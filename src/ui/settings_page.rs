use super::{theme, widgets};
use crate::app::GameAcceleratorApp;

const GAME_PRESETS: [(&str, &str, &str); 3] = [
    ("无畏契约", "VALORANT", "VALORANT-Win64-Shipping.exe"),
    ("英雄联盟", "LEAGUE OF LEGENDS", "League of Legends.exe"),
    ("穿越火线", "CROSSFIRE", "crossfire.exe"),
];

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    widgets::page_header(ui, "设置", "选择常玩的游戏，配置一键加速的行为。");

    // Keep save controls visible while the preference cards scroll.
    let footer_height = 58.0
        + app.settings_status.as_ref().map_or(0.0, |message| {
            ui.painter()
                .layout(
                    message.clone(),
                    egui::FontId::proportional(12.0),
                    theme::TEXT_PRIMARY,
                    (ui.available_width() - 32.0).max(1.0),
                )
                .size()
                .y
                + 38.0
        });
    egui::ScrollArea::vertical()
        .id_salt("settings_preferences")
        .max_height((ui.available_height() - footer_height).max(0.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "常玩的游戏");

                let selected_file = app
                    .config
                    .selected_game
                    .as_deref()
                    .unwrap_or_default()
                    .rsplit(['\\', '/'])
                    .next()
                    .unwrap_or_default()
                    .to_string();
                let mut selected_preset = None;

                if ui.available_width() >= 440.0 {
                    ui.columns(3, |columns| {
                        for (index, &(name, subtitle, exe)) in GAME_PRESETS.iter().enumerate() {
                            let selected = selected_file.eq_ignore_ascii_case(exe);
                            if preset_button(&mut columns[index], name, subtitle, selected) {
                                selected_preset = Some(exe);
                            }
                        }
                    });
                } else {
                    for &(name, subtitle, exe) in &GAME_PRESETS {
                        let selected = selected_file.eq_ignore_ascii_case(exe);
                        if preset_button(ui, name, subtitle, selected) {
                            selected_preset = Some(exe);
                        }
                        ui.add_space(6.0);
                    }
                }

                if let Some(exe) = selected_preset {
                    app.config.select_competitive_game(exe);
                    app.save_config();
                }

                ui.add_space(14.0);
                let current_game = app.config.selected_game.as_deref().unwrap_or_default();
                let current_file = current_game
                    .rsplit(['\\', '/'])
                    .next()
                    .unwrap_or_default();
                let current_name = GAME_PRESETS
                    .iter()
                    .find(|(_, _, exe)| current_file.eq_ignore_ascii_case(exe))
                    .map(|(name, _, _)| *name)
                    .unwrap_or(if current_game.is_empty() {
                        "尚未选择"
                    } else {
                        "自定义游戏"
                    });
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("当前游戏")
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                    widgets::status_badge(ui, current_name, theme::ACCENT);
                });
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "预设选择后自动保存。保留游戏、反作弊、语音和同步进程，不裁剪内存或提升优先级。",
                    )
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
            });

            ui.add_space(14.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "一键加速选项");
                ui.label(
                    egui::RichText::new("这些选项会在点击「启动加速」后执行。")
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(14.0);

                config_row(
                    ui,
                    &mut app.config.enable_high_perf_power,
                    "高性能电源计划",
                    "使用机器支持的高性能计划；停止加速或正常退出时恢复原计划。",
                );
                row_separator(ui);
                config_row(
                    ui,
                    &mut app.config.enable_game_mode,
                    "Windows 游戏模式",
                    "启用系统内置游戏模式；停止加速或正常退出时恢复原值。",
                );
                row_separator(ui);
                config_row(
                    ui,
                    &mut app.config.kill_background_processes,
                    "关闭后台进程",
                    "仅处理明确加入黑名单的程序。关闭前请保存工作，默认不启用。",
                );
            });

            ui.add_space(14.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "游戏可执行文件");
                ui.label(
                    egui::RichText::new("游戏 EXE 文件名或完整路径")
                        .size(14.0)
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(8.0);
                let mut game = app.config.selected_game.clone().unwrap_or_default();
                let text_edit = egui::TextEdit::singleline(&mut game)
                    .hint_text("例如 game.exe 或 E:\\Games\\game.exe")
                    .desired_width(ui.available_width());
                if ui.add(text_edit).changed() {
                    app.config.selected_game = if game.is_empty() { None } else { Some(game) };
                }
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "文件名用于识别游戏进程。若要设置显卡偏好，请在 GPU 设置页填写完整路径。",
                    )
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
            });

            ui.add_space(14.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "应用信息");
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("Game Accelerator")
                            .size(14.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    widgets::status_badge(
                        ui,
                        concat!("v", env!("CARGO_PKG_VERSION")),
                        theme::TEXT_SECONDARY,
                    );
                });
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(
                        "应用启动后只进行监控。游戏结束后，点击「停止并恢复原设置」。部分系统操作需要管理员权限。",
                    )
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(format!(
                        "配置文件：{}",
                        crate::config::AppConfig::config_path().display()
                    ))
                    .size(12.0)
                    .color(theme::TEXT_DIM),
                );
            });
            ui.add_space(4.0);
        });

    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        if ui.add(theme::primary_button("保存设置")).clicked() {
            app.save_config();
        }
        if ui.add(theme::secondary_button("恢复默认")).clicked() {
            app.config = crate::config::AppConfig::default();
            app.save_config();
        }
    });

    if let Some(status) = &app.settings_status {
        ui.add_space(8.0);
        widgets::notice(
            ui,
            status,
            if status.starts_with('✓') {
                theme::SUCCESS
            } else {
                theme::WARNING
            },
        );
    }
}

fn preset_button(ui: &mut egui::Ui, name: &str, subtitle: &str, selected: bool) -> bool {
    let mut text = egui::text::LayoutJob::default();
    text.append(
        name,
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(14.0),
            color: theme::TEXT_PRIMARY,
            ..Default::default()
        },
    );
    text.append(
        &format!("\n{}", if selected { "已选择" } else { subtitle }),
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(11.0),
            color: if selected {
                theme::ACCENT
            } else {
                theme::TEXT_SECONDARY
            },
            ..Default::default()
        },
    );
    ui.add_sized(
        egui::vec2(ui.available_width(), 76.0),
        egui::Button::new(text)
            .fill(if selected {
                theme::ACCENT_BG
            } else {
                theme::SURFACE
            })
            .stroke(egui::Stroke::new(
                1.0,
                if selected {
                    theme::ACCENT
                } else {
                    theme::CARD_BORDER
                },
            ))
            .rounding(egui::Rounding::same(12.0)),
    )
    .clicked()
}

fn config_row(ui: &mut egui::Ui, checked: &mut bool, title: &str, description: &str) {
    if widgets::toggle_row(ui, title, description, *checked) {
        *checked = !*checked;
    }
}

fn row_separator(ui: &mut egui::Ui) {
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
}
