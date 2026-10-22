//! `#[tauri::command]` wrappers over `razer_hid_client::RazerDevice`. Each
//! one is a thin pass-through: pull the open device out of `AppState`, call
//! the existing client method, map `ClientError` to a `String` via its
//! `Display` impl (Tauri commands return `Result<T, String>` idiomatically,
//! and `ClientError` only ever has the two variants the old UI already
//! treated as one opaque message — see `crates/client/src/lib.rs`).

use std::sync::atomic::Ordering;

use razer_hid_abi::DeviceProfile;
use razer_hid_abi::commands::LedEffect;
use razer_hid_client::RazerDevice;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Library name (not a file path) of the device-catalog plugin —
/// `razer_hid_client::load_known_devices` looks for it next to the running
/// executable using the platform's DLL naming convention. Same mechanism
/// the old egui GUI used (`crates/gui/src/app.rs`'s `known_devices`).
const DEVICES_PLUGIN: &str = "razer_hid_devices_mice";

pub fn known_devices() -> &'static [DeviceProfile] {
    static CELL: std::sync::OnceLock<&'static [DeviceProfile]> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        razer_hid_client::load_known_devices(DEVICES_PLUGIN)
            .expect("couldn't load device catalog plugin")
    })
}

/// Serializable summary of a `LedZoneProfile` for the frontend — just
/// identity + whether it has a brightness control, not the command bytes.
#[derive(Serialize, Clone)]
pub struct LedZoneInfo {
    pub id: u8,
    pub name: &'static str,
    pub has_brightness: bool,
}

/// Lowercase effect name matching `UserProfile.effect`'s string
/// convention from the old egui GUI's config format — kept as a plain
/// string rather than a shared enum since the frontend just needs to
/// compare it against button labels, not round-trip it through Rust.
fn effect_name(effect: LedEffect) -> &'static str {
    match effect {
        LedEffect::Off => "off",
        LedEffect::Static => "static",
        LedEffect::Breathing => "breathing",
        LedEffect::Spectrum => "spectrum",
        LedEffect::Wave => "wave",
        LedEffect::Custom => "custom",
    }
}

/// Serializable summary of a `DeviceProfile` for the frontend. Plain
/// `DeviceProfile` can't cross the IPC boundary as-is (it derives
/// `Clone`/`Copy`, not `Serialize`).
#[derive(Serialize, Clone)]
pub struct DeviceInfo {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub dpi_min: Option<u16>,
    pub dpi_max: Option<u16>,
    pub supported_poll_hz: Vec<u32>,
    pub has_identity: bool,
    pub has_battery: bool,
    pub idle_min_seconds: Option<u16>,
    pub idle_max_seconds: Option<u16>,
    pub has_low_battery_threshold: bool,
    pub led_zones: Vec<LedZoneInfo>,
    pub matrix_rows: Option<u8>,
    pub matrix_cols: Option<u8>,
    pub unsupported_led_effects: Vec<&'static str>,
}

pub fn device_info(profile: &'static DeviceProfile) -> DeviceInfo {
    DeviceInfo {
        name: profile.name,
        vendor_id: profile.vendor_id,
        product_id: profile.product_id,
        dpi_min: profile.dpi.map(|d| d.min),
        dpi_max: profile.dpi.map(|d| d.max),
        supported_poll_hz: profile
            .poll_rate
            .map(|p| p.supported_hz.to_vec())
            .unwrap_or_default(),
        has_identity: profile.identity.is_some(),
        has_battery: profile.battery.is_some(),
        idle_min_seconds: profile.idle_time.map(|i| i.min_seconds),
        idle_max_seconds: profile.idle_time.map(|i| i.max_seconds),
        has_low_battery_threshold: profile.low_battery_threshold.is_some(),
        led_zones: profile
            .leds
            .iter()
            .map(|z| LedZoneInfo {
                id: z.id,
                name: z.name,
                has_brightness: z.brightness_command_id.is_some(),
            })
            .collect(),
        matrix_rows: profile.matrix.map(|m| m.rows),
        matrix_cols: profile.matrix.map(|m| m.cols),
        unsupported_led_effects: profile
            .unsupported_led_effects
            .iter()
            .map(|e| effect_name(*e))
            .collect(),
    }
}

fn zone_by_id(
    dev: &RazerDevice,
    zone_id: u8,
) -> Result<&'static razer_hid_abi::LedZoneProfile, String> {
    dev.profile
        .leds
        .iter()
        .find(|z| z.id == zone_id)
        .ok_or_else(|| format!("no LED zone with id {zone_id} on this device"))
}

fn with_device<T>(
    state: &State<AppState>,
    f: impl FnOnce(&RazerDevice) -> Result<T, razer_hid_client::ClientError>,
) -> Result<T, String> {
    let guard = state.device.lock().unwrap();
    let dev = guard
        .as_ref()
        .ok_or_else(|| "no device connected".to_string())?;
    f(dev).map_err(|e| e.to_string())
}

fn with_zone<T>(
    state: &State<AppState>,
    zone_id: u8,
    f: impl FnOnce(
        &RazerDevice,
        &'static razer_hid_abi::LedZoneProfile,
    ) -> Result<T, razer_hid_client::ClientError>,
) -> Result<T, String> {
    let guard = state.device.lock().unwrap();
    let dev = guard
        .as_ref()
        .ok_or_else(|| "no device connected".to_string())?;
    let zone = zone_by_id(dev, zone_id)?;
    f(dev, zone).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_connected() -> Result<Vec<DeviceInfo>, String> {
    razer_hid_client::list_connected(known_devices())
        .map(|found| found.into_iter().map(device_info).collect())
        .map_err(|e| e.to_string())
}

/// Opens a specific known device by product id and makes it the active
/// device. The background worker (`worker.rs`) also auto-connects the
/// first device it finds when none is open — this command exists for when
/// more than one supported device is plugged in and the frontend wants to
/// switch which one is active.
#[tauri::command]
pub fn connect_device(state: State<AppState>, product_id: u16) -> Result<DeviceInfo, String> {
    let profile = known_devices()
        .iter()
        .find(|p| p.product_id == product_id)
        .ok_or_else(|| format!("unknown product id 0x{product_id:04x}"))?;
    let dev = RazerDevice::open(profile).map_err(|e| e.to_string())?;
    *state.device.lock().unwrap() = Some(dev);
    state.active_pid.store(product_id as u32, Ordering::SeqCst);
    Ok(device_info(profile))
}

#[tauri::command]
pub fn active_device(state: State<AppState>) -> Option<DeviceInfo> {
    let guard = state.device.lock().unwrap();
    guard.as_ref().map(|dev| device_info(dev.profile))
}

#[derive(Serialize)]
pub struct DpiValue {
    pub x: u16,
    pub y: u16,
}

#[tauri::command]
pub fn get_dpi(state: State<AppState>) -> Result<DpiValue, String> {
    with_device(&state, |dev| dev.get_dpi()).map(|d| DpiValue { x: d.x, y: d.y })
}

#[tauri::command]
pub fn set_dpi(state: State<AppState>, x: u16, y: u16) -> Result<(), String> {
    let result = with_device(&state, |dev| dev.set_dpi(x, y));
    if result.is_ok() {
        state.current_dpi.store(x, Ordering::SeqCst);
    }
    result
}

#[tauri::command]
pub fn get_poll_rate(state: State<AppState>) -> Result<u32, String> {
    with_device(&state, |dev| dev.get_poll_rate())
}

#[tauri::command]
pub fn set_poll_rate(state: State<AppState>, hz: u32) -> Result<(), String> {
    with_device(&state, |dev| dev.set_poll_rate(hz))
}

#[tauri::command]
pub fn get_battery_level(state: State<AppState>) -> Result<u8, String> {
    with_device(&state, |dev| dev.get_battery_level())
}

#[tauri::command]
pub fn get_charging_status(state: State<AppState>) -> Result<bool, String> {
    with_device(&state, |dev| dev.get_charging_status())
}

#[tauri::command]
pub fn get_idle_time(state: State<AppState>) -> Result<u16, String> {
    with_device(&state, |dev| dev.get_idle_time())
}

#[tauri::command]
pub fn set_idle_time(state: State<AppState>, seconds: u16) -> Result<(), String> {
    with_device(&state, |dev| dev.set_idle_time(seconds))
}

#[tauri::command]
pub fn get_low_battery_threshold(state: State<AppState>) -> Result<u8, String> {
    with_device(&state, |dev| dev.get_low_battery_threshold())
}

#[tauri::command]
pub fn set_low_battery_threshold(state: State<AppState>, percent: u8) -> Result<(), String> {
    with_device(&state, |dev| dev.set_low_battery_threshold(percent))
}

#[tauri::command]
pub fn get_firmware_version(state: State<AppState>) -> Result<String, String> {
    with_device(&state, |dev| dev.get_firmware_version())
}

#[tauri::command]
pub fn get_serial(state: State<AppState>) -> Result<String, String> {
    with_device(&state, |dev| dev.get_serial())
}

// -- Lighting --------------------------------------------------------------

#[tauri::command]
pub fn set_led_off(state: State<AppState>, zone_id: u8) -> Result<(), String> {
    with_zone(&state, zone_id, |dev, zone| dev.set_led_off(zone))
}

#[tauri::command]
pub fn set_led_static(state: State<AppState>, zone_id: u8, rgb: [u8; 3]) -> Result<(), String> {
    with_zone(&state, zone_id, |dev, zone| dev.set_led_static(zone, rgb))
}

#[tauri::command]
pub fn set_led_breathing(state: State<AppState>, zone_id: u8, rgb: [u8; 3]) -> Result<(), String> {
    with_zone(&state, zone_id, |dev, zone| {
        dev.set_led_breathing(zone, rgb)
    })
}

#[tauri::command]
pub fn set_led_spectrum(state: State<AppState>, zone_id: u8) -> Result<(), String> {
    with_zone(&state, zone_id, |dev, zone| dev.set_led_spectrum(zone))
}

#[tauri::command]
pub fn set_led_wave(state: State<AppState>, zone_id: u8, direction: u8) -> Result<(), String> {
    with_zone(&state, zone_id, |dev, zone| {
        dev.set_led_wave(zone, direction)
    })
}

#[tauri::command]
pub fn set_led_brightness(state: State<AppState>, zone_id: u8, percent: u8) -> Result<(), String> {
    with_zone(&state, zone_id, |dev, zone| {
        dev.set_led_brightness(zone, percent)
    })
}

#[tauri::command]
pub fn set_led_off_all(state: State<AppState>) -> Result<(), String> {
    with_device(&state, |dev| dev.set_led_off_all())
}

#[tauri::command]
pub fn set_led_static_all(state: State<AppState>, rgb: [u8; 3]) -> Result<(), String> {
    with_device(&state, |dev| dev.set_led_static_all(rgb))
}

#[tauri::command]
pub fn set_led_breathing_all(state: State<AppState>, rgb: [u8; 3]) -> Result<(), String> {
    with_device(&state, |dev| dev.set_led_breathing_all(rgb))
}

#[tauri::command]
pub fn set_led_spectrum_all(state: State<AppState>) -> Result<(), String> {
    with_device(&state, |dev| dev.set_led_spectrum_all())
}

#[tauri::command]
pub fn set_led_wave_all(state: State<AppState>, direction: u8) -> Result<(), String> {
    with_device(&state, |dev| dev.set_led_wave_all(direction))
}

#[tauri::command]
pub fn set_led_brightness_all(state: State<AppState>, percent: u8) -> Result<(), String> {
    with_device(&state, |dev| dev.set_led_brightness_all(percent))
}

// -- Matrix / underglow ------------------------------------------------------

#[tauri::command]
pub fn set_matrix_row(
    state: State<AppState>,
    row: u8,
    start_col: u8,
    colors: Vec<[u8; 3]>,
) -> Result<(), String> {
    with_device(&state, |dev| dev.set_matrix_row(row, start_col, &colors))
}

#[tauri::command]
pub fn commit_matrix_frame(state: State<AppState>) -> Result<(), String> {
    with_device(&state, |dev| dev.commit_matrix_frame())
}

#[tauri::command]
pub fn fill_matrix(state: State<AppState>, rgb: [u8; 3]) -> Result<(), String> {
    with_device(&state, |dev| dev.fill_matrix(rgb))
}

#[tauri::command]
pub fn set_matrix_spectrum(state: State<AppState>) -> Result<(), String> {
    with_device(&state, |dev| dev.set_matrix_spectrum())
}

#[tauri::command]
pub fn set_matrix_wave(state: State<AppState>, direction: u8) -> Result<(), String> {
    with_device(&state, |dev| dev.set_matrix_wave(direction))
}
