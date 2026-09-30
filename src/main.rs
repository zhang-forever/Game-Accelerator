// Hide the console window in release builds. Without this, double-clicking the
// exe from Explorer pops up an empty black console alongside the GUI. Debug
// builds keep the console so println!/eprintln! output is still visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod cli;
mod config;
mod core;
mod monitor;
mod smoke_test;
mod ui;

use app::GameAcceleratorApp;

fn main() {
    let arguments = match cli::Arguments::parse(std::env::args_os().skip(1)) {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(2);
        }
    };
    if arguments.help {
        println!("Game Accelerator {}\n--config-dir <directory>\n--diagnose <report.toml> (read-only)\n--smoke-test <directory> (read-only GUI captures)", env!("CARGO_PKG_VERSION"));
        return;
    }
    if let Some(directory) = arguments.config_dir {
        if let Err(error) = config::settings::set_config_directory(directory) {
            eprintln!("{}", error);
            std::process::exit(2);
        }
    }
    if let Some(report) = arguments.diagnose {
        if let Err(error) = cli::write_diagnostics(&report) {
            eprintln!("{}", error);
            std::process::exit(1);
        }
        return;
    }
    let window_size = arguments.window_size.unwrap_or([1080, 720]);
    let smoke_directory = arguments.smoke_test;
    let window_state_path = smoke_directory.as_ref().map_or_else(
        || config::AppConfig::config_path().with_file_name("window.ron"),
        |directory| directory.join("window.ron"),
    );
    let icon = image::load_from_memory(include_bytes!("../assets/app-icon.png"))
        .expect("Bundled app icon must be a valid PNG")
        .into_rgba8();
    let icon = egui::IconData {
        width: icon.width(),
        height: icon.height(),
        rgba: icon.into_raw(),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window_size[0] as f32, window_size[1] as f32])
            .with_min_inner_size([880.0, 560.0])
            .with_icon(icon),
        persistence_path: Some(window_state_path),
        persist_window: smoke_directory.is_none(),
        ..Default::default()
    };

    eframe::run_native(
        "Game Accelerator",
        options,
        Box::new(move |cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            apply_dark_theme(&cc.egui_ctx);
            Ok(Box::new(GameAcceleratorApp::new(cc, smoke_directory)))
        }),
    )
    .unwrap_or_else(|e| {
        eprintln!("Fatal: failed to launch GUI: {}", e);
        std::process::exit(1);
    });
}

fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Load Microsoft YaHei for Chinese support
    let font_candidates = [
        ("C:/Windows/Fonts/msyh.ttc", 0),   // Microsoft YaHei Regular
        ("C:/Windows/Fonts/msyhbd.ttc", 0), // Microsoft YaHei Bold
        ("C:/Windows/Fonts/simhei.ttf", 0), // SimHei
    ];

    let mut loaded = false;
    for (path, _index) in &font_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("chinese".to_owned(), egui::FontData::from_owned(data));

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
    ui::theme::apply(ctx);
}
