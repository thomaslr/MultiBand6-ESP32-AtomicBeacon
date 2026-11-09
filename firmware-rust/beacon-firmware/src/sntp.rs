//! Network Time Protocol (NTP) Time Synchronization.

use esp_idf_svc::sntp::{EspSntp, SntpConf, SyncMode};
use log::{info, warn};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[allow(dead_code)]
pub struct SntpManager {
    _sntp: EspSntp<'static>,
    server: String,
    last_sync_epoch: Arc<AtomicU32>,
}

#[allow(dead_code)]
impl SntpManager {
    pub fn init(server: &str) -> Result<Arc<Self>, esp_idf_svc::sys::EspError> {
        info!("[SNTP] Initializing persistent time synchronization with server: '{}'", server);
        let mut conf = SntpConf::default();
        let target_server = if server.is_empty() {
            "pool.ntp.org"
        } else {
            server
        };

        conf.servers[0] = target_server;

        conf.sync_mode = SyncMode::Immediate;

        let last_sync_epoch = Arc::new(AtomicU32::new(0));
        let last_sync_cb = Arc::clone(&last_sync_epoch);

        let sntp = EspSntp::new_with_callback(&conf, move |duration| {
            let now_sec = duration.as_secs() as u32;
            last_sync_cb.store(now_sec, Ordering::Relaxed);
            info!("[SNTP] Atomic RTC synchronized to: {} UTC", now_sec);
        })?;

        // Configure lwIP SNTP background sync interval to 1 hour (3,600,000 ms)
        unsafe {
            esp_idf_svc::sys::sntp_set_sync_interval(3_600_000);
        }

        Ok(Arc::new(Self {
            _sntp: sntp,
            server: target_server.to_string(),
            last_sync_epoch,
        }))
    }

    /// Triggers an immediate re-synchronization request with the NTP server.
    pub fn request_sync(&self) {
        info!("[SNTP] Forcing immediate NTP synchronization poll...");
        unsafe {
            esp_idf_svc::sys::sntp_restart();
        }
    }

    /// Returns true if the system clock has been synchronized to a modern epoch (> 1.7B sec).
    pub fn is_synced(&self) -> bool {
        let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };
        now_sec > 1_700_000_000
    }

    /// Wait up to `timeout` for time synchronization to complete.
    pub fn sync(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.is_synced() {
                let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) as u32 };
                info!("[SNTP] RTC validated! Current epoch: {}", now_sec);
                return true;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        warn!("[SNTP] Synchronization wait timed out, RTC epoch: {}", unsafe {
            esp_idf_svc::sys::time(core::ptr::null_mut())
        });
        false
    }

    pub fn last_sync_epoch(&self) -> u32 {
        self.last_sync_epoch.load(Ordering::Relaxed)
    }

    pub fn server_name(&self) -> &str {
        &self.server
    }
}
