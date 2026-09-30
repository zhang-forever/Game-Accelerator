use super::{theme, widgets};
use crate::app::{ActionTarget, GameAcceleratorApp};
use crate::core::{disk_optimizer, game_mode, power_manager};

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    widgets::page_header(
        ui,
        "系统优化",
        "查看当前系统设置，按需调整游戏模式与系统服务。",
    );

    let snap = app.stats.lock().clone();
    let enabled = snap.flags_ready
        && !app.action_busy
        && !app.is_boosting
        && !app.is_restoring
        && !app.session_active();

    egui::ScrollArea::vertical()
        .id_salt("system_preferences")
        .show(ui, |ui| {
            widgets::notice(
                ui,
                "此页的手动修改会持续生效。一键加速的临时设置，请在仪表盘停止并恢复。",
                theme::ACCENT,
            );
            ui.add_space(14.0);

            if let Some(message) = &app.sysopt_status {
                widgets::notice(
                    ui,
                    message,
                    if message.starts_with('✓') {
                        theme::SUCCESS
                    } else {
                        theme::WARNING
                    },
                );
                ui.add_space(12.0);
            }
            if !snap.flags_ready {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        egui::RichText::new("正在读取当前系统设置…")
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                });
                ui.add_space(12.0);
            }
            for error in &snap.settings_query_errors {
                widgets::notice(ui, error, theme::WARNING);
                ui.add_space(12.0);
            }
            if app.action_busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        egui::RichText::new("正在应用设置，请稍候…")
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                });
                ui.add_space(12.0);
            }
            if app.session_active() {
                widgets::notice(
                    ui,
                    "当前加速会话正在管理电源和游戏模式，请先停止并恢复，再手动修改。",
                    theme::WARNING,
                );
                ui.add_space(12.0);
            }

            ui.add_enabled_ui(enabled, |ui| {
                theme::card_frame().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    widgets::section_header(ui, "游戏体验");

                    if let Some(enable) = widgets::setting_row(
                        ui,
                        "高性能电源计划",
                        "使用机器支持的计划。可能增加耗电和温度，效果请在游戏中比较。",
                        snap.power_high_perf,
                    ) {
                        app.start_action(ActionTarget::System, move || {
                            if enable {
                                power_manager::set_high_performance()
                                    .map(|_| "✓ 已切换到高性能计划".to_string())
                            } else {
                                power_manager::set_balanced()
                                    .map(|_| "✓ 已切换到平衡计划".to_string())
                            }
                        });
                    }
                    row_separator(ui);

                    if let Some(enable) = widgets::setting_row(
                        ui,
                        "Windows 游戏模式",
                        "使用 Windows 内置游戏模式。",
                        snap.game_mode_on,
                    ) {
                        app.start_action(ActionTarget::System, move || {
                            if enable {
                                game_mode::enable_game_mode()
                                    .map(|_| "✓ 已开启 Windows 游戏模式".to_string())
                            } else {
                                game_mode::disable_game_mode()
                                    .map(|_| "✓ 已关闭 Windows 游戏模式".to_string())
                            }
                        });
                    }
                    row_separator(ui);

                    if let Some(enable) = widgets::setting_row(
                        ui,
                        "手柄按钮打开 Game Bar",
                        "控制手柄按钮是否呼出 Game Bar。Win+G 和工具栏本身仍然可用。",
                        snap.game_bar_on,
                    ) {
                        app.start_action(ActionTarget::System, move || {
                            game_mode::toggle_game_bar(enable).map(|_| {
                                if enable {
                                    "✓ 已允许手柄按钮打开 Game Bar".to_string()
                                } else {
                                    "✓ 已关闭手柄按钮打开 Game Bar".to_string()
                                }
                            })
                        });
                    }
                });

                ui.add_space(14.0);
                theme::card_frame().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    widgets::section_header(ui, "硬件与系统服务");
                    ui.horizontal_wrapped(|ui| {
                        widgets::status_badge(ui, "需要管理员权限", theme::WARNING);
                        ui.label(
                            egui::RichText::new("请按实际需要调整")
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                        );
                    });
                    ui.add_space(16.0);

                    if let Some(enable) = widgets::setting_row(
                        ui,
                        "硬件加速 GPU 调度",
                        "需要硬件与驱动支持，修改后重启电脑生效。不保证每款游戏都受益。",
                        snap.hw_gpu_sched_on,
                    ) {
                        app.start_action(ActionTarget::System, move || {
                            game_mode::toggle_hardware_gpu_scheduling(enable)
                                .map(|_| "✓ GPU 调度设置已写入，重启后生效".to_string())
                        });
                    }
                    row_separator(ui);

                    if let Some(pause) = widgets::setting_row(
                        ui,
                        "暂停磁盘索引",
                        "停止或启动 Windows Search 服务。暂停会影响系统搜索。",
                        snap.search_indexer_running.map(|running| !running),
                    ) {
                        app.start_action(ActionTarget::System, move || {
                            if pause {
                                disk_optimizer::stop_windows_search_service()
                                    .map(|_| "✓ 已暂停 Windows Search 服务".to_string())
                            } else {
                                disk_optimizer::start_windows_search_service()
                                    .map(|_| "✓ 已恢复 Windows Search 服务".to_string())
                            }
                        });
                    }
                });
            });
            ui.add_space(4.0);
        });
}

fn row_separator(ui: &mut egui::Ui) {
    ui.add_space(14.0);
    ui.separator();
    ui.add_space(14.0);
}
