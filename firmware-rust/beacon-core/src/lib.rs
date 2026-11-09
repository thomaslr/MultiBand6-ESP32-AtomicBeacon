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
}

