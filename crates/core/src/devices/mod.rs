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

mod abyssus_2000;
mod abyssus_elite_dva_edition;
mod abyssus_essential;
mod abyssus_v2;
mod basilisk;
mod basilisk_essential;
mod basilisk_mobile_wired;
mod basilisk_ultimate_wired;
mod basilisk_v2;
mod basilisk_v3;
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
mod mamba_elite;
mod mamba_te_wired;
mod mamba_wired;
mod mamba_wireless;
mod mamba_wireless_wired;
mod naga_2014;
mod naga_chroma;
mod naga_epic_chroma;
mod naga_epic_chroma_dock;
mod naga_hex_v2;
mod naga_left_handed_2020;
mod naga_pro_wired;
mod naga_pro_wireless;
mod naga_trinity;
mod naga_v2_pro_wired;
mod naga_v2_pro_wireless;
mod naga_x;
mod orochi_2013;
mod orochi_chroma;
mod orochi_v2_bluetooth;
mod ouroboros;
mod pro_click_v2_vertical_edition_wired;
mod pro_click_v2_vertical_edition_wireless;
mod pro_click_v2_wired;
mod pro_click_v2_wireless;
mod pro_click_wired;
mod taipan;
mod viper;
mod viper_8k;
mod viper_mini;
mod viper_mini_se_wired;
mod viper_mini_se_wireless;
mod viper_ultimate;
mod viper_v2_pro_wired;
mod viper_v2_pro_wireless;
mod viper_v3_hyperspeed;
mod viper_v3_pro_se_wired;
mod viper_v3_pro_se_wireless;
mod viper_v3_pro_wired;
mod viper_v3_pro_wireless;

pub use abyssus_2000::ABYSSUS_2000;
pub use abyssus_elite_dva_edition::ABYSSUS_ELITE_DVA_EDITION;
pub use abyssus_essential::ABYSSUS_ESSENTIAL;
pub use abyssus_v2::ABYSSUS_V2;
pub use basilisk::BASILISK;
pub use basilisk_essential::BASILISK_ESSENTIAL;
pub use basilisk_mobile_wired::BASILISK_MOBILE_WIRED;
pub use basilisk_ultimate_wired::BASILISK_ULTIMATE_WIRED;
pub use basilisk_v2::BASILISK_V2;
pub use basilisk_v3::BASILISK_V3;
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
pub use mamba_elite::MAMBA_ELITE;
pub use mamba_te_wired::MAMBA_TE_WIRED;
pub use mamba_wired::MAMBA_WIRED;
pub use mamba_wireless::MAMBA_WIRELESS;
pub use mamba_wireless_wired::MAMBA_WIRELESS_WIRED;
pub use naga_2014::NAGA_2014;
pub use naga_chroma::NAGA_CHROMA;
pub use naga_epic_chroma::NAGA_EPIC_CHROMA;
pub use naga_epic_chroma_dock::NAGA_EPIC_CHROMA_DOCK;
pub use naga_hex_v2::NAGA_HEX_V2;
pub use naga_left_handed_2020::NAGA_LEFT_HANDED_2020;
pub use naga_pro_wired::NAGA_PRO_WIRED;
pub use naga_pro_wireless::NAGA_PRO_WIRELESS;
pub use naga_trinity::NAGA_TRINITY;
pub use naga_v2_pro_wired::NAGA_V2_PRO_WIRED;
pub use naga_v2_pro_wireless::NAGA_V2_PRO_WIRELESS;
pub use naga_x::NAGA_X;
pub use orochi_2013::OROCHI_2013;
pub use orochi_chroma::OROCHI_CHROMA;
pub use orochi_v2_bluetooth::OROCHI_V2_BLUETOOTH;
pub use ouroboros::OUROBOROS;
pub use pro_click_v2_vertical_edition_wired::PRO_CLICK_V2_VERTICAL_EDITION_WIRED;
pub use pro_click_v2_vertical_edition_wireless::PRO_CLICK_V2_VERTICAL_EDITION_WIRELESS;
pub use pro_click_v2_wired::PRO_CLICK_V2_WIRED;
pub use pro_click_v2_wireless::PRO_CLICK_V2_WIRELESS;
pub use pro_click_wired::PRO_CLICK_WIRED;
pub use taipan::TAIPAN;
pub use viper::VIPER;
pub use viper_8k::VIPER_8K;
pub use viper_mini::VIPER_MINI;
pub use viper_mini_se_wired::VIPER_MINI_SE_WIRED;
pub use viper_mini_se_wireless::VIPER_MINI_SE_WIRELESS;
pub use viper_ultimate::{VIPER_ULTIMATE_WIRED, VIPER_ULTIMATE_WIRELESS};
pub use viper_v2_pro_wired::VIPER_V2_PRO_WIRED;
pub use viper_v2_pro_wireless::VIPER_V2_PRO_WIRELESS;
pub use viper_v3_hyperspeed::VIPER_V3_HYPERSPEED;
pub use viper_v3_pro_se_wired::VIPER_V3_PRO_SE_WIRED;
pub use viper_v3_pro_se_wireless::VIPER_V3_PRO_SE_WIRELESS;
pub use viper_v3_pro_wired::VIPER_V3_PRO_WIRED;
pub use viper_v3_pro_wireless::VIPER_V3_PRO_WIRELESS;

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
    /// `GeneralCommand::GetPollRate` for the common single-byte variant
    /// most mice use — queries the device's actual current rate instead
    /// of assuming whatever was last requested.
    pub get_command_id: u8,
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
    /// Some matrices *also* answer the same SetEffect command a normal
    /// [`LedZoneProfile`] does — just addressed to this LED id instead of
    /// a zone id — giving genuine hardware-animated Wave/Spectrum (and
    /// Static/Brightness) on the whole strip, rather than only being able
    /// to paint a static frame via raw pixel writes. `None` if the matrix
    /// is custom-frame-only (no confirmed source evidence of this).
    pub effect_led_id: Option<u8>,
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
    /// Effects confirmed to be rejected outright for every zone/matrix on
    /// this device (e.g. Basilisk V3 rejects Breathing and Off). Lets the
    /// GUI hide those controls instead of surfacing a firmware error.
    pub unsupported_led_effects: &'static [crate::commands::LedEffect],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Verified against real hardware by at least one contributor.
    Verified,
    /// Written from protocol patterns seen on other models, not yet
    /// confirmed on this exact one.
    Unverified,
}

pub const KNOWN_DEVICES: &[DeviceProfile] = &[
    BASILISK_V3,
    VIPER_ULTIMATE_WIRED,
    VIPER_ULTIMATE_WIRELESS,
    ABYSSUS_2000,
    ABYSSUS_ELITE_DVA_EDITION,
    ABYSSUS_ESSENTIAL,
    ABYSSUS_V2,
    BASILISK,
    BASILISK_ESSENTIAL,
    BASILISK_MOBILE_WIRED,
    BASILISK_ULTIMATE_WIRED,
    BASILISK_V2,
    BASILISK_V3_35K,
    BASILISK_V3_PRO_35K_PHANTOM_GREEN_EDITION_WIRED,
    BASILISK_V3_PRO_35K_PHANTOM_GREEN_EDITION_WIRELESS,
    BASILISK_V3_PRO_35K_WIRED,
    BASILISK_V3_PRO_35K_WIRELESS,
    BASILISK_V3_PRO_WIRED,
    BASILISK_V3_PRO_WIRELESS,
    BASILISK_V3_X_HYPERSPEED,
    BASILISK_X_HYPERSPEED,
    COBRA,
    COBRA_HYPERSPEED_WIRED,
    COBRA_HYPERSPEED_WIRELESS,
    COBRA_PRO_WIRED,
    COBRA_PRO_WIRELESS,
    DEATHADDER_2000,
    DEATHADDER_3500,
    DEATHADDER_CHROMA,
    DEATHADDER_ELITE,
    DEATHADDER_ESSENTIAL,
    DEATHADDER_ESSENTIAL_2021,
    DEATHADDER_ESSENTIAL_WHITE_EDITION,
    DEATHADDER_V2,
    DEATHADDER_V2_LITE,
    DEATHADDER_V2_MINI,
    DEATHADDER_V2_PRO_WIRED,
    DEATHADDER_V2_PRO_WIRELESS,
    DEATHADDER_V2_X_HYPERSPEED,
    DEATHADDER_V3,
    DEATHADDER_V3_HYPERSPEED_WIRED,
    DEATHADDER_V3_HYPERSPEED_WIRELESS,
    DEATHADDER_V3_PRO_WIRED,
    DEATHADDER_V3_PRO_WIRED_ALT,
    DEATHADDER_V3_PRO_WIRELESS,
    DEATHADDER_V3_PRO_WIRELESS_ALT,
    DEATHADDER_V4_PRO_WIRED,
    DEATHADDER_V4_PRO_WIRELESS,
    DIAMONDBACK_CHROMA,
    IMPERATOR,
    LANCEHEAD_TE_WIRED,
    LANCEHEAD_WIRED,
    LANCEHEAD_WIRELESS,
    LANCEHEAD_WIRELESS_WIRED,
    MAMBA_2012_WIRED,
    MAMBA_2012_WIRELESS,
    MAMBA_ELITE,
    MAMBA_TE_WIRED,
    MAMBA_WIRED,
    MAMBA_WIRELESS,
    MAMBA_WIRELESS_WIRED,
    NAGA_2014,
    NAGA_CHROMA,
    NAGA_EPIC_CHROMA,
    NAGA_EPIC_CHROMA_DOCK,
    NAGA_HEX_V2,
    NAGA_LEFT_HANDED_2020,
    NAGA_PRO_WIRED,
    NAGA_PRO_WIRELESS,
    NAGA_TRINITY,
    NAGA_V2_PRO_WIRED,
    NAGA_V2_PRO_WIRELESS,
    NAGA_X,
    OROCHI_2013,
    OROCHI_CHROMA,
    OROCHI_V2_BLUETOOTH,
    OUROBOROS,
    PRO_CLICK_V2_VERTICAL_EDITION_WIRED,
    PRO_CLICK_V2_VERTICAL_EDITION_WIRELESS,
    PRO_CLICK_V2_WIRED,
    PRO_CLICK_V2_WIRELESS,
    PRO_CLICK_WIRED,
    TAIPAN,
    VIPER,
    VIPER_8K,
    VIPER_MINI,
    VIPER_MINI_SE_WIRED,
    VIPER_MINI_SE_WIRELESS,
    VIPER_V2_PRO_WIRED,
    VIPER_V2_PRO_WIRELESS,
    VIPER_V3_HYPERSPEED,
    VIPER_V3_PRO_SE_WIRED,
    VIPER_V3_PRO_SE_WIRELESS,
    VIPER_V3_PRO_WIRED,
    VIPER_V3_PRO_WIRELESS,
];

pub fn find_by_pid(vendor_id: u16, product_id: u16) -> Option<&'static DeviceProfile> {
    KNOWN_DEVICES
        .iter()
        .find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}
