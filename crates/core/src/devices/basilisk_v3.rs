//! Device profile for the Razer Basilisk V3 (wired).
//!
//! Confidence: the command classes/ids/argument layouts below were
//! cross-checked line-by-line against the OpenRazer kernel driver source
//! (`driver/razermouse_driver.c` and `razerchromacommon.c`, GPL-3.0),
//! which explicitly lists `USB_DEVICE_ID_RAZER_BASILISK_V3` for each of
//! these code paths — this isn't "standard mouse bytes applied on spec,"
//! it's the exact bytes OpenRazer's Linux driver sends to this model.
//! Still marked `Unverified` here because no maintainer of *this* repo
//! has run it against real hardware yet. If you have, please say so (and
//! what did/didn't work) in an issue — see `docs/PROTOCOL.md`.

use super::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    PollRateProfile,
};
use crate::commands::{CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand};
use crate::constants::RAZER_VENDOR_ID;

pub const BASILISK_V3: DeviceProfile = DeviceProfile {
    name: "Razer Basilisk V3",
    vendor_id: RAZER_VENDOR_ID,
    // Verify with `lsusb -d 1532:` (Linux) or Device Manager (Windows) —
    // Razer has shipped more than one PID for variants of this mouse.
    product_id: 0x0099,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        // OpenRazer clamps the raw protocol value to 100-45000 for every
        // device that takes this command shape; the sensor's actual
        // practical ceiling is lower, but this matches what the firmware
        // itself will accept without silently reinterpreting the bytes.
        min: 100,
        max: 45000,
    }),
    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        supported_hz: &[125, 500, 1000],
    }),
    leds: &[
        LedZoneProfile {
            id: CommonLedId::ScrollWheel as u8,
            name: "Scroll wheel",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
        },
        LedZoneProfile {
            id: CommonLedId::Logo as u8,
            name: "Logo",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
        },
        // The third zone: this mouse's bottom/underglow strip, which
        // OpenRazer (and this protocol generally) calls "Backlight"
        // (BACKLIGHT_LED = 0x05 in razercommon.h) — it's the same zone id
        // the static-effect example in docs/PROTOCOL.md and the
        // regression test in protocol.rs are built from.
        LedZoneProfile {
            id: CommonLedId::Backlight as u8,
            name: "Underglow",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
        },
    ],
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
    }),
};
