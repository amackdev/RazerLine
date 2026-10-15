//! Byte-level framing/CRC for a Razer HID control report ([`protocol`]).
//! No transport dependency (no `hidapi`) — this crate only knows about
//! bytes, not USB.
//!
//! Compiled as a `cdylib` and loaded at runtime by `razer-hid-core`
//! (not linked at compile time) — see [`ffi`] for the two exported
//! entry points. Command vocabulary and other compile-time-shared
//! shapes live in `razer-hid-abi`, not here.

pub mod ffi;
pub mod protocol;

pub use protocol::{ProtocolError, RazerReport};
