//! Small reusable UI pieces shared by the different sections of the app.

use eframe::egui::{self, Color32, Ui};

use crate::theme;

/// A small filled circle indicating connected (accent) vs. not (muted).
pub fn status_dot(ui: &mut Ui, connected: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
    let color = if connected {
        theme::SUCCESS
    } else {
        theme::MUTED
    };
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

/// A small battery icon, color-coded by charge level (muted/low →
/// accent-green/full) and outlined in the accent color while charging.
/// Hand-painted rather than an emoji/icon-font glyph — this project has
/// already hit missing-glyph tofu boxes once (see the typography
/// fallback fix), and a battery indicator that's supposed to always be
/// visible is the wrong place to risk that again.
pub fn battery_badge(ui: &mut Ui, percent: u8, charging: bool) {
    let body_size = egui::vec2(18.0, 9.0);
    let nub_size = egui::vec2(2.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(body_size.x + nub_size.x + 2.0, body_size.y),
        egui::Sense::hover(),
    );
    let painter = ui.painter();

    let body = egui::Rect::from_min_size(rect.min, body_size);
    let nub = egui::Rect::from_min_size(
        egui::pos2(body.max.x + 2.0, rect.center().y - nub_size.y / 2.0),
        nub_size,
    );

    let outline_color = if charging {
        theme::ACCENT
    } else {
        Color32::from_white_alpha(90)
    };
    painter.rect_stroke(body, 2.0, egui::Stroke::new(1.2_f32, outline_color));
    painter.rect_filled(nub, 1.0, outline_color);

    let fill_color = if percent <= 15 {
        theme::DANGER
    } else if percent <= 35 {
        Color32::from_rgb(224, 170, 64)
    } else {
        theme::SUCCESS
    };
    let inset = 1.5;
    let max_fill_w = body_size.x - inset * 2.0;
    let fill_w = max_fill_w * (percent.min(100) as f32 / 100.0);
    if fill_w > 0.5 {
        let fill_rect = egui::Rect::from_min_size(
            body.min + egui::vec2(inset, inset),
            egui::vec2(fill_w, body_size.y - inset * 2.0),
        );
        painter.rect_filled(fill_rect, 1.0, fill_color);
    }
}

/// A small rounded color sample with a faint border, for showing the
/// currently selected LED color next to its picker.
pub fn color_swatch(ui: &mut Ui, rgb: [u8; 3], size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    ui.painter().rect_filled(rect, 4.0, color);
    ui.painter().rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0_f32, Color32::from_white_alpha(40)),
    );
}

/// A titled card with an accent bar, used to group each section of
/// controls (Performance, Lighting, Info, ...) into a visually distinct
/// block instead of one long scrolling list.
pub fn card(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(theme::CARD_BG)
        .rounding(10.0)
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(4.0, 18.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 2.0, theme::ACCENT);
                ui.label(egui::RichText::new(title).strong().size(15.0));
            });
            ui.add_space(10.0);
            add_contents(ui);
        });
    ui.add_space(12.0);
}

/// What kind of LED effect to animate in [`effect_preview`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind {
    Off,
    Static,
    Breathing,
    Spectrum,
    Wave,
}

/// A live-animated strip showing roughly what the selected effect looks
/// like, so changing the color/effect gives immediate visual feedback
/// before it's sent to the device.
pub fn effect_preview(ui: &mut Ui, kind: PreviewKind, rgb: [u8; 3], time: f64) {
    let size = egui::vec2(ui.available_width(), 36.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    let color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    let off = Color32::from_gray(22);

    match kind {
        PreviewKind::Off => {
            painter.rect_filled(rect, 6.0, off);
        }
        PreviewKind::Static => {
            painter.rect_filled(rect, 6.0, color);
        }
        PreviewKind::Breathing => {
            let t = ((time * 1.4).sin() as f32 + 1.0) / 2.0;
            painter.rect_filled(rect, 6.0, lerp_color(off, color, t));
            ui.ctx().request_repaint();
        }
        PreviewKind::Spectrum => {
            const STEPS: i32 = 28;
            let w = rect.width() / STEPS as f32;
            for i in 0..STEPS {
                let hue = ((time * 50.0) as f32 + i as f32 * (360.0 / STEPS as f32)) % 360.0;
                let c = hsv_to_rgb(hue, 0.85, 0.95);
                let r = egui::Rect::from_min_size(
                    rect.min + egui::vec2(i as f32 * w, 0.0),
                    egui::vec2(w + 1.0, rect.height()),
                );
                painter.rect_filled(r, 0.0, c);
            }
            ui.ctx().request_repaint();
        }
        PreviewKind::Wave => {
            // The device's Wave effect takes no color argument at all —
            // it's always a traveling rainbow band, same as Synapse's
            // own Wave effect. Preview it that way rather than blending
            // the picked color, which would be actively misleading.
            const STEPS: i32 = 28;
            let w = rect.width() / STEPS as f32;
            for i in 0..STEPS {
                let hue = ((time * 90.0) as f32 + i as f32 * 10.0) % 360.0;
                let c = hsv_to_rgb(hue, 0.85, 0.95);
                let r = egui::Rect::from_min_size(
                    rect.min + egui::vec2(i as f32 * w, 0.0),
                    egui::vec2(w + 1.0, rect.height()),
                );
                painter.rect_filled(r, 0.0, c);
            }
            ui.ctx().request_repaint();
        }
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(lerp(a.r(), b.r()), lerp(a.g(), b.g()), lerp(a.b(), b.b()))
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match hp as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    Color32::from_rgb(
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}
