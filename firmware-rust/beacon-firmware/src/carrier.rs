//! RF Carrier Generation & 50ms Pulse Modulator for GPIO 1.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use beacon_core::config::AppConfig;
use beacon_core::protocols::{
    generate_frame, get_tick_level, TimeStation, ALL_STATIONS, TIMEPROTO_MAX_LEVEL_BYTES,
    TOTAL_FRAME_TICKS,
};
use esp_idf_svc::sys::{
    esp_timer_create, esp_timer_create_args_t, esp_timer_handle_t, esp_timer_start_periodic,
    esp_timer_stop, ledc_channel_config, ledc_channel_config_t, ledc_channel_t_LEDC_CHANNEL_0,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_duty, ledc_timer_bit_t_LEDC_TIMER_8_BIT,
    ledc_timer_config, ledc_timer_config_t, ledc_timer_t_LEDC_TIMER_0, ledc_update_duty,
};
use log::info;
use crate::pins::PIN_ANTENNA;

pub struct CarrierState {
    pub is_transmitting: AtomicBool,
    pub active_station: Mutex<TimeStation>,
    pub carrier_freq_hz: AtomicU32,
    pub tick_buffer: Mutex<[u8; TIMEPROTO_MAX_LEVEL_BYTES]>,
    pub current_tick_idx: AtomicUsize,
    pub transmit_start_sec: AtomicU32,
    pub transmit_duration_sec: AtomicU32,
    pub user_offset_hours: Mutex<i32>,
    pub trigger_source: Mutex<String>,

    // Carousel state
    pub carousel_enabled: AtomicBool,
    pub carousel_stations: Mutex<Vec<TimeStation>>,
    pub carousel_stage_idx: AtomicUsize,
    pub stage_start_sec: AtomicU32,
    pub stage_duration_sec: AtomicU32,
}

static mut GLOBAL_STATE: Option<Arc<CarrierState>> = None;
static mut TIMER_HANDLE: esp_timer_handle_t = core::ptr::null_mut();

unsafe extern "C" fn on_tick_timer_callback(_arg: *mut core::ffi::c_void) {
    if let Some(ref state) = GLOBAL_STATE {
        if !state.is_transmitting.load(Ordering::Relaxed) {
            return;
        }

        let mut idx = state.current_tick_idx.load(Ordering::Relaxed);

        // Frame boundary check (at 1200 ticks = 60s)
        if idx >= TOTAL_FRAME_TICKS {
            let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };

            // Carousel stage handoff check
            if state.carousel_enabled.load(Ordering::Relaxed) {
                let stage_start = state.stage_start_sec.load(Ordering::Relaxed);
                let stage_dur = state.stage_duration_sec.load(Ordering::Relaxed);
                if (now_sec.saturating_sub(stage_start)) >= stage_dur {
                    let stations = state.carousel_stations.lock().unwrap();
                    let current_stage = state.carousel_stage_idx.load(Ordering::Relaxed);
                    if current_stage + 1 < stations.len() {
                        let next_stage = current_stage + 1;
                        state.carousel_stage_idx.store(next_stage, Ordering::Relaxed);
                        state.stage_start_sec.store(now_sec, Ordering::Relaxed);

                        let next_st = stations[next_stage];
                        *state.active_station.lock().unwrap() = next_st;
                        let freq = next_st.carrier_freq_hz();
                        state.carrier_freq_hz.store(freq, Ordering::Relaxed);
                        setup_ledc_pwm(freq);
                    }
                }
            }

            // Regenerate frame for upcoming minute epoch (robust against +/- 1s boundary jitter)
            let station = *state.active_station.lock().unwrap();
            let offset_hours = *state.user_offset_hours.lock().unwrap();
            let frame_minute_sec = ((now_sec + 1) - ((now_sec + 1) % 60)) as u64;
            let (new_frame, _) = generate_frame(station, frame_minute_sec, offset_hours * 3600, 0);

            *state.tick_buffer.lock().unwrap() = new_frame;
            idx = 0;
            state.current_tick_idx.store(0, Ordering::Relaxed);
        }

        let tick_val = {
            let buf = state.tick_buffer.lock().unwrap();
            get_tick_level(&*buf, idx)
        };

        state.current_tick_idx.store(idx + 1, Ordering::Relaxed);

        if tick_val {
            ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0, 128); // 50% duty
            ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0);
        } else {
            ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0, 0);   // 0% duty
            ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0);
        }
    }
}

fn setup_ledc_pwm(freq_hz: u32) {
    unsafe {
        let timer_cfg = ledc_timer_config_t {
            speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
            duty_resolution: ledc_timer_bit_t_LEDC_TIMER_8_BIT,
            timer_num: ledc_timer_t_LEDC_TIMER_0,
            freq_hz,
            clk_cfg: 0,
            deconfigure: false,
        };
        ledc_timer_config(&timer_cfg);

        let ch_cfg = ledc_channel_config_t {
            gpio_num: PIN_ANTENNA,
            speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
            channel: ledc_channel_t_LEDC_CHANNEL_0,
            intr_type: 0,
            timer_sel: ledc_timer_t_LEDC_TIMER_0,
            duty: 0, // Carrier OFF initially
            hpoint: 0,
            sleep_mode: 0,
            flags: Default::default(),
        };
        ledc_channel_config(&ch_cfg);
    }
}

pub struct CarrierManager {
    pub state: Arc<CarrierState>,
}

impl CarrierManager {
    pub fn new(config: &AppConfig) -> Self {
        let state = Arc::new(CarrierState {
            is_transmitting: AtomicBool::new(false),
            active_station: Mutex::new(config.station),
            carrier_freq_hz: AtomicU32::new(config.station.carrier_freq_hz()),
            tick_buffer: Mutex::new([0u8; TIMEPROTO_MAX_LEVEL_BYTES]),
            current_tick_idx: AtomicUsize::new(0),
            transmit_start_sec: AtomicU32::new(0),
            transmit_duration_sec: AtomicU32::new(0),
            user_offset_hours: Mutex::new(config.offset_hours),
            trigger_source: Mutex::new("standby".to_string()),
            carousel_enabled: AtomicBool::new(config.carousel_enabled),
            carousel_stations: Mutex::new(Self::build_station_list(config.carousel_mask, config.station)),
            carousel_stage_idx: AtomicUsize::new(0),
            stage_start_sec: AtomicU32::new(0),
            stage_duration_sec: AtomicU32::new(config.carousel_duration_min * 60),
        });

        unsafe {
            GLOBAL_STATE = Some(Arc::clone(&state));
            setup_ledc_pwm(config.station.carrier_freq_hz());

            let timer_args = esp_timer_create_args_t {
                callback: Some(on_tick_timer_callback),
                arg: core::ptr::null_mut(),
                dispatch_method: esp_idf_svc::sys::esp_timer_dispatch_t_ESP_TIMER_TASK,
                name: b"50ms_tick\0".as_ptr() as *const _,
                skip_unhandled_events: true,
            };
            esp_timer_create(&timer_args, &raw mut TIMER_HANDLE);
        }

        CarrierManager { state }
    }

    pub fn build_station_list(mask: u8, fallback: TimeStation) -> Vec<TimeStation> {
        let mut list = Vec::new();
        for (i, &st) in ALL_STATIONS.iter().enumerate() {
            if (mask & (1 << i)) != 0 {
                list.push(st);
            }
        }
        if list.is_empty() {
            list.push(fallback);
        }
        list
    }

    pub fn start(&self, config: &AppConfig, duration_sec: Option<u32>, source: &str) {
        if self.state.is_transmitting.load(Ordering::Relaxed) {
            return;
        }

        *self.state.trigger_source.lock().unwrap() = source.to_string();

        // Lock Wi-Fi modem sleep OFF during broadcast to ensure clean power rail
        unsafe {
            esp_idf_svc::sys::esp_wifi_set_ps(esp_idf_svc::sys::wifi_ps_type_t_WIFI_PS_NONE);
        }

        let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };
        let is_carousel = config.carousel_enabled;

        let (first_station, total_duration, stage_dur) = if is_carousel {
            let stations = Self::build_station_list(config.carousel_mask, config.station);
            let s_dur = config.carousel_duration_min * 60;
            let tot_dur = duration_sec.unwrap_or(stations.len() as u32 * s_dur);
            let first = stations[0];
            *self.state.carousel_stations.lock().unwrap() = stations;
            self.state.carousel_stage_idx.store(0, Ordering::Relaxed);
            self.state.stage_start_sec.store(now_sec, Ordering::Relaxed);
            self.state.stage_duration_sec.store(s_dur, Ordering::Relaxed);
            self.state.carousel_enabled.store(true, Ordering::Relaxed);
            (first, tot_dur, s_dur)
        } else {
            let tot_dur = duration_sec.unwrap_or(config.broadcast_duration_min * 60);
            self.state.carousel_enabled.store(false, Ordering::Relaxed);
            self.state.carousel_stage_idx.store(0, Ordering::Relaxed);
            self.state.stage_start_sec.store(now_sec, Ordering::Relaxed);
            self.state.stage_duration_sec.store(tot_dur, Ordering::Relaxed);
            (config.station, tot_dur, tot_dur)
        };

        *self.state.active_station.lock().unwrap() = first_station;
        *self.state.user_offset_hours.lock().unwrap() = config.offset_hours;
        let freq = first_station.carrier_freq_hz();
        self.state.carrier_freq_hz.store(freq, Ordering::Relaxed);
        setup_ledc_pwm(freq);

        // Align initial frame and tick index precisely to wall-clock time
        let mut tv = esp_idf_svc::sys::timeval { tv_sec: 0, tv_usec: 0 };
        unsafe {
            esp_idf_svc::sys::gettimeofday(&mut tv, core::ptr::null_mut());
        }
        let delay_us = 50_000 - (tv.tv_usec % 50_000);
        if delay_us > 0 && delay_us < 50_000 {
            let target_us = unsafe { esp_idf_svc::sys::esp_timer_get_time() } + (delay_us as i64);
            while unsafe { esp_idf_svc::sys::esp_timer_get_time() } < target_us {}
        }

        // Re-read exact wall-clock time right at the 50ms boundary
        unsafe {
            esp_idf_svc::sys::gettimeofday(&mut tv, core::ptr::null_mut());
        }
        let clk_sec = tv.tv_sec as u32;
        let sec_in_min = (clk_sec % 60) as usize;
        let tick_in_sec = ((tv.tv_usec / 50_000) as usize).min(19);
        let now_tick_idx = sec_in_min * 20 + tick_in_sec;

        // Base the initial frame on the start of the current minute
        let minute_start_sec = (clk_sec - (clk_sec % 60)) as u64;
        let (frame, _) = generate_frame(first_station, minute_start_sec, config.offset_hours * 3600, 0);
        let initial_tick_val = get_tick_level(&frame, now_tick_idx);
        *self.state.tick_buffer.lock().unwrap() = frame;
        self.state.transmit_start_sec.store(clk_sec, Ordering::Relaxed);
        self.state.transmit_duration_sec.store(total_duration, Ordering::Relaxed);

        // Immediately assert the current 50ms tick level and schedule next tick in 50ms
        self.state.current_tick_idx.store(now_tick_idx + 1, Ordering::Relaxed);
        self.state.is_transmitting.store(true, Ordering::SeqCst);

        unsafe {
            let duty = if initial_tick_val { 128 } else { 0 };
            ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0, duty);
            ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0);

            if !TIMER_HANDLE.is_null() {
                esp_timer_start_periodic(TIMER_HANDLE, 50_000); // 50,000 us = 50 ms
            }
        }
        info!(
            "[Carrier] Broadcast started for {} ({} Hz, total {}s, stage {}s)",
            first_station.station_name(), freq, total_duration, stage_dur
        );
    }

    pub fn stop(&self, modem_sleep_enabled: bool) {
        if !self.state.is_transmitting.load(Ordering::Relaxed) {
            return;
        }
        self.state.is_transmitting.store(false, Ordering::SeqCst);
        *self.state.trigger_source.lock().unwrap() = "standby".to_string();

        unsafe {
            if !TIMER_HANDLE.is_null() {
                esp_timer_stop(TIMER_HANDLE);
            }
            // Shut off carrier completely
            ledc_set_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0, 0);
            ledc_update_duty(ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_channel_t_LEDC_CHANNEL_0);

            // Restore Wi-Fi modem sleep if configured
            if modem_sleep_enabled {
                esp_idf_svc::sys::esp_wifi_set_ps(esp_idf_svc::sys::wifi_ps_type_t_WIFI_PS_MIN_MODEM);
            }
        }
        info!("[Carrier] Broadcast stopped. Carrier OFF.");
    }
}
