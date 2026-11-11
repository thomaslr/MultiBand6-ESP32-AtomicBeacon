// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

//! Onboard Status LED Controller with Nightstand Breathing Wave and Physical Button ACK Patterns.
//!
//! ESP32-C3 SuperMini Status LED is located on GPIO 8 (Active LOW).
//! Patterns:
//! - Standby: completely OFF (frozen HIGH, zero glow / zero leakage)
//! - Button Start ACK: 3 rapid crisp blips (100ms ON / 100ms OFF x 3 = 600ms) then transitions to Breathing
//! - Button Stop ACK: 2 deliberate blinks (200ms ON / 150ms OFF x 2 = 700ms) then Off
//! - Button Error ACK: 5 fast strobes (50ms ON / 50ms OFF x 5 = 500ms) then Off
//! - Breathing Wave: 4.0s gentle sine wave during active RF transmission (capped at ~30% peak brightness)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use esp_idf_svc::sys::{
    ledc_channel_config, ledc_channel_config_t, ledc_channel_t_LEDC_CHANNEL_1,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_duty, ledc_stop, ledc_timer_bit_t_LEDC_TIMER_8_BIT,
    ledc_timer_config, ledc_timer_config_t, ledc_timer_t_LEDC_TIMER_1, ledc_update_duty,
};
use crate::pins::PIN_STATUS_LED;

const LED_BREATHE_PERIOD_MS: u32 = 4000;
const LED_MAX_BRIGHTNESS: f32 = 75.0; // ~30% peak brightness for nightstand comfort
const LED_ACK_BRIGHTNESS: u32 = 180;  // Crisp, unmistakable brightness for button acknowledgments

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LedPattern {
    Standby,
    Breathing,
    ButtonStartAck(u32), // start_ms
    ButtonStopAck(u32),  // start_ms
    ButtonErrorAck(u32), // start_ms
}

pub struct StatusLed {
    pattern: Mutex<LedPattern>,
    was_transmitting: AtomicBool,
}

impl StatusLed {
    pub fn init() -> Self {
        unsafe {
            let timer_cfg = ledc_timer_config_t {
                speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
                duty_resolution: ledc_timer_bit_t_LEDC_TIMER_8_BIT,
                timer_num: ledc_timer_t_LEDC_TIMER_1,
                freq_hz: 1000,
                clk_cfg: 0,
                deconfigure: false,
            };
            ledc_timer_config(&timer_cfg);

            let ch_cfg = ledc_channel_config_t {
                gpio_num: PIN_STATUS_LED,
                speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
                channel: ledc_channel_t_LEDC_CHANNEL_1,
                intr_type: 0,
                timer_sel: ledc_timer_t_LEDC_TIMER_1,
                duty: 256, // 100% duty (pin held HIGH = Active-LOW LED completely OFF)
                hpoint: 0,
                sleep_mode: 0,
                flags: Default::default(),
            };
            ledc_channel_config(&ch_cfg);
            // Freeze channel at logic 1 to guarantee 0V across LED (zero glow / zero leakage)
            ledc_stop(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, 1);
        }
        StatusLed {
            pattern: Mutex::new(LedPattern::Standby),
            was_transmitting: AtomicBool::new(false),
        }
    }

    /// Triggers the 3-blip start acknowledgment (~600ms) when the physical sync button is pressed.
    pub fn trigger_button_start(&self, now_ms: u32) {
        if let Ok(mut pat) = self.pattern.lock() {
            *pat = LedPattern::ButtonStartAck(now_ms);
        }
    }

    /// Triggers the 2-blink stop acknowledgment (~700ms) when the sync button cancels a broadcast.
    pub fn trigger_button_stop(&self, now_ms: u32) {
        if let Ok(mut pat) = self.pattern.lock() {
            *pat = LedPattern::ButtonStopAck(now_ms);
        }
    }

    /// Triggers the 5-strobe error acknowledgment (~500ms) if the button is pressed without valid RTC time.
    pub fn trigger_button_error(&self, now_ms: u32) {
        if let Ok(mut pat) = self.pattern.lock() {
            *pat = LedPattern::ButtonErrorAck(now_ms);
        }
    }

    /// Updates status LED output based on active pattern and transmission status.
    pub fn update(&self, is_transmitting: bool, now_ms: u32) {
        let mut pat = match self.pattern.lock() {
            Ok(p) => *p,
            Err(_) => return,
        };

        unsafe {
            match pat {
                LedPattern::ButtonStartAck(start_ms) => {
                    let elapsed = now_ms.saturating_sub(start_ms);
                    if elapsed < 600 {
                        // 3 rapid pulses: 100ms ON / 100ms OFF x 3
                        let is_on = match elapsed {
                            0..=99 | 200..=299 | 400..=499 => true,
                            _ => false,
                        };
                        let duty = if is_on {
                            256u32.saturating_sub(LED_ACK_BRIGHTNESS)
                        } else {
                            256
                        };
                        ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, duty);
                        ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
                    } else {
                        // Transition to Breathing wave once the 3 blips complete
                        pat = LedPattern::Breathing;
                        if let Ok(mut p) = self.pattern.lock() {
                            *p = LedPattern::Breathing;
                        }
                    }
                }
                LedPattern::ButtonStopAck(start_ms) => {
                    let elapsed = now_ms.saturating_sub(start_ms);
                    if elapsed < 700 {
                        // 2 deliberate blinks: 200ms ON, 150ms OFF, 200ms ON, 150ms OFF
                        let is_on = match elapsed {
                            0..=199 | 350..=549 => true,
                            _ => false,
                        };
                        let duty = if is_on {
                            256u32.saturating_sub(LED_ACK_BRIGHTNESS)
                        } else {
                            256
                        };
                        ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, duty);
                        ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
                    } else {
                        // Transition to Standby (dark) once stop blinks complete
                        if let Ok(mut p) = self.pattern.lock() {
                            *p = LedPattern::Standby;
                        }
                        ledc_stop(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, 1);
                    }
                    return;
                }
                LedPattern::ButtonErrorAck(start_ms) => {
                    let elapsed = now_ms.saturating_sub(start_ms);
                    if elapsed < 500 {
                        // 5 fast strobe bursts: 50ms ON / 50ms OFF x 5
                        let is_on = (elapsed % 100) < 50;
                        let duty = if is_on {
                            256u32.saturating_sub(LED_ACK_BRIGHTNESS)
                        } else {
                            256
                        };
                        ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, duty);
                        ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
                    } else {
                        // Transition to Standby once strobe finishes
                        if let Ok(mut p) = self.pattern.lock() {
                            *p = LedPattern::Standby;
                        }
                        ledc_stop(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, 1);
                    }
                    return;
                }
                LedPattern::Standby => {
                    if is_transmitting {
                        // Automated or Web sync: starts breathing directly with zero start blips
                        pat = LedPattern::Breathing;
                        if let Ok(mut p) = self.pattern.lock() {
                            *p = LedPattern::Breathing;
                        }
                    }
                }
                LedPattern::Breathing => {}
            }

            // Execute Breathing Wave when in Breathing state
            if pat == LedPattern::Breathing {
                if is_transmitting {
                    self.was_transmitting.store(true, Ordering::Relaxed);
                    let phase = ((now_ms % LED_BREATHE_PERIOD_MS) as f32 / LED_BREATHE_PERIOD_MS as f32)
                        * 2.0
                        * core::f32::consts::PI;
                    let factor = (1.0 - phase.cos()) * 0.5;
                    let brightness = (factor * factor * LED_MAX_BRIGHTNESS) as u32;
                    let duty = 256u32.saturating_sub(brightness);
                    ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, duty);
                    ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
                } else if self.was_transmitting.swap(false, Ordering::Relaxed) {
                    if let Ok(mut p) = self.pattern.lock() {
                        *p = LedPattern::Standby;
                    }
                    ledc_stop(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, 1);
                }
            }
        }
    }
}
