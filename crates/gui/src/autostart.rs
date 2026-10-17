//! Autostart integration for Windows (via HKCU\...\Run registry key).

#[cfg(windows)]
pub fn is_autostart_enabled() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
        key.get_value::<String, _>("RazerLine").is_ok()
    } else {
        false
    }
}

#[cfg(windows)]
pub fn set_autostart(enable: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .map_err(|e| format!("Registry error: {e}"))?;

    if enable {
        let exe = std::env::current_exe().map_err(|e| format!("Current exe path error: {e}"))?;
        let exe_str = format!("\"{}\" --background", exe.to_string_lossy());
        key.set_value("RazerLine", &exe_str)
            .map_err(|e| format!("Failed to set Run key: {e}"))?;
    } else {
        let _ = key.delete_value("RazerLine");
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn is_autostart_enabled() -> bool {
    false
}

#[cfg(not(windows))]
pub fn set_autostart(_enable: bool) -> Result<(), String> {
    Ok(())
}
