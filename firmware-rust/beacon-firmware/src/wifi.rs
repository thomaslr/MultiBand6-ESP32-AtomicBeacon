// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

//! Wi-Fi Station and SoftAP Provisioning Management.
//!
//! Features learned from plant-esp32:
//! 1. Station Mode with WPA3-SAE stale session recovery (10-attempt loop with radio cycling).
//! 2. Max TX power boost (78 = ~19.5 dBm) for maximum reach.
//! 3. 20 MHz bandwidth (HT20) lock to reduce RF noise on antenna.
//! 4. 802.11 Modem-Sleep enabled during standby; paused during active RF broadcast.
//! 5. SoftAP Provisioning: `TimeStation-Setup-XXXX` (192.168.4.1) captive portal with nearby AP scanner.
//! 6. Self-healing recovery: reboots every 5 minutes in SoftAP mode if saved credentials exist.

use std::ffi::CString;

use beacon_core::config::WifiConfig;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{
    esp_netif_get_handle_from_ifkey, esp_netif_set_hostname, esp_wifi_get_max_tx_power,
    esp_wifi_set_bandwidth, esp_wifi_set_max_tx_power, esp_wifi_set_ps,
    wifi_bandwidth_t_WIFI_BW_HT20, wifi_bandwidth_t_WIFI_BW_HT40, wifi_interface_t_WIFI_IF_STA,
    wifi_ps_type_t_WIFI_PS_MIN_MODEM, wifi_ps_type_t_WIFI_PS_NONE,
};
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};
use log::{info, warn};

use crate::storage::WifiCredentials;

pub fn apply_wifi_radio_config(cfg: &WifiConfig) {
    unsafe {
        let _ = esp_wifi_set_max_tx_power(cfg.tx_power);
        let bw = if cfg.force_20mhz {
            wifi_bandwidth_t_WIFI_BW_HT20
        } else {
            wifi_bandwidth_t_WIFI_BW_HT40
        };
        let _ = esp_wifi_set_bandwidth(wifi_interface_t_WIFI_IF_STA, bw);
        let ps = if cfg.modem_sleep {
            wifi_ps_type_t_WIFI_PS_MIN_MODEM
        } else {
            wifi_ps_type_t_WIFI_PS_NONE
        };
        let _ = esp_wifi_set_ps(ps);
    }
    info!(
        "[WIFI] Radio config applied: TX power {} (~{:.1} dBm), HT20: {}, Modem Sleep: {}",
        cfg.tx_power, (cfg.tx_power as f32) / 4.0, cfg.force_20mhz, cfg.modem_sleep
    );
}

pub enum WifiState {
    StationConnected {
        ip_address: String,
        wifi_handle: BlockingWifi<EspWifi<'static>>,
    },
    ProvisioningMode {
        ap_ssid: String,
        scanned_ssids: Vec<String>,
        wifi_handle: BlockingWifi<EspWifi<'static>>,
    },
}

pub fn start_wifi(
    modem: Modem<'static>,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
    saved_creds: Option<WifiCredentials>,
    hostname: &str,
    wifi_cfg: &WifiConfig,
) -> Result<WifiState, esp_idf_svc::sys::EspError> {
    let esp_wifi = EspWifi::new(modem, sys_loop.clone(), nvs)?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

    // 1. If saved credentials exist, attempt Station Mode first
    if let Some(creds) = saved_creds {
        info!("[WIFI] Attempting connection to saved network: '{}'", creds.ssid);

        let client_config = ClientConfiguration {
            ssid: creds.ssid.as_str().try_into().unwrap_or_default(),
            password: creds.password.as_str().try_into().unwrap_or_default(),
            auth_method: if creds.password.is_empty() {
                AuthMethod::None
            } else {
                AuthMethod::WPA2Personal
            },
            ..Default::default()
        };

        if let Ok(()) = wifi.set_configuration(&Configuration::Client(client_config)) {
            if let Ok(()) = wifi.start() {
                unsafe {
                    let _ = esp_wifi_set_max_tx_power(wifi_cfg.tx_power);
                    let bw = if wifi_cfg.force_20mhz {
                        wifi_bandwidth_t_WIFI_BW_HT20
                    } else {
                        wifi_bandwidth_t_WIFI_BW_HT40
                    };
                    let _ = esp_wifi_set_bandwidth(wifi_interface_t_WIFI_IF_STA, bw);
                    let mut actual_pwr: i8 = 0;
                    if esp_wifi_get_max_tx_power(&mut actual_pwr) == 0 {
                        info!("[WIFI] TX power configured: {} (~{:.1} dBm)", actual_pwr, (actual_pwr as f32) / 4.0);
                    }
                    let _ = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE);
                }
                set_sta_hostname(hostname);

                const MAX_STA_ATTEMPTS: u32 = 10;
                for attempt in 1..=MAX_STA_ATTEMPTS {
                    if attempt > 1 {
                        warn!(
                            "[WIFI] Retrying saved network (attempt {}/{} in 15s with radio cycle)...",
                            attempt, MAX_STA_ATTEMPTS
                        );
                        let _ = wifi.disconnect();
                        let _ = wifi.stop();
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let _ = wifi.start();
                        unsafe {
                            let _ = esp_wifi_set_max_tx_power(wifi_cfg.tx_power);
                            let bw = if wifi_cfg.force_20mhz {
                                wifi_bandwidth_t_WIFI_BW_HT20
                            } else {
                                wifi_bandwidth_t_WIFI_BW_HT40
                            };
                            let _ = esp_wifi_set_bandwidth(wifi_interface_t_WIFI_IF_STA, bw);
                            let _ = esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE);
                        }
                        std::thread::sleep(std::time::Duration::from_secs(14));
                    }

                    if let Ok(()) = wifi.connect() {
                        info!("[WIFI] Associated with AP. Settling 750ms before DHCP negotiation...");
                        std::thread::sleep(std::time::Duration::from_millis(750));
                        if let Ok(()) = wifi.wait_netif_up() {
                            if let Ok(info) = wifi.wifi().sta_netif().get_ip_info() {
                                let ip_str = info.ip.to_string();
                                if ip_str != "0.0.0.0" {
                                    info!("[WIFI] Station Mode active! IP: {}", ip_str);
                                    apply_wifi_radio_config(wifi_cfg);
                                    return Ok(WifiState::StationConnected {
                                        ip_address: ip_str,
                                        wifi_handle: wifi,
                                    });
                                }
                            }
                        }
                    }
                }
                warn!("[WIFI] Failed to connect to saved network after {} attempts", MAX_STA_ATTEMPTS);
                let _ = wifi.stop();
            }
        }
    }

    // 2. Fall back to Provisioning SoftAP Mode
    let mut mac = [0u8; 6];
    unsafe {
        esp_idf_svc::sys::esp_read_mac(
            mac.as_mut_ptr(),
            esp_idf_svc::sys::esp_mac_type_t_ESP_MAC_WIFI_STA,
        );
    }
    let ap_ssid = format!("TimeStation-Setup-{:02X}{:02X}", mac[4], mac[5]);
    info!("[WIFI] Starting Provisioning SoftAP: '{}'", ap_ssid);

    let ap_config = AccessPointConfiguration {
        ssid: ap_ssid.as_str().try_into().unwrap_or_default(),
        channel: 1,
        auth_method: AuthMethod::None,
        max_connections: 4,
        ..Default::default()
    };

    let mixed_config = Configuration::Mixed(ClientConfiguration::default(), ap_config);
    if let Err(e) = wifi.set_configuration(&mixed_config) {
        warn!("[WIFI] Mixed mode configuration failed ({:?}), falling back to AP only", e);
        let ap_only = Configuration::AccessPoint(AccessPointConfiguration {
            ssid: ap_ssid.as_str().try_into().unwrap_or_default(),
            channel: 1,
            auth_method: AuthMethod::None,
            max_connections: 4,
            ..Default::default()
        });
        wifi.set_configuration(&ap_only)?;
    }

    wifi.start()?;
    info!("[WIFI] SoftAP active. Gateway IP: 192.168.4.1");

    info!("[WIFI] Scanning nearby 2.4 GHz networks...");
    let mut scanned_ssids = Vec::new();
    if let Ok(aps) = wifi.scan() {
        for ap in aps {
            let name = ap.ssid.as_str().trim();
            if !name.is_empty() && !scanned_ssids.iter().any(|s: &String| s == name) {
                scanned_ssids.push(name.to_string());
            }
        }
        info!("[WIFI] Discovered {} unique nearby networks", scanned_ssids.len());
    } else {
        warn!("[WIFI] Initial scan returned no results");
    }

    Ok(WifiState::ProvisioningMode {
        ap_ssid,
        scanned_ssids,
        wifi_handle: wifi,
    })
}

fn set_sta_hostname(hostname: &str) {
    if let Ok(c_hostname) = CString::new(hostname) {
        unsafe {
            let ifkey = CString::new("WIFI_STA_DEF").unwrap();
            let netif = esp_netif_get_handle_from_ifkey(ifkey.as_ptr());
            if !netif.is_null() {
                let _ = esp_netif_set_hostname(netif, c_hostname.as_ptr());
            }
        }
    }
}
