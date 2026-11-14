// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

//! MultiBand 6 ESP32 Beacon Core Library
//!
//! Provides atomic clock radio protocols (BPC, WWVB, MSF, DCF77, JJY),
//! date/time calendar conversions, and typed configuration schemas.

pub mod config;
pub mod datetime;
pub mod protocols;

/// Parse semver strings such as "v0.1.4" or "0.1.5" into a numeric tuple (major, minor, patch).
pub fn parse_semver(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let mut parts = clean.split('.').filter_map(|s| s.parse::<u32>().ok());
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

/// Compare two semver strings, returning true if `remote` is strictly newer than `local`.
pub fn is_newer_version(remote: &str, local: &str) -> bool {
    parse_semver(remote) > parse_semver(local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parsing_and_comparison() {
        assert_eq!(parse_semver("v0.1.4"), (0, 1, 4));
        assert_eq!(parse_semver("0.1.5"), (0, 1, 5));
        assert_eq!(parse_semver("V1.2.3"), (1, 2, 3));
        assert!(is_newer_version("v0.1.5", "0.1.4"));
        assert!(is_newer_version("v1.0.0", "v0.9.9"));
        assert!(!is_newer_version("v0.1.4", "v0.1.4"));
        assert!(!is_newer_version("v0.1.3", "v0.1.4"));
    }

    #[test]
    fn test_default_wifi_config_and_nvs_serialization() {
        let def = config::AppConfig::default();
        assert_eq!(def.wifi.tx_power, 78);
        assert!(def.wifi.modem_sleep);
        assert!(def.wifi.force_20mhz);

        // Verify NVS JSON round-trip preserves user overrides
        let mut custom = def.clone();
        custom.wifi.modem_sleep = false;
        custom.wifi.tx_power = 68;
        let json = serde_json::to_string(&custom).unwrap();
        let loaded: config::AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, custom);

        // Verify partial/older NVS JSON missing wifi fields falls back to new defaults
        let partial: config::AppConfig = serde_json::from_str(r#"{"station":"Wwvb"}"#).unwrap();
        assert_eq!(partial.station, protocols::TimeStation::Wwvb);
        assert_eq!(partial.wifi.tx_power, 78);
        assert!(partial.wifi.modem_sleep);
        assert!(partial.wifi.force_20mhz);
    }
}

