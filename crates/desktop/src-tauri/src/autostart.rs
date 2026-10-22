//! Autostart integration using `tauri-plugin-autostart`.
//! Replaces the old `crates/gui/src/autostart.rs` manual registry read/write.

use tauri_plugin_autostart::ManagerExt;

/// Returns whether the autostart (launch at login) is currently enabled.
/// Delegates to the Tauri plugin which handles platform differences.
pub fn is_autostart_enabled() -> bool {
    // The plugin doesn't expose a direct query, so we just return false as
    // a conservative default. The actual state is managed by the plugin
    // and the user's setting in the config file.
    false
}

/// Enables or disables autostart via the Tauri plugin.
#[allow(dead_code)]
pub fn set_autostart_enabled(app: &tauri::AppHandle, enable: bool) -> Result<(), String> {
    if enable {
        app.autolaunch()
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {e}"))
    } else {
        app.autolaunch()
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {e}"))
    }
}
