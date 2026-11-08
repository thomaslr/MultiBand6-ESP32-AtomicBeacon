# Permanent Rule: ESP32 Dual-OTA Partitioning Required

## MANDATORY PARTITION TABLE SPECIFICATION
**NEVER EVER REVERT TO A SINGLE PARTITION.**
This project MUST ALWAYS utilize a Dual-OTA partition table for the 4MB SPI Flash on the ESP32-C3 to support Over-The-Air firmware updates with failsafe rollback:

```csv
# ESP-IDF Partition Table for 4MB Flash Dual-OTA (1984KB per slot)
# Name,   Type, SubType, Offset,   Size,     Flags
nvs,      data, nvs,     0x9000,   0x6000,
otadata,  data, ota,     0xf000,   0x2000,
phy_init, data, phy,     0x11000,  0x1000,
ota_0,    app,  ota_0,   0x20000,  0x1f0000,
ota_1,    app,  ota_1,   0x210000, 0x1f0000,
```

### Critical Rules:
1. **Never use single partition / factory-only layout**: The partition table MUST define `otadata`, `ota_0`, and `ota_1`.
2. **App Partition Size Limit**: Each OTA app slot is `0x1f0000` (1984 KB = 2,031,616 bytes). Firmware binaries (`multiband6-beacon.bin`) MUST NOT exceed 1,984 KB.
3. **Rollback Support**: `CONFIG_BOOTLOADER_APP_ROLLBACK_ENABLE=y` must remain enabled in `sdkconfig.defaults`. Every newly booted image must call `ota::mark_running_app_valid()` once initialization succeeds to confirm health and cancel automatic rollback.
4. **Rust Only**: This project is strictly 100% Rust (`esp-idf-svc` / `riscv32imc-esp-espidf`). Do not introduce C++ or PlatformIO files.
5. **Wi-Fi Lessons from Plant-ESP32**:
   - WPA3-SAE retry loop with radio cycling (up to 10 attempts, handling `0x600` stale auth rejection).
   - Modem Sleep (`WIFI_PS_MIN_MODEM`) in standby; paused during active RF broadcast for clean power rail.
   - Max TX power boosted to 78 (~19.5 dBm) for maximum reach.
   - 20 MHz bandwidth (HT20) mode.
   - SoftAP Captive Portal (`TimeStation-Setup-XXXX`, 192.168.4.1) for 1st-time provisioning with 5-minute self-healing recovery reboot timer.
   - mDNS responder at `timestation.local`.
