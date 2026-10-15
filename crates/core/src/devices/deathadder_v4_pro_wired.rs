//! Device profile for the Razer DeathAdder V4 Pro (Wired) (PID 0x00BE).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_DEATHADDER_V4_PRO_WIRED` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use super::{Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile};
use crate::commands::{CommandClass, GeneralCommand, MouseCommand};
use crate::constants::RAZER_VENDOR_ID;

pub const DEATHADDER_V4_PRO_WIRED: DeviceProfile = DeviceProfile {
    name: "Razer DeathAdder V4 Pro (Wired)",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x00BE,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 45000,
        transaction_id: 0x1F,
    }),
    poll_rate: None,
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
