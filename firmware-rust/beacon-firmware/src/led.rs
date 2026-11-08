//! Onboard Status LED Controller with Nightstand Breathing Wave.
//!
//! ESP32-C3 SuperMini Status LED is located on GPIO 8 (Active LOW).
//! Duty cycle 255 = 100% OFF (dark in standby).
//! During active RF transmission, breathes smoothly with a 4.0s sine cycle.

use esp_idf_svc::sys::{
    ledc_channel_config_t, ledc_channel_t_LEDC_CHANNEL_1, ledc_channel_config,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_duty, ledc_timer_bit_t_LEDC_TIMER_8_BIT,
    ledc_timer_config, ledc_timer_config_t, ledc_timer_t_LEDC_TIMER_1,
    ledc_update_duty,
};
use crate::pins::PIN_STATUS_LED;

const LED_BREATHE_PERIOD_MS: u32 = 4000;
const LED_MAX_BRIGHTNESS: f32 = 75.0; // ~30% peak brightness for nightstand comfort

pub struct StatusLed;

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
                duty: 255, // Active LOW: 255 = completely OFF
                hpoint: 0,
                sleep_mode: 0,
                flags: Default::default(),
            };
            ledc_channel_config(&ch_cfg);
        }
        StatusLed
    }

    /// Updates breathing pulse based on current milliseconds timestamp.
    pub fn update(&self, is_transmitting: bool, now_ms: u32) {
        unsafe {
            if is_transmitting {
                let phase = ((now_ms % LED_BREATHE_PERIOD_MS) as f32 / LED_BREATHE_PERIOD_MS as f32)
                    * 2.0
                    * core::f32::consts::PI;
                let factor = (1.0 - phase.cos()) * 0.5;
                let brightness = (factor * factor * LED_MAX_BRIGHTNESS) as u32;
                let duty = 255 - brightness;
                ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, duty);
                ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
            } else {
                ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1, 255);
                ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_1);
            }
        }
    }
}
