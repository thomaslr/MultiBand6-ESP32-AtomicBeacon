//! Non-Volatile Flash Configuration & Wi-Fi Storage using ESP32 NVS.

use beacon_core::config::AppConfig;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use log::{error, info, warn};

const CFG_NAMESPACE: &str = "beacon_cfg";
const NVS_KEY_CONFIG: &str = "config_json";

const WIFI_NAMESPACE: &str = "beacon_wifi";
const NVS_KEY_SSID: &str = "wifi_ssid";
const NVS_KEY_PASS: &str = "wifi_pass";

const BUFFER_SIZE: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiCredentials {
    pub ssid: String,
    pub password: String,
}

pub struct NvsConfigStorage {
    nvs_cfg: EspNvs<NvsDefault>,
    nvs_wifi: EspNvs<NvsDefault>,
}

impl NvsConfigStorage {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self, esp_idf_svc::sys::EspError> {
        let nvs_cfg = EspNvs::new(partition.clone(), CFG_NAMESPACE, true)?;
        let nvs_wifi = EspNvs::new(partition, WIFI_NAMESPACE, true)?;
        Ok(Self { nvs_cfg, nvs_wifi })
    }

    pub fn load_config(&self) -> AppConfig {
        let mut buf = vec![0u8; BUFFER_SIZE];
        match self.nvs_cfg.get_str(NVS_KEY_CONFIG, &mut buf) {
            Ok(Some(json_str)) => match serde_json::from_str::<AppConfig>(json_str) {
                Ok(cfg) => {
                    info!("[NVS] Loaded existing beacon configuration from flash");
                    cfg
                }
                Err(e) => {
                    warn!("[NVS] Corrupt config JSON in flash ({}). Using defaults.", e);
                    let def = AppConfig::default();
                    let _ = self.save_config(&def);
                    def
                }
            },
            Ok(None) => {
                info!("[NVS] No saved beacon config found. Initializing defaults.");
                let def = AppConfig::default();
                let _ = self.save_config(&def);
                def
            }
            Err(e) => {
                error!("[NVS] Failed to read beacon config ({}). Using defaults.", e);
                AppConfig::default()
            }
        }
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
        let json_str = serde_json::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        self.nvs_cfg
            .set_str(NVS_KEY_CONFIG, &json_str)
            .map_err(|e| format!("Failed to write config to NVS: {}", e))?;

        info!("[NVS] Configuration saved to flash");
        Ok(())
    }

    pub fn load_wifi(&self) -> Option<WifiCredentials> {
        let mut ssid_buf = [0u8; 64];
        let mut pass_buf = [0u8; 64];

        let ssid = match self.nvs_wifi.get_str(NVS_KEY_SSID, &mut ssid_buf) {
            Ok(Some(s)) if !s.trim().is_empty() => s.to_string(),
            _ => return None,
        };

        let password = match self.nvs_wifi.get_str(NVS_KEY_PASS, &mut pass_buf) {
            Ok(Some(p)) => p.to_string(),
            _ => String::new(),
        };

        info!("[NVS] Loaded saved Wi-Fi credentials for SSID: '{}'", ssid);
        Some(WifiCredentials { ssid, password })
    }

    pub fn save_wifi(&self, ssid: &str, password: &str) -> Result<(), String> {
        self.nvs_wifi
            .set_str(NVS_KEY_SSID, ssid)
            .map_err(|e| format!("Failed to save Wi-Fi SSID to NVS: {}", e))?;

        self.nvs_wifi
            .set_str(NVS_KEY_PASS, password)
            .map_err(|e| format!("Failed to save Wi-Fi password to NVS: {}", e))?;

        info!("[NVS] Saved Wi-Fi credentials for SSID: '{}'", ssid);
        Ok(())
    }

    pub fn clear_wifi(&self) -> Result<(), String> {
        self.nvs_wifi
            .erase_all()
            .map_err(|e| format!("Failed to erase Wi-Fi credentials: {}", e))?;
        info!("[NVS] Cleared Wi-Fi credentials from NVS");
        Ok(())
    }
}
