//! Unofficial, open-source control library for Razer HID peripherals.
//!
//! Not affiliated with or endorsed by Razer Inc. Built from the public
//! Razer Chroma HID protocol reverse-engineered by the OpenRazer community.
//!
//! This crate is the generic control engine: [`device::RazerDevice`] and
//! the [`profile::DeviceProfile`] shape it's driven by. It depends on
//! `razer-hid-protocol` for byte-level framing/command vocabulary, but
//! deliberately knows nothing about any *specific* device — the catalog
//! of concrete profiles (one per supported model) lives in the separate
//! `razer-hid-devices` crate, which depends on this one. That split lets
//! the device database grow independently without the control engine
//! needing to change.

pub mod device;
pub mod profile;

pub use device::{DeviceError, DpiSetting, InterfaceInfo, RazerDevice};
pub use profile::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    MatrixPacketLength, MatrixProfile, PollRateProfile,
};
