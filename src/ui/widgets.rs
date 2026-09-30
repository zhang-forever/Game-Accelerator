use super::{icons, theme};

pub fn page_header(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(26.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(description)
            .size(12.5)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(12.0);
}

pub fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(15.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.add_space(10.0);
}

pub fn notice(ui: &mut egui::Ui, message: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.gamma_multiply(0.10))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(14.0, 11.0))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.3)))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(message.trim_start_matches(['✓', '⚠', ' ']))
                        .size(12.5)
                        .color(color),
                );
            });
        });
    ui.add_space(10.0);
}

pub fn status_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.gamma_multiply(0.12))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(11.0).color(color));
        });
}

pub fn gauge_row(ui: &mut egui::Ui, label: &str, percent: f32) {
    let percent = if percent.is_finite() {
        percent.clamp(0.0, 100.0)
    } else {
        0.0
    };
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(58.0, 20.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
            },
        );
        let width = (ui.available_width() - 54.0).max(20.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 8.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(4.0), theme::SURFACE);
        let filled = rect.width() * percent / 100.0;
        if filled > 0.0 {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(filled, rect.height())),
                egui::Rounding::same(4.0),
                theme::gauge_color(percent),
            );
        }
        ui.label(
            egui::RichText::new(format!("{percent:.0}%"))
                .size(12.0)
                .color(theme::TEXT_PRIMARY),
        );
    });
}

pub fn metric_card(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    detail: &str,
    percent: Option<f32>,
    icon: icons::Icon,
) {
    let width = ui.available_width();
    theme::card_frame().show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.set_min_width((width - 36.0).max(10.0));
        ui.horizontal(|ui| {
            icons::show(ui, icon, 18.0, theme::ACCENT);
            ui.label(
                egui::RichText::new(label)
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            );
        });
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new(value)
                .size(28.0)
                .strong()
                .color(theme::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(detail)
                .size(11.0)
                .color(theme::TEXT_DIM),
        );
        ui.add_space(4.0);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 4.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(2.0), theme::SURFACE);
        if let Some(percent) = percent {
            let width = rect.width() * (percent / 100.0).clamp(0.0, 1.0);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(width, 4.0)),
                egui::Rounding::same(2.0),
                theme::ACCENT,
            );
        }
    });
}

pub fn toggle_switch(ui: &mut egui::Ui, on: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(40.0, 24.0), egui::Sense::click());
    let on = ui.ctx().animate_bool(response.id, on);
    let color = if on > 0.5 {
        theme::ACCENT
    } else {
        theme::SURFACE_HOVER
    };
    ui.painter()
        .rect_filled(rect, egui::Rounding::same(12.0), color);
    let x = egui::lerp(rect.left() + 12.0..=rect.right() - 12.0, on);
    ui.painter()
        .circle_filled(egui::pos2(x, rect.center().y), 8.0, theme::TEXT_PRIMARY);
    response.clicked()
}

pub fn toggle_row(ui: &mut egui::Ui, title: &str, description: &str, on: bool) -> bool {
    let mut clicked = false;
    let text_width = (ui.available_width() - 142.0).max(120.0);
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(text_width, 48.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(14.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.label(
                    egui::RichText::new(description)
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
            },
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            clicked = toggle_switch(ui, on);
            ui.label(
                egui::RichText::new(if on { "已开启" } else { "已关闭" })
                    .size(12.0)
                    .color(if on { theme::SUCCESS } else { theme::TEXT_DIM }),
            );
        });
    });
    clicked
}

pub fn setting_row(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    state: Option<bool>,
) -> Option<bool> {
    if let Some(on) = state {
        return toggle_row(ui, title, description, on).then_some(!on);
    }
    let mut choice = None;
    ui.label(
        egui::RichText::new(title)
            .size(14.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.label(
        egui::RichText::new(description)
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        status_badge(ui, "系统默认 / 状态未知", theme::TEXT_DIM);
        if ui.add(theme::secondary_button("开启")).clicked() {
            choice = Some(true);
        }
        if ui.add(theme::secondary_button("关闭")).clicked() {
            choice = Some(false);
        }
    });
    choice
}
