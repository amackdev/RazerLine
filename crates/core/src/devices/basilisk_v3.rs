//! Device profile for the Razer Basilisk V3 (wired).
//!
//! Confirmed against real hardware: DPI get/set, polling rate, and the
//! Static/Spectrum/Wave LED effects plus brightness on both the scroll
//! wheel and logo zones. Also confirmed on real hardware: Breathing and
//! Off genuinely get rejected by the firmware (`STATUS_NOT_SUPPORTED`/
//! `STATUS_FAILURE`) — not a bug here, OpenRazer's own driver doesn't
//! attempt them for this exact PID either. The third LED (bottom/
//! underglow) is deliberately not listed as a zone — see the comment
//! below and `docs/PROTOCOL.md`.

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
    confidence: Confidence::Verified,
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
        // This mouse actually has a third LED (bottom/underglow), but it
        // is deliberately NOT listed here — real hardware rejects the
        // simple single-zone effect command (class 0x0F id 0x02) for it
        // with STATUS_FAILURE, confirmed against a real Basilisk V3.
        // OpenRazer's own driver never attempts this either: none of its
        // `USB_DEVICE_ID_RAZER_BASILISK_V3` probe-time cases fall inside
        // the block that creates `backlight_matrix_effect_*` sysfs files,
        // for any PID. The underglow is most likely only addressable
        // through the separate per-pixel "custom frame" matrix command
        // (class 0x0F id 0x08, `razer_chroma_extended_matrix_set_custom_frame2`
        // in OpenRazer) rather than a single-zone color — the same
        // category of not-yet-implemented work as keyboard per-key RGB.
        // Don't add a "Backlight"/"Underglow" zone back here without
        // confirming the correct mechanism against real traffic first.
    ],
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
    }),
};
