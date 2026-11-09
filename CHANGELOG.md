# Changelog

All notable changes to the MultiBand 6 ESP32 Atomic Beacon project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-11-09

### Fixed
- **Persistent SNTP Daemon Lifecycle**: Fixed critical bug where `EspSntp` was dropped immediately after initial boot attempt, causing the SNTP daemon to stop and leaving the system running on an un-disciplined 1970 boot timer. Retained `SntpManager` across application lifecycle with automatic 1-hour periodic re-sync (`sntp_set_sync_interval`).
- **Boot SNTP Background Retry**: Added background retry loop polling every 15 seconds if initial boot sync does not complete immediately, recovering gracefully from Wi-Fi router / DNS delays.

### Added
- **Pre-Broadcast 2-Minute Fresh Sync**: Implemented pre-broadcast NTP synchronization window occurring 2 minutes before the scheduled transmission (e.g. at `01:53` for `01:55`). This disciplines the ESP32 RTC to sub-second precision while ensuring all 2.4 GHz Wi-Fi activity and 3.3V rail noise cease before the RF carrier starts.
- **Station-Locked Local Time & Scheduling**: Aligned local wall clock and nightly schedule triggers with the active station's native timezone offset (`station.native_utc_offset_sec() + user_offset_hours`), matching watch Home City settings.
- **Midnight Maintenance Sync**: Scheduled an automatic daily NTP refresh at midnight (`00:00`) station local time when idle.
- **Web UI "Sync Time Now" Action**: Added a manual `POST /api/sync-ntp` endpoint and web UI button to allow immediate time synchronization on demand.
- **Web UI NTP Syncing Indicator**: Displays `Syncing NTP...` on the live clocks when RTC epoch is un-synchronized (`< 1_700_000_000`) instead of misleading 1970 timestamps.

## [0.1.2] - 2026-11-08

### Fixed
- **BPC P1 Sub-Frame Indexing**: Aligned BPC 20-second sub-frame frame counter bits.
- **JJY :15/:45 Parity & DOW**: Corrected year and day-of-week bit positions on 15s and 45s markers.
- **WWVB DST Transition Bits**: Handled DST status flags.
- **50ms Boundary Lock**: Fixed tick timer alignment.

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
