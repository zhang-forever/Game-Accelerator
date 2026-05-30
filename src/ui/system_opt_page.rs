use crate::app::GameAcceleratorApp;
use crate::core::{cpu_optimizer, disk_optimizer, game_mode, power_manager};
use super::{theme, widgets};

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("系统优化")
                .size(22.0)
                .strong()
                .color(theme::TEXT_PRIMARY),
        );
        ui.add_space(8.0);

        // Help tooltip
        let help_icon = ui.label(
            egui::RichText::new("❓")
                .size(16.0)
                .color(theme::TEXT_DIM),
        );
        help_icon.on_hover_text(
            "Windows 系统级游戏优化开关\n\n💡 使用方法：\n• 绿色开关 = 已开启优化\n• 点击开关切换状态\n• 部分设置需要重启生效\n\n⚠️ 注意：\n• 大部分功能需要管理员权限\n• 建议在游戏前开启，游戏后关闭\n• 高性能模式会增加耗电"
        );
    });
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("打开下面的开关即可优化。绿色 = 已开启游戏优化状态。")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(10.0);

    // Status feedback banner
    if let Some(ref msg) = app.sysopt_status {
        let color = if msg.starts_with("✓") {
            theme::SUCCESS
        } else {
            theme::WARNING
        };
        egui::Frame::none()
            .fill(color.linear_multiply(0.12))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(msg).size(13.0).color(color));
            });
        ui.add_space(10.0);
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Read all toggle states from the cached background snapshot instead of
        // shelling out (powercfg / reg query) on every repaint. This is what keeps
        // the page responsive.
        let snap = app.stats.lock().clone();

        // ===== Power plan =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let on = snap.power_high_perf;
            if widgets::toggle_row(
                ui,
                "高性能电源模式",
                "让 CPU 全速运行，不再为省电降频。插电玩游戏必开。",
                on,
            ) {
                let res = if on {
                    power_manager::set_balanced()
                        .map(|_| "✓ 已切换回平衡电源模式".to_string())
                } else {
                    power_manager::set_high_performance()
                        .map(|_| "✓ 已切换到高性能电源模式".to_string())
                };
                app.sysopt_status = Some(unwrap_result(res));
            }
        });

        ui.add_space(8.0);

        // ===== Game Mode =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let on = snap.game_mode_on;
            if widgets::toggle_row(
                ui,
                "Windows 游戏模式",
                "系统自带功能，玩游戏时优先把资源分给游戏。",
                on,
            ) {
                let res = if on {
                    game_mode::disable_game_mode().map(|_| "✓ 已关闭 Windows 游戏模式".to_string())
                } else {
                    game_mode::enable_game_mode().map(|_| "✓ 已开启 Windows 游戏模式".to_string())
                };
                app.sysopt_status = Some(unwrap_result(res));
            }
        });

        ui.add_space(8.0);

        // ===== Hardware GPU Scheduling =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let on = snap.hw_gpu_sched_on;
            if widgets::toggle_row(
                ui,
                "硬件加速 GPU 调度",
                "让显卡自己管理显存调度，部分游戏能降低延迟（重启生效）。",
                on,
            ) {
                let res = game_mode::toggle_hardware_gpu_scheduling(!on).map(|_| {
                    if on {
                        "✓ 已关闭硬件加速 GPU 调度（重启生效）".to_string()
                    } else {
                        "✓ 已开启硬件加速 GPU 调度（重启生效）".to_string()
                    }
                });
                app.sysopt_status = Some(unwrap_result(res));
            }
        });

        ui.add_space(8.0);

        // ===== Game Bar =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let on = snap.game_bar_on;
            if widgets::toggle_row(
                ui,
                "Xbox Game Bar",
                "Win+G 呼出的录屏工具栏。用不到的话关掉能省一点资源。",
                on,
            ) {
                let res = game_mode::toggle_game_bar(!on).map(|_| {
                    if on {
                        "✓ 已关闭 Xbox Game Bar".to_string()
                    } else {
                        "✓ 已开启 Xbox Game Bar".to_string()
                    }
                });
                app.sysopt_status = Some(unwrap_result(res));
            }
        });

        ui.add_space(8.0);

        // ===== Windows Search index =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let running = snap.search_indexer_running;
            // "on" here means "indexing running" — for gaming we want it OFF,
            // so the toggle represents "停止索引" being active when NOT running.
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("暂停磁盘索引")
                            .size(14.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new("Windows 后台扫描硬盘的服务，玩游戏时暂停可减少卡顿。")
                            .size(11.0)
                            .color(theme::TEXT_DIM),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // toggle ON = indexing paused (not running)
                    let paused = !running;
                    if widgets::toggle_switch(ui, paused) {
                        let res = if paused {
                            // currently paused -> resume
                            disk_optimizer::start_windows_search_service()
                                .map(|_| "✓ 已恢复磁盘索引服务".to_string())
                        } else {
                            disk_optimizer::stop_windows_search_service()
                                .map(|_| "✓ 已暂停磁盘索引服务".to_string())
                        };
                        app.sysopt_status = Some(unwrap_result(res));
                    }
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(if paused { "已暂停" } else { "运行中" })
                            .size(12.0)
                            .color(if paused { theme::SUCCESS } else { theme::TEXT_DIM }),
                    );
                });
            });
        });

        ui.add_space(8.0);

        // ===== Advanced one-shot actions =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new("进阶优化（点一下执行）")
                    .size(13.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if action_button(ui, "禁用 CPU 核心停车").clicked() {
                    let res = cpu_optimizer::disable_core_parking()
                        .map(|_| "✓ 已禁用 CPU 核心停车，避免游戏中降频".to_string());
                    app.sysopt_status = Some(unwrap_result(res));
                }
                ui.add_space(6.0);
                if action_button(ui, "清理系统缓存").clicked() {
                    let res = disk_optimizer::flush_file_cache()
                        .map(|_| "✓ 已清理系统文件缓存".to_string());
                    app.sysopt_status = Some(unwrap_result(res));
                }
            });
        });
    });
}

fn unwrap_result(r: Result<String, String>) -> String {
    match r {
        Ok(s) => s,
        Err(e) => format!("⚠ {}（可能需要管理员权限）", e),
    }
}

fn action_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(13.0)
                .strong()
                .color(egui::Color32::from_rgb(8, 10, 14)),
        )
        .fill(theme::ACCENT)
        .rounding(egui::Rounding::same(6.0))
        .min_size(egui::vec2(140.0, 32.0)),
    )
}
