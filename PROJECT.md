# MultiBand 6 ESP32 Atomic Beacon Architecture & Engineering Guide

## 1. System Overview
The **MultiBand6-ESP32-AtomicBeacon** simulates longwave radio time signal stations (BPC 68.5 kHz, WWVB 60.0 kHz, MSF 60.0 kHz, DCF77 77.5 kHz, JJY40 40.0 kHz, JJY60 60.0 kHz) using an ESP32-C3 microcontroller. It synchronizes its real-time clock (RTC) with Stratum-1 atomic clocks over Wi-Fi via NTP, modulates an RF magnetic antenna on GPIO 1, and synchronizes atomic/radio-controlled wristwatches (such as Casio G-Shock MultiBand 6, Wave Ceptor, Citizen, and Seiko).

The project is implemented in **100% Rust** using `esp-idf-svc` and `beacon-core`.

---

## 2. Flash Partitioning & Mandatory Dual-OTA (RULE)
> [!IMPORTANT]
> **Dual-OTA Partitioning is strictly mandatory.** Never revert to a single-partition or factory-only layout.

```csv
# ESP-IDF Partition Table for 4MB Flash Dual-OTA (1984KB per slot)
# Name,   Type, SubType, Offset,   Size,     Flags
nvs,      data, nvs,     0x9000,   0x6000,
otadata,  data, ota,     0xf000,   0x2000,
phy_init, data, phy,     0x11000,  0x1000,
ota_0,    app,  ota_0,   0x20000,  0x1f0000,
ota_1,    app,  ota_1,   0x210000, 0x1f0000,
```

- Each application partition (`ota_0`, `ota_1`) has a maximum size of **1,984 KB** (2,031,616 bytes).
- **Rollback Failsafe**: `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y` ensures that if a newly flashed firmware panics or crashes, the bootloader automatically rolls back to the prior working partition.
- `ota::mark_running_app_valid()` is called upon successful startup to confirm image health and finalize the active partition.

---

## 3. Wi-Fi Architecture & Lessons Learned from Plant-ESP32

### 3.1 WPA3-SAE Stale Session Recovery
- Following abrupt restarts or OTA reboots, modern routers and mesh access points (e.g., TP-Link Deco, ASUS AiMesh) frequently reject immediate reconnection attempts (`reason: 0x600`).
- The firmware implements a 10-attempt retry loop with complete radio cycling (`wifi.disconnect()`, `wifi.stop()`, brief sleep, `wifi.start()`, `wifi.connect()`) before falling back.

### 3.2 SoftAP Provisioning & Captive Portal
- When no Wi-Fi credentials exist in NVS flash (or if connection fails), the device automatically starts SoftAP `TimeStation-Setup-XXXX` (using last 4 hex characters of the MAC address) at `192.168.4.1`.
- Captive portal detection endpoints (`/hotspot-detect.html`, `/generate_204`, `/gen_204`, `/ncsi.txt`, `/connecttest.txt`) serve `setup.html` directly to mobile devices upon connecting.
- Nearby 2.4 GHz networks are scanned via `/api/scan` and presented in a dropdown.
- Submissions to `/api/provision` write credentials to NVS and restart the device into station mode.

### 3.3 Self-Healing Recovery Timer
- When saved credentials exist in NVS, remaining in SoftAP indefinitely after a temporary router power outage would strand the device.
- A 5-minute watchdog timer automatically triggers `esp_restart()` to reconnect to the home router once it finishes rebooting.

### 3.4 RF Noise Suppression & Rail Stability
- **Standby Mode**: Wi-Fi 802.11 Modem-Sleep (`WIFI_PS_MIN_MODEM`) and CPU clock scaling to 80 MHz minimize power consumption and thermal rise.
- **Broadcast Mode**: During active LF RF carrier transmission (40.0–77.5 kHz), modem sleep is paused (`WIFI_PS_NONE`) to eliminate rail voltage ripple and RF burst hash that could interfere with wristwatch antenna reception.
- **Bandwidth**: Station radio locked to 20 MHz bandwidth (HT20).
- **TX Power**: Configurable up to 78 (~19.5 dBm max) for maximum range through walls.

---

## 4. Hardware Pin Mapping (ESP32-C3 SuperMini)

| GPIO Pin | Function | Electrical Properties | Description |
| :--- | :--- | :--- | :--- |
| **GPIO 1** | `ANTENNA` | LEDC Ch 0 PWM | RF Carrier Output (40.0–77.5 kHz) driving C1815 booster or direct LC tank |
| **GPIO 3** | `SYNC_BUTTON` | Input (Pull-Up, Active LOW) | Tactile push button SW1 / External header J3 with 50ms software debounce |
| **GPIO 8** | `STATUS_LED` | LEDC Ch 1 PWM (Active LOW) | Breathing nightstand wave during broadcast; 100% dark in standby |
| **GPIO 4** | `I2C SDA` | Open-Drain (4.7 kΩ Pull-Up) | Optional I2C Display Header J2 |
| **GPIO 5** | `I2C SCL` | Open-Drain (4.7 kΩ Pull-Up) | Optional I2C Display Header J2 |

---

## 5. Web Dashboard & REST API

### Static Endpoints (Pre-Compressed Gzip)
- `GET /`, `GET /index.html`: Main responsive dashboard (Broadcast Control, Network Settings, OTA Firmware).
- `GET /app.js`: Client-side polling, live UTC & station clock synchronization, and configuration orchestration.
- `GET /style.css`: Modern responsive UI theme with light and dark mode.
- `GET /setup`, `GET /setup.html`: Captive portal Wi-Fi setup.
- `GET /favicon.svg`: Beacon icon.

### REST API Endpoints
| Endpoint | Method | Payload | Description |
| :--- | :--- | :--- | :--- |
| `/api/status` | `GET` | None | Returns transmitter state, active station, frequencies, clocks, and Wi-Fi RSSI |
| `/api/config` | `GET` | None | Returns full active `AppConfig` JSON |
| `/api/config` | `POST` | JSON patch | Updates broadcast schedule, station, NTP, or Wi-Fi radio parameters in NVS |
| `/api/transmit` | `POST` | None | Manually initiates RF time signal transmission |
| `/api/stop` | `POST` | None | Manually terminates active RF transmission |
| `/api/scan` | `GET` | None | Returns JSON array of discovered 2.4 GHz network SSIDs |
| `/api/provision` | `POST` | `{"ssid":"...","password":"..."}` | Saves Wi-Fi credentials to NVS and initiates reboot |
| `/api/wifi/reset` | `POST` | None | Erases saved Wi-Fi credentials from NVS and reboots into setup mode |
| `/api/system/reboot`| `POST` | None | Restarts the ESP32 microcontroller |
| `/api/ota/status` | `GET` | None | Reports running slot, standby slot, firmware version, and rollback state |
| `/api/ota/update` | `POST` | `{"url":"..."}` | Initiates background HTTPS OTA download from GitHub Releases |
| `/api/ota/upload` | `POST` | Binary `.bin` | Direct browser-based firmware binary upload |

---

## 6. Continuous Integration & Docker Builds
- **Docker Image**: `.github/ci/Dockerfile` packages Ubuntu 24.04 with pinned `nightly-2026-04-01`, `rust-src`, `ldproxy`, and `espflash`.
- **Image Pipeline**: `.github/workflows/build-ci-image.yml` publishes the builder container to GHCR.
- **Firmware Pipeline**: `.github/workflows/build-and-release.yml` compiles release binaries, verifies size against the 1984 KB OTA limit, generates `version.json`, and attaches release assets to GitHub Releases upon pushing `v*` tags.
