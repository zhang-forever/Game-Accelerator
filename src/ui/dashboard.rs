use super::{theme, widgets};
use crate::app::GameAcceleratorApp;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("仪表盘")
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
            "实时监控系统资源使用情况\n点击「启动加速」优化游戏性能\n\n💡 提示：\n• 加速前先在「设置」中配置游戏路径\n• 部分功能需要管理员权限\n• 建议在启动游戏前执行加速"
        );
    });
    ui.add_space(6.0);

    // Quick guide banner for first-time users
    if app.boost_result.is_none() {
        egui::Frame::none()
            .fill(theme::ACCENT_BG)
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("💡").size(14.0));
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("快速开始")
                                .size(12.0)
                                .strong()
                                .color(theme::ACCENT),
                        );
                        ui.label(
                            egui::RichText::new(
                                "设置中选游戏预设 → 启动加速 → 启动游戏；游戏结束后停止并恢复。",
                            )
                            .size(10.0)
                            .color(theme::TEXT_SECONDARY),
                        );
                    });
                });
            });
        ui.add_space(8.0);
    }

    ui.add_space(8.0);

    let stats = app.stats.lock().clone();

    // Top stat cards - 4 equal columns (responsive)
    ui.columns(4, |cols| {
        widgets::stat_card(
            &mut cols[0],
            &format!("{:.0}%", stats.cpu_usage_total),
            "CPU",
            &format!("{} 核 / {} 线程", stats.cpu_cores, stats.cpu_threads),
        );
        widgets::stat_card(
            &mut cols[1],
            &format!("{:.1}G", stats.ram_used_gb),
            "内存",
            &format!("{:.0}% 已用", stats.ram_usage_percent),
        );
        widgets::stat_card(
            &mut cols[2],
            &if stats.gpu_available {
                format!("{:.0}%", stats.gpu_usage)
            } else {
                "—".to_string()
            },
            "显卡",
            &if stats.gpu_temp > 0.0 {
                format!("{:.0}°C", stats.gpu_temp)
            } else {
                "暂无读数".to_string()
            },
        );
        widgets::stat_card(
            &mut cols[3],
            &format!("{}", stats.process_count),
            "进程",
            "运行中",
        );
    });

    ui.add_space(14.0);

    // Two-column main area - using columns for proper vertical layout
    ui.columns(2, |cols| {
        // LEFT: CPU per-core
        theme::card_frame().show(&mut cols[0], |ui| {
            ui.label(
                egui::RichText::new("CPU 各核心使用率")
                    .size(14.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            if stats.cpu_usage_per_core.is_empty() {
                ui.label(
                    egui::RichText::new("正在采集...")
                        .size(12.0)
                        .color(theme::TEXT_DIM),
                );
            }
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .id_salt("cpu_cores")
                .show(ui, |ui| {
                    for (i, usage) in stats.cpu_usage_per_core.iter().enumerate() {
                        widgets::gauge_row(ui, &format!("线程 {}", i + 1), *usage);
                        ui.add_space(5.0);
                    }
                });
        });

        // RIGHT: Memory/GPU + Boost
        let right = &mut cols[1];

        theme::card_frame().show(right, |ui| {
            ui.label(
                egui::RichText::new("内存 & GPU")
                    .size(14.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            widgets::gauge_row(ui, "内存", stats.ram_usage_percent);
            ui.label(
                egui::RichText::new(format!(
                    "{:.1} / {:.1} GB",
                    stats.ram_used_gb, stats.ram_total_gb
                ))
                .size(11.0)
                .color(theme::TEXT_DIM),
            );
            ui.add_space(8.0);

            if stats.gpu_available {
                let gpu_mem_pct =
                    stats.gpu_mem_used_mb as f32 / stats.gpu_mem_total_mb as f32 * 100.0;
                widgets::gauge_row(ui, "显存", gpu_mem_pct);
                ui.label(
                    egui::RichText::new(format!(
                        "{} / {} MB",
                        stats.gpu_mem_used_mb, stats.gpu_mem_total_mb
                    ))
                    .size(11.0)
                    .color(theme::TEXT_DIM),
                );
                ui.add_space(8.0);
                widgets::gauge_row(ui, "GPU", stats.gpu_usage);
            } else {
                ui.label(
                    egui::RichText::new("暂无 GPU 监控读数")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
            }
        });

        right.add_space(12.0);

        // Boost card
        theme::card_frame().show(right, |ui| {
            ui.label(
                egui::RichText::new("一键加速")
                    .size(14.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            ui.vertical_centered_justified(|ui| {
                let label = if app.is_boosting {
                    "⏳  加 速 中…"
                } else if app.is_restoring {
                    "⏳  恢 复 中…"
                } else if app.session_active() {
                    "✓  加 速 已 开 启"
                } else {
                    "🚀  启 动 加 速"
                };
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::from_rgb(8, 10, 14)),
                )
                .fill(if app.is_boosting {
                    theme::TEXT_DIM
                } else {
                    theme::ACCENT
                })
                .rounding(egui::Rounding::same(8.0))
                .min_size(egui::vec2(0.0, 42.0));

                // Disable the button while a boost is running in the background.
                let clicked = ui
                    .add_enabled(
                        !app.is_boosting
                            && !app.is_restoring
                            && !app.action_busy
                            && !app.session_active(),
                        btn,
                    )
                    .clicked();

                if clicked {
                    app.start_boost();
                }
                if app.session_active()
                    && ui
                        .add_enabled(
                            !app.is_restoring && !app.action_busy,
                            theme::secondary_button("停止并恢复原设置"),
                        )
                        .clicked()
                {
                    app.restore_boost();
                }
            });

            ui.add_space(8.0);

            if app.is_boosting {
                ui.label(
                    egui::RichText::new("正在应用所选设置，请稍候…")
                        .size(11.0)
                        .color(theme::TEXT_SECONDARY),
                );
            } else if let Some(ref result) = app.boost_result {
                let summary = if result.errors.is_empty() {
                    "本次操作完成"
                } else {
                    "部分操作未完成"
                };
                ui.label(egui::RichText::new(summary).size(12.0).color(
                    if result.errors.is_empty() {
                        theme::TEXT_SECONDARY
                    } else {
                        theme::WARNING
                    },
                ));
                if result.processes_killed > 0 {
                    ui.label(format!(
                        "关闭 {} 个已选择的后台进程",
                        result.processes_killed
                    ));
                }
                for message in &result.messages {
                    ui.label(
                        egui::RichText::new(message)
                            .size(10.5)
                            .color(theme::TEXT_SECONDARY),
                    );
                }
                if let Some(ref t) = app.last_boost_time {
                    ui.label(
                        egui::RichText::new(format!("上次加速: {}", t))
                            .size(10.0)
                            .color(theme::TEXT_DIM),
                    );
                }
                for err in &result.errors {
                    ui.label(
                        egui::RichText::new(format!("⚠ {}", err))
                            .size(10.0)
                            .color(theme::WARNING),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new("默认调整电源和游戏模式；保留游戏、反作弊与语音进程。")
                        .size(11.0)
                        .color(theme::TEXT_SECONDARY),
                );
            }
        });
    });
}
