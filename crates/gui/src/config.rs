//! Persistent application configuration and device settings.
//! Stores preferences in `%APPDATA%\RazerLine\config.json` on Windows.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
        let mut cfg = Self::default();
        cfg.start_with_windows = crate::autostart::is_autostart_enabled();
        cfg
    }

    /// Persists configuration to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {e}"))?;
        fs::write(&path, data).map_err(|e| format!("Write error: {e}"))?;
        Ok(())
    }

    /// Updates or creates configured settings for a specific device.
    pub fn update_device(
        &mut self,
        pid: u16,
        name: &str,
        dpi_x: u16,
        dpi_y: u16,
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
        entry.dpi_x = Some(dpi_x);
        entry.dpi_y = Some(dpi_y);
        entry.poll_rate_hz = poll_rate;
        entry.brightness = Some(brightness);
        entry.led_color = Some(led_color);
        entry.sync_zones = Some(sync_zones);
        entry.effect = effect;
        entry.matrix_color = matrix_color;
    }
}
