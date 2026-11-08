//! Web Server and REST API Implementation.

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
use crate::ota::{flash_stream, get_ota_status, update_from_url};
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

pub struct AppState {
    pub config: AppConfig,
    pub storage: NvsConfigStorage,
    pub carrier: CarrierManager,
    pub ip_address: String,
    pub wifi_ssid: String,
    pub wifi_rssi: i32,
    pub scanned_ssids: Vec<String>,
}

#[derive(serde::Serialize)]
struct StatusResponse {
    transmitting: bool,
    active_station: u8,
    station_name: &'static str,
    carrier_freq: u32,
    offset_hours: i32,
    epoch_sec: u64,
    uptime_sec: u32,
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
}

pub fn create_web_server(state: Arc<Mutex<AppState>>) -> Result<EspHttpServer<'static>, EspIOError> {
    let server_config = ServerConfig {
        stack_size: 10240,
        max_open_sockets: 6,
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

        let resp = StatusResponse {
            transmitting: is_tx,
            active_station: station as u8,
            station_name: station.station_name(),
            carrier_freq: freq,
            offset_hours: *c_state.user_offset_hours.lock().unwrap(),
            epoch_sec: now_sec as u64,
            uptime_sec,
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
                app.config.apply_patch(patch);
                let _ = app.storage.save_config(&app.config);
                apply_wifi_radio_config(&app.config.wifi);
                send_json(req, 200, r#"{"status":"saved"}"#)
            }
            Err(e) => {
                let err = format!(r#"{{"error":"Failed to parse config: {}"}}"#, e);
                send_json(req, 400, &err)
            }
        }
    })?;

    // 5. POST /api/transmit
    let s_tx = Arc::clone(&state);
    server.fn_handler("/api/transmit", Method::Post, move |req| {
        let app = s_tx.lock().unwrap();
        app.carrier.start(&app.config, None);
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
    server.fn_handler("/api/ota/status", Method::Get, |req| {
        let status = get_ota_status();
        let json = serde_json::to_string(&status).unwrap_or_default();
        send_json(req, 200, &json)
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
        max_open_sockets: 6,
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

    let mut resp = req.into_response(
        200,
        Some("OK"),
        &[
            ("Content-Type", content_type),
            ("Cache-Control", "public, max-age=3600"),
            ("Connection", "close"),
            if use_gzip {
                ("Content-Encoding", "gzip")
            } else {
                ("Content-Encoding", "identity")
            },
        ],
    )?;

    if use_gzip {
        for chunk in gz.chunks(2048) {
            let _ = resp.write_all(chunk);
        }
    } else {
        for chunk in raw.chunks(2048) {
            let _ = resp.write_all(chunk);
        }
    }
    Ok(())
}

fn send_json(
    req: Request<&mut EspHttpConnection>,
    status: u16,
    json: &str,
) -> Result<(), EspIOError> {
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
    for chunk in json.as_bytes().chunks(2048) {
        let _ = resp.write_all(chunk);
    }
    Ok(())
}
