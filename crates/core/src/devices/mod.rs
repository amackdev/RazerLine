//! The device registry: one [`DeviceProfile`] per supported Razer
//! peripheral, describing which capabilities it has and which HID command
//! bytes drive them.
//!
//! This is the extension point for the whole project. Supporting a new
//! mouse/keyboard/etc. means writing one new file here and adding it to
//! [`KNOWN_DEVICES`] — it does **not** require touching the protocol
//! framing, the CLI, or the GUI, all of which are written generically
//! against this struct. See `CONTRIBUTING.md` for the step-by-step guide
//! and `devices/_template.rs` for an annotated starting point.

mod basilisk_v3;

pub use basilisk_v3::BASILISK_V3;

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
}

#[derive(Debug, Clone, Copy)]
pub struct PollRateProfile {
    pub command_class: u8,
    pub set_command_id: u8,
    pub supported_hz: &'static [u32],
}

/// One independently controllable RGB zone (e.g. "Scroll wheel", "Logo").
#[derive(Debug, Clone, Copy)]
pub struct LedZoneProfile {
    pub id: u8,
    pub name: &'static str,
    pub command_class: u8,
    pub effect_command_id: u8,
    pub brightness_command_id: Option<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct IdentityProfile {
    pub command_class: u8,
    pub firmware_command_id: u8,
    pub serial_command_id: u8,
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

pub const KNOWN_DEVICES: &[DeviceProfile] = &[BASILISK_V3];

pub fn find_by_pid(vendor_id: u16, product_id: u16) -> Option<&'static DeviceProfile> {
    KNOWN_DEVICES
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}
