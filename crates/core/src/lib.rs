//! Unofficial, open-source control engine for Razer HID peripherals.
//!
//! Not affiliated with or endorsed by Razer Inc. Built from the public
//! Razer Chroma HID protocol reverse-engineered by the OpenRazer community.
//!
//! This crate is the generic control engine: [`device::RazerDevice`],
//! driven by whichever [`razer_hid_abi::DeviceProfile`] matches the
//! connected hardware. It's compiled as a `cdylib` and loaded at
//! runtime (not linked at compile time) by `razer-hid-client` — see
//! [`ffi`] for the dispatch-style entry points that boundary crosses
//! through. Device-shape types and command vocabulary live in
//! `razer-hid-abi`, which both this crate and its caller compile
//! against directly; the catalog of concrete device profiles lives in
//! the separate `razer-hid-devices-mice` crate (also loaded at
//! runtime), not here.

pub mod device;
pub mod ffi;

pub use device::{DeviceError, DpiSetting, InterfaceInfo, RazerDevice};
