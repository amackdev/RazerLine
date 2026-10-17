use std::thread::sleep;
use std::time::{Duration, Instant};

/// Delay between consecutive zone writes when "All zones" fires more than
/// one LED command in a row. The CLI only ever sends one command per
/// invocation and works reliably; sending several back-to-back with no
/// gap appears to let the firmware drop/overwrite an earlier zone's
/// effect — observed as Scroll wheel's color getting clobbered by Logo's
/// write immediately after it. Keep this short; it's not meant to be
/// perceptible as lag, just enough settling time between HID writes.
const INTER_ZONE_DELAY: Duration = Duration::from_millis(60);

use eframe::egui;
use razer_hid_abi::{DeviceProfile, LedZoneProfile};
use razer_hid_client::RazerDevice;

/// Library name (not a file path) of the device-catalog plugin —
/// `razer_hid_client::load_known_devices` looks for this next to the
/// running executable using the platform's DLL naming convention.
const DEVICES_PLUGIN: &str = "razer_hid_devices_mice";

fn known_devices() -> &'static [DeviceProfile] {
    static CELL: std::sync::OnceLock<&'static [DeviceProfile]> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        razer_hid_client::load_known_devices(DEVICES_PLUGIN)
            .expect("couldn't load device catalog plugin")
    })
}

use crate::config::AppConfig;
use crate::theme;
use crate::tray::TrayManager;
use crate::widgets::{self, PreviewKind};

/// How often to actually re-enumerate USB devices. Decoupled from egui's
/// repaint rate — HID enumeration is cheap but not free, and the UI
/// doesn't need to re-check more often than a human would notice a mouse
/// being plugged in.
const SCAN_INTERVAL: Duration = Duration::from_millis(1000);

pub struct GuiApp {
    device: Option<RazerDevice>,
    connected: Vec<&'static DeviceProfile>,
    active_pid: Option<u16>,
    last_scan: Instant,
    status: String,
    status_is_error: bool,

    dpi_x: u16,
    dpi_y: u16,
    /// The device's actual current polling rate, queried on connect —
    /// `None` until that query returns (or if it's not supported), so the
    /// UI shows "no selection" rather than guessing one.
    current_poll_hz: Option<u32>,

    selected_zone_idx: usize,
    /// When true, lighting actions apply to every zone at once (Razer
    /// Synapse's "All" view); when false, only to the selected zone.
    sync_zones: bool,
    led_color: [u8; 3],
    brightness: u8,
    last_preview: PreviewKind,
    current_effect_name: Option<String>,

    matrix_color: [u8; 3],

    info_text: Option<String>,

    // Background & Tray & Config state
    config: AppConfig,
    _tray: Option<TrayManager>,
}

impl GuiApp {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let tray = TrayManager::new().ok();
        let mut app = Self {
            device: None,
            connected: Vec::new(),
            active_pid: None,
            last_scan: Instant::now(),
            status: "Scanning for devices...".to_string(),
            status_is_error: false,
            dpi_x: 800,
            dpi_y: 800,
            current_poll_hz: None,
            selected_zone_idx: 0,
            sync_zones: true,
            led_color: [64, 209, 128],
            brightness: 100,
            last_preview: PreviewKind::Off,
            current_effect_name: None,
            matrix_color: [64, 209, 128],
            info_text: None,
            config,
            _tray: tray,
        };
        app.rescan();
        app
    }

    /// Re-enumerates connected devices and reconciles state against what
    /// was already open — called on a timer from `update()`, not just on
    /// startup, so plugging/unplugging a device is picked up live without
    /// the user doing anything. Does **not** reopen the active device's
    /// handle on every tick: only touches `self.device` when the active
    /// device actually disappeared or something new needs auto-selecting.
    fn rescan(&mut self) {
        let found = match razer_hid_client::list_connected(known_devices()) {
            Ok(found) => found,
            Err(e) => {
                self.set_status(format!("Scan error: {e}"), true);
                return;
            }
        };

        let active_still_present = self
            .active_pid
            .is_some_and(|pid| found.iter().any(|p| p.product_id == pid));

        self.connected = found;

        if !active_still_present {
            self.device = None;
            self.active_pid = None;
            match self.connected.first().copied() {
                Some(profile) => self.open(profile),
                None => self.set_status("No device connected", false),
            }
        }
    }

    /// Opens the specified device profile and restores any previously configured
    /// settings (DPI, polling rate, lighting effects, brightness).
    fn open(&mut self, profile: &'static DeviceProfile) {
        match RazerDevice::open(profile) {
            Ok(dev) => {
                let pid = profile.product_id;
                self.active_pid = Some(pid);
                self.info_text = None;
                self.selected_zone_idx = 0;

                // Load configured settings if present for this device
                if let Some(cfg) = self.config.devices.get(&pid).cloned() {
                    let mut applied_items = Vec::new();

                    // 1. Restore configured DPI
                    if let (Some(x), Some(y)) = (cfg.dpi_x, cfg.dpi_y) {
                        if profile.dpi.is_some() {
                            if dev.set_dpi(x, y).is_ok() {
                                self.dpi_x = x;
                                self.dpi_y = y;
                                applied_items.push("DPI");
                            }
                        }
                    } else if let Ok(current) = dev.get_dpi() {
                        self.dpi_x = current.x;
                        self.dpi_y = current.y;
                    }

                    // 2. Restore configured Polling Rate
                    if let Some(hz) = cfg.poll_rate_hz {
                        if profile.poll_rate.is_some() {
                            if dev.set_poll_rate(hz).is_ok() {
                                self.current_poll_hz = Some(hz);
                                applied_items.push("Polling rate");
                            }
                        }
                    } else {
                        self.current_poll_hz = dev.get_poll_rate().ok();
                    }

                    // 3. Restore configured lighting properties
                    if let Some(sync) = cfg.sync_zones {
                        self.sync_zones = sync;
                    }
                    if let Some(color) = cfg.led_color {
                        self.led_color = color;
                    }
                    if let Some(b) = cfg.brightness {
                        self.brightness = b;
                        for zone in profile.leds {
                            let _ = dev.set_led_brightness(zone, b);
                        }
                    }

                    // 4. Restore configured lighting effect
                    if let Some(ref eff) = cfg.effect {
                        self.current_effect_name = Some(eff.clone());
                        let color = self.led_color;
                        let zones = profile.leds;
                        let has_matrix = profile.matrix.is_some();
                        let matrix_has_effects =
                            profile.matrix.and_then(|m| m.effect_led_id).is_some();

                        match eff.as_str() {
                            "static" => {
                                self.last_preview = PreviewKind::Static;
                                for (i, zone) in zones.iter().enumerate() {
                                    if i > 0 {
                                        sleep(INTER_ZONE_DELAY);
                                    }
                                    let _ = dev.set_led_static(zone, color);
                                }
                                if has_matrix {
                                    sleep(INTER_ZONE_DELAY);
                                    let _ = dev.fill_matrix(color);
                                }
                                applied_items.push("Lighting");
                            }
                            "spectrum" => {
                                self.last_preview = PreviewKind::Spectrum;
                                for (i, zone) in zones.iter().enumerate() {
                                    if i > 0 {
                                        sleep(INTER_ZONE_DELAY);
                                    }
                                    let _ = dev.set_led_spectrum(zone);
                                }
                                if matrix_has_effects {
                                    sleep(INTER_ZONE_DELAY);
                                    let _ = dev.set_matrix_spectrum();
                                }
                                applied_items.push("Lighting");
                            }
                            "breathing" => {
                                self.last_preview = PreviewKind::Breathing;
                                for (i, zone) in zones.iter().enumerate() {
                                    if i > 0 {
                                        sleep(INTER_ZONE_DELAY);
                                    }
                                    let _ = dev.set_led_breathing(zone, color);
                                }
                                applied_items.push("Lighting");
                            }
                            "wave" => {
                                self.last_preview = PreviewKind::Wave;
                                for (i, zone) in zones.iter().enumerate() {
                                    if i > 0 {
                                        sleep(INTER_ZONE_DELAY);
                                    }
                                    let _ = dev.set_led_wave(zone, 1);
                                }
                                if matrix_has_effects {
                                    sleep(INTER_ZONE_DELAY);
                                    let _ = dev.set_matrix_wave(1);
                                }
                                applied_items.push("Lighting");
                            }
                            "off" => {
                                self.last_preview = PreviewKind::Off;
                                for (i, zone) in zones.iter().enumerate() {
                                    if i > 0 {
                                        sleep(INTER_ZONE_DELAY);
                                    }
                                    let _ = dev.set_led_off(zone);
                                }
                                if has_matrix {
                                    sleep(INTER_ZONE_DELAY);
                                    let _ = dev.fill_matrix([0, 0, 0]);
                                }
                                applied_items.push("Lighting");
                            }
                            _ => {}
                        }
                    }

                    // 5. Restore matrix/underglow
                    if let Some(m_color) = cfg.matrix_color {
                        if profile.matrix.is_some() {
                            self.matrix_color = m_color;
                            let _ = dev.fill_matrix(m_color);
                        }
                    }

                    self.device = Some(dev);
                    if applied_items.is_empty() {
                        self.set_status(format!("Connected to {}", profile.name), false);
                    } else {
                        self.set_status(
                            format!(
                                "Connected to {} (restored configured {})",
                                profile.name,
                                applied_items.join(", ")
                            ),
                            false,
                        );
                    }
                } else {
                    // No saved settings for this device yet — query current hardware values
                    if let Ok(current) = dev.get_dpi() {
                        self.dpi_x = current.x;
                        self.dpi_y = current.y;
                    }
                    self.current_poll_hz = dev.get_poll_rate().ok();
                    self.device = Some(dev);
                    self.set_status(format!("Connected to {}", profile.name), false);
                }
            }
            Err(e) => {
                self.device = None;
                self.active_pid = None;
                self.set_status(format!("Couldn't open {}: {e}", profile.name), true);
            }
        }
    }

    /// Persists current device settings to the configuration file.
    fn save_current_device_config(&mut self) {
        if let Some(pid) = self.active_pid {
            let name = self
                .device
                .as_ref()
                .map(|d| d.profile.name)
                .unwrap_or("Razer Device");
            let (x, y) = (self.dpi_x, self.dpi_y);
            let poll = self.current_poll_hz;
            let brightness = self.brightness;
            let color = self.led_color;
            let sync = self.sync_zones;
            let effect = self.current_effect_name.clone();
            let matrix = self
                .device
                .as_ref()
                .and_then(|d| d.matrix_dims())
                .map(|_| self.matrix_color);

            self.config
                .update_device(pid, name, x, y, poll, brightness, color, sync, effect, matrix);
            let _ = self.config.save();
        }
    }

    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = msg.into();
        self.status_is_error = is_error;
    }

    fn apply<T>(&mut self, result: Result<T, razer_hid_client::ClientError>, ok_msg: &str) {
        match result {
            Ok(_) => self.set_status(ok_msg, false),
            Err(e) => self.set_status(format!("Error: {e}"), true),
        }
    }

    /// Runs `f` against either just the selected zone or every zone on the
    /// device, depending on `sync_zones` — the "All" vs. per-zone toggle.
    /// Reports the first failure (naming which zone), or success.
    fn apply_to_zones(
        &mut self,
        zones: &'static [LedZoneProfile],
        selected_idx: usize,
        mut f: impl FnMut(
            &RazerDevice,
            &'static LedZoneProfile,
        ) -> Result<(), razer_hid_client::ClientError>,
        ok_msg: &str,
    ) {
        let targets: &'static [LedZoneProfile] = if self.sync_zones {
            zones
        } else {
            &zones[selected_idx..=selected_idx]
        };
        let dev = self.device.as_ref().unwrap();
        let mut failure = None;
        for (i, zone) in targets.iter().enumerate() {
            if i > 0 {
                sleep(INTER_ZONE_DELAY);
            }
            if let Err(e) = f(dev, zone) {
                failure.get_or_insert_with(|| format!("{} failed: {e}", zone.name));
            }
        }
        match failure {
            None => self.set_status(ok_msg, false),
            Some(msg) => self.set_status(msg, true),
        }
    }

    /// Like [`apply_to_zones`](Self::apply_to_zones), but when "All zones"
    /// is active and `matrix_f` is `Some`, also runs it against the
    /// underglow/matrix so it doesn't sit outside the sync — Synapse
    /// treats it as just another zone for actions it actually supports.
    /// Pass `None` for an action the matrix can't do at all (e.g.
    /// Breathing). Only applies when `sync_zones` is true; in per-zone
    /// mode Underglow has its own card/controls.
    fn apply_to_zones_and_matrix(
        &mut self,
        zones: &'static [LedZoneProfile],
        selected_idx: usize,
        mut zone_f: impl FnMut(
            &RazerDevice,
            &'static LedZoneProfile,
        ) -> Result<(), razer_hid_client::ClientError>,
        matrix_f: Option<impl FnOnce(&RazerDevice) -> Result<(), razer_hid_client::ClientError>>,
        ok_msg: &str,
    ) {
        let targets: &'static [LedZoneProfile] = if self.sync_zones {
            zones
        } else {
            &zones[selected_idx..=selected_idx]
        };
        let dev = self.device.as_ref().unwrap();
        let mut failure = None;
        for (i, zone) in targets.iter().enumerate() {
            if i > 0 {
                sleep(INTER_ZONE_DELAY);
            }
            if let Err(e) = zone_f(dev, zone) {
                failure.get_or_insert_with(|| format!("{} failed: {e}", zone.name));
            }
        }
        if self.sync_zones {
            if let Some(matrix_f) = matrix_f {
                if !targets.is_empty() {
                    sleep(INTER_ZONE_DELAY);
                }
                if let Err(e) = matrix_f(dev) {
                    failure.get_or_insert_with(|| format!("Underglow failed: {e}"));
                }
            }
        }
        match failure {
            None => self.set_status(ok_msg, false),
            Some(msg) => self.set_status(msg, true),
        }
    }
}

impl Default for GuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Window close interception: if close_to_tray is true, hide to tray instead of quitting
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.config.close_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                crate::win_util::hide_window();
            }
        }

        // Live USB detection: re-scan periodically
        if self.last_scan.elapsed() >= SCAN_INTERVAL {
            self.rescan();
            self.last_scan = Instant::now();
        }
        ctx.request_repaint_after(SCAN_INTERVAL);

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(190.0)
            .frame(
                egui::Frame::none()
                    .fill(theme::SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(12.0, 16.0)),
            )
            .show(ctx, |ui| self.ui_sidebar(ui));

        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::none()
                    .fill(theme::SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| self.ui_status_bar(ui));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::APP_BG)
                    .inner_margin(egui::Margin::same(20.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.ui_content(ui));
            });
    }
}

impl GuiApp {
    fn ui_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("RazerLine")
                    .strong()
                    .size(16.0)
                    .color(theme::ACCENT),
            );
        });
        ui.add_space(14.0);
        ui.label(egui::RichText::new("DEVICES").small().color(theme::MUTED));
        ui.add_space(8.0);

        if self.connected.is_empty() {
            ui.label(
                egui::RichText::new("No devices connected")
                    .small()
                    .color(theme::MUTED),
            );
            return;
        }

        let mut clicked: Option<&'static DeviceProfile> = None;
        for profile in self.connected.iter().copied() {
            let is_active = self.active_pid == Some(profile.product_id);

            egui::Frame::none()
                .fill(if is_active {
                    theme::CARD_BG
                } else {
                    egui::Color32::TRANSPARENT
                })
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        widgets::status_dot(ui, true);
                        ui.add_space(4.0);
                        if ui.selectable_label(is_active, profile.name).clicked() {
                            clicked = Some(profile);
                        }
                    });
                });
        }

        if let Some(profile) = clicked {
            if self.active_pid != Some(profile.product_id) {
                self.open(profile);
            }
        }
    }

    fn ui_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let color = if self.status_is_error {
                theme::DANGER
            } else {
                theme::MUTED
            };
            ui.label(egui::RichText::new(&self.status).small().color(color));
        });
    }

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        let Some(profile) = self.device.as_ref().map(|d| d.profile) else {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No device connected")
                        .size(16.0)
                        .color(theme::MUTED),
                );
                ui.label(
                    egui::RichText::new(
                        "Plug in a supported Razer device — it'll show up here automatically.",
                    )
                    .small()
                    .color(theme::MUTED),
                );
            });
            ui.add_space(30.0);
            self.ui_system_settings_card(ui);
            return;
        };

        ui.heading(profile.name);
        ui.add_space(14.0);

        if profile.dpi.is_some() || profile.poll_rate.is_some() {
            self.ui_performance_card(ui, profile);
        }
        if !profile.leds.is_empty() {
            self.ui_lighting_card(ui, profile);
        }
        // In "All zones" mode the Lighting card's Static/Off/Spectrum/Wave
        // buttons already drive the underglow (see `apply_to_zones_and_matrix`)
        // — showing this card too would let the user set it to a second,
        // conflicting color/effect right next to the sync controls.
        if profile.matrix.is_some() && !self.sync_zones {
            self.ui_matrix_card(ui);
        }
        if profile.identity.is_some() {
            self.ui_info_card(ui);
        }

        // System & background settings card
        self.ui_system_settings_card(ui);
    }

    fn ui_performance_card(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        widgets::card(ui, "Performance", |ui| {
            if let Some(dpi) = profile.dpi {
                ui.label(egui::RichText::new("DPI").color(theme::MUTED).small());

                // A slider spanning a device's full DPI range (often
                // 100-45000) is too coarse to land on an exact value —
                // type the number directly instead, with presets for the
                // common steps.
                let mut x = self.dpi_x as u32;
                let mut y = self.dpi_y as u32;

                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(
                        egui::DragValue::new(&mut x)
                            .range(dpi.min..=dpi.max)
                            .speed(50.0),
                    );
                    ui.add_space(10.0);
                    ui.label("Y:");
                    ui.add(
                        egui::DragValue::new(&mut y)
                            .range(dpi.min..=dpi.max)
                            .speed(50.0),
                    );
                });
                self.dpi_x = x as u16;
                self.dpi_y = y as u16;

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Presets:").small().color(theme::MUTED));
                    for preset in [400, 800, 1600, 3200, 6400] {
                        if preset >= dpi.min && preset <= dpi.max {
                            if ui.button(preset.to_string()).clicked() {
                                self.dpi_x = preset as u16;
                                self.dpi_y = preset as u16;
                                let result = self
                                    .device
                                    .as_ref()
                                    .unwrap()
                                    .set_dpi(self.dpi_x, self.dpi_y);
                                self.apply(result, &format!("DPI set to {preset}"));
                                self.save_current_device_config();
                            }
                        }
                    }
                });

                ui.add_space(8.0);
                if ui.button("Apply DPI").clicked() {
                    let (x, y) = (self.dpi_x, self.dpi_y);
                    let result = self.device.as_ref().unwrap().set_dpi(x, y);
                    self.apply(result, &format!("DPI set to {x}"));
                    self.save_current_device_config();
                }
                ui.add_space(10.0);
            }

            if let Some(poll) = profile.poll_rate {
                ui.label(
                    egui::RichText::new("Polling rate")
                        .color(theme::MUTED)
                        .small(),
                );
                ui.horizontal(|ui| {
                    for hz in poll.supported_hz {
                        let is_current = self.current_poll_hz == Some(*hz);
                        if ui
                            .selectable_label(is_current, format!("{hz} Hz"))
                            .clicked()
                        {
                            let result = self.device.as_ref().unwrap().set_poll_rate(*hz);
                            if result.is_ok() {
                                self.current_poll_hz = Some(*hz);
                                self.save_current_device_config();
                            }
                            self.apply(result, &format!("Polling rate set to {hz}Hz"));
                        }
                    }
                });
                if self.current_poll_hz.is_none() {
                    ui.label(
                        egui::RichText::new(
                            "Couldn't read the device's current polling rate — none shown as selected.",
                        )
                        .small()
                        .color(theme::MUTED),
                    );
                }
            }
        });
    }

    fn ui_lighting_card(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        widgets::card(ui, "Lighting", |ui| {
            let zones = profile.leds;
            let zone_idx = self.selected_zone_idx.min(zones.len() - 1);

            let has_matrix = profile.matrix.is_some();
            let matrix_has_effects = profile.matrix.and_then(|m| m.effect_led_id).is_some();
            let supports = |effect: razer_hid_abi::commands::LedEffect| {
                !profile.unsupported_led_effects.contains(&effect)
            };

            if zones.len() > 1 || has_matrix {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sync_zones, true, "All zones");
                    ui.selectable_value(&mut self.sync_zones, false, "Individually");
                });
                ui.add_space(6.0);

                if !self.sync_zones {
                    if zones.len() > 1 {
                        egui::ComboBox::from_label("Zone")
                            .selected_text(zones[zone_idx].name)
                            .show_ui(ui, |ui| {
                                for (i, z) in zones.iter().enumerate() {
                                    ui.selectable_value(&mut self.selected_zone_idx, i, z.name);
                                }
                            });
                    }
                }
            }

            let any_brightness = if self.sync_zones {
                zones.iter().any(|z| z.brightness_command_id.is_some())
            } else {
                zones[zone_idx].brightness_command_id.is_some()
            };

            widgets::effect_preview(ui, self.last_preview, self.led_color, ui.input(|i| i.time));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut self.led_color);
                widgets::color_swatch(ui, self.led_color, 20.0);
            });
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button("Static").clicked() {
                    let color = self.led_color;
                    self.last_preview = PreviewKind::Static;
                    self.current_effect_name = Some("static".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_static(zone, color),
                        has_matrix.then_some(move |dev: &RazerDevice| dev.fill_matrix(color)),
                        "LED set to static color",
                    );
                    self.save_current_device_config();
                }
                if supports(razer_hid_abi::commands::LedEffect::Breathing)
                    && ui.button("Breathing").clicked()
                {
                    let color = self.led_color;
                    self.last_preview = PreviewKind::Breathing;
                    self.current_effect_name = Some("breathing".to_string());
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_breathing(zone, color),
                        "LED breathing",
                    );
                    self.save_current_device_config();
                }
                if ui.button("Spectrum").clicked() {
                    self.last_preview = PreviewKind::Spectrum;
                    self.current_effect_name = Some("spectrum".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_spectrum(zone),
                        matrix_has_effects.then_some(|dev: &RazerDevice| dev.set_matrix_spectrum()),
                        "LED cycling spectrum",
                    );
                    self.save_current_device_config();
                }
                if ui.button("Wave").clicked() {
                    self.last_preview = PreviewKind::Wave;
                    self.current_effect_name = Some("wave".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_wave(zone, 1),
                        matrix_has_effects.then_some(|dev: &RazerDevice| dev.set_matrix_wave(1)),
                        "LED wave",
                    );
                    self.save_current_device_config();
                }
                if ui.button("Off").clicked() {
                    self.last_preview = PreviewKind::Off;
                    self.current_effect_name = Some("off".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_off(zone),
                        has_matrix.then_some(|dev: &RazerDevice| dev.fill_matrix([0, 0, 0])),
                        "LED off",
                    );
                    self.save_current_device_config();
                }
            });

            if !self.sync_zones && has_matrix {
                ui.label(
                    egui::RichText::new(
                        "Underglow only supports a solid color or off — Breathing/Spectrum/Wave don't reach it.",
                    )
                    .small()
                    .color(theme::MUTED),
                );
            } else if self.sync_zones && has_matrix {
                ui.label(
                    egui::RichText::new(
                        "Underglow follows Static/Off/Spectrum/Wave too, just not Breathing.",
                    )
                    .small()
                    .color(theme::MUTED),
                );
            }

            if any_brightness {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Brightness")
                        .color(theme::MUTED)
                        .small(),
                );
                let slider = ui.add(egui::Slider::new(&mut self.brightness, 0..=100).suffix("%"));
                if slider.drag_stopped() || slider.lost_focus() {
                    let b = self.brightness;
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_brightness(zone, b),
                        "Brightness updated",
                    );
                    self.save_current_device_config();
                }
            }
        });
    }

    fn ui_matrix_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, "Underglow", |ui| {
            let dims = self.device.as_ref().and_then(|d| d.matrix_dims());
            let Some((rows, cols)) = dims else {
                return;
            };
            ui.label(
                egui::RichText::new(format!("{rows} row x {cols} individually addressable LEDs"))
                    .small()
                    .color(theme::MUTED),
            );
            ui.add_space(6.0);

            widgets::effect_preview(ui, PreviewKind::Static, self.matrix_color, 0.0);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut self.matrix_color);
                widgets::color_swatch(ui, self.matrix_color, 20.0);
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Fill").clicked() {
                    let color = self.matrix_color;
                    let r = self.device.as_ref().unwrap().fill_matrix(color);
                    self.apply(r, "Underglow filled");
                    self.save_current_device_config();
                }
                if ui.button("Off").clicked() {
                    let r = self.device.as_ref().unwrap().fill_matrix([0, 0, 0]);
                    self.apply(r, "Underglow off");
                    self.save_current_device_config();
                }
            });
        });
    }

    fn ui_info_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, "Device info", |ui| {
            if ui.button("Refresh").clicked() {
                let dev = self.device.as_ref().unwrap();
                let fw = dev.get_firmware_version();
                let serial = dev.get_serial();
                match (fw, serial) {
                    (Ok(fw), Ok(serial)) => {
                        self.info_text = Some(format!("Firmware  {fw}\nSerial    {serial}"));
                        self.set_status("Info refreshed", false);
                    }
                    (Err(e), _) | (_, Err(e)) => self.set_status(format!("Error: {e}"), true),
                }
            }
            if let Some(text) = &self.info_text {
                ui.add_space(8.0);
                ui.monospace(text);
            }
        });
    }

    fn ui_system_settings_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, "System & Background", |ui| {
            let mut autostart_changed = false;
            let mut close_tray_changed = false;

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.start_with_windows, "Start with Windows")
                    .changed()
                {
                    autostart_changed = true;
                }
            });
            ui.label(
                egui::RichText::new("Automatically start RazerLine in the system tray on Windows startup.")
                    .small()
                    .color(theme::MUTED),
            );

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.close_to_tray, "Minimize to tray on close")
                    .changed()
                {
                    close_tray_changed = true;
                }
            });
            ui.label(
                egui::RichText::new("Keep running in the background when closing the window with the 'X' button.")
                    .small()
                    .color(theme::MUTED),
            );

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Save Settings as Default").clicked() {
                    self.save_current_device_config();
                    self.set_status("Configured settings saved", false);
                }
                if ui.button("Reload from Disk").clicked() {
                    self.config = AppConfig::load();
                    if let Some(profile) = self
                        .connected
                        .iter()
                        .find(|p| Some(p.product_id) == self.active_pid)
                        .copied()
                    {
                        self.open(profile);
                    }
                    self.set_status("Settings reloaded from disk", false);
                }
            });

            if autostart_changed {
                let enabled = self.config.start_with_windows;
                if let Err(e) = crate::autostart::set_autostart(enabled) {
                    self.set_status(format!("Autostart setting failed: {e}"), true);
                } else {
                    let msg = if enabled {
                        "Start with Windows enabled (runs in background on boot)"
                    } else {
                        "Start with Windows disabled"
                    };
                    self.set_status(msg, false);
                    let _ = self.config.save();
                }
            }

            if close_tray_changed {
                let _ = self.config.save();
            }

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!(
                    "Config file: {}",
                    AppConfig::file_path().display()
                ))
                .small()
                .color(theme::MUTED),
            );
        });
    }
}

pub fn run(start_in_background: bool) -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon.png is a valid PNG — generated from assets/razer-inspired-logo.svg");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 720.0])
            .with_icon(icon)
            .with_visible(!start_in_background),
        ..Default::default()
    };
    eframe::run_native(
        "RazerLine",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(GuiApp::new()))
        }),
    )
}
