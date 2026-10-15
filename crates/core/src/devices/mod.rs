//! The device registry: one [`DeviceProfile`] per supported Razer
//! peripheral, describing which capabilities it has and which HID command
//! bytes drive them.
//!
//! This is the extension point for the whole project. Supporting a new
//! mouse/keyboard/etc. means writing one new file here and adding it to
//! [`KNOWN_DEVICES`] — it does **not** require touching the protocol
//! framing, the CLI, or the GUI, all of which are written generically
//! against this struct. See README.md's "Adding support for your device"
//! for the step-by-step guide and `devices/_template.rs` for an
//! annotated starting point.

mod basilisk_v3;
mod viper_ultimate;

pub use basilisk_v3::BASILISK_V3;
pub use viper_ultimate::{VIPER_ULTIMATE_WIRED, VIPER_ULTIMATE_WIRELESS};

/// The broad category of peripheral. Currently only `Mouse` capabilities
/// (DPI, polling rate, two-zone RGB) are modeled end to end; keyboards and
/// other classes need their own capability types added here as profiles
/// for them are contributed (e.g. per-key matrix effects).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Mouse,
    Keyboard,
    Mousemat,
    Headset,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct DpiProfile {
    pub command_class: u8,
    pub set_command_id: u8,
    pub get_command_id: u8,
    pub min: u16,
    pub max: u16,
    /// Transaction id this exact command needs. This is **not** always
    /// the same across a whole device: e.g. the Viper Ultimate uses
    /// 0xFF for DPI/poll-rate/identity but 0x3F for LED commands. Always
    /// check per-command, not just per-device, against the real source.
    pub transaction_id: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct PollRateProfile {
    pub command_class: u8,
    pub set_command_id: u8,
    pub supported_hz: &'static [u32],
    pub transaction_id: u8,
}

/// One independently controllable RGB zone (e.g. "Scroll wheel", "Logo").
#[derive(Debug, Clone, Copy)]
pub struct LedZoneProfile {
    pub id: u8,
    pub name: &'static str,
    pub command_class: u8,
    pub effect_command_id: u8,
    pub brightness_command_id: Option<u8>,
    pub transaction_id: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct IdentityProfile {
    pub command_class: u8,
    pub firmware_command_id: u8,
    pub serial_command_id: u8,
    /// Firmware and serial queries are **not guaranteed to share a
    /// transaction id** — e.g. the Mamba Elite uses 0x1F for firmware but
    /// 0xFF for serial. Check both independently against the real source.
    pub firmware_transaction_id: u8,
    pub serial_transaction_id: u8,
}

/// A per-pixel addressable LED strip/grid driven by writing one or more
/// rows of RGB data and then a separate "commit" command that tells the
/// device to actually display what was written (OpenRazer's "custom
/// frame" mechanism). This is a different, lower-level mechanism than
/// [`LedZoneProfile`] — use it for strips with more than one
/// individually addressable LED (e.g. a mouse's underglow), not for a
/// single-color zone.
/// How the "custom frame" write command's declared data length is
/// computed. OpenRazer's `razer_chroma_extended_matrix_set_custom_frame`
/// wrapper always passes a fixed 0x47; `..._set_custom_frame2(..., 0)`
/// (used by a different set of devices, e.g. the Mamba Elite/Naga Pro)
/// computes it from how many columns are actually being written.
#[derive(Debug, Clone, Copy)]
pub enum MatrixPacketLength {
    Fixed(u8),
    /// `5 + 3 * columns_written`.
    Computed,
}

#[derive(Debug, Clone, Copy)]
pub struct MatrixProfile {
    pub rows: u8,
    pub cols: u8,
    pub write_command_class: u8,
    pub write_command_id: u8,
    /// How to size the declared data length of the write command. Varies
    /// by device — check the real source rather than assuming either one.
    pub write_packet_length: MatrixPacketLength,
    pub write_transaction_id: u8,
    pub commit_command_class: u8,
    pub commit_command_id: u8,
    pub commit_transaction_id: u8,
}

/// Everything the control library needs to know about one device model.
#[derive(Debug, Clone, Copy)]
pub struct DeviceProfile {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub kind: DeviceKind,
    /// How confident this profile is, for display to the user. Profiles
    /// contributed without hardware to test against should be marked
    /// `Unverified` so the CLI/GUI can warn accordingly.
    pub confidence: Confidence,
    pub dpi: Option<DpiProfile>,
    pub poll_rate: Option<PollRateProfile>,
    pub leds: &'static [LedZoneProfile],
    pub matrix: Option<MatrixProfile>,
    pub identity: Option<IdentityProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Verified against real hardware by at least one contributor.
    Verified,
    /// Written from protocol patterns seen on other models, not yet
    /// confirmed on this exact one.
    Unverified,
}

pub const KNOWN_DEVICES: &[DeviceProfile] =
    &[BASILISK_V3, VIPER_ULTIMATE_WIRED, VIPER_ULTIMATE_WIRELESS];

pub fn find_by_pid(vendor_id: u16, product_id: u16) -> Option<&'static DeviceProfile> {
    KNOWN_DEVICES
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}
