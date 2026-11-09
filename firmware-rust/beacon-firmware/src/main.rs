//! MultiBand 6 Worldwide Radio Atomic Clock Simulator Firmware
//!
//! ESP32-C3 SuperMini / Custom Carrier Board

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::{PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::mdns::EspMdns;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::esp_wifi_sta_get_rssi;
use log::{info, warn};

mod carrier;
mod led;
mod ota;
mod pins;
mod secrets;
mod sntp;
mod storage;
mod web;
mod wifi;

use carrier::CarrierManager;
use led::StatusLed;
use ota::mark_running_app_valid;
use storage::NvsConfigStorage;
use web::{create_setup_server, create_web_server, AppState};
use wifi::{start_wifi, WifiState};

fn main() -> anyhow::Result<()> {
    // Bind esp-idf-svc logger
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("==================================================");
    info!("  TimeStation ESP32-C3 MultiBand 6 Simulator Booting");
    info!("  Dual-OTA Partition Table & Rollback Protected");
    info!("==================================================");

    // 1. Confirm boot health and cancel bootloader rollback
    mark_running_app_valid();

    // 2. Hardware initialization
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs_default = EspDefaultNvsPartition::take()?;

    let status_led = StatusLed::init();
    status_led.update(false, 0); // Completely dark in standby

    // Bedside sync button (SW1 / J3: GPIO 3, Active LOW)
    let _sync_button = PinDriver::input(peripherals.pins.gpio3, Pull::Up)?;
    unsafe {
        // Configure internal pullup for GPIO 3
        esp_idf_svc::sys::gpio_set_pull_mode(
            pins::PIN_SYNC_BUTTON,
            esp_idf_svc::sys::gpio_pull_mode_t_GPIO_PULLUP_ONLY,
        );
    }

    // 3. Storage initialization
    let storage = NvsConfigStorage::new(nvs_default.clone())
        .map_err(|e| anyhow::anyhow!("NVS init error: {:?}", e))?;
    let config = storage.load_config();

    // Load or seed saved Wi-Fi credentials
    let mut saved_creds = storage.load_wifi();
    if saved_creds.is_none() && !secrets::WIFI_SSID.is_empty() {
        info!("[Setup] Seeding Wi-Fi credentials from secrets.rs");
        let _ = storage.save_wifi(secrets::WIFI_SSID, secrets::WIFI_PASSWORD);
        saved_creds = storage.load_wifi();
    }

    // 4. Carrier Manager initialization
    let carrier = CarrierManager::new(&config);

    // 5. Wi-Fi Startup (Station or SoftAP Provisioning Mode)
    let has_saved_wifi = saved_creds.is_some();
    let wifi_state = start_wifi(
        peripherals.modem,
        sys_loop,
        Some(nvs_default),
        saved_creds,
        &config.system.hostname,
        &config.wifi,
    )
    .map_err(|e| anyhow::anyhow!("Wi-Fi startup error: {:?}", e))?;

    match wifi_state {
        WifiState::StationConnected { ip_address, wifi_handle } => {
            info!("[Main] Wi-Fi connected! Station IP: {}", ip_address);

            // Start mDNS Responder (timestation.local)
            let _mdns = match EspMdns::take() {
                Ok(mut mdns) => {
                    let host = &config.system.hostname;
                    if let Err(e) = mdns.set_hostname(host) {
                        warn!("[mDNS] Failed to set hostname: {:?}", e);
                    }
                    let _ = mdns.set_instance_name(&format!("TimeStation ({})", host));
                    let _ = mdns.add_service(None, "_http", "_tcp", 80, &[("board", "esp32c3")]);
                    info!("[mDNS] Responder active at http://{}.local", host);
                    Some(mdns)
                }
                Err(e) => {
                    warn!("[mDNS] Failed to start responder: {:?}", e);
                    None
                }
            };

            // Start Persistent NTP Time Sync
            let sntp = match sntp::SntpManager::init(&config.system.ntp_server) {
                Ok(mgr) => {
                    info!("[SNTP] Attempting initial atomic time sync on boot...");
                    let _ = mgr.sync(Duration::from_secs(8));
                    Some(mgr)
                }
                Err(e) => {
                    warn!("[SNTP] Failed to initialize SNTP: {:?}", e);
                    None
                }
            };

            // Shared application state for Web Server
            let app_state = Arc::new(Mutex::new(AppState {
                config: config.clone(),
                storage,
                carrier: CarrierManager { state: Arc::clone(&carrier.state) },
                ip_address: ip_address.clone(),
                wifi_ssid: wifi_handle.get_configuration().ok().and_then(|c| match c {
                    esp_idf_svc::wifi::Configuration::Client(client) => Some(client.ssid.to_string()),
                    _ => None,
                }).unwrap_or_default(),
                wifi_rssi: 0,
                scanned_ssids: Vec::new(),
                sntp: sntp.clone(),
                ota_update: crate::ota::OtaUpdateState::default(),
            }));

            // Launch Main Web Server
            let _server = create_web_server(Arc::clone(&app_state))?;

            // Prevent network and servers from dropping
            core::mem::forget(wifi_handle);
            if let Some(mdns) = _mdns {
                core::mem::forget(mdns);
            }
            core::mem::forget(_server);

            // Station operational loop
            let mut last_button_state = true; // High when not pressed
            let mut last_schedule_day = -1i32;
            let mut last_presync_day = -1i32;
            let mut last_midnight_sync_day = -1i32;
            let mut last_midnight_ota_day = -1i32;
            let mut boot_ota_checked = false;
            let boot_time_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
            let mut last_bg_retry_ms = 0u32;
            let mut last_is_tx = false;

            loop {
                let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
                let is_tx = carrier.state.is_transmitting.load(Ordering::Relaxed);
                let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };
                let is_synced = now_sec > 1_700_000_000;

                // Track transmission state transitions to suspend/resume SNTP service
                if is_tx != last_is_tx {
                    if is_tx {
                        // Suspend background NTP sync during RF broadcast window to eliminate 2.4 GHz bursts
                        if let Some(ref mgr) = sntp {
                            mgr.suspend();
                        }
                    } else {
                        // Resume background NTP sync service when broadcast completes
                        if let Some(ref mgr) = sntp {
                            mgr.resume();
                        }
                    }
                    last_is_tx = is_tx;
                }

                // Update LED breathing pulse
                status_led.update(is_tx, now_ms);

                // Update RSSI periodically
                if now_ms % 5000 < 25 {
                    let mut rssi = 0i32;
                    if unsafe { esp_wifi_sta_get_rssi(&mut rssi) } == 0 {
                        if let Ok(mut app) = app_state.lock() {
                            app.wifi_rssi = rssi;
                        }
                    }
                }

                // Background SNTP retry if boot sync failed or hasn't settled yet (every 15s)
                if !is_synced && !is_tx && now_ms.saturating_sub(last_bg_retry_ms) >= 15_000 {
                    last_bg_retry_ms = now_ms;
                    if let Some(ref mgr) = sntp {
                        info!("[SNTP] RTC not yet synchronized. Requesting background NTP poll...");
                        mgr.request_sync();
                    }
                }

                // Initial OTA update check quickly after boot (~15s or when RTC synchronized)
                if !boot_ota_checked && !is_tx && (is_synced || now_ms.saturating_sub(boot_time_ms) >= 15_000) {
                    boot_ota_checked = true;
                    info!("[Main] Performing initial post-boot OTA update check...");
                    crate::web::trigger_ota_check_background(Arc::clone(&app_state));
                }

                // Check transmission duration timeout
                if is_tx {
                    let tx_start = carrier.state.transmit_start_sec.load(Ordering::Relaxed);
                    let tx_dur = carrier.state.transmit_duration_sec.load(Ordering::Relaxed);
                    if tx_dur > 0 && (now_sec.saturating_sub(tx_start)) >= tx_dur {
                        info!("[Main] Scheduled/test broadcast window complete.");
                        carrier.stop(config.wifi.modem_sleep);
                    }
                }

                // Time synchronization & schedule triggers (only when clock is valid and not transmitting)
                if is_synced && !is_tx {
                    let current_cfg = {
                        app_state.lock().map(|a| a.config.clone()).unwrap_or_else(|_| config.clone())
                    };

                    let offset_sec = current_cfg.station.native_utc_offset_sec() + (current_cfg.offset_hours * 3600);
                    let local_epoch = (now_sec as i64) + (offset_sec as i64);
                    let mut tm_val = unsafe { core::mem::zeroed() };
                    unsafe {
                        esp_idf_svc::sys::gmtime_r(&(local_epoch as esp_idf_svc::sys::time_t), &mut tm_val);
                    }

                    // Midnight local time periodic maintenance sync
                    if tm_val.tm_hour == 0
                        && tm_val.tm_min == 0
                        && tm_val.tm_yday != last_midnight_sync_day
                    {
                        last_midnight_sync_day = tm_val.tm_yday;
                        info!("[Schedule] Midnight station local time: triggering NTP maintenance sync.");
                        if let Some(ref mgr) = sntp {
                            mgr.request_sync();
                        }
                    }

                    // Daily firmware update check: 5 minutes after midnight (00:05 station local time)
                    if tm_val.tm_hour == 0
                        && tm_val.tm_min == 5
                        && tm_val.tm_yday != last_midnight_ota_day
                    {
                        last_midnight_ota_day = tm_val.tm_yday;
                        info!("[Schedule] Post-midnight scheduled check for firmware updates...");
                        crate::web::trigger_ota_check_background(Arc::clone(&app_state));
                    }

                    // Scheduled broadcast logic
                    if current_cfg.schedule_enabled {
                        // Pre-broadcast NTP sync: 2 minutes prior to broadcast start
                        let (pre_hour, pre_min) = if current_cfg.broadcast_minute >= 2 {
                            (current_cfg.broadcast_hour as i32, (current_cfg.broadcast_minute - 2) as i32)
                        } else {
                            (((current_cfg.broadcast_hour as i32) + 23) % 24, (current_cfg.broadcast_minute + 58) as i32)
                        };

                        if tm_val.tm_hour == pre_hour
                            && tm_val.tm_min == pre_min
                            && tm_val.tm_yday != last_presync_day
                        {
                            last_presync_day = tm_val.tm_yday;
                            info!(
                                "[Schedule] Pre-broadcast NTP synchronization window at {:02}:{:02} (2m before {:02}:{:02} broadcast)...",
                                tm_val.tm_hour, tm_val.tm_min, current_cfg.broadcast_hour, current_cfg.broadcast_minute
                            );
                            if let Some(ref mgr) = sntp {
                                mgr.request_sync();
                                let _ = mgr.sync(Duration::from_secs(6));
                            }
                        }

                        // Scheduled broadcast trigger
                        if tm_val.tm_hour == current_cfg.broadcast_hour as i32
                            && tm_val.tm_min == current_cfg.broadcast_minute as i32
                            && tm_val.tm_yday != last_schedule_day
                        {
                            last_schedule_day = tm_val.tm_yday;
                            info!(
                                "[Schedule] Triggering scheduled broadcast at {:02}:{:02} ({} min)...",
                                tm_val.tm_hour, tm_val.tm_min, current_cfg.broadcast_duration_min
                            );
                            carrier.start(&current_cfg, None);
                        }
                    }
                }

                // Manual Bedside Sync Button (SW1 / J3: GPIO 3, Active LOW)
                let btn_pin_low = unsafe { esp_idf_svc::sys::gpio_get_level(pins::PIN_SYNC_BUTTON) == 0 };
                if btn_pin_low && last_button_state {
                    // Button pressed edge
                    if is_tx {
                        info!("[Button] Sync button pressed: Stopping transmission.");
                        carrier.stop(config.wifi.modem_sleep);
                    } else {
                        info!("[Button] Sync button pressed: Starting transmission.");
                        if let Some(ref mgr) = sntp {
                            if !mgr.is_synced() {
                                info!("[Button] RTC not yet synchronized; waiting up to 3s for NTP reference...");
                                let _ = mgr.sync(Duration::from_secs(3));
                            }
                        }
                        let cur_cfg = app_state.lock().map(|a| a.config.clone()).unwrap_or_else(|_| config.clone());
                        carrier.start(&cur_cfg, None);
                    }
                }
                last_button_state = !btn_pin_low;

                std::thread::sleep(Duration::from_millis(20));
            }
        }
        WifiState::ProvisioningMode { ap_ssid, scanned_ssids, wifi_handle } => {
            warn!("[Main] Running in SoftAP Provisioning Mode: '{}'", ap_ssid);
            let shared_storage = Arc::new(Mutex::new(storage));
            let shared_scanned = Arc::new(Mutex::new(scanned_ssids));

            let _server = create_setup_server(shared_storage, shared_scanned)?;
            core::mem::forget(wifi_handle);
            core::mem::forget(_server);

            // 5-minute self-healing recovery timer if saved credentials exist
            let start_time = std::time::Instant::now();
            loop {
                let now_ms = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1000) as u32 };
                status_led.update(false, now_ms);

                if has_saved_wifi && start_time.elapsed() >= Duration::from_secs(300) {
                    warn!("[Main] 5 minutes elapsed in SoftAP with saved credentials. Rebooting to reconnect...");
                    unsafe { esp_idf_svc::sys::esp_restart(); }
                }

                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
