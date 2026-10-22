//! Operation codes for `razer-hid-core`'s dispatch-style FFI entry point
//! (`razer_core_call`). The core control engine is loaded as a DLL at
//! runtime rather than linked at compile time, so its ~20-odd
//! `RazerDevice` methods can't cross the boundary as ordinary Rust
//! function calls — there's one shared entry point instead, keyed by
//! one of these codes, with a small fixed binary layout per op for the
//! input/output byte buffers (documented next to each variant).
//! `razer-hid-client` encodes calls against this; `razer-hid-core`
//! decodes and handles them. Both compile against this same enum, so
//! the numbering only needs to stay internally consistent, not
//! cross-version stable.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// in: none. out: x:u16 LE, y:u16 LE (4 bytes).
    GetDpi = 1,
    /// in: x:u16 LE, y:u16 LE (4 bytes). out: none.
    SetDpi = 2,
    /// in: none. out: hz:u32 LE (4 bytes).
    GetPollRate = 3,
    /// in: hz:u32 LE (4 bytes). out: none.
    SetPollRate = 4,
    /// in: zone_ptr:usize LE (8 bytes, a `*const LedZoneProfile`). out: none.
    SetLedOff = 5,
    /// in: zone_ptr:usize LE (8) + r,g,b (3) = 11 bytes. out: none.
    SetLedStatic = 6,
    /// in: zone_ptr:usize LE (8) + r,g,b (3) = 11 bytes. out: none.
    SetLedBreathing = 7,
    /// in: zone_ptr:usize LE (8 bytes). out: none.
    SetLedSpectrum = 8,
    /// in: zone_ptr:usize LE (8) + direction:u8 (1) = 9 bytes. out: none.
    SetLedWave = 9,
    /// in: zone_ptr:usize LE (8) + percent:u8 (1) = 9 bytes. out: none.
    SetLedBrightness = 10,
    /// in: r:u8, g:u8, b:u8 (3 bytes). out: none.
    SetLedStaticAll = 11,
    /// in: none. out: none.
    SetLedOffAll = 12,
    /// in: r:u8, g:u8, b:u8 (3 bytes). out: none.
    SetLedBreathingAll = 13,
    /// in: none. out: none.
    SetLedSpectrumAll = 14,
    /// in: direction:u8 (1 byte). out: none.
    SetLedWaveAll = 15,
    /// in: percent:u8 (1 byte). out: none.
    SetLedBrightnessAll = 16,
    /// in: row:u8, start_col:u8, then N*3 bytes of RGB triples. out: none.
    SetMatrixRow = 17,
    /// in: none. out: none.
    CommitMatrixFrame = 18,
    /// in: r:u8, g:u8, b:u8 (3 bytes). out: none.
    FillMatrix = 19,
    /// in: none. out: none.
    SetMatrixSpectrum = 20,
    /// in: direction:u8 (1 byte). out: none.
    SetMatrixWave = 21,
    /// in: none. out: UTF-8 bytes of the version string.
    GetFirmwareVersion = 22,
    /// in: none. out: UTF-8 bytes of the serial string.
    GetSerial = 23,
    /// in: none. out: percent:u8 (1 byte, 0-100).
    GetBatteryLevel = 24,
    /// in: none. out: charging:u8 (1 byte, 0 or 1).
    GetChargingStatus = 25,
}
