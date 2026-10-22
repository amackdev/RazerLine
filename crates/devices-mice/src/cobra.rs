//! Device profile for the Razer Cobra (PID 0x00A3).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_COBRA` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use razer_hid_abi::commands::{
    CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand,
};
use razer_hid_abi::constants::RAZER_VENDOR_ID;
use razer_hid_abi::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    PollRateProfile,
};

pub const COBRA: DeviceProfile = DeviceProfile {
    name: "Razer Cobra",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x00A3,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 8500,
        transaction_id: 0xFF,
    }),
    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        get_command_id: GeneralCommand::GetPollRate as u8,
        supported_hz: &[125, 500, 1000],
        transaction_id: 0xFF,
    }),
    leds: &[LedZoneProfile {
        id: CommonLedId::Logo as u8,
        name: "Logo",
        command_class: CommandClass::Led as u8,
        effect_command_id: LedCommand::SetEffect as u8,
        brightness_command_id: Some(LedCommand::SetBrightness as u8),
        transaction_id: 0x1F,
    }],
    matrix: None,
    identity: Some(IdentityProfile {
        command_class: CommandClass::General as u8,
        firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
        serial_command_id: GeneralCommand::GetSerial as u8,
        firmware_transaction_id: 0xFF,
        serial_transaction_id: 0xFF,
    }),
    battery: None,
    idle_time: None,
    low_battery_threshold: None,
    unsupported_led_effects: &[],
};
