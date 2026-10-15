//! Compile-time-shared vocabulary between the loaded DLLs
//! (`razer-hid-protocol`, `razer-hid-core`, `razer-hid-devices-mice`)
//! and their loader (`razer-hid-client`, used by the CLI/GUI). Think of
//! it like a C header: pure data shapes and constants, no logic, no
//! platform/transport dependency — everyone depends on this at compile
//! time; nothing here is itself loaded as a DLL.

pub mod commands;
pub mod constants;
pub mod ffi_types;
pub mod op;
pub mod profile;

pub use ffi_types::FfiInterfaceInfo;
pub use op::Op;
pub use profile::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    MatrixPacketLength, MatrixProfile, PollRateProfile,
};
