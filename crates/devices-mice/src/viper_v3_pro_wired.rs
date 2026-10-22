//! Device profile for the Razer Viper V3 Pro (Wired) (PID 0x00C0).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_VIPER_V3_PRO_WIRED` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use razer_hid_abi::commands::{CommandClass, GeneralCommand, MiscCommand, MouseCommand};
use razer_hid_abi::constants::RAZER_VENDOR_ID;
use razer_hid_abi::{
    BatteryProfile, ChargingStatusSupport, Confidence, DeviceKind, DeviceProfile, DpiProfile,
    IdentityProfile, PollRateProfile,
};

pub const VIPER_V3_PRO_WIRED: DeviceProfile = DeviceProfile {
    name: "Razer Viper V3 Pro (Wired)",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x00C0,
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
    leds: &[],
    matrix: None,
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
