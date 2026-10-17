//! Device-specific Vector Blueprint Visualizer (Top View & Bottom View) with authentic hardware contours,
//! button hotspots, live Chroma RGB lighting, and physical profile indicator LED.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Ui};

use crate::buttons::MouseButtonId;
use crate::theme;

static SVG_BASILISK_TOP: &[u8] = include_bytes!("../assets/basilisk_v3.svg");
static SVG_BASILISK_BOTTOM: &[u8] = include_bytes!("../assets/basilisk_v3_bottom.svg");
static SVG_GENERIC: &[u8] = include_bytes!("../assets/generic_mouse.svg");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChassis {
    BasiliskV3,       // Razer Basilisk V3 specific model
    GenericErgonomic, // Fallback when device specific model is missing
}

impl DeviceChassis {
    pub fn from_device_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("basilisk") {
            DeviceChassis::BasiliskV3
        } else {
            DeviceChassis::GenericErgonomic
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DeviceChassis::BasiliskV3 => "Razer Basilisk V3 (Specific Hardware Model)",
            DeviceChassis::GenericErgonomic => "Generic Razer Gaming Mouse (Fallback Model)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseViewMode {
    TopDown,
    BottomView,
}

pub struct Mouse3dView {
    pub view_mode: MouseViewMode,
    pub hovered_button: Option<MouseButtonId>,
}

impl Default for Mouse3dView {
    fn default() -> Self {
        Self {
            view_mode: MouseViewMode::TopDown,
            hovered_button: None,
        }
    }
}

impl Mouse3dView {
    /// Renders the device-specific 3D mouse visualizer and returns the button clicked (if any).
    pub fn show(
        &mut self,
        ui: &mut Ui,
        device_name: &str,
        selected_button: Option<MouseButtonId>,
        led_color: [u8; 3],
        time: f64,
    ) -> Option<MouseButtonId> {
        let chassis = DeviceChassis::from_device_name(device_name);
        let available_size = egui::vec2(ui.available_width().max(560.0), 450.0);
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());
        let rect = response.rect;
        let center = rect.center();

        // High-tech dark background grid
        painter.rect_filled(rect, 14.0_f32, Color32::from_rgb(17, 18, 22));
        painter.rect_stroke(
            rect,
            14.0_f32,
            Stroke::new(1.0_f32, Color32::from_white_alpha(18)),
        );

        let rgb = Color32::from_rgb(led_color[0], led_color[1], led_color[2]);

        // Device model badge at top-left of canvas
        painter.text(
            rect.left_top() + egui::vec2(18.0, 18.0),
            egui::Align2::LEFT_TOP,
            format!(
                "HARDWARE SCHEMATIC: {} ({})",
                device_name.to_uppercase(),
                chassis.display_name()
            ),
            egui::FontId::proportional(11.0),
            theme::ACCENT,
        );

        let hovered = None;

        let clicked_btn = match chassis {
            DeviceChassis::BasiliskV3 => match self.view_mode {
                MouseViewMode::TopDown => {
                    self.render_basilisk_top_view(ui, &painter, center, selected_button, rgb, time)
                }
                MouseViewMode::BottomView => self.render_basilisk_bottom_view(
                    ui,
                    &painter,
                    center,
                    selected_button,
                    rgb,
                    time,
                ),
            },
            DeviceChassis::GenericErgonomic => {
                self.render_generic_view(ui, &painter, center, selected_button, rgb, time)
            }
        };

        self.hovered_button = hovered;

        // View Mode Switcher Pills (Top View / Bottom View)
        let modes = [
            (MouseViewMode::TopDown, "Top View"),
            (MouseViewMode::BottomView, "Bottom View"),
        ];
        let btn_w = 100.0;
        let btn_h = 24.0;
        let gap = 8.0;
        let total_w = 2.0 * btn_w + gap;
        let start_pos = rect.right_top() + egui::vec2(-total_w - 18.0, 14.0);

        for (i, (mode, label)) in modes.iter().enumerate() {
            let btn_rect = Rect::from_min_size(
                start_pos + egui::vec2(i as f32 * (btn_w + gap), 0.0),
                egui::vec2(btn_w, btn_h),
            );
            let is_sel = self.view_mode == *mode;
            let is_hovered = ui.rect_contains_pointer(btn_rect);

            if is_hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
                self.view_mode = *mode;
            }

            let bg = if is_sel {
                Color32::from_rgb(26, 56, 38)
            } else if is_hovered {
                Color32::from_rgb(32, 35, 42)
            } else {
                Color32::from_rgb(20, 22, 26)
            };

            let stroke = if is_sel {
                Stroke::new(1.0_f32, theme::ACCENT)
            } else {
                Stroke::new(1.0_f32, Color32::from_white_alpha(30))
            };

            painter.rect_filled(btn_rect, 5.0_f32, bg);
            painter.rect_stroke(btn_rect, 5.0_f32, stroke);

            let txt_color = if is_sel {
                theme::ACCENT
            } else if is_hovered {
                Color32::WHITE
            } else {
                Color32::from_gray(175)
            };

            painter.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(11.0),
                txt_color,
            );
        }

        // Instructions footer
        let footer_text = match self.view_mode {
            MouseViewMode::TopDown => "Top Blueprint — click any numbered pin to reassign",
            MouseViewMode::BottomView => {
                "Bottom Blueprint — Profile Switch button & Focus+ optical sensor"
            }
        };
        painter.text(
            rect.left_bottom() + egui::vec2(18.0, -16.0),
            egui::Align2::LEFT_BOTTOM,
            footer_text,
            egui::FontId::proportional(11.0),
            theme::MUTED,
        );

        clicked_btn
    }

    // ═════════════════════════════════════════════════════════════════════
    // RAZER BASILISK V3: TOP VIEW (SVG Vector Model with Live Chroma)
    // ═════════════════════════════════════════════════════════════════════
    fn render_basilisk_top_view(
        &self,
        ui: &mut Ui,
        painter: &egui::Painter,
        center: Pos2,
        selected: Option<MouseButtonId>,
        led_color: Color32,
        _time: f64,
    ) -> Option<MouseButtonId> {
        let mut clicked = None;

        // 1. Dynamic 11-Zone Chroma Underglow desk halo
        painter.circle_filled(
            center + egui::vec2(0.0, 30.0),
            160.0_f32,
            Color32::from_rgba_unmultiplied(led_color.r(), led_color.g(), led_color.b(), 32),
        );

        // 2. Render Authentic Basilisk V3 SVG Vector Model
        let img_size = egui::vec2(280.0, 392.0);
        let img_rect = Rect::from_center_size(center, img_size);
        let img = egui::Image::from_bytes("bytes://basilisk_v3.svg", SVG_BASILISK_TOP);
        img.paint_at(ui, img_rect);

        // 3. Live Chroma Illumination on Wheel & Tri-Snake Logo
        let wheel_center = center + egui::vec2(0.0, -96.0);
        painter.circle_filled(
            wheel_center,
            12.0_f32,
            Color32::from_rgba_unmultiplied(led_color.r(), led_color.g(), led_color.b(), 45),
        );

        let logo_center = center + egui::vec2(0.0, 68.0);
        painter.circle_filled(
            logo_center,
            15.0_f32,
            Color32::from_rgba_unmultiplied(led_color.r(), led_color.g(), led_color.b(), 55),
        );

        // 4. Interactive Callout Pins with Leader Lines (Official Razer Layout)
        let pins = [
            (
                MouseButtonId::MiddleClick,
                wheel_center,
                center + egui::vec2(-198.0, -90.0),
                "Scroll Click",
            ),
            (
                MouseButtonId::TiltLeft,
                wheel_center + egui::vec2(-14.0, 0.0),
                center + egui::vec2(-198.0, -48.0),
                "Repeat Scroll Left",
            ),
            (
                MouseButtonId::MultiFunctionClutch,
                center + egui::vec2(-82.0, -15.0),
                center + egui::vec2(-198.0, -6.0),
                "Sensitivity Clutch",
            ),
            (
                MouseButtonId::SideForward,
                center + egui::vec2(-56.0, -32.0),
                center + egui::vec2(-198.0, 36.0),
                "Mouse Button 5",
            ),
            (
                MouseButtonId::SideBack,
                center + egui::vec2(-54.0, 2.0),
                center + egui::vec2(-198.0, 78.0),
                "Mouse Button 4",
            ),
            (
                MouseButtonId::ProfileSwitch,
                center + egui::vec2(-14.0, 140.0),
                center + egui::vec2(-198.0, 120.0),
                "Cycle Up Profile",
            ),
            (
                MouseButtonId::ScrollUp,
                wheel_center + egui::vec2(0.0, -12.0),
                center + egui::vec2(198.0, -90.0),
                "Scroll Up",
            ),
            (
                MouseButtonId::TiltRight,
                wheel_center + egui::vec2(14.0, 0.0),
                center + egui::vec2(198.0, -48.0),
                "Repeat Scroll Right",
            ),
            (
                MouseButtonId::ScrollDown,
                wheel_center + egui::vec2(0.0, 12.0),
                center + egui::vec2(198.0, -6.0),
                "Scroll Down",
            ),
            (
                MouseButtonId::ScrollMode,
                center + egui::vec2(0.0, -64.0),
                center + egui::vec2(198.0, 36.0),
                "Scroll Mode Toggle",
            ),
            (
                MouseButtonId::DpiSwitch,
                center + egui::vec2(0.0, -44.0),
                center + egui::vec2(198.0, 78.0),
                "Cycle Up Sensitivity Stages",
            ),
        ];

        for (btn_id, target_pos, badge_pos, text) in pins {
            let is_sel = selected == Some(btn_id);
            if self.draw_callout_pin(ui, painter, target_pos, badge_pos, text, is_sel) {
                clicked = Some(btn_id);
            }
        }

        clicked
    }

    // ═════════════════════════════════════════════════════════════════════
    // RAZER BASILISK V3: BOTTOM VIEW (Profile Switch Button & Glides)
    // ═════════════════════════════════════════════════════════════════════
    fn render_basilisk_bottom_view(
        &self,
        ui: &mut Ui,
        painter: &egui::Painter,
        center: Pos2,
        selected: Option<MouseButtonId>,
        led_color: Color32,
        _time: f64,
    ) -> Option<MouseButtonId> {
        let mut clicked = None;

        // 1. Dynamic Desk Chroma Underglow halo
        painter.circle_filled(
            center + egui::vec2(0.0, 20.0),
            160.0_f32,
            Color32::from_rgba_unmultiplied(led_color.r(), led_color.g(), led_color.b(), 35),
        );

        // 2. Render Authentic Basilisk V3 Bottom SVG Vector Model
        let img_size = egui::vec2(280.0, 392.0);
        let img_rect = Rect::from_center_size(center, img_size);
        let img = egui::Image::from_bytes("bytes://basilisk_v3_bottom.svg", SVG_BASILISK_BOTTOM);
        img.paint_at(ui, img_rect);

        // 3. Dedicated Hardware Profile Status LED (Active Profile Indicator)
        let profile_btn_pos = center + egui::vec2(-8.0, 92.0);
        let profile_led_pos = center + egui::vec2(10.0, 92.0);
        painter.circle_filled(profile_led_pos, 4.5_f32, led_color);
        painter.circle_stroke(
            profile_led_pos,
            7.5_f32,
            Stroke::new(
                1.5_f32,
                Color32::from_rgba_unmultiplied(led_color.r(), led_color.g(), led_color.b(), 110),
            ),
        );
        painter.circle_filled(profile_led_pos, 1.8_f32, Color32::WHITE);

        // 4. Optical Sensor Center Target
        let sensor_pos = center + egui::vec2(0.0, 18.0);
        painter.circle_filled(sensor_pos, 3.0_f32, Color32::from_rgb(255, 30, 30));

        // 5. Interactive Callout Pins for Bottom Features
        let is_prof_sel = selected == Some(MouseButtonId::ProfileSwitch);
        if self.draw_callout_pin(
            ui,
            painter,
            profile_btn_pos,
            center + egui::vec2(-198.0, 92.0),
            "Cycle Up Profile",
            is_prof_sel,
        ) {
            clicked = Some(MouseButtonId::ProfileSwitch);
        }

        // Secondary Callout: Optical Sensor & PTFE Glides info
        self.draw_info_badge(
            painter,
            sensor_pos,
            center + egui::vec2(185.0, 18.0),
            "Focus+ 26K Sensor",
        );

        self.draw_info_badge(
            painter,
            center + egui::vec2(0.0, 138.0),
            center + egui::vec2(185.0, 92.0),
            "100% Pure PTFE Glides",
        );

        clicked
    }

    // ═════════════════════════════════════════════════════════════════════
    // GENERIC RAZER GAMING MOUSE (Fallback Model when device specific missing)
    // ═════════════════════════════════════════════════════════════════════
    fn render_generic_view(
        &self,
        ui: &mut Ui,
        painter: &egui::Painter,
        center: Pos2,
        selected: Option<MouseButtonId>,
        led_color: Color32,
        _time: f64,
    ) -> Option<MouseButtonId> {
        let mut clicked = None;

        // Ambient glow
        painter.circle_filled(
            center + egui::vec2(0.0, 30.0),
            150.0_f32,
            Color32::from_rgba_unmultiplied(led_color.r(), led_color.g(), led_color.b(), 28),
        );

        // Render generic SVG
        let img_size = egui::vec2(280.0, 392.0);
        let img_rect = Rect::from_center_size(center, img_size);
        let img = egui::Image::from_bytes("bytes://generic_mouse.svg", SVG_GENERIC);
        img.paint_at(ui, img_rect);

        // Standard Generic Pins
        let pins = [
            (
                MouseButtonId::MiddleClick,
                center + egui::vec2(0.0, -95.0),
                center + egui::vec2(-185.0, -50.0),
                "3: Scroll Click",
            ),
            (
                MouseButtonId::SideForward,
                center + egui::vec2(-55.0, -20.0),
                center + egui::vec2(-185.0, 10.0),
                "5: Forward (M5)",
            ),
            (
                MouseButtonId::SideBack,
                center + egui::vec2(-52.0, 15.0),
                center + egui::vec2(-185.0, 70.0),
                "4: Back (M4)",
            ),
            (
                MouseButtonId::DpiSwitch,
                center + egui::vec2(0.0, -45.0),
                center + egui::vec2(185.0, -20.0),
                "DPI Cycle Button",
            ),
        ];

        for (btn_id, target_pos, badge_pos, text) in pins {
            let is_sel = selected == Some(btn_id);
            if self.draw_callout_pin(ui, painter, target_pos, badge_pos, text, is_sel) {
                clicked = Some(btn_id);
            }
        }

        clicked
    }

    /// Draws an interactive callout pin badge connected to the physical mouse button with an orthogonal knee line.
    fn draw_callout_pin(
        &self,
        ui: &mut Ui,
        painter: &egui::Painter,
        target_pos: Pos2,
        badge_pos: Pos2,
        text: &str,
        is_selected: bool,
    ) -> bool {
        let badge_size = egui::vec2(156.0, 24.0);
        let badge_rect = Rect::from_center_size(badge_pos, badge_size);

        let is_hovered = ui.rect_contains_pointer(badge_rect);

        let stroke_color = if is_selected {
            theme::ACCENT
        } else if is_hovered {
            Color32::from_rgb(140, 220, 160)
        } else {
            Color32::from_rgb(55, 60, 72)
        };

        let bg_color = if is_selected {
            Color32::from_rgb(26, 56, 38)
        } else if is_hovered {
            Color32::from_rgb(34, 38, 46)
        } else {
            Color32::from_rgb(20, 22, 26)
        };

        // Leader line with orthogonal knee joints connecting cleanly to the nearest badge edge
        let is_left = badge_pos.x < target_pos.x;
        let connect_pos = Pos2::new(
            if is_left {
                badge_rect.max.x
            } else {
                badge_rect.min.x
            },
            badge_rect.center().y,
        );

        let knee_x = if is_left {
            connect_pos.x + 14.0
        } else {
            connect_pos.x - 14.0
        };

        let knee_1 = Pos2::new(knee_x, target_pos.y);
        let knee_2 = Pos2::new(knee_x, connect_pos.y);

        painter.line_segment([target_pos, knee_1], Stroke::new(1.0_f32, stroke_color));
        painter.line_segment([knee_1, knee_2], Stroke::new(1.0_f32, stroke_color));
        painter.line_segment([knee_2, connect_pos], Stroke::new(1.0_f32, stroke_color));

        // Hotspot target dot on mouse body
        if is_selected {
            painter.circle_filled(target_pos, 4.5_f32, theme::ACCENT);
            painter.circle_stroke(
                target_pos,
                7.5_f32,
                Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(64, 209, 128, 90)),
            );
        } else {
            painter.circle_filled(target_pos, 3.5_f32, stroke_color);
            painter.circle_stroke(
                target_pos,
                5.5_f32,
                Stroke::new(1.0_f32, Color32::from_white_alpha(35)),
            );
        }

        // Badge pill box
        painter.rect_filled(badge_rect, 6.0_f32, bg_color);
        painter.rect_stroke(
            badge_rect,
            6.0_f32,
            Stroke::new(if is_selected { 1.5_f32 } else { 1.0_f32 }, stroke_color),
        );

        let text_color = if is_selected {
            theme::ACCENT
        } else if is_hovered {
            Color32::WHITE
        } else {
            Color32::from_gray(215)
        };

        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(10.5),
            text_color,
        );

        // Click detection
        ui.input(|i| i.pointer.primary_clicked() && is_hovered)
    }

    /// Informational badge on the blueprint with a leader line.
    fn draw_info_badge(
        &self,
        painter: &egui::Painter,
        target_pos: Pos2,
        badge_pos: Pos2,
        text: &str,
    ) {
        let badge_size = egui::vec2(136.0, 22.0);
        let badge_rect = Rect::from_center_size(badge_pos, badge_size);
        let stroke_color = Color32::from_rgb(50, 54, 64);
        let bg_color = Color32::from_rgb(18, 19, 23);

        let is_left = badge_pos.x < target_pos.x;
        let connect_pos = Pos2::new(
            if is_left {
                badge_rect.max.x
            } else {
                badge_rect.min.x
            },
            badge_rect.center().y,
        );

        let knee_x = if is_left {
            connect_pos.x + 14.0
        } else {
            connect_pos.x - 14.0
        };

        let knee_1 = Pos2::new(knee_x, target_pos.y);
        let knee_2 = Pos2::new(knee_x, connect_pos.y);

        painter.line_segment([target_pos, knee_1], Stroke::new(1.0_f32, stroke_color));
        painter.line_segment([knee_1, knee_2], Stroke::new(1.0_f32, stroke_color));
        painter.line_segment([knee_2, connect_pos], Stroke::new(1.0_f32, stroke_color));

        painter.circle_filled(target_pos, 3.0_f32, stroke_color);

        painter.rect_filled(badge_rect, 5.0_f32, bg_color);
        painter.rect_stroke(badge_rect, 5.0_f32, Stroke::new(1.0_f32, stroke_color));

        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(10.5),
            Color32::from_gray(160),
        );
    }
}
