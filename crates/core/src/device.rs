//! Generic control interface, driven by a [`DeviceProfile`] rather than
//! being hardcoded to one model. Every supported peripheral goes through
//! this same code path.

use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};
use razer_hid_abi::constants::*;
use razer_hid_abi::{DeviceProfile, LedZoneProfile};
use razer_hid_protocol::RazerReport;

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("no supported Razer device found (check cable / udev rules)")]
    NotFound,
    #[error("no device matching product id 0x{0:04x} is connected")]
    NotConnected(u16),
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("protocol error: {0}")]
    Protocol(#[from] razer_hid_protocol::ProtocolError),
    #[error("device rejected command class=0x{class:02x} id=0x{id:02x}")]
    Rejected { class: u8, id: u8 },
    #[error("command class=0x{class:02x} id=0x{id:02x} not supported by this firmware")]
    NotSupported { class: u8, id: u8 },
    #[error(
        "class=0x{class:02x} id=0x{id:02x}: device never confirmed this command after 5 \
         attempts (last response status=0x{status:02x}) — it may be accepted at the USB \
         level but not actually processed by the firmware for this command"
    )]
    UnconfirmedCommand { class: u8, id: u8, status: u8 },
    #[error("value out of range: {0}")]
    OutOfRange(String),
    #[error("{0} is not supported by {1}")]
    Unsupported(&'static str, &'static str),
    #[error(
        "found {0} matching USB interface(s) for {1}, but none of them accepted a feature \
         report (last error: {2}) — on Windows in particular, the plain mouse/keyboard \
         interface rejects Set/GetFeature and only a separate vendor-control interface accepts it."
    )]
    NoWorkingInterface(usize, &'static str, Box<DeviceError>),
}

pub struct DpiSetting {
    pub x: u16,
    pub y: u16,
}

/// Raw info about one HID interface, for diagnosing which one a device's
/// control traffic actually lives on.
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
        let mut report = RazerReport::new(
            identity.command_class,
            identity.firmware_command_id,
            vec![0u8; 2],
        )
        .expect("2-byte probe args never exceed ARGS_LENGTH");
        report.transaction_id = identity.firmware_transaction_id;
        report
    } else if let Some(dpi) = profile.dpi {
        let mut report = RazerReport::new(dpi.command_class, dpi.get_command_id, vec![0u8; 7])
            .expect("7-byte probe args never exceed ARGS_LENGTH");
        report.transaction_id = dpi.transaction_id;
        report
    } else {
        RazerReport::new(0x00, 0x00, Vec::new()).expect("empty probe args never exceed ARGS_LENGTH")
    }
}

impl RazerDevice {
    /// Enumerates which of `known_devices` are currently connected,
    /// without opening them. This crate has no device database of its
    /// own — pass `razer_hid_devices::KNOWN_DEVICES`, or any other slice
    /// of profiles you want to check against.
    pub fn list_connected(
        known_devices: &'static [DeviceProfile],
    ) -> Result<Vec<&'static DeviceProfile>, DeviceError> {
        let api = HidApi::new()?;
        let mut found = Vec::new();
        for profile in known_devices {
            if api
                .device_list()
                .any(|d| d.vendor_id() == profile.vendor_id && d.product_id() == profile.product_id)
            {
                found.push(profile);
            }
        }
        Ok(found)
    }

    /// Opens the first connected device that matches any profile in
    /// `known_devices`.
    pub fn open_first_supported(
        known_devices: &'static [DeviceProfile],
    ) -> Result<Self, DeviceError> {
        let connected = Self::list_connected(known_devices)?;
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

    /// Mirrors OpenRazer's `razer_send_payload` retry loop exactly (see
    /// `driver/razermouse_driver.c`): on each of up to 5 attempts, resend
    /// the full command, then read back a response and accept it only if
    /// it actually echoes the same command class/id (not a stale or
    /// all-zero buffer left over from something else) AND its status is
    /// `SUCCESSFUL` or `BUSY` — OpenRazer treats "busy" as success too.
    /// Anything else retries. This matters because a naive "anything
    /// except FAILURE/NOT_SUPPORTED is fine" check (what this used to do)
    /// treats a stale status=0x00 readback as success, which looks like
    /// the command worked right up until you check the hardware and
    /// nothing happened.
    fn transact(&self, report: &RazerReport) -> Result<RazerReport, DeviceError> {
        let payload = report.pack();
        let mut last_response: Option<RazerReport> = None;
        let mut last_err: Option<DeviceError> = None;

        for _ in 0..5 {
            if let Err(e) = self.hid.send_feature_report(&payload) {
                last_err = Some(e.into());
                sleep(Duration::from_millis(10));
                continue;
            }

            sleep(Duration::from_millis(10));

            let mut buf = [0u8; REPORT_LENGTH];
            buf[0] = 0x00; // report id to fetch
            let response = match self.hid.get_feature_report(&mut buf) {
                Ok(n) => match RazerReport::unpack(&buf[..n]) {
                    Ok(r) => r,
                    Err(e) => {
                        last_err = Some(e.into());
                        sleep(Duration::from_millis(10));
                        continue;
                    }
                },
                Err(e) => {
                    last_err = Some(e.into());
                    sleep(Duration::from_millis(10));
                    continue;
                }
            };

            let echoes_request = response.command_class == report.command_class
                && response.command_id == report.command_id;

            if echoes_request && matches!(response.status, STATUS_SUCCESSFUL | STATUS_BUSY) {
                return Ok(response);
            }

            last_response = Some(response);
            sleep(Duration::from_millis(10));
        }

        if let Some(response) = last_response {
            return Err(match response.status {
                STATUS_FAILURE => DeviceError::Rejected {
                    class: report.command_class,
                    id: report.command_id,
                },
                STATUS_NOT_SUPPORTED => DeviceError::NotSupported {
                    class: report.command_class,
                    id: report.command_id,
                },
                other => DeviceError::UnconfirmedCommand {
                    class: report.command_class,
                    id: report.command_id,
                    status: other,
                },
            });
        }

        Err(last_err.unwrap_or(DeviceError::UnconfirmedCommand {
            class: report.command_class,
            id: report.command_id,
            status: 0xFF,
        }))
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
            razer_hid_abi::commands::Storage::Variable as u8,
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            0x00,
            0x00,
        ];
        let mut report = RazerReport::new(dpi.command_class, dpi.set_command_id, args)?;
        report.transaction_id = dpi.transaction_id;
        self.transact(&report)?;
        Ok(())
    }

    pub fn get_dpi(&self) -> Result<DpiSetting, DeviceError> {
        let dpi = self
            .profile
            .dpi
            .ok_or(DeviceError::Unsupported("DPI", self.profile.name))?;
        let args = vec![
            razer_hid_abi::commands::Storage::None as u8,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut report = RazerReport::new(dpi.command_class, dpi.get_command_id, args)?;
        report.transaction_id = dpi.transaction_id;
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
        let rate_byte = razer_hid_abi::commands::PollRateHz::from_hz(hz)
            .ok_or_else(|| DeviceError::OutOfRange(format!("unsupported poll rate {hz}Hz")))?;
        let mut report = RazerReport::new(
            poll.command_class,
            poll.set_command_id,
            vec![rate_byte as u8],
        )?;
        report.transaction_id = poll.transaction_id;
        self.transact(&report)?;
        Ok(())
    }

    /// Queries the device's actual current polling rate, rather than
    /// assuming whatever was last requested (useful on connect, so the
    /// UI can show/highlight the real state instead of a guess).
    pub fn get_poll_rate(&self) -> Result<u32, DeviceError> {
        let poll = self
            .profile
            .poll_rate
            .ok_or(DeviceError::Unsupported("polling rate", self.profile.name))?;
        let mut report = RazerReport::new(poll.command_class, poll.get_command_id, vec![0u8])?;
        report.transaction_id = poll.transaction_id;
        let resp = self.transact(&report)?;
        let raw = resp.arguments.first().copied().unwrap_or(0);
        razer_hid_abi::commands::PollRateHz::from_byte(raw).ok_or_else(|| {
            DeviceError::OutOfRange(format!(
                "device reported an unrecognized poll rate byte 0x{raw:02x}"
            ))
        })
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
        self.send_led_effect(zone, razer_hid_abi::commands::LedEffect::Off, &[0; 3])
    }

    pub fn set_led_static(&self, zone: &LedZoneProfile, rgb: [u8; 3]) -> Result<(), DeviceError> {
        // args[3..5] = 0, args[5] = 0x01, args[6..8] = rgb
        let tail = [0x00, 0x00, 0x01, rgb[0], rgb[1], rgb[2]];
        self.send_led_effect(zone, razer_hid_abi::commands::LedEffect::Static, &tail)
    }

    pub fn set_led_breathing(
        &self,
        zone: &LedZoneProfile,
        rgb: [u8; 3],
    ) -> Result<(), DeviceError> {
        // args[3] = 0x01 (one color), args[4] = 0, args[5] = 0x01, args[6..8] = rgb
        let tail = [0x01, 0x00, 0x01, rgb[0], rgb[1], rgb[2]];
        self.send_led_effect(zone, razer_hid_abi::commands::LedEffect::Breathing, &tail)
    }

    pub fn set_led_spectrum(&self, zone: &LedZoneProfile) -> Result<(), DeviceError> {
        self.send_led_effect(zone, razer_hid_abi::commands::LedEffect::Spectrum, &[0; 3])
    }

    pub fn set_led_wave(&self, zone: &LedZoneProfile, direction: u8) -> Result<(), DeviceError> {
        // args[3] = direction (0-2), args[4] = speed (0x28 is OpenRazer's
        // fixed default — lower is faster), args[5] = 0
        let direction = direction.min(2);
        let tail = [direction, 0x28, 0x00];
        self.send_led_effect(zone, razer_hid_abi::commands::LedEffect::Wave, &tail)
    }

    /// `tail` is always exactly 3 or 6 bytes: the args *after* the 3-byte
    /// [storage, led_id, effect_id] header, giving a 6- or 9-byte report
    /// matching OpenRazer's `arg_size` for that effect.
    fn send_led_effect(
        &self,
        zone: &LedZoneProfile,
        effect: razer_hid_abi::commands::LedEffect,
        tail: &[u8],
    ) -> Result<(), DeviceError> {
        self.send_effect_report(
            zone.command_class,
            zone.effect_command_id,
            zone.transaction_id,
            zone.id,
            effect,
            tail,
        )
    }

    /// Shared wire format between a real zone and the matrix's own effect
    /// commands (see [`set_matrix_wave`](Self::set_matrix_wave)) — both
    /// just address a different LED id.
    fn send_effect_report(
        &self,
        command_class: u8,
        command_id: u8,
        transaction_id: u8,
        led_id: u8,
        effect: razer_hid_abi::commands::LedEffect,
        tail: &[u8],
    ) -> Result<(), DeviceError> {
        let mut args = vec![
            razer_hid_abi::commands::Storage::Variable as u8,
            led_id,
            effect as u8,
        ];
        args.extend_from_slice(tail);
        let mut report = RazerReport::new(command_class, command_id, args)?;
        report.transaction_id = transaction_id;
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
        let args = vec![
            razer_hid_abi::commands::Storage::Variable as u8,
            zone.id,
            value,
        ];
        let mut report = RazerReport::new(zone.command_class, cmd_id, args)?;
        report.transaction_id = zone.transaction_id;
        self.transact(&report)?;
        Ok(())
    }

    // -- Apply one action to every zone (Razer Synapse's "All" view) --------

    /// Delay between consecutive zone/matrix writes when one action
    /// touches more than one LED in a row. Observed necessity: without
    /// it, sending two zone writes back-to-back can silently drop the
    /// *first* zone's effect (as if the firmware doesn't have enough
    /// settling time between HID writes) — a single write on its own
    /// always works. The CLI's single-zone commands never hit this since
    /// they only ever send one write per process invocation.
    const INTER_ZONE_DELAY: Duration = Duration::from_millis(60);

    /// Runs `f` against every LED zone on the device, in order, pausing
    /// [`INTER_ZONE_DELAY`](Self::INTER_ZONE_DELAY) between writes.
    /// Continues past a failing zone rather than stopping early, and
    /// returns the *first* error encountered (if any) only after every
    /// zone has been attempted.
    fn for_each_zone(
        &self,
        mut f: impl FnMut(&LedZoneProfile) -> Result<(), DeviceError>,
    ) -> Result<(), DeviceError> {
        let mut first_err = None;
        for (i, zone) in self.led_zones().iter().enumerate() {
            if i > 0 {
                sleep(Self::INTER_ZONE_DELAY);
            }
            if let Err(e) = f(zone) {
                first_err.get_or_insert(e);
            }
        }
        match first_err {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }

    /// Sets every zone to the same solid color, and — since the matrix/
    /// underglow mechanism's only comparable capability is painting a
    /// solid color too — also fills it with the same color if the device
    /// has one. This is "All zones" from the CLI/GUI's point of view.
    pub fn set_led_static_all(&self, rgb: [u8; 3]) -> Result<(), DeviceError> {
        let mut result = self.for_each_zone(|zone| self.set_led_static(zone, rgb));
        if self.profile.matrix.is_some() {
            sleep(Self::INTER_ZONE_DELAY);
            if let Err(e) = self.fill_matrix(rgb) {
                result = result.and(Err(e));
            }
        }
        result
    }

    /// Turns every zone off, including the underglow/matrix if present.
    pub fn set_led_off_all(&self) -> Result<(), DeviceError> {
        let mut result = self.for_each_zone(|zone| self.set_led_off(zone));
        if self.profile.matrix.is_some() {
            sleep(Self::INTER_ZONE_DELAY);
            if let Err(e) = self.fill_matrix([0, 0, 0]) {
                result = result.and(Err(e));
            }
        }
        result
    }

    /// Sets every zone to Breathing with the same color. No matrix
    /// equivalent — confirmed devices reject Breathing addressed to the
    /// matrix's own LED id the same way they reject it on real zones.
    pub fn set_led_breathing_all(&self, rgb: [u8; 3]) -> Result<(), DeviceError> {
        self.for_each_zone(|zone| self.set_led_breathing(zone, rgb))
    }

    /// Sets every zone to Spectrum cycling, and the matrix too if it
    /// answers the same effect command under its own LED id (see
    /// [`set_matrix_spectrum`](Self::set_matrix_spectrum)).
    pub fn set_led_spectrum_all(&self) -> Result<(), DeviceError> {
        let mut result = self.for_each_zone(|zone| self.set_led_spectrum(zone));
        if self.profile.matrix.and_then(|m| m.effect_led_id).is_some() {
            sleep(Self::INTER_ZONE_DELAY);
            if let Err(e) = self.set_matrix_spectrum() {
                result = result.and(Err(e));
            }
        }
        result
    }

    /// Sets every zone to Wave, and the matrix too if it answers the same
    /// effect command under its own LED id (see
    /// [`set_matrix_wave`](Self::set_matrix_wave)).
    pub fn set_led_wave_all(&self, direction: u8) -> Result<(), DeviceError> {
        let mut result = self.for_each_zone(|zone| self.set_led_wave(zone, direction));
        if self.profile.matrix.and_then(|m| m.effect_led_id).is_some() {
            sleep(Self::INTER_ZONE_DELAY);
            if let Err(e) = self.set_matrix_wave(direction) {
                result = result.and(Err(e));
            }
        }
        result
    }

    /// Sets brightness on every zone that supports it, silently skipping
    /// zones that don't (brightness support can vary per zone on the same
    /// device) rather than failing the whole batch over one zone.
    pub fn set_led_brightness_all(&self, percent: u8) -> Result<(), DeviceError> {
        self.for_each_zone(|zone| {
            if zone.brightness_command_id.is_none() {
                return Ok(());
            }
            self.set_led_brightness(zone, percent)
        })
    }

    // -- Addressable LED matrix (e.g. an underglow strip) --------------------

    /// `(rows, cols)` of the addressable matrix, if this device has one
    /// separate from its simple single-color [`LedZoneProfile`] zones.
    pub fn matrix_dims(&self) -> Option<(u8, u8)> {
        self.profile.matrix.map(|m| (m.rows, m.cols))
    }

    /// Writes one row of raw RGB pixel data starting at `start_col`.
    /// Doesn't take effect until [`commit_matrix_frame`](Self::commit_matrix_frame)
    /// is called afterward — this only stages the data.
    pub fn set_matrix_row(
        &self,
        row: u8,
        start_col: u8,
        colors: &[[u8; 3]],
    ) -> Result<(), DeviceError> {
        let matrix = self
            .profile
            .matrix
            .ok_or(DeviceError::Unsupported("LED matrix", self.profile.name))?;

        if row >= matrix.rows {
            return Err(DeviceError::OutOfRange(format!(
                "row {row} out of range (0..{})",
                matrix.rows
            )));
        }
        let end_col = start_col as usize + colors.len();
        if colors.is_empty() || end_col > matrix.cols as usize {
            return Err(DeviceError::OutOfRange(format!(
                "columns {start_col}..{end_col} out of range (0..{})",
                matrix.cols
            )));
        }

        let data_size = match matrix.write_packet_length {
            razer_hid_abi::MatrixPacketLength::Fixed(n) => n as usize,
            razer_hid_abi::MatrixPacketLength::Computed => 5 + colors.len() * 3,
        };
        let mut args = vec![0u8; data_size];
        args[2] = row;
        args[3] = start_col;
        args[4] = start_col + colors.len() as u8 - 1; // stop_col, inclusive
        for (i, rgb) in colors.iter().enumerate() {
            let offset = 5 + i * 3;
            args[offset..offset + 3].copy_from_slice(rgb);
        }

        let mut report =
            RazerReport::new(matrix.write_command_class, matrix.write_command_id, args)?;
        report.transaction_id = matrix.write_transaction_id;
        self.transact(&report)?;
        Ok(())
    }

    /// Tells the device to display whatever was most recently staged via
    /// [`set_matrix_row`](Self::set_matrix_row).
    pub fn commit_matrix_frame(&self) -> Result<(), DeviceError> {
        let matrix = self
            .profile
            .matrix
            .ok_or(DeviceError::Unsupported("LED matrix", self.profile.name))?;

        let mut args = vec![0u8; 12];
        args[2] = razer_hid_abi::commands::LedEffect::Custom as u8;

        let mut report =
            RazerReport::new(matrix.commit_command_class, matrix.commit_command_id, args)?;
        report.transaction_id = matrix.commit_transaction_id;
        self.transact(&report)?;
        Ok(())
    }

    /// Convenience: fills every column of row 0 with one color and
    /// commits immediately. For a device with more than one row, use
    /// [`set_matrix_row`](Self::set_matrix_row) directly per row instead.
    pub fn fill_matrix(&self, rgb: [u8; 3]) -> Result<(), DeviceError> {
        let (_, cols) = self
            .matrix_dims()
            .ok_or(DeviceError::Unsupported("LED matrix", self.profile.name))?;
        let colors = vec![rgb; cols as usize];
        self.set_matrix_row(0, 0, &colors)?;
        self.commit_matrix_frame()
    }

    /// LED id/command bytes for the matrix's *own* effect commands (as
    /// opposed to the raw-pixel custom-frame write), for devices
    /// confirmed to answer them — see [`MatrixProfile::effect_led_id`].
    fn matrix_effect_target(&self) -> Result<(u8, u8, u8, u8), DeviceError> {
        let matrix = self
            .profile
            .matrix
            .ok_or(DeviceError::Unsupported("LED matrix", self.profile.name))?;
        let led_id = matrix
            .effect_led_id
            .ok_or(DeviceError::Unsupported("matrix effect", self.profile.name))?;
        Ok((
            matrix.commit_command_class,
            matrix.commit_command_id,
            matrix.commit_transaction_id,
            led_id,
        ))
    }

    /// Hardware-animated Spectrum cycling across the matrix, unlike
    /// [`fill_matrix`](Self::fill_matrix) which can only paint one static
    /// frame. Only on devices where [`MatrixProfile::effect_led_id`] is `Some`.
    pub fn set_matrix_spectrum(&self) -> Result<(), DeviceError> {
        let (class, cmd, txn, led_id) = self.matrix_effect_target()?;
        self.send_effect_report(
            class,
            cmd,
            txn,
            led_id,
            razer_hid_abi::commands::LedEffect::Spectrum,
            &[0; 3],
        )
    }

    /// Hardware-animated Wave across the matrix. Only on devices where
    /// [`MatrixProfile::effect_led_id`] is `Some`.
    pub fn set_matrix_wave(&self, direction: u8) -> Result<(), DeviceError> {
        let (class, cmd, txn, led_id) = self.matrix_effect_target()?;
        let direction = direction.min(2);
        let tail = [direction, 0x28, 0x00];
        self.send_effect_report(
            class,
            cmd,
            txn,
            led_id,
            razer_hid_abi::commands::LedEffect::Wave,
            &tail,
        )
    }

    // -- Identity ------------------------------------------------------------

    pub fn get_firmware_version(&self) -> Result<String, DeviceError> {
        let identity = self.profile.identity.ok_or(DeviceError::Unsupported(
            "firmware query",
            self.profile.name,
        ))?;
        let mut report = RazerReport::new(
            identity.command_class,
            identity.firmware_command_id,
            vec![0u8; 2],
        )?;
        report.transaction_id = identity.firmware_transaction_id;
        let resp = self.transact(&report)?;
        Ok(format!("v{}.{}", resp.arguments[0], resp.arguments[1]))
    }

    pub fn get_serial(&self) -> Result<String, DeviceError> {
        let identity = self
            .profile
            .identity
            .ok_or(DeviceError::Unsupported("serial query", self.profile.name))?;
        let mut report = RazerReport::new(
            identity.command_class,
            identity.serial_command_id,
            vec![0u8; 22],
        )?;
        report.transaction_id = identity.serial_transaction_id;
        let resp = self.transact(&report)?;
        let end = resp
            .arguments
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(resp.arguments.len());
        Ok(String::from_utf8_lossy(&resp.arguments[..end]).into_owned())
    }
}
