//! Unofficial, open-source control library for Razer HID peripherals.
//!
//! Not affiliated with or endorsed by Razer Inc. Built from the public
//! Razer Chroma HID protocol reverse-engineered by the OpenRazer community.
//!
//! The crate is split into:
//! - [`protocol`]: byte-level framing of a Razer HID control report (the
//!   same for every device).
//! - [`commands`]: the shared command-class/id vocabulary most Razer mice
//!   reuse; a starting point for writing device profiles.
//! - [`devices`]: the registry of supported models — this is where you add
//!   support for a new device, see README.md's "Adding support for your
//!   device" and `devices/_template.rs`.
//! - [`device`]: the generic [`device::RazerDevice`] control handle, driven
//!   by whichever profile matches the connected hardware.

pub mod commands;
pub mod constants;
pub mod device;
pub mod devices;
pub mod protocol;

pub use device::{DeviceError, DpiSetting, RazerDevice};
pub use devices::{DeviceKind, DeviceProfile, KNOWN_DEVICES};
pub use protocol::{ProtocolError, RazerReport};
