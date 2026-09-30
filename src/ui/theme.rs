use egui::{Color32, FontFamily, FontId, Rounding, Stroke, TextStyle};

pub const BG: Color32 = Color32::from_rgb(12, 18, 29);
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(16, 23, 36);
pub const CARD_BG: Color32 = Color32::from_rgb(21, 30, 45);
pub const CARD_BORDER: Color32 = Color32::from_rgb(39, 52, 72);
pub const SURFACE: Color32 = Color32::from_rgb(29, 41, 59);
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(38, 53, 75);
pub const ACCENT: Color32 = Color32::from_rgb(111, 166, 255);
pub const ACCENT_BG: Color32 = Color32::from_rgb(28, 47, 76);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(235, 241, 250);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(162, 178, 200);
pub const TEXT_DIM: Color32 = Color32::from_rgb(116, 136, 161);
pub const SUCCESS: Color32 = Color32::from_rgb(77, 203, 171);
pub const WARNING: Color32 = Color32::from_rgb(237, 190, 105);
pub const DANGER: Color32 = Color32::from_rgb(244, 119, 139);
pub const GAUGE_LOW: Color32 = ACCENT;
pub const GAUGE_MID: Color32 = WARNING;
pub const GAUGE_HIGH: Color32 = DANGER;

pub fn gauge_color(percent: f32) -> Color32 {
    if percent >= 80.0 {
        GAUGE_HIGH
    } else if percent >= 60.0 {
        GAUGE_MID
    } else {
        GAUGE_LOW
    }
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(CARD_BG)
        .rounding(Rounding::same(16.0))
        .inner_margin(egui::Margin::same(18.0))
        .stroke(Stroke::new(1.0, CARD_BORDER))
}

pub fn primary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(14.0).strong().color(BG))
        .fill(ACCENT)
        .rounding(Rounding::same(9.0))
        .min_size(egui::vec2(124.0, 40.0))
}

pub fn secondary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).size(13.0).color(TEXT_PRIMARY))
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, CARD_BORDER))
        .rounding(Rounding::same(9.0))
        .min_size(egui::vec2(110.0, 38.0))
}

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(26.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();
    let visuals = &mut style.visuals;
    *visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG;
    visuals.window_fill = CARD_BG;
    visuals.extreme_bg_color = SIDEBAR_BG;
    visuals.faint_bg_color = SURFACE;
    visuals.window_rounding = Rounding::same(14.0);
    visuals.menu_rounding = Rounding::same(10.0);
    visuals.selection.bg_fill = ACCENT_BG;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.hyperlink_color = ACCENT;
    visuals.widgets.noninteractive.bg_fill = CARD_BG;
    visuals.widgets.noninteractive.weak_bg_fill = CARD_BG;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, CARD_BORDER);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.rounding = Rounding::same(8.0);
        widget.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        widget.bg_stroke = Stroke::new(1.0, CARD_BORDER);
    }
    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.weak_bg_fill = SURFACE;
    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.bg_fill = ACCENT_BG;
    visuals.widgets.active.weak_bg_fill = ACCENT_BG;
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.interact_size.y = 34.0;
    style.spacing.window_margin = egui::Margin::same(20.0);
    ctx.set_style(style);
}
