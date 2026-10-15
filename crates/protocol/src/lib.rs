//! Byte-level Razer HID control-report protocol: framing/CRC
//! ([`protocol`]) and the shared command-class/id vocabulary
//! ([`commands`]) most Razer peripherals reuse. No transport
//! dependency (no `hidapi`) — this crate only knows about bytes, not
//! USB. [`razer_hid_core`](https://docs.rs/razer-hid-core) builds the
//! actual device control handle on top of it.

pub mod commands;
pub mod constants;
pub mod protocol;

pub use protocol::{ProtocolError, RazerReport};
