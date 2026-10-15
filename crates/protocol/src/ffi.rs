//! `extern "C"` entry points `razer-hid-core` calls via `libloading`
//! after dlopen'ing this crate's `cdylib` artifact at runtime. Only two
//! operations exist (pack a command into a report, unpack a raw report
//! read back from the device), so these are plain 1:1 wrappers rather
//! than going through the dispatch-op convention `razer-hid-core`
//! itself uses for its much larger `RazerDevice` surface.

use razer_hid_abi::constants::{ARGS_LENGTH, REPORT_LENGTH};

use crate::protocol::RazerReport;

/// Packs one command into a [`REPORT_LENGTH`]-byte report, writing it
/// into `out_ptr` (caller-owned, must be at least `REPORT_LENGTH` bytes).
/// Returns `0` on success, `-1` if `out_cap` is too small, `-2` if
/// `args_len` exceeds [`razer_hid_abi::constants::ARGS_LENGTH`].
///
/// # Safety
/// `args_ptr` must be valid for `args_len` bytes; `out_ptr` for `out_cap`.
#[no_mangle]
pub unsafe extern "C" fn razer_protocol_pack(
    command_class: u8,
    command_id: u8,
    transaction_id: u8,
    args_ptr: *const u8,
    args_len: usize,
    out_ptr: *mut u8,
    out_cap: usize,
) -> i32 {
    if out_cap < REPORT_LENGTH {
        return -1;
    }
    let args = if args_len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(args_ptr, args_len).to_vec()
    };
    let mut report = match RazerReport::new(command_class, command_id, args) {
        Ok(r) => r,
        Err(_) => return -2,
    };
    report.transaction_id = transaction_id;
    let packed = report.pack();
    std::ptr::copy_nonoverlapping(packed.as_ptr(), out_ptr, REPORT_LENGTH);
    0
}

/// Fixed-layout mirror of [`RazerReport`] for crossing the FFI boundary
/// without needing `Vec`. `arguments`/`arguments_len` follow the same
/// convention as the real struct: only the first `arguments_len` bytes
/// of `arguments` are meaningful.
#[repr(C)]
pub struct FfiRazerReport {
    pub status: u8,
    pub transaction_id: u8,
    pub command_class: u8,
    pub command_id: u8,
    pub arguments_len: u8,
    pub arguments: [u8; ARGS_LENGTH],
}

/// Unpacks a raw report read back from the device. Returns `0` on
/// success, `-1` if `buf_len` is shorter than [`REPORT_LENGTH`].
///
/// # Safety
/// `buf_ptr` must be valid for `buf_len` bytes; `out` must point to a
/// valid, writable `FfiRazerReport`.
#[no_mangle]
pub unsafe extern "C" fn razer_protocol_unpack(
    buf_ptr: *const u8,
    buf_len: usize,
    out: *mut FfiRazerReport,
) -> i32 {
    let buf = std::slice::from_raw_parts(buf_ptr, buf_len);
    let report = match RazerReport::unpack(buf) {
        Ok(r) => r,
        Err(_) => return -1,
    };
    let mut arguments = [0u8; ARGS_LENGTH];
    let n = report.arguments.len().min(ARGS_LENGTH);
    arguments[..n].copy_from_slice(&report.arguments[..n]);
    (*out) = FfiRazerReport {
        status: report.status,
        transaction_id: report.transaction_id,
        command_class: report.command_class,
        command_id: report.command_id,
        arguments_len: n as u8,
        arguments,
    };
    0
}
