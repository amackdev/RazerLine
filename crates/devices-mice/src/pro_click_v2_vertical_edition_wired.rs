//! Device profile for the Razer Pro Click V2 Vertical Edition (Wired) (PID 0x00C7).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_PRO_CLICK_V2_VERTICAL_EDITION_WIRED` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use razer_hid_abi::commands::{CommandClass, GeneralCommand, MouseCommand};
use razer_hid_abi::constants::RAZER_VENDOR_ID;
use razer_hid_abi::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, PollRateProfile,
};

pub const PRO_CLICK_V2_VERTICAL_EDITION_WIRED: DeviceProfile = DeviceProfile {
    name: "Razer Pro Click V2 Vertical Edition (Wired)",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x00C7,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 30000,
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
    unsupported_led_effects: &[],
};
