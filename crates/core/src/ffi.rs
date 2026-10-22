//! Dispatch-style FFI boundary: `razer-hid-client` dlopen's this crate's
//! `cdylib` artifact and calls through these functions instead of
//! `RazerDevice`'s normal Rust methods directly, since this crate isn't
//! a compile-time dependency of the CLI/GUI anymore — everything past
//! [`razer_core_call`] stays a normal, panic-safe Rust call internally.
//! See [`razer_hid_abi::Op`] for the per-operation wire format this
//! decodes. Zone-targeted ops take a pointer to the caller's
//! `LedZoneProfile` rather than re-deriving it from an id — both sides
//! compile against the identical `razer-hid-abi` type, and the pointer
//! is into the device-catalog plugin's `'static` data, so this is just
//! a pointer handoff, not a cross-ABI struct copy.

use std::os::raw::c_void;

use razer_hid_abi::{DeviceProfile, FfiInterfaceInfo, LedZoneProfile, Op};

use crate::device::RazerDevice;

thread_local! {
    static LAST_ERROR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn set_last_error(msg: impl Into<String>) {
    LAST_ERROR.with(|e| *e.borrow_mut() = msg.into());
}

/// Writes the last error message (UTF-8, not nul-terminated) into `buf`,
/// truncated to `cap` bytes. Returns the full message's byte length
/// (which may exceed what was actually written if `cap` was too small).
///
/// # Safety
/// `buf` must be valid for `cap` bytes.
#[no_mangle]
pub unsafe extern "C" fn razer_core_last_error(buf: *mut u8, cap: usize) -> usize {
    LAST_ERROR.with(|e| {
        let msg = e.borrow();
        let bytes = msg.as_bytes();
        let n = bytes.len().min(cap);
        if n > 0 {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
        }
        bytes.len()
    })
}

/// Opens `profile` (a pointer into the device-catalog plugin's static
/// data, valid for the process's lifetime). Returns an opaque handle for
/// [`razer_core_call`]/[`razer_core_close`], or null on failure (see
/// [`razer_core_last_error`]).
///
/// # Safety
/// `profile` must point to a valid `DeviceProfile` that outlives the
/// returned handle.
#[no_mangle]
pub unsafe extern "C" fn razer_core_open(profile: *const DeviceProfile) -> *mut c_void {
    let profile: &'static DeviceProfile = &*profile;
    match RazerDevice::open(profile) {
        Ok(dev) => Box::into_raw(Box::new(dev)) as *mut c_void,
        Err(e) => {
            set_last_error(e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Frees a handle returned by [`razer_core_open`]. Must not be called
/// twice, or on a null/foreign pointer.
///
/// # Safety
/// `handle` must be a live pointer previously returned by
/// `razer_core_open` and not yet freed.
#[no_mangle]
pub unsafe extern "C" fn razer_core_close(handle: *mut c_void) {
    if !handle.is_null() {
        drop(Box::from_raw(handle as *mut RazerDevice));
    }
}

/// Fills `out_ptr` (capacity `out_cap` entries) with pointers to every
/// profile in `devices_ptr`/`devices_len` that's currently connected.
/// Returns the count written, or `-1` on a HID enumeration error (see
/// [`razer_core_last_error`]).
///
/// # Safety
/// `devices_ptr` must be valid for `devices_len` entries; `out_ptr` for
/// `out_cap` entries.
#[no_mangle]
pub unsafe extern "C" fn razer_core_list_connected(
    devices_ptr: *const DeviceProfile,
    devices_len: usize,
    out_ptr: *mut *const DeviceProfile,
    out_cap: usize,
) -> i64 {
    let devices: &'static [DeviceProfile] = std::slice::from_raw_parts(devices_ptr, devices_len);
    match RazerDevice::list_connected(devices) {
        Ok(found) => {
            let n = found.len().min(out_cap);
            for (i, p) in found.into_iter().take(n).enumerate() {
                *out_ptr.add(i) = p as *const DeviceProfile;
            }
            n as i64
        }
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

fn copy_truncated(src: &str, dst: &mut [u8]) -> usize {
    let n = src.len().min(dst.len());
    dst[..n].copy_from_slice(&src.as_bytes()[..n]);
    n
}

/// Fills `out_ptr` (capacity `out_cap` entries) with every raw HID
/// interface under `vendor_id`, regardless of whether it matches a
/// known profile. Returns the count written, or `-1` on a HID
/// enumeration error (see [`razer_core_last_error`]).
///
/// # Safety
/// `out_ptr` must be valid for `out_cap` entries.
#[no_mangle]
pub unsafe extern "C" fn razer_core_debug_list_interfaces(
    vendor_id: u16,
    out_ptr: *mut FfiInterfaceInfo,
    out_cap: usize,
) -> i64 {
    match RazerDevice::debug_list_interfaces(vendor_id) {
        Ok(found) => {
            let n = found.len().min(out_cap);
            for (i, info) in found.into_iter().take(n).enumerate() {
                let mut ffi = FfiInterfaceInfo {
                    vendor_id: info.vendor_id,
                    product_id: info.product_id,
                    interface_number: info.interface_number,
                    usage_page: info.usage_page,
                    usage: info.usage,
                    has_product_string: info.product_string.is_some(),
                    product_string: [0u8; 128],
                    product_string_len: 0,
                    path: [0u8; 256],
                    path_len: 0,
                };
                if let Some(s) = &info.product_string {
                    ffi.product_string_len = copy_truncated(s, &mut ffi.product_string) as u8;
                }
                ffi.path_len = copy_truncated(&info.path, &mut ffi.path) as u16;
                *out_ptr.add(i) = ffi;
            }
            n as i64
        }
        Err(e) => {
            set_last_error(e.to_string());
            -1
        }
    }
}

/// Runs one [`Op`] against an open device handle. `in_ptr`/`in_len`
/// carry the op-specific input payload; `out_ptr`/`out_cap` receive the
/// op-specific output payload. Returns the number of bytes written to
/// `out_ptr` (>= 0) on success, or a negative value on failure (see
/// [`razer_core_last_error`]). A malformed/mismatched payload is caught
/// as an internal panic rather than unwinding across the FFI boundary.
///
/// # Safety
/// `handle` must be a live pointer from `razer_core_open`. `in_ptr` must
/// be valid for `in_len` bytes, `out_ptr` for `out_cap` bytes.
#[no_mangle]
pub unsafe extern "C" fn razer_core_call(
    handle: *mut c_void,
    op: u32,
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
) -> i64 {
    let dev = &*(handle as *const RazerDevice);
    let input = std::slice::from_raw_parts(in_ptr, in_len);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dispatch(dev, op, input, out_ptr, out_cap)
    }));
    match result {
        Ok(Ok(n)) => n as i64,
        Ok(Err(e)) => {
            set_last_error(e);
            -1
        }
        Err(_) => {
            set_last_error("internal panic handling this op (likely a malformed payload)");
            -2
        }
    }
}

/// Reconstructs the `'static` [`LedZoneProfile`] reference a zone-scoped
/// op's payload starts with — see the module doc for why a raw pointer
/// handoff is sound here.
fn zone_from_ptr(bytes: &[u8]) -> &'static LedZoneProfile {
    let raw = usize::from_le_bytes(bytes[0..8].try_into().unwrap());
    unsafe { &*(raw as *const LedZoneProfile) }
}

fn write_out(out_ptr: *mut u8, out_cap: usize, data: &[u8]) -> Result<usize, String> {
    if data.len() > out_cap {
        return Err(format!(
            "output buffer too small: need {}, have {out_cap}",
            data.len()
        ));
    }
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), out_ptr, data.len()) };
    Ok(data.len())
}

fn dispatch(
    dev: &RazerDevice,
    op: u32,
    input: &[u8],
    out_ptr: *mut u8,
    out_cap: usize,
) -> Result<usize, String> {
    let op = decode_op(op)?;
    match op {
        Op::GetDpi => {
            let dpi = dev.get_dpi().map_err(|e| e.to_string())?;
            let mut buf = [0u8; 4];
            buf[0..2].copy_from_slice(&dpi.x.to_le_bytes());
            buf[2..4].copy_from_slice(&dpi.y.to_le_bytes());
            write_out(out_ptr, out_cap, &buf)
        }
        Op::SetDpi => {
            let x = u16::from_le_bytes(input[0..2].try_into().unwrap());
            let y = u16::from_le_bytes(input[2..4].try_into().unwrap());
            dev.set_dpi(x, y).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::GetPollRate => {
            let hz = dev.get_poll_rate().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, &hz.to_le_bytes())
        }
        Op::SetPollRate => {
            let hz = u32::from_le_bytes(input[0..4].try_into().unwrap());
            dev.set_poll_rate(hz).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedOff => {
            dev.set_led_off(zone_from_ptr(input))
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedStatic => {
            let rgb = [input[8], input[9], input[10]];
            dev.set_led_static(zone_from_ptr(input), rgb)
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedBreathing => {
            let rgb = [input[8], input[9], input[10]];
            dev.set_led_breathing(zone_from_ptr(input), rgb)
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedSpectrum => {
            dev.set_led_spectrum(zone_from_ptr(input))
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedWave => {
            let direction = input[8];
            dev.set_led_wave(zone_from_ptr(input), direction)
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedBrightness => {
            let percent = input[8];
            dev.set_led_brightness(zone_from_ptr(input), percent)
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedStaticAll => {
            let rgb = [input[0], input[1], input[2]];
            dev.set_led_static_all(rgb).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedOffAll => {
            dev.set_led_off_all().map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedBreathingAll => {
            let rgb = [input[0], input[1], input[2]];
            dev.set_led_breathing_all(rgb).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedSpectrumAll => {
            dev.set_led_spectrum_all().map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedWaveAll => {
            dev.set_led_wave_all(input[0]).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetLedBrightnessAll => {
            dev.set_led_brightness_all(input[0])
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetMatrixRow => {
            let row = input[0];
            let start_col = input[1];
            // `as_chunks` (clippy's suggested replacement) is still
            // unstable on some toolchains this needs to build with —
            // keep `chunks_exact` and silence the lint instead. The
            // lint itself doesn't exist on older clippy, hence
            // `unknown_lints` too.
            #[allow(unknown_lints)]
            #[allow(clippy::chunks_exact_to_as_chunks)]
            let colors: Vec<[u8; 3]> = input[2..]
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            dev.set_matrix_row(row, start_col, &colors)
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::CommitMatrixFrame => {
            dev.commit_matrix_frame().map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::FillMatrix => {
            let rgb = [input[0], input[1], input[2]];
            dev.fill_matrix(rgb).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetMatrixSpectrum => {
            dev.set_matrix_spectrum().map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::SetMatrixWave => {
            dev.set_matrix_wave(input[0]).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::GetFirmwareVersion => {
            let s = dev.get_firmware_version().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, s.as_bytes())
        }
        Op::GetSerial => {
            let s = dev.get_serial().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, s.as_bytes())
        }
        Op::GetBatteryLevel => {
            let percent = dev.get_battery_level().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, &[percent])
        }
        Op::GetChargingStatus => {
            let charging = dev.get_charging_status().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, &[charging as u8])
        }
        Op::GetIdleTime => {
            let seconds = dev.get_idle_time().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, &seconds.to_le_bytes())
        }
        Op::SetIdleTime => {
            let seconds = u16::from_le_bytes(input[0..2].try_into().unwrap());
            dev.set_idle_time(seconds).map_err(|e| e.to_string())?;
            Ok(0)
        }
        Op::GetLowBatteryThreshold => {
            let percent = dev.get_low_battery_threshold().map_err(|e| e.to_string())?;
            write_out(out_ptr, out_cap, &[percent])
        }
        Op::SetLowBatteryThreshold => {
            let percent = input[0];
            dev.set_low_battery_threshold(percent)
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
    }
}

fn decode_op(raw: u32) -> Result<Op, String> {
    const ALL: &[Op] = &[
        Op::GetDpi,
        Op::SetDpi,
        Op::GetPollRate,
        Op::SetPollRate,
        Op::SetLedOff,
        Op::SetLedStatic,
        Op::SetLedBreathing,
        Op::SetLedSpectrum,
        Op::SetLedWave,
        Op::SetLedBrightness,
        Op::SetLedStaticAll,
        Op::SetLedOffAll,
        Op::SetLedBreathingAll,
        Op::SetLedSpectrumAll,
        Op::SetLedWaveAll,
        Op::SetLedBrightnessAll,
        Op::SetMatrixRow,
        Op::CommitMatrixFrame,
        Op::FillMatrix,
        Op::SetMatrixSpectrum,
        Op::SetMatrixWave,
        Op::GetFirmwareVersion,
        Op::GetSerial,
        Op::GetBatteryLevel,
        Op::GetChargingStatus,
        Op::GetIdleTime,
        Op::SetIdleTime,
        Op::GetLowBatteryThreshold,
        Op::SetLowBatteryThreshold,
    ];
    ALL.iter()
        .copied()
        .find(|op| *op as u32 == raw)
        .ok_or_else(|| format!("unknown op code {raw}"))
}
