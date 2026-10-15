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
mod abyssus_2000;
mod abyssus_elite_dva_edition;
mod abyssus_essential;
mod abyssus_v2;
mod basilisk;
mod basilisk_essential;
mod basilisk_mobile_wired;
mod basilisk_ultimate_wired;
mod basilisk_v2;
mod basilisk_v3_35k;
mod basilisk_v3_pro_35k_phantom_green_edition_wired;
mod basilisk_v3_pro_35k_phantom_green_edition_wireless;
mod basilisk_v3_pro_35k_wired;
mod basilisk_v3_pro_35k_wireless;
mod basilisk_v3_pro_wired;
mod basilisk_v3_pro_wireless;
mod basilisk_v3_x_hyperspeed;
mod basilisk_x_hyperspeed;
mod cobra;
mod cobra_hyperspeed_wired;
mod cobra_hyperspeed_wireless;
mod cobra_pro_wired;
mod cobra_pro_wireless;
mod deathadder_2000;
mod deathadder_3500;
mod deathadder_chroma;
mod deathadder_elite;
mod deathadder_essential;
mod deathadder_essential_2021;
mod deathadder_essential_white_edition;
mod deathadder_v2;
mod deathadder_v2_lite;
mod deathadder_v2_mini;
mod deathadder_v2_pro_wired;
mod deathadder_v2_pro_wireless;
mod deathadder_v2_x_hyperspeed;
mod deathadder_v3;
mod deathadder_v3_hyperspeed_wired;
mod deathadder_v3_hyperspeed_wireless;
mod deathadder_v3_pro_wired;
mod deathadder_v3_pro_wired_alt;
mod deathadder_v3_pro_wireless;
mod deathadder_v3_pro_wireless_alt;
mod deathadder_v4_pro_wired;
mod deathadder_v4_pro_wireless;
mod diamondback_chroma;
mod imperator;
mod lancehead_te_wired;
mod lancehead_wired;
mod lancehead_wireless;
mod lancehead_wireless_wired;
mod mamba_2012_wired;
mod mamba_2012_wireless;

pub use basilisk_v3::BASILISK_V3;
pub use viper_ultimate::{VIPER_ULTIMATE_WIRED, VIPER_ULTIMATE_WIRELESS};
pub use abyssus_2000::ABYSSUS_2000;
pub use abyssus_elite_dva_edition::ABYSSUS_ELITE_DVA_EDITION;
pub use abyssus_essential::ABYSSUS_ESSENTIAL;
pub use abyssus_v2::ABYSSUS_V2;
pub use basilisk::BASILISK;
pub use basilisk_essential::BASILISK_ESSENTIAL;
pub use basilisk_mobile_wired::BASILISK_MOBILE_WIRED;
pub use basilisk_ultimate_wired::BASILISK_ULTIMATE_WIRED;
pub use basilisk_v2::BASILISK_V2;
pub use basilisk_v3_35k::BASILISK_V3_35K;
pub use basilisk_v3_pro_35k_phantom_green_edition_wired::BASILISK_V3_PRO_35K_PHANTOM_GREEN_EDITION_WIRED;
pub use basilisk_v3_pro_35k_phantom_green_edition_wireless::BASILISK_V3_PRO_35K_PHANTOM_GREEN_EDITION_WIRELESS;
pub use basilisk_v3_pro_35k_wired::BASILISK_V3_PRO_35K_WIRED;
pub use basilisk_v3_pro_35k_wireless::BASILISK_V3_PRO_35K_WIRELESS;
pub use basilisk_v3_pro_wired::BASILISK_V3_PRO_WIRED;
pub use basilisk_v3_pro_wireless::BASILISK_V3_PRO_WIRELESS;
pub use basilisk_v3_x_hyperspeed::BASILISK_V3_X_HYPERSPEED;
pub use basilisk_x_hyperspeed::BASILISK_X_HYPERSPEED;
pub use cobra::COBRA;
pub use cobra_hyperspeed_wired::COBRA_HYPERSPEED_WIRED;
pub use cobra_hyperspeed_wireless::COBRA_HYPERSPEED_WIRELESS;
pub use cobra_pro_wired::COBRA_PRO_WIRED;
pub use cobra_pro_wireless::COBRA_PRO_WIRELESS;
pub use deathadder_2000::DEATHADDER_2000;
pub use deathadder_3500::DEATHADDER_3500;
pub use deathadder_chroma::DEATHADDER_CHROMA;
pub use deathadder_elite::DEATHADDER_ELITE;
pub use deathadder_essential::DEATHADDER_ESSENTIAL;
pub use deathadder_essential_2021::DEATHADDER_ESSENTIAL_2021;
pub use deathadder_essential_white_edition::DEATHADDER_ESSENTIAL_WHITE_EDITION;
pub use deathadder_v2::DEATHADDER_V2;
pub use deathadder_v2_lite::DEATHADDER_V2_LITE;
pub use deathadder_v2_mini::DEATHADDER_V2_MINI;
pub use deathadder_v2_pro_wired::DEATHADDER_V2_PRO_WIRED;
pub use deathadder_v2_pro_wireless::DEATHADDER_V2_PRO_WIRELESS;
pub use deathadder_v2_x_hyperspeed::DEATHADDER_V2_X_HYPERSPEED;
pub use deathadder_v3::DEATHADDER_V3;
pub use deathadder_v3_hyperspeed_wired::DEATHADDER_V3_HYPERSPEED_WIRED;
pub use deathadder_v3_hyperspeed_wireless::DEATHADDER_V3_HYPERSPEED_WIRELESS;
pub use deathadder_v3_pro_wired::DEATHADDER_V3_PRO_WIRED;
pub use deathadder_v3_pro_wired_alt::DEATHADDER_V3_PRO_WIRED_ALT;
pub use deathadder_v3_pro_wireless::DEATHADDER_V3_PRO_WIRELESS;
pub use deathadder_v3_pro_wireless_alt::DEATHADDER_V3_PRO_WIRELESS_ALT;
pub use deathadder_v4_pro_wired::DEATHADDER_V4_PRO_WIRED;
pub use deathadder_v4_pro_wireless::DEATHADDER_V4_PRO_WIRELESS;
pub use diamondback_chroma::DIAMONDBACK_CHROMA;
pub use imperator::IMPERATOR;
pub use lancehead_te_wired::LANCEHEAD_TE_WIRED;
pub use lancehead_wired::LANCEHEAD_WIRED;
pub use lancehead_wireless::LANCEHEAD_WIRELESS;
pub use lancehead_wireless_wired::LANCEHEAD_WIRELESS_WIRED;
pub use mamba_2012_wired::MAMBA_2012_WIRED;
pub use mamba_2012_wireless::MAMBA_2012_WIRELESS;

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
    &[BASILISK_V3, VIPER_ULTIMATE_WIRED, VIPER_ULTIMATE_WIRELESS, ABYSSUS_2000, ABYSSUS_ELITE_DVA_EDITION, ABYSSUS_ESSENTIAL, ABYSSUS_V2, BASILISK, BASILISK_ESSENTIAL, BASILISK_MOBILE_WIRED, BASILISK_ULTIMATE_WIRED, BASILISK_V2, BASILISK_V3_35K, BASILISK_V3_PRO_35K_PHANTOM_GREEN_EDITION_WIRED, BASILISK_V3_PRO_35K_PHANTOM_GREEN_EDITION_WIRELESS, BASILISK_V3_PRO_35K_WIRED, BASILISK_V3_PRO_35K_WIRELESS, BASILISK_V3_PRO_WIRED, BASILISK_V3_PRO_WIRELESS, BASILISK_V3_X_HYPERSPEED, BASILISK_X_HYPERSPEED, COBRA, COBRA_HYPERSPEED_WIRED, COBRA_HYPERSPEED_WIRELESS, COBRA_PRO_WIRED, COBRA_PRO_WIRELESS, DEATHADDER_2000, DEATHADDER_3500, DEATHADDER_CHROMA, DEATHADDER_ELITE, DEATHADDER_ESSENTIAL, DEATHADDER_ESSENTIAL_2021, DEATHADDER_ESSENTIAL_WHITE_EDITION, DEATHADDER_V2, DEATHADDER_V2_LITE, DEATHADDER_V2_MINI, DEATHADDER_V2_PRO_WIRED, DEATHADDER_V2_PRO_WIRELESS, DEATHADDER_V2_X_HYPERSPEED, DEATHADDER_V3, DEATHADDER_V3_HYPERSPEED_WIRED, DEATHADDER_V3_HYPERSPEED_WIRELESS, DEATHADDER_V3_PRO_WIRED, DEATHADDER_V3_PRO_WIRED_ALT, DEATHADDER_V3_PRO_WIRELESS, DEATHADDER_V3_PRO_WIRELESS_ALT, DEATHADDER_V4_PRO_WIRED, DEATHADDER_V4_PRO_WIRELESS, DIAMONDBACK_CHROMA, IMPERATOR, LANCEHEAD_TE_WIRED, LANCEHEAD_WIRED, LANCEHEAD_WIRELESS, LANCEHEAD_WIRELESS_WIRED, MAMBA_2012_WIRED, MAMBA_2012_WIRELESS];

pub fn find_by_pid(vendor_id: u16, product_id: u16) -> Option<&'static DeviceProfile> {
    KNOWN_DEVICES
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}
