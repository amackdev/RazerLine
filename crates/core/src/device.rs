//! Generic control interface, driven by a [`DeviceProfile`] rather than
//! being hardcoded to one model. Every supported peripheral goes through
//! this same code path.

use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

use crate::constants::*;
use crate::devices::{DeviceProfile, LedZoneProfile, KNOWN_DEVICES};
use crate::protocol::RazerReport;

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("no supported Razer device found (check cable / udev rules)")]
    NotFound,
    #[error("no device matching product id 0x{0:04x} is connected")]
    NotConnected(u16),
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("protocol error: {0}")]
    Protocol(#[from] crate::protocol::ProtocolError),
    #[error("device rejected command class=0x{class:02x} id=0x{id:02x}")]
    Rejected { class: u8, id: u8 },
    #[error("command class=0x{class:02x} id=0x{id:02x} not supported by this firmware")]
    NotSupported { class: u8, id: u8 },
    #[error("value out of range: {0}")]
    OutOfRange(String),
    #[error("{0} is not supported by {1}")]
    Unsupported(&'static str, &'static str),
    #[error(
        "found {0} matching USB interface(s) for {1}, but none of them accepted a feature \
         report (last error: {2}). Run `razer-ctl debug-usb` and compare against docs/PROTOCOL.md \
         — on Windows in particular, the plain mouse/keyboard interface rejects Set/GetFeature \
         and only a separate vendor-control interface accepts it."
    )]
    NoWorkingInterface(usize, &'static str, Box<DeviceError>),
}

pub struct DpiSetting {
    pub x: u16,
    pub y: u16,
}

/// Raw info about one HID interface, for diagnosing which one a device's
/// control traffic actually lives on (`razer-ctl debug-usb`).
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub product_string: Option<String>,
    pub path: String,
}

pub struct RazerDevice {
    hid: HidDevice,
    pub profile: &'static DeviceProfile,
}

/// Builds a harmless, read-only report to test whether an interface
/// accepts Set/GetFeature at all. Prefers the firmware-version query
/// (profiles that have one will get a genuinely meaningful response);
/// falls back to a DPI query, then to an all-zero report purely to
/// exercise the transport if the profile declares neither.
fn probe_report(profile: &DeviceProfile) -> RazerReport {
    if let Some(identity) = profile.identity {
        RazerReport::new(
            identity.command_class,
            identity.firmware_command_id,
            vec![0u8; 2],
        )
        .expect("2-byte probe args never exceed ARGS_LENGTH")
    } else if let Some(dpi) = profile.dpi {
        RazerReport::new(dpi.command_class, dpi.get_command_id, vec![0u8; 7])
            .expect("7-byte probe args never exceed ARGS_LENGTH")
    } else {
        RazerReport::new(0x00, 0x00, Vec::new()).expect("empty probe args never exceed ARGS_LENGTH")
    }
}

impl RazerDevice {
    /// Enumerates connected, known devices without opening them.
    pub fn list_connected() -> Result<Vec<&'static DeviceProfile>, DeviceError> {
        let api = HidApi::new()?;
        let mut found = Vec::new();
        for profile in KNOWN_DEVICES {
            if api
                .device_list()
                .any(|d| d.vendor_id() == profile.vendor_id && d.product_id() == profile.product_id)
            {
                found.push(profile);
            }
        }
        Ok(found)
    }

    /// Opens the first connected device that matches any known profile.
    pub fn open_first_supported() -> Result<Self, DeviceError> {
        let connected = Self::list_connected()?;
        let profile = connected.first().ok_or(DeviceError::NotFound)?;
        Self::open(profile)
    }

    /// Opens a specific known profile; errors if that model isn't plugged in.
    ///
    /// A Razer peripheral enumerates as *several* HID interfaces (e.g. one
    /// plain input collection for cursor/key events, one vendor-defined
    /// collection for control commands). Which one accepts Set/GetFeature
    /// calls varies by platform and device — notably, Windows refuses
    /// direct Feature-report I/O on collections it treats as the system
    /// mouse/keyboard, so guessing by usage page alone is not reliable.
    /// Instead, every matching interface is tried in turn with a harmless
    /// probe report; whichever one actually accepts it is used.
    pub fn open(profile: &'static DeviceProfile) -> Result<Self, DeviceError> {
        let api = HidApi::new()?;

        // Prefer interfaces that look vendor-defined (usage page 0xFF00 is
        // the most common convention for Razer's control collection), but
        // still try every match — this ranking is just to avoid
        // unnecessary probe attempts, not a hard requirement.
        let mut candidates: Vec<(CString, bool)> = api
            .device_list()
            .filter(|d| d.vendor_id() == profile.vendor_id && d.product_id() == profile.product_id)
            .map(|d| (d.path().to_owned(), d.usage_page() == 0xFF00))
            .collect();

        if candidates.is_empty() {
            return Err(DeviceError::NotConnected(profile.product_id));
        }
        candidates.sort_by_key(|(_, looks_vendor)| !looks_vendor);

        let probe = probe_report(profile).pack();
        let attempted = candidates.len();
        let mut last_err: Option<DeviceError> = None;

        for (path, _) in candidates {
            let hid = match api.open_path(&path) {
                Ok(hid) => hid,
                Err(e) => {
                    last_err = Some(e.into());
                    continue;
                }
            };
            match hid.send_feature_report(&probe) {
                Ok(_) => return Ok(Self { hid, profile }),
                Err(e) => last_err = Some(e.into()),
            }
        }

        Err(DeviceError::NoWorkingInterface(
            attempted,
            profile.name,
            Box::new(last_err.unwrap_or(DeviceError::NotConnected(profile.product_id))),
        ))
    }

    /// Lists every HID interface on the system under the given vendor id,
    /// without filtering to known profiles — useful for figuring out which
    /// interface a new or misbehaving device's control traffic lives on.
    /// See `docs/PROTOCOL.md` / `CONTRIBUTING.md`.
    pub fn debug_list_interfaces(vendor_id: u16) -> Result<Vec<InterfaceInfo>, DeviceError> {
        let api = HidApi::new()?;
        Ok(api
            .device_list()
            .filter(|d| d.vendor_id() == vendor_id)
            .map(|d| InterfaceInfo {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                interface_number: d.interface_number(),
                usage_page: d.usage_page(),
                usage: d.usage(),
                product_string: d.product_string().map(|s| s.to_string()),
                path: d.path().to_string_lossy().into_owned(),
            })
            .collect())
    }

    fn transact(&self, report: &RazerReport) -> Result<RazerReport, DeviceError> {
        let payload = report.pack();
        self.hid.send_feature_report(&payload)?;

        // Give the device a moment to process before polling the response.
        sleep(Duration::from_millis(10));

        let mut buf = [0u8; REPORT_LENGTH];
        buf[0] = 0x00; // report id to fetch
        let n = self.hid.get_feature_report(&mut buf)?;
        let response = RazerReport::unpack(&buf[..n])?;

        match response.status {
            STATUS_FAILURE => Err(DeviceError::Rejected {
                class: report.command_class,
                id: report.command_id,
            }),
            STATUS_NOT_SUPPORTED => Err(DeviceError::NotSupported {
                class: report.command_class,
                id: report.command_id,
            }),
            _ => Ok(response),
        }
    }

    // -- DPI -----------------------------------------------------------

    /// `razer_chroma_misc_set_dpi_xy` always sends `Storage::Variable`
    /// regardless of what's asked for (OpenRazer's own implementation
    /// ignores its `variable_storage` parameter here) — replicated as-is
    /// since that's what the firmware actually receives from the
    /// reference driver.
    pub fn set_dpi(&self, x: u16, y: u16) -> Result<(), DeviceError> {
        let dpi = self
            .profile
            .dpi
            .ok_or(DeviceError::Unsupported("DPI", self.profile.name))?;
        for (value, axis) in [(x, "x"), (y, "y")] {
            if !(dpi.min..=dpi.max).contains(&value) {
                return Err(DeviceError::OutOfRange(format!(
                    "DPI {axis}={value} out of range ({}-{})",
                    dpi.min, dpi.max
                )));
            }
        }
        let args = vec![
            crate::commands::Storage::Variable as u8,
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            0x00,
            0x00,
        ];
        let report = RazerReport::new(dpi.command_class, dpi.set_command_id, args)?;
        self.transact(&report)?;
        Ok(())
    }

    pub fn get_dpi(&self) -> Result<DpiSetting, DeviceError> {
        let dpi = self
            .profile
            .dpi
            .ok_or(DeviceError::Unsupported("DPI", self.profile.name))?;
        let args = vec![crate::commands::Storage::None as u8, 0, 0, 0, 0, 0, 0];
        let report = RazerReport::new(dpi.command_class, dpi.get_command_id, args)?;
        let resp = self.transact(&report)?;
        let a = &resp.arguments;
        Ok(DpiSetting {
            x: ((a[1] as u16) << 8) | a[2] as u16,
            y: ((a[3] as u16) << 8) | a[4] as u16,
        })
    }

    // -- Polling rate -----------------------------------------------------

    pub fn set_poll_rate(&self, hz: u32) -> Result<(), DeviceError> {
        let poll = self
            .profile
            .poll_rate
            .ok_or(DeviceError::Unsupported("polling rate", self.profile.name))?;
        if !poll.supported_hz.contains(&hz) {
            return Err(DeviceError::OutOfRange(format!(
                "unsupported poll rate {hz}Hz (supported: {:?})",
                poll.supported_hz
            )));
        }
        let rate_byte = crate::commands::PollRateHz::from_hz(hz)
            .ok_or_else(|| DeviceError::OutOfRange(format!("unsupported poll rate {hz}Hz")))?;
        let report = RazerReport::new(
            poll.command_class,
            poll.set_command_id,
            vec![rate_byte as u8],
        )?;
        self.transact(&report)?;
        Ok(())
    }

    // -- Lighting -----------------------------------------------------------

    pub fn led_zones(&self) -> &'static [LedZoneProfile] {
        self.profile.leds
    }

    // Argument layout for every extended-matrix effect follows
    // `razer_chroma_extended_matrix_effect_base` in OpenRazer: byte 0 is
    // always `Storage::Variable`, byte 1 the LED zone id, byte 2 the
    // effect id — then effect-specific bytes at fixed offsets (not
    // necessarily packed immediately after the header; there are
    // deliberate zero gaps that match what the firmware expects).

    pub fn set_led_off(&self, zone: &LedZoneProfile) -> Result<(), DeviceError> {
        self.send_led_effect(zone, crate::commands::LedEffect::Off, &[0; 3])
    }

    pub fn set_led_static(&self, zone: &LedZoneProfile, rgb: [u8; 3]) -> Result<(), DeviceError> {
        // args[3..5] = 0, args[5] = 0x01, args[6..8] = rgb
        let tail = [0x00, 0x00, 0x01, rgb[0], rgb[1], rgb[2]];
        self.send_led_effect(zone, crate::commands::LedEffect::Static, &tail)
    }

    pub fn set_led_breathing(
        &self,
        zone: &LedZoneProfile,
        rgb: [u8; 3],
    ) -> Result<(), DeviceError> {
        // args[3] = 0x01 (one color), args[4] = 0, args[5] = 0x01, args[6..8] = rgb
        let tail = [0x01, 0x00, 0x01, rgb[0], rgb[1], rgb[2]];
        self.send_led_effect(zone, crate::commands::LedEffect::Breathing, &tail)
    }

    pub fn set_led_spectrum(&self, zone: &LedZoneProfile) -> Result<(), DeviceError> {
        self.send_led_effect(zone, crate::commands::LedEffect::Spectrum, &[0; 3])
    }

    pub fn set_led_wave(&self, zone: &LedZoneProfile, direction: u8) -> Result<(), DeviceError> {
        // args[3] = direction (0-2), args[4] = speed (0x28 is OpenRazer's
        // fixed default — lower is faster), args[5] = 0
        let direction = direction.min(2);
        let tail = [direction, 0x28, 0x00];
        self.send_led_effect(zone, crate::commands::LedEffect::Wave, &tail)
    }

    /// `tail` is always exactly 3 or 6 bytes: the args *after* the 3-byte
    /// [storage, led_id, effect_id] header, giving a 6- or 9-byte report
    /// matching OpenRazer's `arg_size` for that effect.
    fn send_led_effect(
        &self,
        zone: &LedZoneProfile,
        effect: crate::commands::LedEffect,
        tail: &[u8],
    ) -> Result<(), DeviceError> {
        let mut args = vec![
            crate::commands::Storage::Variable as u8,
            zone.id,
            effect as u8,
        ];
        args.extend_from_slice(tail);
        let report = RazerReport::new(zone.command_class, zone.effect_command_id, args)?;
        self.transact(&report)?;
        Ok(())
    }

    pub fn set_led_brightness(
        &self,
        zone: &LedZoneProfile,
        percent: u8,
    ) -> Result<(), DeviceError> {
        let cmd_id = zone.brightness_command_id.ok_or(DeviceError::Unsupported(
            "LED brightness",
            self.profile.name,
        ))?;
        if percent > 100 {
            return Err(DeviceError::OutOfRange("brightness must be 0-100".into()));
        }
        let value = ((percent as u16) * 255 / 100) as u8;
        let args = vec![crate::commands::Storage::Variable as u8, zone.id, value];
        let report = RazerReport::new(zone.command_class, cmd_id, args)?;
        self.transact(&report)?;
        Ok(())
    }

    // -- Identity ------------------------------------------------------------

    pub fn get_firmware_version(&self) -> Result<String, DeviceError> {
        let identity = self.profile.identity.ok_or(DeviceError::Unsupported(
            "firmware query",
            self.profile.name,
        ))?;
        let report = RazerReport::new(
            identity.command_class,
            identity.firmware_command_id,
            vec![0u8; 2],
        )?;
        let resp = self.transact(&report)?;
        Ok(format!("v{}.{}", resp.arguments[0], resp.arguments[1]))
    }

    pub fn get_serial(&self) -> Result<String, DeviceError> {
        let identity = self
            .profile
            .identity
            .ok_or(DeviceError::Unsupported("serial query", self.profile.name))?;
        let report = RazerReport::new(
            identity.command_class,
            identity.serial_command_id,
            vec![0u8; 22],
        )?;
        let resp = self.transact(&report)?;
        let end = resp
            .arguments
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(resp.arguments.len());
        Ok(String::from_utf8_lossy(&resp.arguments[..end]).into_owned())
    }
}
