//! Visual theme: a dark control-panel look with a single accent color.
//! Kept in one place so the rest of the UI code never hardcodes a color.

use eframe::egui::{self, Color32};

pub const ACCENT: Color32 = Color32::from_rgb(64, 209, 128);
pub const SUCCESS: Color32 = Color32::from_rgb(64, 209, 128);
pub const DANGER: Color32 = Color32::from_rgb(224, 94, 94);
pub const MUTED: Color32 = Color32::from_gray(90);

pub const APP_BG: Color32 = Color32::from_rgb(18, 19, 22);
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(14, 15, 18);
pub const CARD_BG: Color32 = Color32::from_rgb(26, 27, 31);
pub const CARD_BG_HOVER: Color32 = Color32::from_rgb(32, 34, 39);

pub fn apply(ctx: &egui::Context) {
    #[cfg(windows)]
    load_system_fonts(ctx);

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = APP_BG;
    visuals.window_fill = APP_BG;
    visuals.extreme_bg_color = SIDEBAR_BG;
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(64, 209, 128, 45);
    visuals.selection.stroke = egui::Stroke::new(1.0_f32, ACCENT);
    visuals.slider_trailing_fill = true;
    visuals.widgets.hovered.bg_fill = CARD_BG_HOVER;
    visuals.widgets.active.bg_fill = Color32::from_rgb(28, 58, 40);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

#[cfg(windows)]
fn load_system_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
        fonts
            .font_data
            .insert("segoe_ui".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe_ui".to_owned());
    }
    if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
        fonts.font_data.insert(
            "segoe_ui_symbol".to_owned(),
            egui::FontData::from_owned(data),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("segoe_ui_symbol".to_owned());
    }
    ctx.set_fonts(fonts);
}
