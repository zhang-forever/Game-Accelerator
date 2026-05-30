use super::theme;

/// Full-width gauge row: label on left, bar fills remaining space, percent on right.
/// Designed to be used inside a vertical layout (auto-fits container width).
pub fn gauge_row(ui: &mut egui::Ui, label: &str, percent: f32) {
    ui.horizontal(|ui| {
        // Fixed-width label column
        ui.allocate_ui_with_layout(
            egui::vec2(46.0, 16.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
            },
        );

        // Percent label reserved on the right
        let pct_text = format!("{:.0}%", percent);

        // Bar fills the space between label and percent
        let bar_h = 14.0;
        let reserved_right = 42.0;
        let bar_w = (ui.available_width() - reserved_right).max(40.0);

        let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, bar_h), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // Track
            painter.rect_filled(
                rect,
                egui::Rounding::same(4.0),
                egui::Color32::from_rgb(38, 40, 56),
            );
            // Fill
            let fill_w = rect.width() * (percent / 100.0).clamp(0.0, 1.0);
            if fill_w > 1.0 {
                let fill_rect =
                    egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                painter.rect_filled(
                    fill_rect,
                    egui::Rounding::same(4.0),
                    theme::gauge_color(percent),
                );
            }
        }

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(pct_text)
                .size(12.0)
                .strong()
                .color(theme::TEXT_PRIMARY),
        );
    });
}

/// Vertical stat card. Must be called with a column ui so it lays out top-to-bottom.
pub fn stat_card(ui: &mut egui::Ui, value: &str, label: &str, sub: &str) {
    egui::Frame::none()
        .fill(theme::CARD_BG)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(14.0))
        .stroke(egui::Stroke::new(1.0, theme::CARD_BORDER))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(value)
                        .size(24.0)
                        .strong()
                        .color(theme::ACCENT),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(label)
                        .size(13.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.label(
                    egui::RichText::new(sub)
                        .size(10.0)
                        .color(theme::TEXT_DIM),
                );
            });
        });
}

/// Section header with separator.
pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(14.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(8.0);
}

/// Small status badge.
pub fn status_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), egui::FontId::proportional(10.0), color);
    let size = galley.size();
    let pad = egui::vec2(6.0, 2.0);
    let (rect, _) = ui.allocate_exact_size(size + pad * 2.0, egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::Rounding::same(3.0), color.linear_multiply(0.15));
    ui.painter().galley(rect.left_top() + pad, galley, color);
}

/// An iOS-style toggle switch. Returns true if it was clicked this frame.
/// `on` reflects the current state and is used to draw the knob position.
pub fn toggle_switch(ui: &mut egui::Ui, on: bool) -> bool {
    let width = 44.0;
    let height = 24.0;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let radius = height / 2.0;

        // Track
        let track_color = if on {
            theme::ACCENT
        } else {
            egui::Color32::from_rgb(55, 58, 75)
        };
        painter.rect_filled(rect, egui::Rounding::same(radius), track_color);

        // Knob
        let knob_x = if on {
            rect.right() - radius
        } else {
            rect.left() + radius
        };
        let knob_center = egui::pos2(knob_x, rect.center().y);
        painter.circle_filled(
            knob_center,
            radius - 3.0,
            egui::Color32::from_rgb(245, 245, 250),
        );
    }

    response.clicked()
}

/// A full-width settings row: title + description on the left, a toggle on the right.
/// Returns true if the toggle was clicked.
pub fn toggle_row(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    on: bool,
) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(14.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(description)
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Status text + switch
            if toggle_switch(ui, on) {
                clicked = true;
            }
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(if on { "已开启" } else { "已关闭" })
                    .size(12.0)
                    .color(if on { theme::SUCCESS } else { theme::TEXT_DIM }),
            );
        });
    });
    clicked
}
