//! Loads `razer-hid-core` and `razer-hid-devices-mice` as DLLs at
//! runtime (via `libloading`, next to the running executable) and
//! exposes the same `RazerDevice` method names/shapes the project used
//! when core was linked at compile time — so the CLI/GUI barely need to
//! change. See `razer_hid_abi::Op` for the wire format this encodes
//! against, and `razer-hid-core`'s `ffi` module for the other side.

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::OnceLock;

use libloading::{Library, Symbol};
use razer_hid_abi::{DeviceProfile, FfiInterfaceInfo, LedZoneProfile, Op};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("failed to load {0}: {1}")]
    Load(String, libloading::Error),
    #[error("{0}")]
    Device(String),
}

pub struct DpiSetting {
    pub x: u16,
    pub y: u16,
}

pub struct InterfaceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub product_string: Option<String>,
    pub path: String,
}

/// Directory the running executable lives in — where Cargo places
/// every crate's build artifacts (binaries and `cdylib`s alike) by
/// default, so the DLLs this crate loads are expected right next to it.
fn plugin_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn plugin_path(crate_name: &str) -> PathBuf {
    let file = format!(
        "{}{}{}",
        std::env::consts::DLL_PREFIX,
        crate_name,
        std::env::consts::DLL_SUFFIX
    );
    plugin_dir().join(file)
}

fn load_library(crate_name: &str) -> Result<&'static Library, ClientError> {
    // Leaked deliberately: every handle/pointer this crate hands back
    // borrows memory inside the loaded library, so it must never be
    // unloaded for the rest of the process's life.
    let path = plugin_path(crate_name);
    let lib = unsafe { Library::new(&path) }
        .map_err(|e| ClientError::Load(path.display().to_string(), e))?;
    Ok(Box::leak(Box::new(lib)))
}

unsafe fn symbol<'a, T>(lib: &'a Library, name: &str) -> Result<Symbol<'a, T>, ClientError> {
    let name_z = format!("{name}\0");
    lib.get(name_z.as_bytes())
        .map_err(|e| ClientError::Load(name.to_string(), e))
}

type FnOpen = unsafe extern "C" fn(*const DeviceProfile) -> *mut c_void;
type FnClose = unsafe extern "C" fn(*mut c_void);
type FnLastError = unsafe extern "C" fn(*mut u8, usize) -> usize;
type FnListConnected =
    unsafe extern "C" fn(*const DeviceProfile, usize, *mut *const DeviceProfile, usize) -> i64;
type FnCall = unsafe extern "C" fn(*mut c_void, u32, *const u8, usize, *mut u8, usize) -> i64;
type FnDebugListInterfaces = unsafe extern "C" fn(u16, *mut FfiInterfaceInfo, usize) -> i64;
type FnPluginDevices = unsafe extern "C" fn(*mut usize) -> *const DeviceProfile;

struct CoreApi {
    open: FnOpen,
    close: FnClose,
    last_error: FnLastError,
    list_connected: FnListConnected,
    call: FnCall,
    debug_list_interfaces: FnDebugListInterfaces,
}

fn core_api() -> Result<&'static CoreApi, ClientError> {
    static CELL: OnceLock<Result<CoreApi, ClientError>> = OnceLock::new();
    CELL.get_or_init(|| {
        let lib = load_library("razer_hid_core")?;
        unsafe {
            Ok(CoreApi {
                open: *symbol::<FnOpen>(lib, "razer_core_open")?,
                close: *symbol::<FnClose>(lib, "razer_core_close")?,
                last_error: *symbol::<FnLastError>(lib, "razer_core_last_error")?,
                list_connected: *symbol::<FnListConnected>(lib, "razer_core_list_connected")?,
                call: *symbol::<FnCall>(lib, "razer_core_call")?,
                debug_list_interfaces: *symbol::<FnDebugListInterfaces>(
                    lib,
                    "razer_core_debug_list_interfaces",
                )?,
            })
        }
    })
    .as_ref()
    .map_err(|e| ClientError::Device(e.to_string()))
}

fn last_error(api: &CoreApi) -> String {
    let mut buf = vec![0u8; 512];
    let n = unsafe { (api.last_error)(buf.as_mut_ptr(), buf.len()) };
    buf.truncate(n.min(buf.len()));
    String::from_utf8_lossy(&buf).into_owned()
}

/// Loads `razer-hid-devices-mice` (or any other `cdylib` exporting a
/// `razer_hid_plugin_devices` entry point of the same shape) and
/// returns its catalog. Pass the crate's library name (e.g.
/// `"razer_hid_devices_mice"`), not a file path.
pub fn load_known_devices(crate_name: &str) -> Result<&'static [DeviceProfile], ClientError> {
    let lib = load_library(crate_name)?;
    let func: Symbol<FnPluginDevices> = unsafe { symbol(lib, "razer_hid_plugin_devices")? };
    let mut len = 0usize;
    let ptr = unsafe { func(&mut len) };
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// Enumerates which of `known_devices` are currently connected.
pub fn list_connected(
    known_devices: &'static [DeviceProfile],
) -> Result<Vec<&'static DeviceProfile>, ClientError> {
    let api = core_api()?;
    let mut out: Vec<*const DeviceProfile> = vec![std::ptr::null(); known_devices.len()];
    let n = unsafe {
        (api.list_connected)(
            known_devices.as_ptr(),
            known_devices.len(),
            out.as_mut_ptr(),
            out.len(),
        )
    };
    if n < 0 {
        return Err(ClientError::Device(last_error(api)));
    }
    Ok(out[..n as usize].iter().map(|p| unsafe { &**p }).collect())
}

/// Every raw HID interface under `vendor_id`, regardless of whether it
/// matches a known profile (`razer-ctl debug-usb`).
pub fn debug_list_interfaces(vendor_id: u16) -> Result<Vec<InterfaceInfo>, ClientError> {
    let api = core_api()?;
    let mut buf: Vec<FfiInterfaceInfo> = (0..64).map(|_| unsafe { std::mem::zeroed() }).collect();
    let n = unsafe { (api.debug_list_interfaces)(vendor_id, buf.as_mut_ptr(), buf.len()) };
    if n < 0 {
        return Err(ClientError::Device(last_error(api)));
    }
    Ok(buf[..n as usize]
        .iter()
        .map(|i| InterfaceInfo {
            vendor_id: i.vendor_id,
            product_id: i.product_id,
            interface_number: i.interface_number,
            usage_page: i.usage_page,
            usage: i.usage,
            product_string: i.has_product_string.then(|| {
                String::from_utf8_lossy(&i.product_string[..i.product_string_len as usize])
                    .into_owned()
            }),
            path: String::from_utf8_lossy(&i.path[..i.path_len as usize]).into_owned(),
        })
        .collect())
}

pub struct RazerDevice {
    handle: *mut c_void,
    pub profile: &'static DeviceProfile,
}

impl Drop for RazerDevice {
    fn drop(&mut self) {
        if let Ok(api) = core_api() {
            unsafe { (api.close)(self.handle) };
        }
    }
}

/// `in`/`out` are fixed at 512 bytes — generous for every op this
/// project currently defines (the largest is a full matrix row write,
/// and the longest string response is a firmware/serial string).
const BUF_CAP: usize = 512;

impl RazerDevice {
    pub fn open(profile: &'static DeviceProfile) -> Result<Self, ClientError> {
        let api = core_api()?;
        let handle = unsafe { (api.open)(profile) };
        if handle.is_null() {
            return Err(ClientError::Device(last_error(api)));
        }
        Ok(Self { handle, profile })
    }

    fn call(&self, op: Op, input: &[u8]) -> Result<Vec<u8>, ClientError> {
        let api = core_api()?;
        let mut out = vec![0u8; BUF_CAP];
        let n = unsafe {
            (api.call)(
                self.handle,
                op as u32,
                input.as_ptr(),
                input.len(),
                out.as_mut_ptr(),
                out.len(),
            )
        };
        if n < 0 {
            return Err(ClientError::Device(last_error(api)));
        }
        out.truncate(n as usize);
        Ok(out)
    }

    fn call_str(&self, op: Op) -> Result<String, ClientError> {
        let bytes = self.call(op, &[])?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn get_dpi(&self) -> Result<DpiSetting, ClientError> {
        let out = self.call(Op::GetDpi, &[])?;
        Ok(DpiSetting {
            x: u16::from_le_bytes(out[0..2].try_into().unwrap()),
            y: u16::from_le_bytes(out[2..4].try_into().unwrap()),
        })
    }

    pub fn set_dpi(&self, x: u16, y: u16) -> Result<(), ClientError> {
        let mut buf = [0u8; 4];
        buf[0..2].copy_from_slice(&x.to_le_bytes());
        buf[2..4].copy_from_slice(&y.to_le_bytes());
        self.call(Op::SetDpi, &buf)?;
        Ok(())
    }

    pub fn get_poll_rate(&self) -> Result<u32, ClientError> {
        let out = self.call(Op::GetPollRate, &[])?;
        Ok(u32::from_le_bytes(out[0..4].try_into().unwrap()))
    }

    pub fn set_poll_rate(&self, hz: u32) -> Result<(), ClientError> {
        self.call(Op::SetPollRate, &hz.to_le_bytes())?;
        Ok(())
    }

    /// Pure profile data — no FFI call needed, both sides share the
    /// same `DeviceProfile` layout at compile time.
    pub fn led_zones(&self) -> &'static [LedZoneProfile] {
        self.profile.leds
    }

    /// Pure profile data — see [`led_zones`](Self::led_zones).
    pub fn matrix_dims(&self) -> Option<(u8, u8)> {
        self.profile.matrix.map(|m| (m.rows, m.cols))
    }

    fn zone_payload(zone: &'static LedZoneProfile, tail: &[u8]) -> Vec<u8> {
        let mut buf = (zone as *const LedZoneProfile as usize)
            .to_le_bytes()
            .to_vec();
        buf.extend_from_slice(tail);
        buf
    }

    pub fn set_led_off(&self, zone: &'static LedZoneProfile) -> Result<(), ClientError> {
        self.call(Op::SetLedOff, &Self::zone_payload(zone, &[]))?;
        Ok(())
    }

    pub fn set_led_static(
        &self,
        zone: &'static LedZoneProfile,
        rgb: [u8; 3],
    ) -> Result<(), ClientError> {
        self.call(Op::SetLedStatic, &Self::zone_payload(zone, &rgb))?;
        Ok(())
    }

    pub fn set_led_breathing(
        &self,
        zone: &'static LedZoneProfile,
        rgb: [u8; 3],
    ) -> Result<(), ClientError> {
        self.call(Op::SetLedBreathing, &Self::zone_payload(zone, &rgb))?;
        Ok(())
    }

    pub fn set_led_spectrum(&self, zone: &'static LedZoneProfile) -> Result<(), ClientError> {
        self.call(Op::SetLedSpectrum, &Self::zone_payload(zone, &[]))?;
        Ok(())
    }

    pub fn set_led_wave(
        &self,
        zone: &'static LedZoneProfile,
        direction: u8,
    ) -> Result<(), ClientError> {
        self.call(Op::SetLedWave, &Self::zone_payload(zone, &[direction]))?;
        Ok(())
    }

    pub fn set_led_brightness(
        &self,
        zone: &'static LedZoneProfile,
        percent: u8,
    ) -> Result<(), ClientError> {
        self.call(Op::SetLedBrightness, &Self::zone_payload(zone, &[percent]))?;
        Ok(())
    }

    pub fn set_led_static_all(&self, rgb: [u8; 3]) -> Result<(), ClientError> {
        self.call(Op::SetLedStaticAll, &rgb)?;
        Ok(())
    }

    pub fn set_led_off_all(&self) -> Result<(), ClientError> {
        self.call(Op::SetLedOffAll, &[])?;
        Ok(())
    }

    pub fn set_led_breathing_all(&self, rgb: [u8; 3]) -> Result<(), ClientError> {
        self.call(Op::SetLedBreathingAll, &rgb)?;
        Ok(())
    }

    pub fn set_led_spectrum_all(&self) -> Result<(), ClientError> {
        self.call(Op::SetLedSpectrumAll, &[])?;
        Ok(())
    }

    pub fn set_led_wave_all(&self, direction: u8) -> Result<(), ClientError> {
        self.call(Op::SetLedWaveAll, &[direction])?;
        Ok(())
    }

    pub fn set_led_brightness_all(&self, percent: u8) -> Result<(), ClientError> {
        self.call(Op::SetLedBrightnessAll, &[percent])?;
        Ok(())
    }

    pub fn set_matrix_row(
        &self,
        row: u8,
        start_col: u8,
        colors: &[[u8; 3]],
    ) -> Result<(), ClientError> {
        let mut buf = vec![row, start_col];
        for c in colors {
            buf.extend_from_slice(c);
        }
        self.call(Op::SetMatrixRow, &buf)?;
        Ok(())
    }

    pub fn commit_matrix_frame(&self) -> Result<(), ClientError> {
        self.call(Op::CommitMatrixFrame, &[])?;
        Ok(())
    }

    pub fn fill_matrix(&self, rgb: [u8; 3]) -> Result<(), ClientError> {
        self.call(Op::FillMatrix, &rgb)?;
        Ok(())
    }

    pub fn set_matrix_spectrum(&self) -> Result<(), ClientError> {
        self.call(Op::SetMatrixSpectrum, &[])?;
        Ok(())
    }

    pub fn set_matrix_wave(&self, direction: u8) -> Result<(), ClientError> {
        self.call(Op::SetMatrixWave, &[direction])?;
        Ok(())
    }

    pub fn get_firmware_version(&self) -> Result<String, ClientError> {
        self.call_str(Op::GetFirmwareVersion)
    }

    pub fn get_serial(&self) -> Result<String, ClientError> {
        self.call_str(Op::GetSerial)
    }
}
