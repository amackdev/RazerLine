use std::time::{Duration, Instant};

use eframe::egui;
use razer_hid_core::devices::{DeviceProfile, LedZoneProfile};
use razer_hid_core::RazerDevice;

use crate::theme;
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

    selected_zone_idx: usize,
    /// When true, lighting actions apply to every zone at once (Razer
    /// Synapse's "All" view); when false, only to the selected zone.
    sync_zones: bool,
    led_color: [u8; 3],
    brightness: u8,
    last_preview: PreviewKind,

    matrix_color: [u8; 3],

    info_text: Option<String>,
}

impl Default for GuiApp {
    fn default() -> Self {
        let mut app = Self {
            device: None,
            connected: Vec::new(),
            active_pid: None,
            last_scan: Instant::now(),
            status: "Scanning for devices...".to_string(),
            status_is_error: false,
            dpi_x: 800,
            dpi_y: 800,
            selected_zone_idx: 0,
            sync_zones: true,
            led_color: [64, 209, 128],
            brightness: 100,
            last_preview: PreviewKind::Off,
            matrix_color: [64, 209, 128],
            info_text: None,
        };
        app.rescan();
        app
    }
}

impl GuiApp {
    /// Re-enumerates connected devices and reconciles state against what
    /// was already open — called on a timer from `update()`, not just on
    /// startup, so plugging/unplugging a device is picked up live without
    /// the user doing anything. Does **not** reopen the active device's
    /// handle on every tick: only touches `self.device` when the active
    /// device actually disappeared or something new needs auto-selecting.
    fn rescan(&mut self) {
        let found = match RazerDevice::list_connected() {
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

    fn open(&mut self, profile: &'static DeviceProfile) {
        match RazerDevice::open(profile) {
            Ok(dev) => {
                // Show the device's actual current DPI rather than a
                // hardcoded guess — if the query isn't supported or fails,
                // just keep whatever was there before.
                if let Ok(current) = dev.get_dpi() {
                    self.dpi_x = current.x;
                    self.dpi_y = current.y;
                }
                self.active_pid = Some(profile.product_id);
                self.device = Some(dev);
                self.info_text = None;
                self.selected_zone_idx = 0;
                self.set_status(format!("Connected to {}", profile.name), false);
            }
            Err(e) => {
                self.device = None;
                self.active_pid = None;
                self.set_status(format!("Couldn't open {}: {e}", profile.name), true);
            }
        }
    }

    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = msg.into();
        self.status_is_error = is_error;
    }

    fn apply<T>(&mut self, result: Result<T, razer_hid_core::DeviceError>, ok_msg: &str) {
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
        mut f: impl FnMut(&RazerDevice, &LedZoneProfile) -> Result<(), razer_hid_core::DeviceError>,
        ok_msg: &str,
    ) {
        let targets: &[LedZoneProfile] = if self.sync_zones {
            zones
        } else {
            &zones[selected_idx..=selected_idx]
        };
        let dev = self.device.as_ref().unwrap();
        let mut failure = None;
        for zone in targets {
            if let Err(e) = f(dev, zone) {
                failure.get_or_insert_with(|| format!("{} failed: {e}", zone.name));
            }
        }
        match failure {
            None => self.set_status(ok_msg, false),
            Some(msg) => self.set_status(msg, true),
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Live USB detection: egui only calls update() on input by
        // default, so request a wakeup even when idle — this is what
        // makes plugging/unplugging a device show up without the user
        // touching anything.
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
        if profile.matrix.is_some() {
            self.ui_matrix_card(ui);
        }
        if profile.identity.is_some() {
            self.ui_info_card(ui);
        }
    }

    fn ui_performance_card(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        widgets::card(ui, "Performance", |ui| {
            if let Some(dpi) = profile.dpi {
                ui.label(egui::RichText::new("DPI").color(theme::MUTED).small());

                // A slider spanning a device's full DPI range (often
                // 100-45000) is too coarse to land on an exact value —
                // type the number directly instead, with presets for the
                // common values. X and Y are kept equal and not exposed
                // separately; nothing in this UI needs independent axes.
                ui.add(
                    egui::DragValue::new(&mut self.dpi_x)
                        .range(dpi.min..=dpi.max)
                        .speed(5)
                        .suffix(" DPI"),
                );
                self.dpi_y = self.dpi_x;

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    const PRESETS: [u16; 9] =
                        [400, 800, 1200, 1600, 2400, 3200, 6400, 12800, 16000];
                    for preset in PRESETS {
                        if preset < dpi.min || preset > dpi.max {
                            continue;
                        }
                        let is_current = self.dpi_x == preset;
                        if ui
                            .selectable_label(is_current, preset.to_string())
                            .clicked()
                        {
                            self.dpi_x = preset;
                            self.dpi_y = preset;
                        }
                    }
                });

                ui.add_space(8.0);
                if ui.button("Apply DPI").clicked() {
                    let (x, y) = (self.dpi_x, self.dpi_y);
                    let result = self.device.as_ref().unwrap().set_dpi(x, y);
                    self.apply(result, &format!("DPI set to {x}"));
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
                        if ui.button(format!("{hz} Hz")).clicked() {
                            let result = self.device.as_ref().unwrap().set_poll_rate(*hz);
                            self.apply(result, &format!("Polling rate set to {hz}Hz"));
                        }
                    }
                });
            }
        });
    }

    fn ui_lighting_card(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        widgets::card(ui, "Lighting", |ui| {
            let zones = profile.leds;
            let zone_idx = self.selected_zone_idx.min(zones.len() - 1);

            if zones.len() > 1 {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sync_zones, true, "All zones");
                    ui.selectable_value(&mut self.sync_zones, false, "Individually");
                });
                ui.add_space(6.0);

                if !self.sync_zones {
                    egui::ComboBox::from_label("Zone")
                        .selected_text(zones[zone_idx].name)
                        .show_ui(ui, |ui| {
                            for (i, z) in zones.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_zone_idx, i, z.name);
                            }
                        });
                } else {
                    let names: Vec<_> = zones.iter().map(|z| z.name).collect();
                    ui.label(
                        egui::RichText::new(format!("Applies to: {}", names.join(", ")))
                            .small()
                            .color(theme::MUTED),
                    );
                }
                ui.add_space(6.0);
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
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_static(zone, color),
                        "LED set to static color",
                    );
                }
                if ui.button("Breathing").clicked() {
                    let color = self.led_color;
                    self.last_preview = PreviewKind::Breathing;
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_breathing(zone, color),
                        "LED breathing",
                    );
                }
                if ui.button("Spectrum").clicked() {
                    self.last_preview = PreviewKind::Spectrum;
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_spectrum(zone),
                        "LED cycling spectrum",
                    );
                }
                if ui.button("Wave").clicked() {
                    self.last_preview = PreviewKind::Wave;
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_wave(zone, 1),
                        "LED wave",
                    );
                }
                if ui.button("Off").clicked() {
                    self.last_preview = PreviewKind::Off;
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_off(zone),
                        "LED off",
                    );
                }
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Spectrum and Wave always cycle through colors — the picker above only applies to Static and Breathing.")
                    .small()
                    .color(theme::MUTED),
            );

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
                }
                if ui.button("Off").clicked() {
                    let r = self.device.as_ref().unwrap().fill_matrix([0, 0, 0]);
                    self.apply(r, "Underglow off");
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
}

pub fn run() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon.png is a valid PNG — generated from assets/razer-inspired-logo.svg");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 680.0])
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "RazerLine",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(GuiApp::default()))
        }),
    )
}
