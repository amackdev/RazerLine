//! Device profile for the Razer Basilisk V3 (wired).
//!
//! Confirmed against real hardware: DPI get/set, polling rate, and the
//! Static/Spectrum/Wave LED effects plus brightness on both the scroll
//! wheel and logo zones. Also confirmed on real hardware: Breathing and
//! Off genuinely get rejected by the firmware (`STATUS_NOT_SUPPORTED`/
//! `STATUS_FAILURE`) — not a bug here, OpenRazer's own driver doesn't
//! attempt them for this exact PID either.
//!
//! The third LED (bottom/underglow) is a separate addressable 11-LED
//! strip (`MATRIX_DIMS = [1, 11]` in OpenRazer's daemon metadata,
//! `daemon/openrazer_daemon/hardware/mouse.py`), not a single-color zone
//! — real hardware rejects the simple zone-effect command for it
//! outright. It's modeled below as `matrix`, not as a `LedZoneProfile`.

use super::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    MatrixPacketLength, MatrixProfile, PollRateProfile,
};
use crate::commands::{CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand};
use crate::constants::RAZER_VENDOR_ID;

/// Every command this device responds to uses transaction id 0x1F —
/// confirmed by reading every `USB_DEVICE_ID_RAZER_BASILISK_V3` case in
/// OpenRazer's `razermouse_driver.c` (DPI, poll rate, LED effects,
/// brightness, custom frame, firmware/serial query all land in the 0x1F
/// group). Not every device is this uniform — see `viper_ultimate.rs`
/// for a device that splits transaction ids by command type.
const TRANSACTION_ID: u8 = 0x1F;

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
        // OpenRazer's daemon caps this exact model at 26000 (DPI_MAX in
        // RazerBasiliskV3, hardware/mouse.py) — tighter than the generic
        // 100-45000 the kernel driver's argument-packing clamp allows.
        min: 100,
        max: 26000,
        transaction_id: TRANSACTION_ID,
    }),
    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        get_command_id: GeneralCommand::GetPollRate as u8,
        supported_hz: &[125, 500, 1000],
        transaction_id: TRANSACTION_ID,
    }),
    leds: &[
        LedZoneProfile {
            id: CommonLedId::ScrollWheel as u8,
            name: "Scroll wheel",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
            transaction_id: TRANSACTION_ID,
        },
        LedZoneProfile {
            id: CommonLedId::Logo as u8,
            name: "Logo",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
            transaction_id: TRANSACTION_ID,
        },
    ],
    // Bottom/underglow strip: 1 row x 11 LEDs. Write command uses a fixed
    // 0x47-byte declared length regardless of how many columns are
    // actually touched — confirmed by OpenRazer's dispatch for this PID
    // calling `razer_chroma_extended_matrix_set_custom_frame(...)`, whose
    // wrapper always passes packetLength=0x47 (the row_length+5 variant
    // is only used by a different device group, e.g. Mamba Elite).
    matrix: Some(MatrixProfile {
        rows: 1,
        cols: 11,
        write_command_class: CommandClass::Led as u8,
        write_command_id: LedCommand::SetCustomFrame as u8,
        write_packet_length: MatrixPacketLength::Fixed(0x47),
        write_transaction_id: TRANSACTION_ID,
        commit_command_class: CommandClass::Led as u8,
        commit_command_id: LedCommand::SetEffect as u8,
        commit_transaction_id: TRANSACTION_ID,
    }),
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
        firmware_transaction_id: TRANSACTION_ID,
        serial_transaction_id: TRANSACTION_ID,
    }),
};
