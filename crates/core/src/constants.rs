//! Protocol-level constants shared by every Razer HID device.
//!
//! Device-specific values (product IDs, which command IDs a given model
//! actually responds to, LED zones, DPI range, ...) live in
//! [`crate::devices`], not here. This module only holds the framing-level
//! bits of the Razer control-report protocol itself, which is identical
//! across the whole product line.

/// USB vendor id Razer Inc. registers all of its peripherals under.
pub const RAZER_VENDOR_ID: u16 = 0x1532;

/// Total bytes in a Razer control report, including the leading report id.
pub const REPORT_LENGTH: usize = 90;
pub const ARGS_LENGTH: usize = 80;

pub const STATUS_NEW_COMMAND: u8 = 0x00;
pub const STATUS_BUSY: u8 = 0x01;
pub const STATUS_SUCCESSFUL: u8 = 0x02;
pub const STATUS_FAILURE: u8 = 0x03;
pub const STATUS_NOT_SUPPORTED: u8 = 0x05;

/// 0x1F is used by mice/mousemats; keyboards and some other classes use
/// 0xFF. Device profiles may override this if theirs differs.
pub const DEFAULT_TRANSACTION_ID: u8 = 0x1F;
