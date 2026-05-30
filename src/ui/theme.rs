// Neon green accent - brighter and more vibrant
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 255, 140);
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0, 230, 126);
pub const ACCENT_BG: egui::Color32 = egui::Color32::from_rgb(0, 45, 28);

// Card backgrounds - slightly lighter for better contrast
pub const CARD_BG: egui::Color32 = egui::Color32::from_rgb(26, 28, 40);
pub const CARD_BORDER: egui::Color32 = egui::Color32::from_rgb(48, 52, 70);

// Text - high contrast for readability
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(240, 242, 250);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(175, 180, 200);
pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(125, 130, 150);

// Status colors - more vibrant
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(255, 85, 85);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 200, 75);
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0, 255, 140);

// Gauge gradient
pub const GAUGE_LOW: egui::Color32 = egui::Color32::from_rgb(40, 210, 140);
pub const GAUGE_MID: egui::Color32 = egui::Color32::from_rgb(255, 195, 70);
pub const GAUGE_HIGH: egui::Color32 = egui::Color32::from_rgb(255, 95, 95);

pub fn gauge_color(percent: f32) -> egui::Color32 {
    if percent < 50.0 {
        GAUGE_LOW
    } else if percent < 80.0 {
        GAUGE_MID
    } else {
        GAUGE_HIGH
    }
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(CARD_BG)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, CARD_BORDER))
}

/// Enhanced button style for primary actions
pub fn primary_button(text: &str) -> egui::Button {
    egui::Button::new(
        egui::RichText::new(text)
            .size(13.0)
            .strong()
            .color(egui::Color32::from_rgb(8, 10, 14)),
    )
    .fill(ACCENT)
    .rounding(egui::Rounding::same(6.0))
    .min_size(egui::vec2(120.0, 36.0))
}

/// Secondary button style
pub fn secondary_button(text: &str) -> egui::Button {
    egui::Button::new(
        egui::RichText::new(text)
            .size(13.0)
    )
    .fill(egui::Color32::from_rgb(55, 58, 75))
    .rounding(egui::Rounding::same(6.0))
    .min_size(egui::vec2(120.0, 36.0))
}
