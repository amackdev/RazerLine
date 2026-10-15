//! Device profile for the Razer Basilisk V3 (wired).
//!
//! Confidence: **unverified against hardware by the maintainers of this
//! repo** — these are the standard mouse command bytes documented by the
//! OpenRazer project, applied to this model's VID/PID. If you own this
//! mouse and these work (or don't), please say so in an issue/PR; see
//! `docs/PROTOCOL.md` for how to confirm with a USB capture.

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
        min: 50,
        max: 26000,
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
    ],
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
    }),
};
