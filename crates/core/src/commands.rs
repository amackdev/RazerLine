//! Shared command-class/command-id vocabulary for Razer's HID control
//! protocol, as reverse-engineered by the OpenRazer community (GPL-3.0)
//! and widely re-confirmed across many Razer mice.
//!
//! These values are a *starting point* for writing a new device profile
//! under [`crate::devices`], not a guarantee that any specific model uses
//! them. A surprising number of Razer mice do share these exact bytes, but
//! always verify against your own hardware with a USB capture before
//! trusting a new profile — see `docs/PROTOCOL.md`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandClass {
    General = 0x00,
    Mouse = 0x04,
    Led = 0x0F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GeneralCommand {
    SetPollRate = 0x05,
    GetPollRate = 0x85,
    GetFirmwareVersion = 0x81,
    GetSerial = 0x82,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseCommand {
    SetDpiXY = 0x05,
    GetDpiXY = 0x85,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedCommand {
    SetEffect = 0x02,
    SetBrightness = 0x04,
    GetBrightness = 0x84,
}

/// Common LED zone ids. A device profile is free to define its own zone
/// ids instead (see [`crate::devices::LedZoneProfile`]) if its firmware
/// numbers them differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommonLedId {
    ScrollWheel = 0x01,
    Logo = 0x04,
    Backlight = 0x00,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedEffect {
    Off = 0x00,
    Static = 0x01,
    Breathing = 0x02,
    Wave = 0x03,
    Spectrum = 0x04,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PollRateHz {
    Hz125 = 0x08,
    Hz500 = 0x02,
    Hz1000 = 0x01,
}

impl PollRateHz {
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            125 => Some(Self::Hz125),
            500 => Some(Self::Hz500),
            1000 => Some(Self::Hz1000),
            _ => None,
        }
    }
}
