# Changelog

All notable changes to the MultiBand 6 ESP32 Atomic Beacon project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-11-08

### Fixed
- **Subsecond Broadcast Phase-Locking**: Synchronized RF timer phase to real-world 50 ms tick boundaries and aligned initial frame position with wall-clock time, completely eliminating the 50-second watch sync offset.
- **Network Tab Status**: Resolved JSON field serialization between backend snake_case and frontend camelCase, restoring live Wi-Fi SSID and RSSI signal quality readouts.
- **Station Dropdown Persistence**: Resolved enum string values to numerical select options so the active station (e.g. BPC) stays preserved when switching between dashboard tabs.

### Added
- **Live Stop Button Countdown**: Displayed remaining broadcast time on the active Stop button (`⏹ Stop (MM:SS)`), updating dynamically each second.

### Changed
- **Dashboard Footer Cleanup**: Removed third-party date algorithm attribution from user-facing footer.

## [0.1.0] - 2026-11-08

### Added
- **100% Pure Rust Architecture**: Complete migration to `esp-idf-svc` and `beacon-core` on `riscv32imc-esp-espidf`. All C++ and PlatformIO code removed.
- **ESP32 Dual-OTA Partition Table (MANDATORY)**: Configured 4MB flash dual OTA layout (`ota_0` and `ota_1`, 1984 KB each) with bootloader app rollback protection (`CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y`).
- **Wi-Fi Provisioning & Captive Portal Mode**: Automatic fallback to `TimeStation-Setup-XXXX` (192.168.4.1) with captive portal detection and nearby 2.4 GHz network scanner for first-time production setup.
- **Self-Healing Wi-Fi Recovery**: 10-attempt retry loop with radio cycling for WPA3-SAE stale session recovery (`0x600`), plus a 5-minute watchdog reboot in setup mode if saved credentials exist.
- **Dedicated Web Settings Page**: Added Network & Wi-Fi settings tab to the web dashboard displaying live RSSI, IP, mDNS address (`timestation.local`), network scanner, NTP server selector, and Wi-Fi reset button.
- **RF Power & Silence Hygiene**: Modem sleep enabled during standby; automatically paused during active time broadcast to maintain clean 3.3V rail.
- **Over-the-Air Updates**: Direct browser `.bin` upload via REST API (`/api/ota/upload`) and remote HTTPS updates from GitHub Releases (`/api/ota/update`).
- **Containerized CI/CD**: Pinned GitHub Actions build and release workflows with Dockerfile for reproducible builds.
