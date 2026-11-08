//! Network Time Protocol (NTP) Time Synchronization.

use esp_idf_svc::sntp::{EspSntp, SntpConf, SyncStatus};
use log::{info, warn};
use std::time::Duration;

pub struct SntpManager {
    _sntp: EspSntp<'static>,
}

impl SntpManager {
    pub fn init(server: &str) -> Result<Self, esp_idf_svc::sys::EspError> {
        info!("[SNTP] Initializing time synchronization with server: '{}'", server);
        let mut conf = SntpConf::default();
        if !server.is_empty() {
            conf.servers[0] = server;
        }
        let sntp = EspSntp::new(&conf)?;
        Ok(Self { _sntp: sntp })
    }

    pub fn sync(&self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self._sntp.get_sync_status() == SyncStatus::Completed {
                let now_sec = unsafe { esp_idf_svc::sys::time(core::ptr::null_mut()) };
                if now_sec > 1_700_000_000 {
                    info!("[SNTP] RTC synchronized to atomic time reference! Current epoch: {}", now_sec);
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        warn!("[SNTP] Synchronization timed out, proceeding with current RTC.");
        false
    }
}
