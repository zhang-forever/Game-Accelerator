mod app;
mod config;
mod core;
mod monitor;
mod ui;

use app::GameAcceleratorApp;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([750.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Game Accelerator",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            apply_dark_theme(&cc.egui_ctx);
            Ok(Box::new(GameAcceleratorApp::new(cc)))
        }),
    )
    .unwrap();
}

fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Load Microsoft YaHei for Chinese support
    let font_candidates = [
        ("C:/Windows/Fonts/msyh.ttc", 0),       // Microsoft YaHei Regular
        ("C:/Windows/Fonts/msyhbd.ttc", 0),      // Microsoft YaHei Bold
        ("C:/Windows/Fonts/simhei.ttf", 0),      // SimHei
    ];

    let mut loaded = false;
    for (path, _index) in &font_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese".to_owned(),
                egui::FontData::from_owned(data),
            );

            // Add as fallback for proportional font family
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.push("chinese".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push("chinese".to_owned());
            }
            loaded = true;
            break;
        }
    }

    if !loaded {
        eprintln!("Warning: No Chinese font found. Chinese characters may not render correctly.");
    }

    ctx.set_fonts(fonts);
}

fn apply_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Rounded corners for everything
    style.visuals.window_rounding = egui::Rounding::same(10.0);
    style.visuals.menu_rounding = egui::Rounding::same(8.0);
    style.visuals.popup_shadow = egui::epaint::Shadow {
        offset: egui::Vec2::new(0.0, 4.0),
        blur: 12.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(60),
    };

    // Dark cyberpunk colors
    let v = &mut style.visuals;
    v.dark_mode = true;
    v.panel_fill = egui::Color32::from_rgb(13, 13, 20);
    v.window_fill = egui::Color32::from_rgb(20, 20, 30);
    v.extreme_bg_color = egui::Color32::from_rgb(8, 8, 14);
    v.faint_bg_color = egui::Color32::from_rgb(25, 25, 38);
    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 25, 38);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80));
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 32, 48);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 200));
    v.widgets.inactive.rounding = egui::Rounding::same(6.0);
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 255, 136);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(10, 10, 16));
    v.widgets.hovered.rounding = egui::Rounding::same(6.0);
    v.widgets.active.bg_fill = egui::Color32::from_rgb(0, 200, 110);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(10, 10, 16));
    v.widgets.active.rounding = egui::Rounding::same(6.0);

    // Selection = neon green
    v.selection.bg_fill = egui::Color32::from_rgb(0, 255, 136);
    v.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 136));
    v.hyperlink_color = egui::Color32::from_rgb(80, 180, 255);

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);

    ctx.set_style(style);
}
