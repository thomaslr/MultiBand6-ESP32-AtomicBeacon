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

## 4. Web Interface & Provisioning
- Embedded web assets in `data/` (`index.html`, `setup.html`, `app.js`, `style.css`, `favicon.svg`) pre-compressed with Gzip at build time.
- Dedicated Network & Wi-Fi settings page with live signal quality, nearby Wi-Fi scanner, NTP configuration, and credentials reset button.
- Captive portal detection endpoints (`/hotspot-detect.html`, `/generate_204`, etc.) route directly to the setup portal.
