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
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = APP_BG;
    visuals.window_fill = APP_BG;
    visuals.extreme_bg_color = SIDEBAR_BG;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke.color = Color32::BLACK;
    visuals.widgets.hovered.bg_fill = CARD_BG_HOVER;
    visuals.widgets.active.bg_fill = ACCENT;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}
