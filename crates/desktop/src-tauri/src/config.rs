//! Persistent application configuration and device profiles.
//! Stores preferences in `%APPDATA%\RazerLine\config.json` on Windows
//! (or `$XDG_CONFIG_HOME/razerline/config.json` on Linux, `~/Library/Application Support/RazerLine/config.json` on macOS).
//!
//! This is a fresh schema designed for the Tauri frontend, not a carry-forward
//! of the old egui GUI's `DeviceConfig` shape. The field *content* of
//! `UserProfile` (dpi, dpi_stages, poll_rate_hz, brightness, led_color,
//! sync_zones, effect, matrix_color, button_mappings) is preserved — that's
//! good data modeling, not an egui artifact. The flat `DeviceConfig` is gone;
//! profiles live in `DeviceProfileConfig` keyed by device PID.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::profiles::DeviceProfileConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Whether RazerLine starts automatically on system boot.
    #[serde(default = "default_true")]
    pub start_with_windows: bool,

    /// When closing the window (clicking 'X'), minimize to system tray instead of exiting.
    #[serde(default = "default_true")]
    pub close_to_tray: bool,

    /// Whether the application starts hidden in tray (used with --background).
    #[serde(default)]
    pub start_minimized: bool,

    /// Saved per-device profile configs keyed by USB Product ID.
    #[serde(default)]
    pub devices: HashMap<u16, DeviceProfileConfig>,
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            start_with_windows: false, // Will be synced from registry/plugin on load
            close_to_tray: true,
            start_minimized: false,
            devices: HashMap::new(),
        }
    }
}

impl AppConfig {
    /// Returns the configuration file path.
    pub fn file_path() -> PathBuf {
        // Prefer XDG on Linux, AppData on Windows, Application Support on macOS
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            let mut path = PathBuf::from(config_home);
            path.push("razerline");
            path.push("config.json");
            return path;
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            let mut path = PathBuf::from(appdata);
            path.push("RazerLine");
            path.push("config.json");
            return path;
        }
        if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".config");
            path.push("razerline");
            path.push("config.json");
            return path;
        }
        PathBuf::from("razerline_config.json")
    }

    /// Loads configuration from disk, falling back to defaults if not found or invalid.
    pub fn load() -> Self {
        let path = Self::file_path();
        if path.exists()
            && let Ok(data) = fs::read_to_string(&path)
            && let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&data)
        {
            // Sync with actual autostart state (registry on Windows, plugin elsewhere)
            cfg.start_with_windows = crate::autostart::is_autostart_enabled();
            return cfg;
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

    /// Gets or creates the profile config for a device PID.
    #[allow(dead_code)]
    pub fn device_profile(&mut self, pid: u16, name: &str) -> &mut DeviceProfileConfig {
        self.devices
            .entry(pid)
            .or_insert_with(|| DeviceProfileConfig::default_with_name(name))
    }

    /// Gets the profile config for a device PID if it exists.
    #[allow(dead_code)]
    pub fn get_device_profile(&self, pid: u16) -> Option<&DeviceProfileConfig> {
        self.devices.get(&pid)
    }

    /// Gets a mutable reference to the profile config for a device PID if it exists.
    #[allow(dead_code)]
    pub fn get_device_profile_mut(&mut self, pid: u16) -> Option<&mut DeviceProfileConfig> {
        self.devices.get_mut(&pid)
    }
}
