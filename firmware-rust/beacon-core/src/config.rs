// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

//! Configuration models for TimeStation MultiBand 6 Beacon.

use serde::{Deserialize, Serialize};
use crate::protocols::TimeStation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemConfig {
    pub hostname: String,
    pub ntp_server: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            hostname: "timestation".to_string(),
            ntp_server: "pool.ntp.org".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WifiConfig {
    /// Wi-Fi Modem Sleep (false = WIFI_PS_NONE, true = WIFI_PS_MIN_MODEM)
    pub modem_sleep: bool,
    /// Transmit power level in quarter-dBm: 78 (19.5 dBm max), 68 (17 dBm), etc.
    pub tx_power: i8,
    /// Force 20 MHz (HT20) mode to reduce RF noise
    pub force_20mhz: bool,
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            modem_sleep: true,
            tx_power: 78,
            force_20mhz: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub station: TimeStation,
    pub offset_hours: i32,
    pub schedule_enabled: bool,
    pub broadcast_hour: u8,
    pub broadcast_minute: u8,
    pub broadcast_duration_min: u32,
    pub carousel_enabled: bool,
    pub carousel_mask: u8,
    pub carousel_duration_min: u32,
    pub system: SystemConfig,
    pub wifi: WifiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            station: TimeStation::Bpc,
            offset_hours: 0,
            schedule_enabled: true,
            broadcast_hour: 2,
            broadcast_minute: 0,
            broadcast_duration_min: 25,
            carousel_enabled: false,
            carousel_mask: 0x23, // BPC (0), WWVB (1), JJY60 (5)
            carousel_duration_min: 15,
            system: SystemConfig::default(),
            wifi: WifiConfig::default(),
        }
    }
}

/// Partial patch payload from REST API (`POST /api/config`)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigPatch {
    pub station: Option<u8>,
    pub offset_hours: Option<i32>,
    pub schedule_enabled: Option<bool>,
    pub broadcast_time: Option<String>,
    pub broadcast_duration_min: Option<u32>,
    pub carousel_enabled: Option<bool>,
    pub carousel_mask: Option<u8>,
    pub carousel_duration_min: Option<u32>,
    pub system: Option<SystemPatch>,
    pub wifi: Option<WifiPatch>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SystemPatch {
    pub hostname: Option<String>,
    pub ntp_server: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WifiPatch {
    pub modem_sleep: Option<bool>,
    pub tx_power: Option<i8>,
    pub force_20mhz: Option<bool>,
}

impl AppConfig {
    pub fn apply_patch(&mut self, patch: ConfigPatch) {
        if let Some(st_val) = patch.station {
            if let Some(st) = TimeStation::from_u8(st_val) {
                self.station = st;
            }
        }
        if let Some(off) = patch.offset_hours {
            if (-12..=12).contains(&off) {
                self.offset_hours = off;
            }
        }
        if let Some(sched) = patch.schedule_enabled {
            self.schedule_enabled = sched;
        }
        if let Some(t_str) = patch.broadcast_time {
            let parts: Vec<&str> = t_str.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(h), Ok(m)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                    if h < 24 && m < 60 {
                        self.broadcast_hour = h;
                        self.broadcast_minute = m;
                    }
                }
            }
        }
        if let Some(dur) = patch.broadcast_duration_min {
            if dur >= 1 {
                self.broadcast_duration_min = dur;
            }
        }
        if let Some(car_en) = patch.carousel_enabled {
            self.carousel_enabled = car_en;
        }
        if let Some(mask) = patch.carousel_mask {
            self.carousel_mask = mask;
        }
        if let Some(car_dur) = patch.carousel_duration_min {
            if car_dur >= 1 {
                self.carousel_duration_min = car_dur;
            }
        }
        if let Some(sys) = patch.system {
            if let Some(h) = sys.hostname {
                let trimmed = h.trim();
                if !trimmed.is_empty() {
                    self.system.hostname = trimmed.to_string();
                }
            }
            if let Some(ntp) = sys.ntp_server {
                let trimmed = ntp.trim();
                if !trimmed.is_empty() {
                    self.system.ntp_server = trimmed.to_string();
                }
            }
        }
        if let Some(wifi) = patch.wifi {
            if let Some(ms) = wifi.modem_sleep {
                self.wifi.modem_sleep = ms;
            }
            if let Some(tx) = wifi.tx_power {
                self.wifi.tx_power = tx;
            }
            if let Some(ht20) = wifi.force_20mhz {
                self.wifi.force_20mhz = ht20;
            }
        }
    }
}
