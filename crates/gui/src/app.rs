use eframe::egui;
use razer_hid_core::devices::{DeviceProfile, LedZoneProfile, KNOWN_DEVICES};
use razer_hid_core::RazerDevice;

use crate::theme;
use crate::widgets::{self, PreviewKind};

pub struct GuiApp {
    device: Option<RazerDevice>,
    connected: Vec<&'static DeviceProfile>,
    active_pid: Option<u16>,
    status: String,
    status_is_error: bool,

    dpi_x: u16,
    dpi_y: u16,

    selected_zone_idx: usize,
    led_color: [u8; 3],
    brightness: u8,
    last_preview: PreviewKind,

    info_text: Option<String>,
}

impl Default for GuiApp {
    fn default() -> Self {
        let mut app = Self {
            device: None,
            connected: Vec::new(),
            active_pid: None,
            status: "Scanning for devices...".to_string(),
            status_is_error: false,
            dpi_x: 800,
            dpi_y: 800,
            selected_zone_idx: 0,
            led_color: [64, 209, 128],
            brightness: 100,
            last_preview: PreviewKind::Off,
            info_text: None,
        };
        app.rescan();
        app
    }
}

impl GuiApp {
    fn rescan(&mut self) {
        match RazerDevice::list_connected() {
            Ok(profiles) if !profiles.is_empty() => {
                self.connected = profiles;
                let first = self.connected[0];
                self.open(first);
            }
            Ok(_) => {
                self.connected.clear();
                self.device = None;
                self.active_pid = None;
                self.set_status("No supported Razer device found", false);
            }
            Err(e) => {
                self.device = None;
                self.active_pid = None;
                self.set_status(format!("Scan error: {e}"), true);
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
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        ui.label(egui::RichText::new("DEVICES").small().color(theme::MUTED));
        ui.add_space(8.0);

        let mut clicked: Option<&'static DeviceProfile> = None;
        for profile in KNOWN_DEVICES {
            let is_connected = self
                .connected
                .iter()
                .any(|c| c.product_id == profile.product_id);
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
                        widgets::status_dot(ui, is_connected);
                        ui.add_space(4.0);
                        let resp = ui.add_enabled(
                            is_connected,
                            egui::SelectableLabel::new(is_active, profile.name),
                        );
                        if resp.clicked() {
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

        ui.add_space(16.0);
        if ui.button("Rescan").clicked() {
            self.rescan();
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
        let Some(active_pid) = self.active_pid else {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No device connected")
                        .size(16.0)
                        .color(theme::MUTED),
                );
                ui.label(
                    egui::RichText::new("Plug in a supported Razer device and hit Rescan.")
                        .small()
                        .color(theme::MUTED),
                );
            });
            return;
        };
        let Some(profile) = KNOWN_DEVICES.iter().find(|p| p.product_id == active_pid) else {
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
                egui::ComboBox::from_label("Zone")
                    .selected_text(zones[zone_idx].name)
                    .show_ui(ui, |ui| {
                        for (i, z) in zones.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_zone_idx, i, z.name);
                        }
                    });
            }
            let zone: LedZoneProfile = zones[zone_idx];

            ui.add_space(6.0);
            widgets::effect_preview(ui, self.last_preview, self.led_color, ui.input(|i| i.time));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut self.led_color);
                widgets::color_swatch(ui, self.led_color, 20.0);
            });
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button("Static").clicked() {
                    let r = self
                        .device
                        .as_ref()
                        .unwrap()
                        .set_led_static(&zone, self.led_color);
                    self.last_preview = PreviewKind::Static;
                    self.apply(r, "LED set to static color");
                }
                if ui.button("Breathing").clicked() {
                    let r = self
                        .device
                        .as_ref()
                        .unwrap()
                        .set_led_breathing(&zone, self.led_color);
                    self.last_preview = PreviewKind::Breathing;
                    self.apply(r, "LED breathing");
                }
                if ui.button("Spectrum").clicked() {
                    let r = self.device.as_ref().unwrap().set_led_spectrum(&zone);
                    self.last_preview = PreviewKind::Spectrum;
                    self.apply(r, "LED cycling spectrum");
                }
                if ui.button("Wave").clicked() {
                    let r = self.device.as_ref().unwrap().set_led_wave(&zone, 1);
                    self.last_preview = PreviewKind::Wave;
                    self.apply(r, "LED wave");
                }
                if ui.button("Off").clicked() {
                    let r = self.device.as_ref().unwrap().set_led_off(&zone);
                    self.last_preview = PreviewKind::Off;
                    self.apply(r, "LED off");
                }
            });

            if zone.brightness_command_id.is_some() {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Brightness")
                        .color(theme::MUTED)
                        .small(),
                );
                let slider = ui.add(egui::Slider::new(&mut self.brightness, 0..=100).suffix("%"));
                if slider.drag_stopped() || slider.lost_focus() {
                    let b = self.brightness;
                    let r = self.device.as_ref().unwrap().set_led_brightness(&zone, b);
                    self.apply(r, "Brightness updated");
                }
            }
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([620.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Razer HID Control",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(GuiApp::default()))
        }),
    )
}
