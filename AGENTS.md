# MultiBand6-ESP32-AtomicBeacon Agent Instructions

## 1. ESP32 Partitioning & Dual-OTA Updates (PERMANENT RULE)
- **CRITICAL**: The ESP32 partition table MUST ALWAYS be configured for **Dual-OTA** (`ota_0` and `ota_1`, 1984 KB each on 4MB flash).
- **NEVER** revert to a single-partition or factory-only layout.
- The partition table is defined in `firmware-rust/beacon-firmware/partitions.csv` and referenced in `.cargo/config.toml` and GitHub Actions.
- Bootloader failsafe rollback (`CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y`) must always be enabled.

## 2. Language & Toolchain
- **Language**: 100% Rust only. All C++ code (Arduino, PlatformIO, etc.) has been eliminated.
- **Framework**: `esp-idf-svc` (v0.53+) on `riscv32imc-esp-espidf` target with pinned `nightly-2026-04-01`.
- **Linker / Tools**: `ldproxy` and `espflash`.

## 3. Wi-Fi, Power & RF Stability Principles
- **WPA3-SAE Stale Session Recovery**: Handle router stale leases / auth rejections (`reason: 0x600`) with a multi-attempt retry loop and full radio cycling before falling back to SoftAP.
- **SoftAP Provisioning & Captive Portal**: If no credentials exist or router connection fails, start `TimeStation-Setup-XXXX` (192.168.4.1) serving a mobile-friendly captive portal for network configuration.
- **Self-Healing Recovery Timer**: If credentials exist in NVS but connection fails, reboot every 5 minutes in SoftAP mode to reconnect when the router recovers.
- **Power & RF Hygiene**:
  - Wi-Fi modem sleep enabled during standby.
  - Wi-Fi modem sleep paused during active RF time broadcast to maintain clean 3.3V rail and eliminate 2.4 GHz burst jitter.
  - CPU clocked at 80 MHz for 50% dynamic power reduction.
  - Antenna carrier generated via hardware LEDC PWM on GPIO 1.
  - mDNS responder active at `http://timestation.local`.

## 4. Web Interface, HTTP Streaming & Multi-Client Concurrency (PERMANENT RULE)
- Embedded web assets in `data/` (`index.html`, `setup.html`, `app.js`, `style.css`, `favicon.svg`) pre-compressed with Gzip at build time.
- **Chunked HTTP Streaming (`send_binary` / `send_json`)**:
  - **CRITICAL**: Always stream HTTP response bodies in **$\le$ 1024-byte chunks** (`for chunk in payload.chunks(1024) { resp.write_all(chunk)?; }`). Never pass a multi-kilobyte slice (such as `index.html.gz` or `app.js.gz`) to `resp.write_all()` in a single call, as `httpd_resp_send_chunk` over a single ESP32-C3 lwIP TCP MSS (~1436 bytes) will stall after the first TCP segment (~1307 bytes) and block the single-threaded `httpd` worker until timeout.
  - **NEVER** pass a manual `("Content-Length", ...)` header to `req.into_response(...)` in `esp-idf-svc`, because `EspHttpConnection::write` always uses `Transfer-Encoding: chunked`.
- **Multi-Client Socket Hygiene**:
  - Always configure `EspHttpServer` with `lru_purge_enable: true`, `session_timeout: Duration::from_secs(10)`, `so_linger: Some(Duration::from_secs(0))`, and `max_open_sockets: 12` (with `CONFIG_LWIP_MAX_SOCKETS=18` in `sdkconfig.defaults`) combined with `Connection: close` response headers so multiple phones/browsers accessing the device simultaneously never exhaust sockets or hold `TIME_WAIT` connections open.
- Dedicated Network & Wi-Fi settings page with live signal quality, nearby Wi-Fi scanner, NTP configuration, and credentials reset button.
- Captive portal detection endpoints (`/hotspot-detect.html`, `/generate_204`, etc.) route directly to the setup portal.

## 5. SNTP Time Synchronization & Station-Locked Scheduling (PERMANENT RULE)
- **Lifecycle & Persistence**: `EspSntp` / `SntpManager` MUST be retained in memory across the entire application lifetime so lwIP's background SNTP service stays active with automatic hourly re-synchronization (`sntp_set_sync_interval(3_600_000)`).
- **Background Boot Retry**: If initial boot synchronization does not complete immediately, continuously retry in the background every 15 seconds until an atomic epoch (`> 1_700_000_000`) is established.
- **Pre-Broadcast Fresh Sync**: Always trigger an immediate NTP sync 2 minutes prior to any scheduled broadcast (e.g. at `01:53` for an `01:55` broadcast window) so the ESP32 RTC drift is eliminated and the clock is disciplined to sub-second precision before the RF carrier starts.
- **Silent Transmit Window**: Never poll or step NTP during active RF broadcast to eliminate phase jumps and prevent 2.4 GHz supply ripple on the 3.3V rail.
- **Station-Locked Local Time (Option B)**: Local time and nightly schedules always track the active time signal station's native timezone offset (`station.native_utc_offset_sec() + user_offset_hours`), ensuring watches configured for that station's region receive the signal during their expected nighttime auto-receive window.
