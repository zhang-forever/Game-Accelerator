/// Neon green accent color used for primary actions, highlights, and
/// selection indicators throughout the UI.
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0, 255, 140);
/// Darker green used for hover states on accent-colored widgets.
pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(0, 230, 126);
/// Transparent green tint used as background for informational banners.
pub const ACCENT_BG: egui::Color32 = egui::Color32::from_rgb(0, 45, 28);

/// Dark background color for content cards and panels.
pub const CARD_BG: egui::Color32 = egui::Color32::from_rgb(26, 28, 40);
/// Subtle border color used to outline content cards.
pub const CARD_BORDER: egui::Color32 = egui::Color32::from_rgb(48, 52, 70);

/// High-contrast white text used for headings and primary content.
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(240, 242, 250);
/// Muted text color used for descriptions and secondary information.
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(175, 180, 200);
/// Dim text color used for labels, hints, and metadata.
pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(125, 130, 150);

/// Red status color for destructive actions and error states.
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(255, 85, 85);
/// Amber status color for warnings and caution indicators.
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 200, 75);
/// Green status color for success states and "active" indicators.
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0, 255, 140);

/// Green gauge fill color for low utilization (< 50%).
pub const GAUGE_LOW: egui::Color32 = egui::Color32::from_rgb(40, 210, 140);
/// Yellow gauge fill color for medium utilization (50-80%).
pub const GAUGE_MID: egui::Color32 = egui::Color32::from_rgb(255, 195, 70);
/// Red gauge fill color for high utilization (> 80%).
pub const GAUGE_HIGH: egui::Color32 = egui::Color32::from_rgb(255, 95, 95);

/// Return a gauge fill color that transitions from green to yellow to red
/// based on the utilization percentage.
pub fn gauge_color(percent: f32) -> egui::Color32 {
    if percent < 50.0 {
        GAUGE_LOW
    } else if percent < 80.0 {
        GAUGE_MID
    } else {
        GAUGE_HIGH
    }
}

/// Create a standard content card frame with dark background, rounded
/// corners, and a subtle border.
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
    egui::Button::new(egui::RichText::new(text).size(13.0))
        .fill(egui::Color32::from_rgb(55, 58, 75))
        .rounding(egui::Rounding::same(6.0))
        .min_size(egui::vec2(120.0, 36.0))
}
