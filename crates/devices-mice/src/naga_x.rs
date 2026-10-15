//! Device profile for the Razer Naga X (PID 0x0096).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_NAGA_X` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use razer_hid_abi::commands::{
    CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand,
};
use razer_hid_abi::constants::RAZER_VENDOR_ID;
use razer_hid_abi::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    MatrixPacketLength, MatrixProfile, PollRateProfile,
};

pub const NAGA_X: DeviceProfile = DeviceProfile {
    name: "Razer Naga X",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x0096,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 18000,
        transaction_id: 0x1F,
    }),
    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        get_command_id: GeneralCommand::GetPollRate as u8,
        supported_hz: &[125, 500, 1000],
        transaction_id: 0x1F,
    }),
    leds: &[
        LedZoneProfile {
            id: CommonLedId::LeftSide as u8,
            name: "Left",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
            transaction_id: 0x1F,
        },
        LedZoneProfile {
            id: CommonLedId::ScrollWheel as u8,
            name: "Scroll wheel",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
            transaction_id: 0x1F,
        },
    ],
    matrix: Some(MatrixProfile {
        rows: 1,
        cols: 2,
        write_command_class: CommandClass::Led as u8,
        write_command_id: LedCommand::SetCustomFrame as u8,
        write_packet_length: MatrixPacketLength::Computed,
        write_transaction_id: 0x1F,
        commit_command_class: CommandClass::Led as u8,
        commit_command_id: LedCommand::SetEffect as u8,
        commit_transaction_id: 0x1F,
        effect_led_id: None,
    }),
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
        firmware_transaction_id: 0x1F,
        serial_transaction_id: 0x08,
    }),
    unsupported_led_effects: &[],
};
