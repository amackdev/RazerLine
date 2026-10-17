//! Persistent application configuration and device settings.
//! Stores preferences in `%APPDATA%\RazerLine\config.json` on Windows.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::profiles::DeviceProfileConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Whether RazerLine starts automatically on Windows boot.
    #[serde(default = "default_true")]
    pub start_with_windows: bool,

    /// When closing the window (clicking 'X'), minimize to system tray instead of exiting.
    #[serde(default = "default_true")]
    pub close_to_tray: bool,

    /// Whether the application starts hidden in tray (used with --background).
    #[serde(default)]
    pub start_minimized: bool,

    /// Saved per-device settings keyed by USB Product ID.
    #[serde(default)]
    pub devices: HashMap<u16, DeviceConfig>,
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            start_with_windows: crate::autostart::is_autostart_enabled(),
            close_to_tray: true,
            start_minimized: false,
            devices: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceConfig {
    pub name: String,
    pub dpi_x: Option<u16>,
    pub dpi_y: Option<u16>,
    pub poll_rate_hz: Option<u32>,
    pub brightness: Option<u8>,
    pub led_color: Option<[u8; 3]>,
    pub sync_zones: Option<bool>,
    pub effect: Option<String>,
    pub matrix_color: Option<[u8; 3]>,
    #[serde(default)]
    pub profile_data: Option<DeviceProfileConfig>,
}

impl DeviceConfig {
    pub fn ensure_profiles(&mut self) -> &mut DeviceProfileConfig {
        if self.profile_data.is_none() {
            let mut def = DeviceProfileConfig::default();
            let p = def.active_profile_mut();
            if let Some(dpi) = self.dpi_x {
                p.dpi = dpi;
            }
            if let Some(rate) = self.poll_rate_hz {
                p.poll_rate_hz = Some(rate);
            }
            if let Some(b) = self.brightness {
                p.brightness = b;
            }
            if let Some(c) = self.led_color {
                p.led_color = c;
            }
            if let Some(s) = self.sync_zones {
                p.sync_zones = s;
            }
            if let Some(ref eff) = self.effect {
                p.effect = eff.clone();
            }
            if let Some(m) = self.matrix_color {
                p.matrix_color = Some(m);
            }
            self.profile_data = Some(def);
        }
        self.profile_data.as_mut().unwrap()
    }
}

impl AppConfig {
    /// Returns the configuration file path (%APPDATA%\RazerLine\config.json).
    pub fn file_path() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let mut path = PathBuf::from(appdata);
            path.push("RazerLine");
            path.push("config.json");
            path
        } else {
            PathBuf::from("razerline_config.json")
        }
    }

    /// Loads configuration from disk, falling back to defaults if not found or invalid.
    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&data) {
                    // Sync with actual registry state if on Windows
                    cfg.start_with_windows = crate::autostart::is_autostart_enabled();
                    return cfg;
                }
            }
        }
        Self {
            start_with_windows: crate::autostart::is_autostart_enabled(),
            ..Default::default()
        }
    }

    /// Persists configuration to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data =
            serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {e}"))?;
        fs::write(&path, data).map_err(|e| format!("Write error: {e}"))?;
        Ok(())
    }

    /// Updates or creates configured settings for a specific device.
    #[allow(clippy::too_many_arguments)]
    pub fn update_device(
        &mut self,
        pid: u16,
        name: &str,
        dpi: u16,
        poll_rate: Option<u32>,
        brightness: u8,
        led_color: [u8; 3],
        sync_zones: bool,
        effect: Option<String>,
        matrix_color: Option<[u8; 3]>,
    ) {
        let entry = self.devices.entry(pid).or_insert_with(|| DeviceConfig {
            name: name.to_string(),
            ..Default::default()
        });
        entry.name = name.to_string();
        entry.dpi_x = Some(dpi);
        entry.dpi_y = Some(dpi);
        entry.poll_rate_hz = poll_rate;
        entry.brightness = Some(brightness);
        entry.led_color = Some(led_color);
        entry.sync_zones = Some(sync_zones);
        entry.effect = effect.clone();
        entry.matrix_color = matrix_color;

        let prof = entry.ensure_profiles().active_profile_mut();
        prof.dpi = dpi;
        prof.poll_rate_hz = poll_rate;
        prof.brightness = brightness;
        prof.led_color = led_color;
        prof.sync_zones = sync_zones;
        if let Some(eff) = effect {
            prof.effect = eff;
        }
        if let Some(m) = matrix_color {
            prof.matrix_color = Some(m);
        }
    }
}
