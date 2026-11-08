//! Over-The-Air (OTA) Firmware Update Service.
//!
//! Supports:
//! 1. Remote GitHub Releases HTTPS streaming download with automatic redirect following.
//! 2. Direct browser binary upload via REST API.
//! 3. Automatic rollback cancellation (`mark_running_app_valid`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use embedded_svc::http::Method;
use embedded_svc::io::Read;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection, FollowRedirectsPolicy};
use esp_idf_svc::ota::EspOta;
use log::{info, warn};

pub const DEFAULT_GITHUB_OTA_URL: &str =
    "https://github.com/thomaslr/MultiBand6-ESP32-AtomicBeacon/releases/latest/download/multiband6-beacon.bin";

static OTA_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
pub fn is_ota_in_progress() -> bool {
    OTA_IN_PROGRESS.load(Ordering::Relaxed)
}

struct OtaGuard;
impl Drop for OtaGuard {
    fn drop(&mut self) {
        OTA_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct OtaStatus {
    pub running_slot: String,
    pub next_update_slot: String,
    pub firmware_version: &'static str,
    pub default_url: &'static str,
    pub rollback_enabled: bool,
}

pub fn mark_running_app_valid() {
    match EspOta::new() {
        Ok(mut ota) => match ota.mark_running_slot_valid() {
            Ok(_) => {
                info!("[OTA] Boot verification confirmed: running slot marked valid.");
            }
            Err(e) => {
                info!("[OTA] Running slot status (already valid or factory): {:?}", e);
            }
        },
        Err(e) => {
            warn!("[OTA] Unable to access OTA service: {:?}", e);
        }
    }
}

pub fn get_ota_status() -> OtaStatus {
    let mut running_slot = "factory".to_string();
    let mut next_update_slot = "none".to_string();

    if let Ok(ota) = EspOta::new() {
        if let Ok(slot) = ota.get_running_slot() {
            running_slot = slot.label.as_str().to_string();
        }
        if let Ok(slot) = ota.get_update_slot() {
            next_update_slot = slot.label.as_str().to_string();
        }
    }

    OtaStatus {
        running_slot,
        next_update_slot,
        firmware_version: env!("CARGO_PKG_VERSION"),
        default_url: DEFAULT_GITHUB_OTA_URL,
        rollback_enabled: true,
    }
}

pub fn update_from_url(url: &str) -> Result<(), String> {
    info!("[OTA] Starting OTA download from: {}", url);
    OTA_IN_PROGRESS.store(true, Ordering::SeqCst);
    let _guard = OtaGuard;

    let mut ota = EspOta::new().map_err(|e| format!("Failed to initialize OTA: {:?}", e))?;
    let mut update = ota
        .initiate_update()
        .map_err(|e| format!("Failed to initiate OTA update: {:?}", e))?;

    let config = HttpConfig {
        follow_redirects_policy: FollowRedirectsPolicy::FollowGetHead,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        buffer_size: Some(8192),
        buffer_size_tx: Some(4096),
        timeout: Some(Duration::from_secs(60)),
        ..Default::default()
    };

    let mut conn = EspHttpConnection::new(&config)
        .map_err(|e| format!("Failed to create HTTP client: {:?}", e))?;

    conn.initiate_request(Method::Get, url, &[("User-Agent", "multiband6-beacon-ota")])
        .map_err(|e| format!("Failed to initiate HTTP GET: {:?}", e))?;

    conn.initiate_response()
        .map_err(|e| format!("Failed to get HTTP response: {:?}", e))?;

    let status = conn.status();
    if !(200..300).contains(&status) {
        let _ = update.abort();
        return Err(format!("Server returned HTTP {}", status));
    }

    let mut buf = vec![0u8; 4096];
    let mut total_bytes = 0usize;
    let mut last_log_bytes = 0usize;

    loop {
        match conn.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                update
                    .write(&buf[..n])
                    .map_err(|e| format!("Failed writing flash chunk: {:?}", e))?;
                total_bytes += n;
                if total_bytes - last_log_bytes >= 102400 {
                    info!("[OTA] Downloaded and flashed {} KB...", total_bytes / 1024);
                    last_log_bytes = total_bytes;
                }
            }
            Err(e) => {
                let _ = update.abort();
                return Err(format!("Download error after {} bytes: {:?}", total_bytes, e));
            }
        }
    }

    if total_bytes < 200_000 {
        let _ = update.abort();
        return Err(format!(
            "Downloaded file size ({} bytes) too small to be valid firmware",
            total_bytes
        ));
    }

    info!("[OTA] Download complete ({} bytes). Finalizing flash partition...", total_bytes);

    update
        .complete()
        .map_err(|e| format!("Failed to complete OTA: {:?}", e))?;

    info!("[OTA] Firmware written successfully. Restarting in 2 seconds...");
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(2000));
        unsafe {
            esp_idf_svc::sys::esp_restart();
        }
    });

    Ok(())
}

pub fn flash_stream<R: Read>(mut reader: R, expected_len: usize) -> Result<(), String> {
    info!("[OTA] Flashing firmware from stream (expected: {} bytes)...", expected_len);
    OTA_IN_PROGRESS.store(true, Ordering::SeqCst);
    let _guard = OtaGuard;

    let mut ota = EspOta::new().map_err(|e| format!("Failed to initialize OTA: {:?}", e))?;
    let mut update = ota
        .initiate_update()
        .map_err(|e| format!("Failed to initiate OTA update: {:?}", e))?;

    let mut buf = [0u8; 2048];
    let mut total_bytes = 0usize;
    let mut last_log_bytes = 0usize;

    loop {
        let to_read = if expected_len > 0 {
            buf.len().min(expected_len.saturating_sub(total_bytes))
        } else {
            buf.len()
        };
        if expected_len > 0 && to_read == 0 {
            break;
        }

        match reader.read(&mut buf[..to_read]) {
            Ok(0) => break,
            Ok(n) => {
                update
                    .write(&buf[..n])
                    .map_err(|e| format!("Failed writing flash: {:?}", e))?;
                total_bytes += n;
                if total_bytes - last_log_bytes >= 204800 {
                    info!("[OTA] Uploaded and flashed {} KB...", total_bytes / 1024);
                    last_log_bytes = total_bytes;
                }
                if expected_len > 0 && total_bytes >= expected_len {
                    break;
                }
            }
            Err(e) => {
                let _ = update.abort();
                return Err(format!("Stream read error: {:?}", e));
            }
        }
    }

    if total_bytes < 200_000 {
        let _ = update.abort();
        return Err(format!(
            "Uploaded file ({} bytes) too small to be valid firmware",
            total_bytes
        ));
    }

    update
        .complete()
        .map_err(|e| format!("Failed to complete OTA: {:?}", e))?;

    info!("[OTA] Upload complete ({} bytes). Restarting in 2 seconds...", total_bytes);
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(2000));
        unsafe {
            esp_idf_svc::sys::esp_restart();
        }
    });

    Ok(())
}
