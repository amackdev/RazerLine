//! Shared command-class/command-id vocabulary for Razer's HID control
//! protocol, cross-checked against the OpenRazer kernel driver source
//! (`driver/razerchromacommon.c`, GPL-3.0) rather than guessed. Still
//! verify against your own hardware before trusting a *new* profile
//! built from these: OpenRazer confirms these bytes work for the
//! specific models it lists, not for every Razer mouse.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandClass {
    General = 0x00,
    Mouse = 0x04,
    Led = 0x0F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GeneralCommand {
    SetPollRate = 0x05,
    GetPollRate = 0x85,
    GetFirmwareVersion = 0x81,
    GetSerial = 0x82,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MouseCommand {
    SetDpiXY = 0x05,
    GetDpiXY = 0x85,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedCommand {
    SetEffect = 0x02,
    /// Writes one row of raw RGB pixel data for the "custom frame" matrix
    /// mechanism (`razer_chroma_extended_matrix_set_custom_frame2` in
    /// OpenRazer) — see [`crate::devices::MatrixProfile`].
    SetCustomFrame = 0x03,
    SetBrightness = 0x04,
    GetBrightness = 0x84,
}

/// Common LED zone ids (`razercommon.h`'s `*_LED` defines). A device
/// profile is free to define its own zone ids instead (see
/// [`crate::devices::LedZoneProfile`]) if its firmware numbers them
/// differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommonLedId {
    Zero = 0x00,
    ScrollWheel = 0x01,
    Logo = 0x04,
    Backlight = 0x05,
    RightSide = 0x10,
    LeftSide = 0x11,
}

/// Effect ids used by the "extended matrix" LED command
/// (`razer_chroma_extended_matrix_effect_*` in `razerchromacommon.c`).
/// Note the ordering is *not* sequential with "obvious" guesses — e.g.
/// Spectrum is 0x03 and Wave is 0x04, not the other way around.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedEffect {
    Off = 0x00,
    Static = 0x01,
    Breathing = 0x02,
    Spectrum = 0x03,
    Wave = 0x04,
    /// Tells the device to display whatever was most recently written via
    /// the "custom frame" matrix commands (`SetCustomFrame`), rather than
    /// a built-in effect. Sent via `LedCommand::SetEffect`, same as every
    /// other effect id — "custom" is just another effect selection.
    Custom = 0x08,
}

/// "Variable storage" flag used throughout the protocol: whether a
/// setting should persist in the device's onboard memory (`Variable`) or
/// only apply to the current session (`None`). Most modern mice ignore
/// this for some commands (e.g. DPI set always uses `Variable` in
/// practice) and respect it for others — see the specific device profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Storage {
    None = 0x00,
    Variable = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PollRateHz {
    Hz125 = 0x08,
    Hz500 = 0x02,
    Hz1000 = 0x01,
}

impl PollRateHz {
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            125 => Some(Self::Hz125),
            500 => Some(Self::Hz500),
            1000 => Some(Self::Hz1000),
            _ => None,
        }
    }
}
