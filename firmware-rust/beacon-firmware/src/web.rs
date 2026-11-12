// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

//! Web Server and REST API Implementation.

use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use beacon_core::config::{AppConfig, ConfigPatch};
use embedded_svc::http::server::Request;
use embedded_svc::http::{Headers, Method};
use embedded_svc::io::{Read, Write};
use esp_idf_svc::http::server::{Configuration as ServerConfig, EspHttpConnection, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use log::info;

use crate::carrier::CarrierManager;
use crate::ota::{flash_stream, get_ota_status, update_from_url, OtaUpdateState};
use crate::sntp::SntpManager;
use crate::storage::NvsConfigStorage;
use crate::wifi::apply_wifi_radio_config;

// Embed web assets directly into firmware (both raw and pre-compressed Gzip)
static INDEX_HTML: &str = include_str!("../../../data/index.html");
static INDEX_HTML_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.html.gz"));
static SETUP_HTML: &str = include_str!("../../../data/setup.html");
static SETUP_HTML_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/setup.html.gz"));
static STYLE_CSS: &str = include_str!("../../../data/style.css");
static STYLE_CSS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/style.css.gz"));
static APP_JS: &str = include_str!("../../../data/app.js");
static APP_JS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.js.gz"));
static FAVICON_SVG: &str = include_str!("../../../data/favicon.svg");
static FAVICON_SVG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/favicon.svg.gz"));
static MANIFEST_JSON: &str = include_str!("../../../data/manifest.json");
static MANIFEST_JSON_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/manifest.json.gz"));
static SW_JS: &str = include_str!("../../../data/sw.js");
static SW_JS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sw.js.gz"));
static ICON_192_PNG: &[u8] = include_bytes!("../../../data/icon-192.png");
static ICON_192_PNG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon-192.png.gz"));
static ICON_512_PNG: &[u8] = include_bytes!("../../../data/icon-512.png");
static ICON_512_PNG_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon-512.png.gz"));

static DIAGNOSTIC_LOGS: Mutex<Option<VecDeque<String>>> = Mutex::new(None);

/// Record a human-readable telemetry event in the in-memory circular diagnostics buffer.
pub fn log_diagnostic(msg: &str) {
    if let Ok(mut lock) = DIAGNOSTIC_LOGS.lock() {
        let deque = lock.get_or_insert_with(|| VecDeque::with_capacity(80));
        let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };
        let entry = if now_sec > 1_700_000_000 {
            let mut tm = esp_idf_svc::sys::tm { ..Default::default() };
            let t = now_sec as esp_idf_svc::sys::time_t;
            unsafe { esp_idf_svc::sys::gmtime_r(&t, &mut tm) };
            format!("[{:02}:{:02}:{:02} UTC] {}", tm.tm_hour, tm.tm_min, tm.tm_sec, msg)
        } else {
            let uptime_sec = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1_000_000) as u32 };
            format!("[+{}s] {}", uptime_sec, msg)
        };
        if deque.len() >= 80 {
            deque.pop_front();
        }
        deque.push_back(entry);
    }
}

#[derive(serde::Serialize)]
struct LogsResponse {
    logs: Vec<String>,
    free_heap: u32,
    min_free_heap: u32,
    uptime_sec: u32,
}

pub struct AppState {
    pub config: AppConfig,
    pub storage: NvsConfigStorage,
    pub carrier: CarrierManager,
    pub ip_address: String,
    pub wifi_ssid: String,
    pub wifi_rssi: i32,
    pub scanned_ssids: Vec<String>,
    pub sntp: Option<Arc<SntpManager>>,
    pub ota_update: OtaUpdateState,
}

pub fn perform_ota_check(app_state: Arc<Mutex<AppState>>) {
    let (is_tx, is_ota) = match app_state.lock() {
        Ok(state) => (
            state.carrier.state.is_transmitting.load(Ordering::Relaxed),
            crate::ota::is_ota_in_progress(),
        ),
        Err(_) => return,
    };
    if is_tx || is_ota {
        info!("[OTA Check] Skipping update check: RF broadcast or OTA currently active.");
        return;
    }

    info!("[OTA Check] Querying latest version.json from GitHub Releases...");
    let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u64 };
    match crate::ota::query_remote_update() {
        Ok((info, update_available)) => {
            info!(
                "[OTA Check] Remote version: {}, local: {}, update_available: {}",
                info.version,
                env!("CARGO_PKG_VERSION"),
                update_available
            );
            if let Ok(mut app) = app_state.lock() {
                app.ota_update = OtaUpdateState {
                    latest_version: info.version,
                    update_available,
                    binary_url: info.url,
                    last_check_epoch: now_sec,
                    last_check_status: "ok".to_string(),
                };
            }
        }
        Err(err) => {
            log::warn!("[OTA Check] Update check error: {}", err);
            if let Ok(mut app) = app_state.lock() {
                app.ota_update.last_check_epoch = now_sec;
                app.ota_update.last_check_status = format!("error: {}", err);
            }
        }
    }
}

pub fn trigger_ota_check_background(app_state: Arc<Mutex<AppState>>) {
    let state_clone = Arc::clone(&app_state);
    let _ = std::thread::Builder::new()
        .stack_size(10240)
        .name("ota_check".to_string())
        .spawn(move || {
            perform_ota_check(state_clone);
        });
}


#[derive(serde::Serialize)]
struct StatusResponse {
    transmitting: bool,
    trigger_source: String,
    active_station: u8,
    station_name: &'static str,
    carrier_freq: u32,
    offset_hours: i32,
    epoch_sec: u64,
    uptime_sec: u32,
    ntp_synced: bool,
    carousel_enabled: bool,
    carousel_stage: usize,
    carousel_total_stages: usize,
    stage_remain_sec: u32,
    total_remain_sec: u32,
    ip: String,
    wifi_ssid: String,
    wifi_rssi: i32,
    hostname: String,
    version: &'static str,
    gpio3_level: i32,
    gpio9_level: i32,
}

pub fn create_web_server(state: Arc<Mutex<AppState>>) -> Result<EspHttpServer<'static>, EspIOError> {
    let server_config = ServerConfig {
        stack_size: 10240,
        max_open_sockets: 12,
        session_timeout: std::time::Duration::from_secs(10),
        lru_purge_enable: true,
        so_linger: Some(std::time::Duration::from_secs(0)),
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;

    // 1. Static Assets
    server.fn_handler("/", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", INDEX_HTML, INDEX_HTML_GZ)
    })?;
    server.fn_handler("/index.html", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", INDEX_HTML, INDEX_HTML_GZ)
    })?;
    server.fn_handler("/app.js", Method::Get, |req| {
        send_file(req, "application/javascript", APP_JS, APP_JS_GZ)
    })?;
    server.fn_handler("/style.css", Method::Get, |req| {
        send_file(req, "text/css", STYLE_CSS, STYLE_CSS_GZ)
    })?;
    server.fn_handler("/favicon.svg", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;
    server.fn_handler("/favicon.ico", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;

    // PWA Manifest, Service Worker, and App Icons
    server.fn_handler("/manifest.json", Method::Get, |req| {
        send_file(req, "application/manifest+json", MANIFEST_JSON, MANIFEST_JSON_GZ)
    })?;
    server.fn_handler("/sw.js", Method::Get, |req| {
        send_file(req, "application/javascript", SW_JS, SW_JS_GZ)
    })?;
    server.fn_handler("/icon-192.png", Method::Get, |req| {
        send_binary(req, "image/png", ICON_192_PNG, ICON_192_PNG_GZ)
    })?;
    server.fn_handler("/icon-512.png", Method::Get, |req| {
        send_binary(req, "image/png", ICON_512_PNG, ICON_512_PNG_GZ)
    })?;

    // 2. GET /api/status
    let s_status = Arc::clone(&state);
    server.fn_handler("/api/status", Method::Get, move |req| {
        let app = s_status.lock().unwrap();
        let c_state = &app.carrier.state;
        let is_tx = c_state.is_transmitting.load(Ordering::Relaxed);
        let station = *c_state.active_station.lock().unwrap();
        let freq = c_state.carrier_freq_hz.load(Ordering::Relaxed);
        let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };
        let uptime_sec = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1_000_000) as u32 };

        let tx_start = c_state.transmit_start_sec.load(Ordering::Relaxed);
        let tx_dur = c_state.transmit_duration_sec.load(Ordering::Relaxed);
        let total_remain_sec = if is_tx && tx_dur > 0 {
            tx_dur.saturating_sub(now_sec.saturating_sub(tx_start))
        } else {
            0
        };

        let st_start = c_state.stage_start_sec.load(Ordering::Relaxed);
        let st_dur = c_state.stage_duration_sec.load(Ordering::Relaxed);
        let stage_remain_sec = if is_tx && st_dur > 0 {
            st_dur.saturating_sub(now_sec.saturating_sub(st_start))
        } else {
            0
        };

        let total_stages = c_state.carousel_stations.lock().unwrap().len();

        let trigger_source = c_state.trigger_source.lock().unwrap().clone();
        let gpio3_level = unsafe { esp_idf_svc::sys::gpio_get_level(crate::pins::PIN_SYNC_BUTTON) };
        let gpio9_level = unsafe { esp_idf_svc::sys::gpio_get_level(crate::pins::PIN_BOOT_BUTTON) };

        let resp = StatusResponse {
            transmitting: is_tx,
            trigger_source,
            active_station: station as u8,
            station_name: station.station_name(),
            carrier_freq: freq,
            offset_hours: *c_state.user_offset_hours.lock().unwrap(),
            epoch_sec: now_sec as u64,
            uptime_sec,
            ntp_synced: now_sec > 1_700_000_000,
            carousel_enabled: c_state.carousel_enabled.load(Ordering::Relaxed),
            carousel_stage: c_state.carousel_stage_idx.load(Ordering::Relaxed) + 1,
            carousel_total_stages: total_stages,
            stage_remain_sec,
            total_remain_sec,
            ip: app.ip_address.clone(),
            wifi_ssid: app.wifi_ssid.clone(),
            wifi_rssi: app.wifi_rssi,
            hostname: app.config.system.hostname.clone(),
            version: env!("CARGO_PKG_VERSION"),
            gpio3_level,
            gpio9_level,
        };

        let json = serde_json::to_string(&resp).unwrap_or_default();
        send_json(req, 200, &json)
    })?;

    // 3. GET /api/config
    let s_cfg = Arc::clone(&state);
    server.fn_handler("/api/config", Method::Get, move |req| {
        let app = s_cfg.lock().unwrap();
        let json = serde_json::to_string(&app.config).unwrap_or_default();
        send_json(req, 200, &json)
    })?;

    // 4. POST /api/config (apply patch)
    let s_patch = Arc::clone(&state);
    server.fn_handler("/api/config", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        match serde_json::from_slice::<ConfigPatch>(&buf) {
            Ok(patch) => {
                let mut app = s_patch.lock().unwrap();
                let old_ntp = app.config.system.ntp_server.clone();
                app.config.apply_patch(patch);
                let _ = app.storage.save_config(&app.config);
                apply_wifi_radio_config(&app.config.wifi);

                if app.config.system.ntp_server != old_ntp {
                    if let Some(ref sntp) = app.sntp {
                        sntp.request_sync();
                    }
                }
                send_json(req, 200, r#"{"status":"saved"}"#)
            }
            Err(e) => {
                let err = format!(r#"{{"error":"Failed to parse config: {}"}}"#, e);
                send_json(req, 400, &err)
            }
        }
    })?;

    // 5. POST /api/sync-ntp (force immediate NTP synchronization)
    let s_ntp = Arc::clone(&state);
    server.fn_handler("/api/sync-ntp", Method::Post, move |req| {
        let app = s_ntp.lock().unwrap();
        if let Some(ref sntp) = app.sntp {
            sntp.request_sync();
        }
        send_json(req, 200, r#"{"status":"sync_requested"}"#)
    })?;

    // 6. POST /api/transmit
    let s_tx = Arc::clone(&state);
    server.fn_handler("/api/transmit", Method::Post, move |req| {
        let app = s_tx.lock().unwrap();
        if let Some(ref sntp) = app.sntp {
            if !sntp.is_synced() {
                info!("[API] RTC not yet synchronized prior to broadcast; attempting 3s NTP sync...");
                let _ = sntp.sync(std::time::Duration::from_secs(3));
            }
        }
        let mut cur_cfg = app.config.clone();
        cur_cfg.carousel_enabled = false;
        app.carrier.start(&cur_cfg, Some(15 * 60), "web");
        send_json(req, 200, r#"{"status":"transmitting"}"#)
    })?;

    // 6. POST /api/stop
    let s_stop = Arc::clone(&state);
    server.fn_handler("/api/stop", Method::Post, move |req| {
        let app = s_stop.lock().unwrap();
        app.carrier.stop(app.config.wifi.modem_sleep);
        send_json(req, 200, r#"{"status":"stopped"}"#)
    })?;

    // 7. GET /api/scan
    let s_scan = Arc::clone(&state);
    server.fn_handler("/api/scan", Method::Get, move |req| {
        let app = s_scan.lock().unwrap();
        let json = serde_json::to_string(&app.scanned_ssids).unwrap_or_else(|_| "[]".to_string());
        send_json(req, 200, &json)
    })?;

    // 8. POST /api/provision
    let s_prov = Arc::clone(&state);
    server.fn_handler("/api/provision", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        #[derive(serde::Deserialize)]
        struct ProvReq {
            ssid: String,
            password: Option<String>,
        }

        match serde_json::from_slice::<ProvReq>(&buf) {
            Ok(p) => {
                let ssid = p.ssid.trim();
                if ssid.is_empty() {
                    return send_json(req, 400, r#"{"error":"SSID cannot be empty"}"#);
                }
                let pass = p.password.as_deref().unwrap_or("");
                let save_res = {
                    let app = s_prov.lock().unwrap();
                    app.storage.save_wifi(ssid, pass)
                };

                match save_res {
                    Ok(()) => {
                        let res = send_json(req, 200, r#"{"status":"saved","message":"Restarting..."}"#);
                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(1500));
                            unsafe { esp_idf_svc::sys::esp_restart(); }
                        });
                        res
                    }
                    Err(e) => {
                        let err_json = format!(r#"{{"error":"{}"}}"#, e);
                        send_json(req, 500, &err_json)
                    }
                }
            }
            Err(_) => send_json(req, 400, r#"{"error":"Invalid JSON"}"#),
        }
    })?;

    // 9. POST /api/wifi/reset
    let s_reset = Arc::clone(&state);
    server.fn_handler("/api/wifi/reset", Method::Post, move |req| {
        info!("[HTTP] Wi-Fi reset requested. Erasing credentials and restarting into setup AP...");
        {
            let app = s_reset.lock().unwrap();
            let _ = app.storage.clear_wifi();
        }
        let res = send_json(req, 200, r#"{"status":"reset","message":"Entering setup mode..."}"#);
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            unsafe { esp_idf_svc::sys::esp_restart(); }
        });
        res
    })?;

    // 10. POST /api/system/reboot
    server.fn_handler("/api/system/reboot", Method::Post, |req| {
        info!("[HTTP] Reboot requested via API");
        let res = send_json(req, 200, r#"{"status":"rebooting"}"#);
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            unsafe { esp_idf_svc::sys::esp_restart(); }
        });
        res
    })?;

    // 11. OTA Endpoints
    let s_ota = Arc::clone(&state);
    server.fn_handler("/api/ota/status", Method::Get, move |req| {
        let ota_update = s_ota
            .lock()
            .map(|a| a.ota_update.clone())
            .unwrap_or_default();
        let status = get_ota_status(&ota_update);
        let json = serde_json::to_string(&status).unwrap_or_default();
        send_json(req, 200, &json)
    })?;

    let s_check = Arc::clone(&state);
    server.fn_handler("/api/ota/check", Method::Post, move |req| {
        info!("[HTTP] On-demand OTA update check requested via API");
        trigger_ota_check_background(Arc::clone(&s_check));
        send_json(req, 200, r#"{"status":"checking"}"#)
    })?;


    server.fn_handler("/api/ota/update", Method::Post, |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        #[derive(serde::Deserialize)]
        struct UpdateReq { url: String }

        match serde_json::from_slice::<UpdateReq>(&buf) {
            Ok(body) => match update_from_url(&body.url) {
                Ok(()) => send_json(req, 200, r#"{"status":"flashed","message":"Rebooting..."}"#),
                Err(e) => {
                    let err = format!(r#"{{"error":"{}"}}"#, e);
                    send_json(req, 500, &err)
                }
            },
            Err(_) => send_json(req, 400, r#"{"error":"Invalid payload"}"#),
        }
    })?;

    server.fn_handler("/api/ota/upload", Method::Post, |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        if len == 0 {
            return send_json(req, 400, r#"{"error":"Content-Length required"}"#);
        }
        match flash_stream(&mut req, len) {
            Ok(()) => send_json(req, 200, r#"{"status":"flashed","message":"Rebooting..."}"#),
            Err(e) => {
                let err = format!(r#"{{"error":"{}"}}"#, e);
                send_json(req, 500, &err)
            }
        }
    })?;

    // 14. GET /api/logs
    server.fn_handler("/api/logs", Method::Get, |req| {
        let logs: Vec<String> = if let Ok(mut lock) = DIAGNOSTIC_LOGS.lock() {
            let deque = lock.get_or_insert_with(|| VecDeque::with_capacity(80));
            deque.iter().cloned().collect()
        } else {
            Vec::new()
        };
        let free_heap = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };
        let min_free_heap = unsafe { esp_idf_svc::sys::esp_get_minimum_free_heap_size() };
        let uptime_sec = unsafe { (esp_idf_svc::sys::esp_timer_get_time() / 1_000_000) as u32 };
        let resp = LogsResponse {
            logs,
            free_heap,
            min_free_heap,
            uptime_sec,
        };
        let json = serde_json::to_string(&resp).unwrap_or_default();
        send_json(req, 200, &json)
    })?;

    // 15. POST /api/logs/clear
    server.fn_handler("/api/logs/clear", Method::Post, |req| {
        if let Ok(mut lock) = DIAGNOSTIC_LOGS.lock() {
            if let Some(deque) = lock.as_mut() {
                deque.clear();
            }
        }
        send_json(req, 200, r#"{"status":"cleared"}"#)
    })?;

    info!("[HTTP] Web server initialized on port 80");
    Ok(server)
}

/// Creates a lightweight setup web server for Provisioning SoftAP Mode.
pub fn create_setup_server(
    storage: Arc<Mutex<NvsConfigStorage>>,
    scanned_ssids: Arc<Mutex<Vec<String>>>,
) -> Result<EspHttpServer<'static>, EspIOError> {
    let server_config = ServerConfig {
        stack_size: 10240,
        max_open_sockets: 10,
        session_timeout: std::time::Duration::from_secs(10),
        lru_purge_enable: true,
        so_linger: Some(std::time::Duration::from_secs(0)),
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&server_config)?;

    // 1. Setup Page
    server.fn_handler("/", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/setup", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;

    // 2. Captive Portal Detection Probes
    server.fn_handler("/hotspot-detect.html", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/generate_204", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/gen_204", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/ncsi.txt", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/connecttest.txt", Method::Get, |req| {
        send_file(req, "text/html; charset=utf-8", SETUP_HTML, SETUP_HTML_GZ)
    })?;
    server.fn_handler("/favicon.svg", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;
    server.fn_handler("/favicon.ico", Method::Get, |req| {
        send_file(req, "image/svg+xml", FAVICON_SVG, FAVICON_SVG_GZ)
    })?;

    // PWA Manifest, Service Worker, and App Icons
    server.fn_handler("/manifest.json", Method::Get, |req| {
        send_file(req, "application/manifest+json", MANIFEST_JSON, MANIFEST_JSON_GZ)
    })?;
    server.fn_handler("/sw.js", Method::Get, |req| {
        send_file(req, "application/javascript", SW_JS, SW_JS_GZ)
    })?;
    server.fn_handler("/icon-192.png", Method::Get, |req| {
        send_binary(req, "image/png", ICON_192_PNG, ICON_192_PNG_GZ)
    })?;
    server.fn_handler("/icon-512.png", Method::Get, |req| {
        send_binary(req, "image/png", ICON_512_PNG, ICON_512_PNG_GZ)
    })?;

    // 3. GET /api/scan
    let s_scan = Arc::clone(&scanned_ssids);
    server.fn_handler("/api/scan", Method::Get, move |req| {
        let list = s_scan.lock().unwrap();
        let json = serde_json::to_string(&*list).unwrap_or_else(|_| "[]".to_string());
        send_json(req, 200, &json)
    })?;

    // 4. POST /api/provision
    let s_storage = Arc::clone(&storage);
    server.fn_handler("/api/provision", Method::Post, move |mut req| {
        let len = req.content_len().unwrap_or(0) as usize;
        let mut buf = vec![0u8; len];
        let _ = req.read_exact(&mut buf);

        #[derive(serde::Deserialize)]
        struct ProvReq {
            ssid: String,
            password: Option<String>,
        }

        match serde_json::from_slice::<ProvReq>(&buf) {
            Ok(p) => {
                let ssid = p.ssid.trim();
                if ssid.is_empty() {
                    return send_json(req, 400, r#"{"error":"SSID cannot be empty"}"#);
                }
                let pass = p.password.as_deref().unwrap_or("");
                let save_res = {
                    let st = s_storage.lock().unwrap();
                    st.save_wifi(ssid, pass)
                };

                match save_res {
                    Ok(()) => {
                        let res = send_json(req, 200, r#"{"status":"saved","message":"Restarting..."}"#);
                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(1500));
                            unsafe { esp_idf_svc::sys::esp_restart(); }
                        });
                        res
                    }
                    Err(e) => {
                        let err_json = format!(r#"{{"error":"{}"}}"#, e);
                        send_json(req, 500, &err_json)
                    }
                }
            }
            Err(_) => send_json(req, 400, r#"{"error":"Invalid payload"}"#),
        }
    })?;

    info!("[HTTP] Setup captive portal server active on 192.168.4.1:80");
    Ok(server)
}

fn send_file(
    req: Request<&mut EspHttpConnection>,
    content_type: &str,
    raw: &str,
    gz: &[u8],
) -> Result<(), EspIOError> {
    send_binary(req, content_type, raw.as_bytes(), gz)
}

fn send_binary(
    req: Request<&mut EspHttpConnection>,
    content_type: &str,
    raw: &[u8],
    gz: &[u8],
) -> Result<(), EspIOError> {
    let accept_encoding = req.header("Accept-Encoding").unwrap_or("");
    let use_gzip = accept_encoding.contains("gzip");
    let payload = if use_gzip { gz } else { raw };

    // IMPORTANT: Never pass a manual `Content-Length` header here because `esp-idf-svc`'s
    // `EspHttpConnection::write` always calls `httpd_resp_send_chunk` (Transfer-Encoding: chunked),
    // and `initiate_response` ignores `Content-Length`. Furthermore, always write in <= 1024-byte
    // slices so a single chunk never exceeds the ESP32-C3 lwIP TCP MSS (~1436 bytes), preventing
    // multi-packet socket stalls on weak 2.4 GHz links.
    let mut resp = req.into_response(
        200,
        Some("OK"),
        &[
            ("Content-Type", content_type),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
            ("Pragma", "no-cache"),
            ("Expires", "0"),
            ("Connection", "close"),
            if use_gzip {
                ("Content-Encoding", "gzip")
            } else {
                ("Content-Encoding", "identity")
            },
        ],
    )?;

    for chunk in payload.chunks(1024) {
        resp.write_all(chunk)?;
    }
    Ok(())
}

fn send_json(
    req: Request<&mut EspHttpConnection>,
    status: u16,
    json: &str,
) -> Result<(), EspIOError> {
    let payload = json.as_bytes();
    let mut resp = req.into_response(
        status,
        Some(if status == 200 { "OK" } else { "Error" }),
        &[
            ("Content-Type", "application/json"),
            ("Access-Control-Allow-Origin", "*"),
            ("Cache-Control", "no-cache"),
            ("Connection", "close"),
        ],
    )?;
    for chunk in payload.chunks(1024) {
        resp.write_all(chunk)?;
    }
    Ok(())
}
