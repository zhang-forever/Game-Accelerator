use super::{theme, widgets};
use crate::app::{ActionTarget, GameAcceleratorApp};
use crate::core::gpu_manager;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    widgets::page_header(
        ui,
        "GPU 设置",
        "查看显卡实时读数，为游戏选择 Windows 高性能显卡。",
    );
    let snap = app.stats.lock().clone();

    egui::ScrollArea::vertical()
        .id_salt("gpu_preferences")
        .show(ui, |ui| {
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "显卡概览");

                if !snap.gpu_available {
                    ui.label(
                        egui::RichText::new("暂无 NVIDIA 监控读数")
                            .size(18.0)
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(
                            "AMD / Intel 显卡或缺少 nvidia-smi 时，无法显示这里的实时数据。CPU 与内存监控仍可使用。",
                        )
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                    );
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(if snap.gpu_name.is_empty() {
                                "显卡名称未返回"
                            } else {
                                &snap.gpu_name
                            })
                            .size(18.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                        );
                        widgets::status_badge(ui, "监控中", theme::SUCCESS);
                    });
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "驱动版本  {}",
                            if snap.gpu_driver.is_empty() {
                                "暂无信息"
                            } else {
                                &snap.gpu_driver
                            }
                        ))
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                    );
                    ui.add_space(18.0);

                    let usage = format!("{:.0}%", snap.gpu_usage);
                    let temperature = if snap.gpu_temp > 0.0 {
                        format!("{:.0} °C", snap.gpu_temp)
                    } else {
                        "暂无读数".to_string()
                    };
                    ui.columns(2, |columns| {
                        gpu_metric(&mut columns[0], &usage, "GPU 使用率");
                        gpu_metric(&mut columns[1], &temperature, "GPU 温度");
                    });

                    ui.add_space(16.0);
                    widgets::gauge_row(ui, "负载", snap.gpu_usage);
                    if snap.gpu_mem_total_mb > 0 {
                        ui.add_space(10.0);
                        widgets::gauge_row(
                            ui,
                            "显存",
                            snap.gpu_mem_used_mb as f32 / snap.gpu_mem_total_mb as f32 * 100.0,
                        );
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "已使用 {} MB  /  共 {} MB",
                                snap.gpu_mem_used_mb, snap.gpu_mem_total_mb
                            ))
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                        );
                    }
                }
            });

            ui.add_space(14.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "游戏显卡偏好");
                ui.label(
                    egui::RichText::new(
                        "优先使用 Windows 识别的高性能显卡。请填写实际存在的游戏 EXE 完整路径。",
                    )
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(14.0);
                ui.label(
                    egui::RichText::new("游戏 EXE 完整路径")
                        .size(14.0)
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(8.0);

                let mut game = app.config.selected_game.clone().unwrap_or_default();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut game)
                            .hint_text("例如 E:\\Games\\game.exe")
                            .desired_width(ui.available_width()),
                    )
                    .changed()
                {
                    app.config.selected_game = if game.trim().is_empty() {
                        None
                    } else {
                        Some(game)
                    };
                }
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "预设中的文件名只用于识别进程。设置显卡偏好需要完整路径，并在设置后重新启动游戏。",
                    )
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(16.0);

                if ui
                    .add_enabled(
                        !app.action_busy && !app.is_boosting && !app.is_restoring,
                        theme::primary_button("设为高性能显卡"),
                    )
                    .clicked()
                {
                    let game = app.config.selected_game.clone().unwrap_or_default();
                    app.save_config();
                    app.start_action(ActionTarget::Gpu, move || {
                        gpu_manager::force_discrete_gpu_for_game(&game)
                    });
                }

                if app.action_busy {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("正在应用设置，请稍候…")
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                        );
                    });
                }
                if let Some(message) = &app.gpu_status {
                    ui.add_space(12.0);
                    widgets::notice(
                        ui,
                        message,
                        if message.starts_with('✓') {
                            theme::SUCCESS
                        } else {
                            theme::WARNING
                        },
                    );
                }
            });

            ui.add_space(14.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                widgets::section_header(ui, "驱动偏好");
                ui.label(
                    egui::RichText::new("NVIDIA 控制面板")
                        .size(14.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("管理 3D 设置  →  程序设置  →  选择游戏")
                        .size(12.0)
                        .color(theme::ACCENT),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "可在这里按游戏调整电源管理模式。可用选项与支持情况取决于显卡和驱动版本。",
                    )
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
                );
            });
            ui.add_space(4.0);
        });
}

fn gpu_metric(ui: &mut egui::Ui, value: &str, label: &str) {
    egui::Frame::none()
        .fill(theme::SURFACE)
        .rounding(egui::Rounding::same(12.0))
        .inner_margin(egui::Margin::same(14.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new(value)
                    .size(26.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(label)
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            );
        });
}
