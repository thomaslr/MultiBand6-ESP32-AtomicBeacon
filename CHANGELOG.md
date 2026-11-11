# Changelog

All notable changes to the MultiBand 6 ESP32 Atomic Beacon project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.15] - 2026-11-11

### Changed
- **Cardiac "Thump-Thump" 10-Second Wi-Fi Heartbeat (`led.rs`)**: Updated the 10-second Wi-Fi connected standby indicator to a classic double-pulse heartbeat pattern (`100 ms` ON, `120 ms` OFF, `100 ms` ON) at 50% brightness (`128 / 256` PWM duty), clearly distinguishing it from the single 2-second pulse used when Wi-Fi is lost.

## [0.1.14] - 2026-11-11

### Added
- **Onboard Status LED Wi-Fi Heartbeat & Link-Loss Indicators (`led.rs`, `main.rs`)**:
  - **Wi-Fi Connected Heartbeat**: Pulses once every 10 seconds (`120 ms` ON) at 50% brightness (`128 / 256` PWM duty) during standby once connected to Wi-Fi.
  - **Wi-Fi Lost Indicator**: Pulses once every 2 seconds (`120 ms` ON) at 50% brightness (`128 / 256` PWM duty) whenever the Wi-Fi station link is lost (or while waiting in SoftAP provisioning mode), with 1-second RSSI link detection.

## [0.1.13] - 2026-11-11

### Changed
- **Default Wi-Fi & RF Radio Profile (`beacon-core/src/config.rs`, `data/index.html`)**: Set default installation radio settings (used when NVS flash has no stored config) to `+19.5 dBm` (`tx_power: 78`), `Modem Sleep: Off` (`modem_sleep: false`), and `20 MHz Bandwidth (HT20): On` (`force_20mhz: true`) for maximum web UI responsiveness and zero DTIM wake latency on the ESP32-C3 SuperMini ceramic antenna while preserving existing user settings stored in NVS across reboots and OTA updates.
- **Live Radio Config Preservation on Broadcast Stop (`main.rs`, `carrier.rs`)**: When a scheduled or bedside-button broadcast finishes, the radio now reads the live `app_state` NVS-backed `modem_sleep` setting rather than the initial boot snapshot.

## [0.1.12] - 2026-11-11

### Fixed
- **Single-Write HTTP Asset Delivery & Content-Length Headers (`web.rs`)**: Replaced 2048-byte HTTP chunked loops in `send_binary` and `send_json` (which silently ignored socket write errors and truncated `app.js` / `style.css` under Wi-Fi Modem Sleep or marginal signal conditions) with single-write `Content-Length` responses and increased `max_open_sockets` (8) alongside `CONFIG_LWIP_MAX_SOCKETS=16`.
- **Non-Blocking Dashboard Startup & Timeout Resilience (`data/app.js`, `data/index.html`, `data/sw.js`)**:
  - Replaced `window.onload` initialization with immediate `DOMContentLoaded` startup and an inline `switchTab()` fallback so navigation tabs and live clocks initialize immediately without waiting for slow sub-resources.
  - Added `AbortController` timeouts (6s) to all dashboard API fetches (`/api/status`, `/api/config`, `/api/ota/status`, `/api/logs`) so a stalled socket in a Wi-Fi dead zone never freezes the adaptive polling loop.
  - Defined missing `formatDuration()` helper and `appStatus` reference in `data/app.js` that previously threw a `ReferenceError` when opening the Diagnostics tab.
  - Removed the Service Worker (`sw.js`) pass-through `fetch` interceptor that returned `undefined` on uncached navigation failures and doubled request pressure on the single-threaded ESP32 HTTP server.
- **Automatic Wi-Fi Dead-Zone Reconnection (`main.rs`)**: Added periodic station link monitoring in the main operational loop to automatically trigger `esp_wifi_connect()` when the ESP32-C3 returns from a Wi-Fi dead zone while in standby.

## [0.1.11] - 2026-11-10

### Fixed
- **Dashboard UI Syntax Error in `data/app.js`**: Removed duplicate `activeSt` and `st` variable declarations inside `updateStatusDisplay` that caused browser runtime parsing failure (`SyntaxError: Identifier 'activeSt' has already been declared`), restoring unresponsive navigation tabs, clock updates, and live dashboard metrics.

## [0.1.10] - 2026-11-10

### Added
- **Physical Sync Button LED Acknowledgment Flash Patterns**:
  - **Start ACK**: 3 crisp rapid blips (100ms ON / 100ms OFF x 3 = 600ms) on the onboard LED (GPIO 8) confirming the physical bedside button press, transitioning seamlessly into the calm nightstand breathing wave (4.0s sine pulse). Distinct from automated/web syncs which start breathing directly without initial blips.
  - **Stop ACK**: 2 deliberate blinks (200ms ON / 150ms OFF x 2 = 700ms) confirming transmission cancellation before entering completely dark standby (0V across LED, zero glow).
  - **Error ACK**: 5 fast strobe bursts (50ms ON / 50ms OFF x 5 = 500ms) indicating an un-synchronized RTC clock (e.g. before initial NTP lock), preventing invalid carrier generation.
- **Webpage Bedside Sync Feedback & Adaptive Polling**:
  - Live status badge displays `TRANSMITTING (Bedside Button)` when triggered physically.
  - High-visibility notification banner on dashboard: `⚡ Bedside Manual Sync: Broadcasting [Station] for 15 minutes.`
  - Adaptive status polling: automatically accelerates to 5s during active broadcast (and 15s in standby) instead of static 60s intervals.
  - Diagnostics tab displays real-time contact states for `Bedside Sync (GPIO 3)` and `BOOT Pin (GPIO 9)`.

### Fixed
- **Instant Button Responsiveness**: Reduced debouncing loop sample period from 20ms to 10ms and lowered filter threshold to 2 consecutive samples (20ms contact time), ensuring light or quick finger taps register reliably. Added contact transition diagnostics to live system log stream.

## [0.1.9] - 2026-11-10

### Fixed
- **Physical Sync Button Debounce & RTCIO Configuration**: Fixed bedside tactile pushbutton ($SW_1$ on GPIO 3) being ignored due to contact bounce chatter causing an instant start/abort cycle in under $40\text{ ms}$. Implemented a 3-sample ($60\text{ ms}$) digital integrator with a $500\text{ ms}$ action cooldown and explicit `gpio_config` initialization to cleanly disconnect RTC/analog functions from GPIO 3.
- **15-Minute Current Station Broadcast on Button Press**: Pressing the physical sync button now explicitly overrides carousel settings to broadcast the currently selected radio station for a dedicated 15-minute window with live web countdown, and cleanly stops transmission if pressed again while broadcasting. Added dual-mapping for GPIO 9 (ESP32-C3 onboard BOOT button) as a bench-testing convenience.

## [0.1.8] - 2026-11-10

### Changed
- **Extended Broadcast Duration Limit**: Increased maximum scheduled broadcast duration from 180 minutes to 240 minutes (4 hours) in the web UI, allowing extended overnight synchronization sessions for distant or stubborn timepieces.

## [0.1.7] - 2026-11-09

### Fixed
- **Dashboard UI Syntax Error**: Removed a stray unclosed code block in `data/app.js` that caused browser parsing to fail (`SyntaxError: Unexpected token '}'`), resolving unresponsive navigation tabs and uninitialized clock displays.

## [0.1.6] - 2026-11-09

### Added
- **Dual Binary Packaging (Factory 0x0 & OTA)**: Release pipeline now generates both `multiband6-beacon-factory.bin` (compact merged image with bootloader and Dual-OTA table for first-time flashing at `0x0`) and `multiband6-beacon.bin` (application payload for over-the-air updates).
- **1-Click Web Installer**: Hosted on GitHub Pages via ESP Web Tools for zero-software-install browser flashing and direct Wi-Fi configuration over WebSerial.
- **Live Diagnostics Tab**: Web UI now features a real-time Hardware Telemetry dashboard (free heap, minimum free heap watermark, Wi-Fi RSSI) and an in-memory live system log terminal with auto-scroll, copy, and clear functions.
- **Universal Radio Watch Compatibility Guide (`CASIO_GUIDE.md`)**: Detailed breakdown of regional radio-lock behavior, compatible Home City assignments (`HKG`, `TYO`, `BER`, `NYC`, `LON`), and manual sync operations across Casio MultiBand 6, Citizen Eco-Drive, Seiko, Timex, and Junghans.
- **3D Printable Multi-Mount Enclosure & Table Stand (`enclosure/`)**: Parametric OpenSCAD models and print-ready STLs with M3 screw ears, dual zip-tie channels, 3M Command tape pad, and a 45° angled bedside nightstand cradle.
- **Dedicated Quick Start Guide (`QUICKSTART.md`)**: 5-minute setup documentation for new users and makers.

## [0.1.5] - 2026-11-09

### Added
- **ESP32 Firmware-Driven Update Checks**: Shifted OTA update checking from client-side browser JavaScript onto the ESP32 hardware:
  - **Post-Boot Scan**: Runs ~15 seconds after booting once Wi-Fi and SNTP time synchronization are established.
  - **Midnight Daily Scan**: Runs automatically at `00:05` station local time (5 minutes after the midnight SNTP maintenance sync).
  - **Lightweight `version.json` Stream**: ESP32 downloads only the tiny ~180-byte `version.json` over HTTPS with hardware mbedTLS root certificate validation.
  - **Instant Dashboard Rendering**: Cached update state in `OtaStatus` renders immediately in the web UI with zero latency and eliminates browser-side CORS or GitHub API rate-limiting delays.
  - **On-Demand API Trigger**: Added `POST /api/ota/check` so the web dashboard "Check for Updates" button triggers an immediate check on the ESP32.
- **Cache-Control Modernization**: Changed static asset delivery in the web server to `no-cache, no-store, must-revalidate`, guaranteeing browsers never execute stale JavaScript after an OTA firmware upgrade.
- **Manual Flashing Guarantee**: Strict separation of scanning from flashing; automatic routines only query version metadata, and firmware binary downloads/flashing remain 100% manual opt-in.

## [0.1.4] - 2026-11-09

### Added
- **OTA Release Notes Inspection Modal**: Firmware update button now fetches and displays full release notes and changelog in a formatted review popover before asking the user to confirm and apply.
- **OTA Update Available Status Badge**: Added an inline status badge next to the OTA update button reporting `✨ Update Available: vX.Y.Z` or `✓ Up to date` with human-readable check timestamps.
- **Daily Automated Update Check**: System checks GitHub Releases API once every 24 hours with local caching to prevent rate-limiting and unnecessary network overhead, plus an on-demand "Check for Updates" trigger.
- **Broadcasting SNTP Suspension**: Added hardware-level suspension (`esp_sntp_stop()`) during active RF transmission to eliminate 2.4 GHz bursts on the 3.3V rail, automatically resuming (`esp_sntp_init()`) when broadcast stops.
- **1-Minute Lightweight Dashboard Polling**: Reduced status polling from 4 seconds to 60 seconds (93.3% reduction in HTTP traffic), paired with an immediate refresh on tab visibility changes and client-side 1-second clock ticking.

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
