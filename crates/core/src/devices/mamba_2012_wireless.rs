//! Device profile for the Razer Mamba 2012 (Wireless) (PID 0x0025).
//!
//! Generated from OpenRazer's kernel driver source and daemon metadata
//! (`driver/razermouse_driver.c`, `razerchromacommon.c`,
//! `daemon/openrazer_daemon/hardware/mouse.py`) — every field below traces
//! back to the `USB_DEVICE_ID_RAZER_MAMBA_2012_WIRELESS` case in the relevant switch statement, not
//! copied from a similarly-named device. **Not yet tested against real
//! hardware by any maintainer of this repo** — if you own this device,
//! please report what did/didn't work.

use super::{Confidence, DeviceKind, DeviceProfile, DpiProfile, PollRateProfile};
use crate::commands::{CommandClass, GeneralCommand, MouseCommand};
use crate::constants::RAZER_VENDOR_ID;

pub const MAMBA_2012_WIRELESS: DeviceProfile = DeviceProfile {
    name: "Razer Mamba 2012 (Wireless)",
    vendor_id: RAZER_VENDOR_ID,
    product_id: 0x0025,
    kind: DeviceKind::Mouse,
    confidence: Confidence::Unverified,
    dpi: Some(DpiProfile {
        command_class: CommandClass::Mouse as u8,
        set_command_id: MouseCommand::SetDpiXY as u8,
        get_command_id: MouseCommand::GetDpiXY as u8,
        min: 100,
        max: 6400,
        transaction_id: 0xFF,
    }),
    poll_rate: Some(PollRateProfile {
        command_class: CommandClass::General as u8,
        set_command_id: GeneralCommand::SetPollRate as u8,
        get_command_id: GeneralCommand::GetPollRate as u8,
        supported_hz: &[125, 500, 1000],
        transaction_id: 0xFF,
    }),
    leds: &[],
    matrix: None,
    identity: None,
    unsupported_led_effects: &[],
};
