//! TEMPLATE — copy this file to add support for a new device.
//!
//! This file is intentionally not wired into `mod.rs`'s `KNOWN_DEVICES`,
//! so it does not compile as part of the crate. Steps to add a real
//! device:
//!
//! 1. Copy this file to `your_device_name.rs` in this directory.
//! 2. Find your device's vendor/product id (`lsusb -d 1532:` on Linux,
//!    Device Manager on Windows).
//! 3. Fill in which capabilities it actually has, and the command bytes
//!    that drive them. Start from the shared vocabulary in
//!    `crate::commands` — many Razer mice reuse the same bytes — then
//!    verify with a USB capture (see `docs/PROTOCOL.md`) and correct any
//!    that don't match.
//! 4. Add `mod your_device_name;` and `pub use your_device_name::YOUR_DEVICE;`
//!    to `devices/mod.rs`, and add `YOUR_DEVICE` to the `KNOWN_DEVICES`
//!    slice.
//! 5. Set `confidence: Confidence::Verified` only once you've confirmed
//!    the commands actually work on real hardware.
//! 6. Open a PR. Mention what you tested and what you couldn't.
//!
//! If your device is a keyboard or other non-mouse class whose
//! capabilities don't fit [`DpiProfile`]/[`PollRateProfile`]/
//! [`LedZoneProfile`] (e.g. per-key matrix lighting), that's a sign the
//! capability model in `devices/mod.rs` needs a new type — propose it in
//! your PR rather than forcing your device to fit the mouse-shaped one.

#![allow(dead_code, unused_imports)]

use super::{Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile, PollRateProfile};
use crate::commands::{CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand};
use crate::constants::RAZER_VENDOR_ID;

pub const YOUR_DEVICE: DeviceProfile = DeviceProfile {
    name: "Razer Example Mouse",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x0000, // TODO: fill in the real product id
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,

    // Set to `None` for any capability your device doesn't have.
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 20000, // TODO: check your sensor's actual max DPI
    }),

    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        supported_hz: &[125, 500, 1000],
    }),

    // One entry per independently controllable RGB zone. Empty slice if none.
    leds: &[LedZoneProfile {
        id: CommonLedId::Logo as u8,
        name: "Logo",
        command_class: CommandClass::Led as u8,
        effect_command_id: LedCommand::SetEffect as u8,
        brightness_command_id: Some(LedCommand::SetBrightness as u8),
    }],

    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
    }),
};
