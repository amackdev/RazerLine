//! The shapes that describe one device's capabilities — [`DeviceProfile`]
//! and its component profile types. The actual catalog of concrete
//! `DeviceProfile` instances (one per supported model) lives in the
//! separate `razer-hid-devices` crate, which depends on this module for
//! these types but is not a dependency of it — keeps the core control
//! engine usable without pulling in the whole device database, and lets
//! the database be versioned/extended independently.

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

/// How a device reports whether it's currently charging, alongside its
/// battery level. OpenRazer's kernel driver (`razermouse_driver.c`) keys
/// this per USB product id, and not every wireless mouse falls in the
/// same bucket as its battery-level query:
#[derive(Debug, Clone, Copy)]
pub enum ChargingStatusSupport {
    /// Queries the device, same command class as battery level, with its
    /// own (possibly different) transaction id.
    Supported { transaction_id: u8 },
    /// Primary (e.g. AA) battery, no onboard charging circuit — OpenRazer
    /// always reports "not charging" here without touching the hardware
    /// at all. [`crate::device::RazerDevice::get_charging_status`] (in
    /// `razer-hid-core`) mirrors that: `Ok(false)` with no HID round trip.
    NotApplicable,
    /// This exact model has a battery-level query but no charging-status
    /// command at all in the reference driver (e.g. the Basilisk V3 35K).
    Unsupported,
}

#[derive(Debug, Clone, Copy)]
pub struct BatteryProfile {
    pub command_class: u8,
    pub get_level_command_id: u8,
    pub get_charging_command_id: u8,
    /// Transaction id for the battery-*level* query specifically — check
    /// per-command against the real source, same caveat as
    /// [`DpiProfile::transaction_id`]: this is not guaranteed to match
    /// [`ChargingStatusSupport::Supported`]'s transaction id even though
    /// it happens to for every model confirmed so far.
    pub transaction_id: u8,
    pub charging_status: ChargingStatusSupport,
}

/// Wireless power-saving: how long the device sits idle before it drops
/// into its low-power sleep state. OpenRazer's own range (`clamp(60,
/// 900)` in `razer_chroma_misc_set_idle_time`) is the hardware's actual
/// limit, not an arbitrary daemon choice, so it's enforced here too.
///
/// This is a genuinely separate transaction-id grouping from
/// [`BatteryProfile`] — cross-checking both switch statements in
/// `razermouse_driver.c` for several models (e.g. the Basilisk V3 Pro,
/// which is 0x1F for idle time but 0xFF for battery) turned up real
/// differences, not just an assumption worth flagging. Always check this
/// command on its own, not alongside battery.
#[derive(Debug, Clone, Copy)]
pub struct IdleTimeProfile {
    pub command_class: u8,
    pub get_command_id: u8,
    pub set_command_id: u8,
    pub transaction_id: u8,
    pub min_seconds: u16,
    pub max_seconds: u16,
}

/// Low-battery warning threshold: once charge drops below this percent,
/// the device starts blinking its battery LED (hardware-side behavior,
/// not something this crate implements). OpenRazer encodes the
/// percentage as `floor(percent * 255 / 100)` on the wire, clamped to
/// 0x0C..=0x3F (roughly 5%-25%) — the percent/byte conversion and clamp
/// live in `razer-hid-core`'s `RazerDevice`, since that's where the HID
/// round trip actually happens; this struct is pure data.
///
/// Also its own transaction-id grouping, distinct from both
/// [`BatteryProfile`] and [`IdleTimeProfile`] — e.g. the Cobra
/// HyperSpeed Wireless and Wired variants land in *different* groups for
/// this command despite matching on every other command. Don't assume
/// wired/wireless siblings share this.
#[derive(Debug, Clone, Copy)]
pub struct LowBatteryThresholdProfile {
    pub command_class: u8,
    pub get_command_id: u8,
    pub set_command_id: u8,
    pub transaction_id: u8,
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
    /// `None` for wired-only devices with no onboard battery — only
    /// wireless/dock-chargeable models answer this command at all.
    pub battery: Option<BatteryProfile>,
    /// Wireless power-saving idle timer. `None` for devices that don't
    /// answer this command (not always the same set as [`battery`](Self::battery) —
    /// see [`IdleTimeProfile`]'s doc comment).
    pub idle_time: Option<IdleTimeProfile>,
    /// Low-battery warning threshold. `None` for devices that don't
    /// answer this command — see [`LowBatteryThresholdProfile`]'s doc
    /// comment for why this doesn't always line up with [`battery`](Self::battery)
    /// or [`idle_time`](Self::idle_time) either.
    pub low_battery_threshold: Option<LowBatteryThresholdProfile>,
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
