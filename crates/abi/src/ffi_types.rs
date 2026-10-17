//! `#[repr(C)]` data shapes that cross the `razer-hid-core` FFI
//! boundary as raw memory (as opposed to the `Op` dispatch payloads,
//! which are just byte buffers) — shared here so `razer-hid-client` can
//! decode exactly what `razer-hid-core`'s `ffi` module writes.

/// Fixed-layout mirror of one HID interface for interface diagnostics.
/// `product_string`/`path` are truncated to their buffer size rather than
/// allocated, since this is a diagnostic aid, not a hot path.
#[repr(C)]
pub struct FfiInterfaceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub has_product_string: bool,
    pub product_string: [u8; 128],
    pub product_string_len: u8,
    pub path: [u8; 256],
    pub path_len: u16,
}
