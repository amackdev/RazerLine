//! Device profiles for the Razer Viper Ultimate (wired dock-charging /
//! wireless-receiver variants).
//!
//! Confidence: cross-checked against OpenRazer's `razermouse_driver.c`
//! and daemon metadata (`daemon/openrazer_daemon/hardware/mouse.py`,
//! `RazerViperUltimateWired`/`RazerViperUltimateWireless`), the same way
//! as `basilisk_v3.rs` — but **not yet run against real hardware by any
//! maintainer of this repo**. If you own one, please open an issue or PR
//! with what did/didn't work.
//!
//! Unlike the Basilisk V3, this device does **not** use one uniform
//! transaction id: OpenRazer sends DPI/poll-rate/firmware/serial with
//! 0xFF, but LED effects and brightness with 0x3F. Both are confirmed by
//! reading every `USB_DEVICE_ID_RAZER_VIPER_ULTIMATE_WIRED`/`_WIRELESS`
//! case in the driver, not assumed from the Basilisk V3 pattern.
//!
//! Only one LED zone exists (Logo) — this mouse has no scroll wheel
//! light, and its "matrix" is a 1x1 grid (`MATRIX_DIMS = [1, 1]` in the
//! daemon), i.e. just an alternate path to the same single LED, so no
//! separate `matrix` capability is modeled here.

use razer_hid_abi::commands::{
    CommandClass, CommonLedId, GeneralCommand, LedCommand, MouseCommand,
};
use razer_hid_abi::constants::RAZER_VENDOR_ID;
use razer_hid_abi::{
    Confidence, DeviceKind, DeviceProfile, DpiProfile, IdentityProfile, LedZoneProfile,
    PollRateProfile,
};

const GENERAL_TRANSACTION_ID: u8 = 0xFF;
const LED_TRANSACTION_ID: u8 = 0x3F;

const fn profile(name: &'static str, product_id: u16) -> DeviceProfile {
    DeviceProfile {
        name,
        vendor_id: RAZER_VENDOR_ID,
        product_id,
        kind: DeviceKind::Mouse,
        confidence: Confidence::Unverified,
        dpi: Some(DpiProfile {
            command_class: CommandClass::Mouse as u8,
            set_command_id: MouseCommand::SetDpiXY as u8,
            get_command_id: MouseCommand::GetDpiXY as u8,
            // DPI_MAX in OpenRazer's daemon metadata for this model.
            min: 100,
            max: 20000,
            transaction_id: GENERAL_TRANSACTION_ID,
        }),
        poll_rate: Some(PollRateProfile {
            command_class: CommandClass::General as u8,
            set_command_id: GeneralCommand::SetPollRate as u8,
            get_command_id: GeneralCommand::GetPollRate as u8,
            supported_hz: &[125, 500, 1000],
            transaction_id: GENERAL_TRANSACTION_ID,
        }),
        leds: &[LedZoneProfile {
            id: CommonLedId::Logo as u8,
            name: "Logo",
            command_class: CommandClass::Led as u8,
            effect_command_id: LedCommand::SetEffect as u8,
            brightness_command_id: Some(LedCommand::SetBrightness as u8),
            transaction_id: LED_TRANSACTION_ID,
        }],
        matrix: None,
        identity: Some(IdentityProfile {
            command_class: CommandClass::General as u8,
            firmware_command_id: GeneralCommand::GetFirmwareVersion as u8,
            serial_command_id: GeneralCommand::GetSerial as u8,
            firmware_transaction_id: GENERAL_TRANSACTION_ID,
            serial_transaction_id: GENERAL_TRANSACTION_ID,
        }),
        unsupported_led_effects: &[],
    }
}

// Verify with `lsusb -d 1532:` (Linux) or Device Manager (Windows).
pub const VIPER_ULTIMATE_WIRED: DeviceProfile = profile("Razer Viper Ultimate (Wired)", 0x007A);
pub const VIPER_ULTIMATE_WIRELESS: DeviceProfile =
    profile("Razer Viper Ultimate (Wireless)", 0x007B);
