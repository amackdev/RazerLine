//! Generic control interface, driven by a [`DeviceProfile`] rather than
//! being hardcoded to one model. Every supported peripheral goes through
//! this same code path.

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
}

pub struct DpiSetting {
    pub x: u16,
    pub y: u16,
}

pub struct RazerDevice {
    hid: HidDevice,
    pub profile: &'static DeviceProfile,
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
    pub fn open(profile: &'static DeviceProfile) -> Result<Self, DeviceError> {
        let api = HidApi::new()?;

        let info = api
            .device_list()
            .filter(|d| d.vendor_id() == profile.vendor_id && d.product_id() == profile.product_id)
            // Prefer the vendor-defined control collection over the plain
            // input/cursor interface, when the backend reports usage pages.
            .max_by_key(|d| u16::from(d.usage_page() == 0xFF00))
            .ok_or(DeviceError::NotConnected(profile.product_id))?;

        let hid = info.open_device(&api)?;
        Ok(Self { hid, profile })
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
            0x00, // dpi stage, 0 = active profile
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (y >> 8) as u8,
            (y & 0xFF) as u8,
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
        let report = RazerReport::new(dpi.command_class, dpi.get_command_id, vec![0u8; 5])?;
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

    pub fn set_led_off(&self, zone: &LedZoneProfile) -> Result<(), DeviceError> {
        self.set_led_effect(zone, crate::commands::LedEffect::Off, &[])
    }

    pub fn set_led_static(&self, zone: &LedZoneProfile, rgb: [u8; 3]) -> Result<(), DeviceError> {
        self.set_led_effect(zone, crate::commands::LedEffect::Static, &rgb)
    }

    pub fn set_led_breathing(
        &self,
        zone: &LedZoneProfile,
        rgb: [u8; 3],
    ) -> Result<(), DeviceError> {
        let args = [0x01, rgb[0], rgb[1], rgb[2]];
        self.set_led_effect(zone, crate::commands::LedEffect::Breathing, &args)
    }

    pub fn set_led_spectrum(&self, zone: &LedZoneProfile) -> Result<(), DeviceError> {
        self.set_led_effect(zone, crate::commands::LedEffect::Spectrum, &[])
    }

    pub fn set_led_wave(&self, zone: &LedZoneProfile, direction: u8) -> Result<(), DeviceError> {
        self.set_led_effect(zone, crate::commands::LedEffect::Wave, &[direction])
    }

    fn set_led_effect(
        &self,
        zone: &LedZoneProfile,
        effect: crate::commands::LedEffect,
        extra: &[u8],
    ) -> Result<(), DeviceError> {
        let mut args = vec![zone.id, effect as u8];
        args.extend_from_slice(extra);
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
        let report = RazerReport::new(zone.command_class, cmd_id, vec![zone.id, value])?;
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
            vec![0u8; 20],
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
