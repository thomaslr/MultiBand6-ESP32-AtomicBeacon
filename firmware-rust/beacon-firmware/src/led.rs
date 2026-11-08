//! Onboard Status LED Controller with Nightstand Breathing Wave.
//!
//! ESP32-C3 SuperMini Status LED is located on GPIO 8 (Active LOW).
//! In ESP-IDF LEDC, an 8-bit timer ranges from 0 to 256 (where 256 is 100% HIGH).
//! Setting duty to 255 causes a 1/256th low-pulse glitch (0.39% duty), which makes
//! high-efficiency LEDs glow faintly. We freeze the pin firmly HIGH (1) in standby.
//! During active RF transmission, breathes smoothly with a 4.0s sine cycle.

use std::sync::atomic::{AtomicBool, Ordering};

use esp_idf_svc::sys::{
    ledc_channel_config_t, ledc_channel_t_LEDC_CHANNEL_1, ledc_channel_config,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_duty, ledc_stop, ledc_timer_bit_t_LEDC_TIMER_8_BIT,
    ledc_timer_config, ledc_timer_config_t, ledc_timer_t_LEDC_TIMER_1,
    ledc_update_duty,
};
use crate::pins::PIN_STATUS_LED;

const LED_BREATHE_PERIOD_MS: u32 = 4000;
const LED_MAX_BRIGHTNESS: f32 = 75.0; // ~30% peak brightness for nightstand comfort

pub struct StatusLed {
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
            was_transmitting: AtomicBool::new(false),
        }
    }

    /// Updates breathing pulse based on current milliseconds timestamp.
    pub fn update(&self, is_transmitting: bool, now_ms: u32) {
        unsafe {
            if is_transmitting {
                self.was_transmitting.store(true, Ordering::Relaxed);
                let phase = ((now_ms % LED_BREATHE_PERIOD_MS) as f32 / LED_BREATHE_PERIOD_MS as f32)
                    * 2.0
                    * core::f32::consts::PI;
                let factor = (1.0 - phase.cos()) * 0.5;
                let brightness = (factor * factor * LED_MAX_BRIGHTNESS) as u32;
                // Active LOW: 256 = OFF, lower values increase brightness
                let duty = 256u32.saturating_sub(brightness);
                ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, duty);
                ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
            } else if self.was_transmitting.swap(false, Ordering::Relaxed) {
                // When ending transmission, hold pin firmly HIGH at logic 1 (100% dark)
                ledc_stop(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, 1);
            }
        }
    }
}
