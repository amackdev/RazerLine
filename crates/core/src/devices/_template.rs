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
//!    verify with a USB capture (Wireshark + `usbmon` on Linux, USBPcap
//!    on Windows) and correct any that don't match.
//! 4. **Check the transaction id per command type, not just per device.**
//!    Some devices (e.g. the Viper Ultimate) use a different transaction
//!    id for LED commands than for DPI/poll-rate/identity — don't assume
//!    one constant covers everything just because it did for the last
//!    device you added. Grep OpenRazer's `razermouse_driver.c` for your
//!    exact `USB_DEVICE_ID_RAZER_*` PID in each relevant switch statement.
//! 5. Add `mod your_device_name;` and `pub use your_device_name::YOUR_DEVICE;`
//!    to `devices/mod.rs`, and add `YOUR_DEVICE` to the `KNOWN_DEVICES`
//!    slice.
//! 6. Set `confidence: Confidence::Verified` only once you've confirmed
//!    the commands actually work on real hardware.
//! 7. Open a PR. Mention what you tested and what you couldn't.
//!
//! If your device has a multi-LED addressable strip (anything that isn't
//! a single-color zone — e.g. mouse underglow, keyboard per-key RGB),
//! that needs [`MatrixProfile`] instead of (or alongside) [`LedZoneProfile`]
//! — see the commented-out example below and `basilisk_v3.rs` for a real
//! one. Don't guess the LED count: check OpenRazer's daemon metadata
//! (`daemon/openrazer_daemon/hardware/*.py`, the `MATRIX_DIMS` field for
//! your device's class) rather than assuming a round number.
//!
//! If your device needs a capability that doesn't fit any of
//! [`DpiProfile`]/[`PollRateProfile`]/[`LedZoneProfile`]/[`MatrixProfile`]
//! at all, that's a sign the model in `devices/mod.rs` needs a new type —
//! propose it in your PR rather than forcing your device to fit.

#![allow(dead_code, unused_imports)]

use super::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    MatrixPacketLength, MatrixProfile, PollRateProfile,
};
use crate::commands::{CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand};
use crate::constants::RAZER_VENDOR_ID;

// If every command for your device really does use the same transaction
// id (true for the Basilisk V3, not true for the Viper Ultimate), one
// constant like this is fine. Otherwise use separate constants per
// command type and say why in a comment, the way viper_ultimate.rs does.
const TRANSACTION_ID: u8 = 0x1F;

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
        transaction_id: TRANSACTION_ID,
    }),

    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        get_command_id: GeneralCommand::GetPollRate as u8,
        supported_hz: &[125, 500, 1000],
        transaction_id: TRANSACTION_ID,
    }),

    // One entry per independently controllable single-color RGB zone.
    // Empty slice if none.
    leds: &[LedZoneProfile {
        id: CommonLedId::Logo as u8,
        name: "Logo",
        command_class: CommandClass::Led as u8,
        effect_command_id: LedCommand::SetEffect as u8,
        brightness_command_id: Some(LedCommand::SetBrightness as u8),
        transaction_id: TRANSACTION_ID,
    }],

    // Uncomment and fill in if your device has a multi-LED addressable
    // strip. `write_packet_length` matters: some devices need a fixed
    // value regardless of how many columns are written (Basilisk V3
    // uses a fixed 0x47); others (e.g. Mamba Elite, Naga Pro) compute it
    // from the row length (`MatrixPacketLength::Computed`) — check the
    // real source, don't assume either one.
    //
    // matrix: Some(MatrixProfile {
    //     rows: 1,
    //     cols: 0, // TODO: real LED count, from OpenRazer's MATRIX_DIMS
    //     write_command_class: CommandClass::Led as u8,
    //     write_command_id: LedCommand::SetCustomFrame as u8,
    //     write_packet_length: MatrixPacketLength::Fixed(0x47),
    //     write_transaction_id: TRANSACTION_ID,
    //     commit_command_class: CommandClass::Led as u8,
    //     commit_command_id: LedCommand::SetEffect as u8,
    //     commit_transaction_id: TRANSACTION_ID,
    // }),
    matrix: None,

    // Firmware and serial queries can use *different* transaction ids on
    // the same device (e.g. the Mamba Elite) — don't assume they match.
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
        firmware_transaction_id: TRANSACTION_ID,
        serial_transaction_id: TRANSACTION_ID,
    }),
};
