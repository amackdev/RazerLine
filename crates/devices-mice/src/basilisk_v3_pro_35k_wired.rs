//! Device profile for the Razer Basilisk V3 Pro 35K (Wired) (PID 0x00CC).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_BASILISK_V3_PRO_35K_WIRED` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use razer_hid_abi::commands::{
    CommandClass, CommonLedId, GeneralCommand, LedCommand, MiscCommand, MouseCommand,
};
use razer_hid_abi::constants::RAZER_VENDOR_ID;
use razer_hid_abi::{
    BatteryProfile, ChargingStatusSupport, Confidence, DeviceKind, DeviceProfile, DpiProfile,
    IdentityProfile, LedZoneProfile, MatrixPacketLength, MatrixProfile, PollRateProfile,
};

pub const BASILISK_V3_PRO_35K_WIRED: DeviceProfile = DeviceProfile {
    name: "Razer Basilisk V3 Pro 35K (Wired)",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x00CC,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 35000,
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
            id: CommonLedId::Logo as u8,
            name: "Logo",
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
        cols: 13,
        write_command_class: CommandClass::Led as u8,
        write_command_id: LedCommand::SetCustomFrame as u8,
        write_packet_length: MatrixPacketLength::Fixed(0x47),
        write_transaction_id: 0x1F,
        commit_command_class: CommandClass::Led as u8,
        commit_command_id: LedCommand::SetEffect as u8,
        commit_transaction_id: 0x1F,
        effect_led_id: Some(CommonLedId::Zero as u8),
    }),
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
        firmware_transaction_id: 0x1F,
        serial_transaction_id: 0x1F,
    }),
    battery: Some(BatteryProfile {
        command_class: CommandClass::Misc as u8,
        get_level_command_id: MiscCommand::GetBatteryLevel as u8,
        get_charging_command_id: MiscCommand::GetChargingStatus as u8,
        transaction_id: 0x1F,
        charging_status: ChargingStatusSupport::Supported {
            transaction_id: 0x1F,
        },
    }),
    unsupported_led_effects: &[],
};
