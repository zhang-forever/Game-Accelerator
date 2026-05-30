use super::{theme, widgets};
use crate::app::GameAcceleratorApp;
use crate::core::gpu_manager;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("GPU 设置")
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
            "GPU 性能优化和显卡设置\n\n💡 功能说明：\n• 最大性能模式：锁定显卡高频运行\n• 关闭遥测：停止 NVIDIA 后台数据收集\n• 独显运行：强制游戏使用独立显卡\n\n⚠️ 注意：\n• 仅支持 NVIDIA 显卡\n• 需要安装 nvidia-smi 工具\n• 部分设置需要重启生效"
        );
    });
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("优化显卡性能，确保游戏使用独立显卡运行")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(14.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // ===== GPU Info =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_header(ui, "GPU 信息");

            // Read cached GPU stats from the background monitor instead of running
            // nvidia-smi on every repaint (that was the main source of UI lag).
            let snap = app.stats.lock().clone();

            if snap.gpu_name.is_empty() {
                ui.label(
                    egui::RichText::new("未检测到 NVIDIA 显卡（需要 nvidia-smi 可用）")
                        .size(12.0)
                        .color(theme::WARNING),
                );
            } else {
                ui.label(
                    egui::RichText::new(&snap.gpu_name)
                        .size(15.0)
                        .strong()
                        .color(theme::ACCENT),
                );
                ui.add_space(6.0);

                widgets::gauge_row(ui, "使用", snap.gpu_usage);
                ui.add_space(4.0);

                let vram_pct = if snap.gpu_mem_total_mb > 0 {
                    snap.gpu_mem_used_mb as f32 / snap.gpu_mem_total_mb as f32 * 100.0
                } else {
                    0.0
                };
                widgets::gauge_row(ui, "显存", vram_pct);
                ui.label(
                    egui::RichText::new(format!(
                        "{} / {} MB",
                        snap.gpu_mem_used_mb, snap.gpu_mem_total_mb
                    ))
                    .size(11.0)
                    .color(theme::TEXT_DIM),
                );
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    if snap.gpu_temp > 0.0 {
                        ui.label(
                            egui::RichText::new(format!("温度 {}°C", snap.gpu_temp))
                                .size(12.0)
                                .strong()
                                .color(theme::gauge_color(snap.gpu_temp)),
                        );
                        ui.add_space(12.0);
                    }
                    ui.label(
                        egui::RichText::new(format!("驱动 {}", snap.gpu_driver))
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                });
            }
        });

        ui.add_space(10.0);

        // ===== Status feedback banner =====
        if let Some(ref msg) = app.gpu_status {
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

        // ===== Optimization actions =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_header(ui, "性能优化");

            // 1. Max performance mode
            ui.horizontal(|ui| {
                if action_button(ui, "开启最大性能模式").clicked() {
                    app.gpu_status = Some(string_result(gpu_manager::set_nvidia_max_performance()));
                }
                ui.label(
                    egui::RichText::new("锁定显卡高频运行，重启后生效")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
            });
            ui.add_space(8.0);

            // 2. Disable telemetry
            ui.horizontal(|ui| {
                if action_button(ui, "关闭 NVIDIA 后台遥测").clicked() {
                    app.gpu_status = Some(string_result(gpu_manager::disable_nvidia_telemetry()));
                }
                ui.label(
                    egui::RichText::new("停掉 NVIDIA 的后台上报任务，省一点资源")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
            });
        });

        ui.add_space(10.0);

        // ===== Force discrete GPU =====
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_header(ui, "强制游戏使用独立显卡");

            ui.label(
                egui::RichText::new(
                    "笔记本默认可能用核显跑游戏。填入游戏名，强制它用更强的 GTX 1650。",
                )
                .size(11.0)
                .color(theme::TEXT_SECONDARY),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("游戏程序")
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
                let mut game_exe = app.config.selected_game.clone().unwrap_or_default();
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut game_exe)
                        .hint_text("例如 game.exe 或完整路径")
                        .desired_width(ui.available_width() - 120.0),
                );
                if resp.changed() {
                    app.config.selected_game = if game_exe.is_empty() {
                        None
                    } else {
                        Some(game_exe)
                    };
                }
            });
            ui.add_space(8.0);

            if action_button(ui, "设为独显运行").clicked() {
                let game = app.config.selected_game.clone().unwrap_or_default();
                app.gpu_status = Some(string_result(gpu_manager::force_discrete_gpu_for_game(
                    &game,
                )));
                app.config.save();
            }
        });
    });
}

/// Convert a Result into a single display string.
fn string_result(r: Result<String, String>) -> String {
    match r {
        Ok(s) => s,
        Err(e) => format!("⚠ {}", e),
    }
}

/// A consistent accent-outlined action button.
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
        .min_size(egui::vec2(150.0, 32.0)),
    )
}
