//! egui-based control panel. Generic over whatever `DeviceProfile` is
//! connected — tabs/controls only appear for capabilities the connected
//! device's profile actually declares.

use eframe::egui;
use razer_hid_core::devices::DeviceProfile;
use razer_hid_core::RazerDevice;

struct GuiApp {
    device: Option<RazerDevice>,
    connected: Vec<&'static DeviceProfile>,
    selected_device_idx: usize,
    status: String,

    dpi_x: u16,
    dpi_y: u16,
    link_xy: bool,

    selected_zone_idx: usize,
    led_color: [u8; 3],
    brightness: u8,

    info_text: String,
}

impl Default for GuiApp {
    fn default() -> Self {
        let mut app = Self {
            device: None,
            connected: Vec::new(),
            selected_device_idx: 0,
            status: "Scanning...".to_string(),
            dpi_x: 800,
            dpi_y: 800,
            link_xy: true,
            selected_zone_idx: 0,
            led_color: [0, 255, 0],
            brightness: 100,
            info_text: String::new(),
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
                self.selected_device_idx = 0;
                self.open_selected();
            }
            Ok(_) => {
                self.connected.clear();
                self.device = None;
                self.status = "No supported Razer device found".to_string();
            }
            Err(e) => {
                self.device = None;
                self.status = format!("Scan error: {e}");
            }
        }
    }

    fn open_selected(&mut self) {
        if let Some(profile) = self.connected.get(self.selected_device_idx) {
            match RazerDevice::open(profile) {
                Ok(dev) => {
                    self.status = format!("Connected: {}", profile.name);
                    self.device = Some(dev);
                }
                Err(e) => {
                    self.device = None;
                    self.status = format!("Error opening device: {e}");
                }
            }
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Razer HID Control (unofficial)");
            ui.horizontal(|ui| {
                ui.label(&self.status);
                if ui.button("Rescan").clicked() {
                    self.rescan();
                }
            });

            if self.connected.len() > 1 {
                let mut changed_idx = self.selected_device_idx;
                egui::ComboBox::from_label("Device")
                    .selected_text(self.connected[self.selected_device_idx].name)
                    .show_ui(ui, |ui| {
                        for (i, p) in self.connected.iter().enumerate() {
                            ui.selectable_value(&mut changed_idx, i, p.name);
                        }
                    });
                if changed_idx != self.selected_device_idx {
                    self.selected_device_idx = changed_idx;
                    self.open_selected();
                }
            }

            ui.separator();

            let Some(dev) = &self.device else {
                return;
            };
            let profile = dev.profile;

            if let Some(dpi) = profile.dpi {
                ui.heading("DPI");
                ui.add(egui::Slider::new(&mut self.dpi_x, dpi.min..=dpi.max).text("X"));
                ui.checkbox(&mut self.link_xy, "Lock Y to X");
                if self.link_xy {
                    self.dpi_y = self.dpi_x;
                }
                ui.add_enabled(
                    !self.link_xy,
                    egui::Slider::new(&mut self.dpi_y, dpi.min..=dpi.max).text("Y"),
                );
                if ui.button("Apply DPI").clicked() {
                    let result = dev.set_dpi(self.dpi_x, self.dpi_y);
                    self.status = match result {
                        Ok(()) => format!("DPI set to {}x{}", self.dpi_x, self.dpi_y),
                        Err(e) => format!("Error: {e}"),
                    };
                }
                ui.separator();
            }

            if let Some(poll) = profile.poll_rate {
                ui.heading("Polling Rate");
                ui.horizontal(|ui| {
                    for hz in poll.supported_hz {
                        if ui.button(format!("{hz} Hz")).clicked() {
                            let result = dev.set_poll_rate(*hz);
                            self.status = match result {
                                Ok(()) => format!("Polling rate set to {hz}Hz"),
                                Err(e) => format!("Error: {e}"),
                            };
                        }
                    }
                });
                ui.separator();
            }

            let zones = dev.led_zones();
            if !zones.is_empty() {
                ui.heading("Lighting");
                let mut zone_idx = self.selected_zone_idx.min(zones.len() - 1);
                egui::ComboBox::from_label("Zone")
                    .selected_text(zones[zone_idx].name)
                    .show_ui(ui, |ui| {
                        for (i, z) in zones.iter().enumerate() {
                            ui.selectable_value(&mut zone_idx, i, z.name);
                        }
                    });
                self.selected_zone_idx = zone_idx;
                let zone = zones[zone_idx];

                ui.color_edit_button_srgb(&mut self.led_color);

                ui.horizontal(|ui| {
                    if ui.button("Static").clicked() {
                        let r = dev.set_led_static(&zone, self.led_color);
                        self.status = result_msg(r, "LED set");
                    }
                    if ui.button("Breathing").clicked() {
                        let r = dev.set_led_breathing(&zone, self.led_color);
                        self.status = result_msg(r, "LED breathing");
                    }
                    if ui.button("Spectrum").clicked() {
                        let r = dev.set_led_spectrum(&zone);
                        self.status = result_msg(r, "LED cycling spectrum");
                    }
                    if ui.button("Wave").clicked() {
                        let r = dev.set_led_wave(&zone, 1);
                        self.status = result_msg(r, "LED wave");
                    }
                    if ui.button("Off").clicked() {
                        let r = dev.set_led_off(&zone);
                        self.status = result_msg(r, "LED off");
                    }
                });

                if zone.brightness_command_id.is_some() {
                    let slider =
                        ui.add(egui::Slider::new(&mut self.brightness, 0..=100).text("Brightness"));
                    if slider.drag_stopped() || slider.lost_focus() {
                        let r = dev.set_led_brightness(&zone, self.brightness);
                        self.status = result_msg(r, "Brightness updated");
                    }
                }
                ui.separator();
            }

            if profile.identity.is_some() {
                ui.heading("Info");
                if ui.button("Refresh info").clicked() {
                    let fw = dev.get_firmware_version();
                    let serial = dev.get_serial();
                    self.info_text = match (fw, serial) {
                        (Ok(fw), Ok(serial)) => format!("Firmware: {fw}\nSerial: {serial}"),
                        (Err(e), _) | (_, Err(e)) => format!("Error: {e}"),
                    };
                }
                ui.label(&self.info_text);
            }
        });
    }
}

fn result_msg(result: Result<(), razer_hid_core::DeviceError>, ok_msg: &str) -> String {
    match result {
        Ok(()) => ok_msg.to_string(),
        Err(e) => format!("Error: {e}"),
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Razer HID Control (unofficial)",
        options,
        Box::new(|_cc| Ok(Box::new(GuiApp::default()))),
    )
}
