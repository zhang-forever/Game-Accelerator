use super::{theme, widgets};
use crate::app::{ActionTarget, GameAcceleratorApp};
use crate::core::{disk_optimizer, game_mode, power_manager};

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("系统优化")
            .size(22.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("此页手动修改会持续生效；一键加速的临时设置请在仪表盘停止并恢复。")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(10.0);
    if let Some(message) = &app.sysopt_status {
        ui.label(
            egui::RichText::new(message).color(if message.starts_with('✓') {
                theme::SUCCESS
            } else {
                theme::WARNING
            }),
        );
    }

    let snap = app.stats.lock().clone();
    if !snap.flags_ready {
        ui.label("正在读取当前系统设置…");
    }
    for error in &snap.settings_query_errors {
        ui.label(
            egui::RichText::new(format!("⚠ {}", error))
                .size(11.0)
                .color(theme::WARNING),
        );
    }
    if app.action_busy {
        ui.label("正在应用设置，请稍候…");
    }
    if app.session_active() {
        ui.label("当前加速会话正在管理电源和游戏模式，请先停止并恢复后再手动修改。");
    }
    let enabled = snap.flags_ready
        && !app.action_busy
        && !app.is_boosting
        && !app.is_restoring
        && !app.session_active();

    ui.add_enabled_ui(enabled, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(enable) = widgets::setting_row(
                    ui,
                    "高性能电源模式",
                    "根据机器支持的计划切换；可能增加耗电和温度，效果需在游戏中比较。",
                    snap.power_high_perf,
                ) {
                    app.start_action(ActionTarget::System, move || {
                        if enable {
                            power_manager::set_high_performance()
                                .map(|_| "✓ 已切换到高性能计划".to_string())
                        } else {
                            power_manager::set_balanced().map(|_| "✓ 已切换到平衡计划".to_string())
                        }
                    });
                }
            });
            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
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
            });
            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(enable) = widgets::setting_row(
                    ui,
                    "硬件加速 GPU 调度",
                    "需要硬件和驱动支持、管理员权限及重启；不保证每款游戏受益。",
                    snap.hw_gpu_sched_on,
                ) {
                    app.start_action(ActionTarget::System, move || {
                        game_mode::toggle_hardware_gpu_scheduling(enable)
                            .map(|_| "✓ GPU 调度设置已写入，重启后生效".to_string())
                    });
                }
            });
            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(enable) = widgets::setting_row(
                    ui,
                    "手柄按钮打开 Game Bar",
                    "控制手柄按钮是否呼出 Game Bar；Win+G 和工具栏本身保持可用。",
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
            ui.add_space(8.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(pause) = widgets::setting_row(
                    ui,
                    "暂停磁盘索引",
                    "仅停止或启动 Windows Search 服务，需要管理员权限，会影响系统搜索。",
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
    });
}
