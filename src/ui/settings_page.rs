use super::theme;
use crate::app::GameAcceleratorApp;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("设置")
            .size(22.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("配置游戏加速行为和应用偏好")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(16.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Boost behavior section
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("🎮 加速行为")
                    .size(15.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("选择启动游戏加速时要执行的优化操作")
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                for (label, exe) in [
                    ("无畏契约", "VALORANT-Win64-Shipping.exe"),
                    ("英雄联盟", "League of Legends.exe"),
                    ("穿越火线", "crossfire.exe"),
                ] {
                    if ui.add(theme::secondary_button(label)).clicked() {
                        app.config.select_competitive_game(exe);
                        app.save_config();
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "竞技游戏预设：保留游戏、反作弊、语音和同步进程，不裁剪内存或提升优先级。",
                )
                .size(11.0)
                .color(theme::TEXT_SECONDARY),
            );
            ui.add_space(12.0);

            checkbox_with_desc(
                ui,
                &mut app.config.kill_background_processes,
                "关闭后台进程",
                "仅处理明确加入黑名单的程序；关闭前保存工作，默认不启用",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.enable_high_perf_power,
                "切换高性能电源计划",
                "切换可用的高性能计划；停止加速或正常退出时恢复原计划",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.enable_game_mode,
                "启用 Windows Game Mode",
                "启用系统内置游戏模式；停止加速或正常退出时恢复原值",
            );
        });

        ui.add_space(12.0);

        // Only expose application behavior that is actually implemented.
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("⚙️ 应用行为")
                    .size(15.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("配置应用程序的启动和运行方式")
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(12.0);

            ui.label(
                egui::RichText::new(
                    "启动只进行监控。点击启动加速才执行设置；游戏结束后点击停止并恢复。",
                )
                .size(12.0)
                .color(theme::TEXT_SECONDARY),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!(
                    "配置位置：{}",
                    crate::config::AppConfig::config_path().display()
                ))
                .size(10.5)
                .color(theme::TEXT_DIM),
            );
        });

        ui.add_space(12.0);

        // Game path section
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("🎯 游戏路径")
                    .size(15.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("指定要优化的游戏可执行文件路径")
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(12.0);

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("游戏 EXE 路径")
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
                ui.add_space(4.0);

                let mut game = app.config.selected_game.clone().unwrap_or_default();
                let text_edit = egui::TextEdit::singleline(&mut game)
                    .hint_text("例如: game.exe 或 C:\\Games\\game.exe")
                    .desired_width(ui.available_width());

                if ui.add(text_edit).changed() {
                    app.config.selected_game = if game.is_empty() { None } else { Some(game) };
                }

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("💡 提示: 可以只填写文件名，也可以填写完整路径")
                        .size(10.0)
                        .color(theme::TEXT_DIM),
                );
            });
        });

        ui.add_space(16.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(theme::primary_button("💾 保存设置")).clicked() {
                app.save_config();
            }

            ui.add_space(8.0);

            if ui.add(theme::secondary_button("🔄 恢复默认")).clicked() {
                app.config = crate::config::AppConfig::default();
                app.save_config();
            }
        });

        if let Some(status) = &app.settings_status {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(status).color(if status.starts_with('✓') {
                    theme::SUCCESS
                } else {
                    theme::WARNING
                }),
            );
        }

        ui.add_space(16.0);

        // About section
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(
                egui::RichText::new("ℹ️ 关于")
                    .size(15.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(concat!("Game Accelerator v", env!("CARGO_PKG_VERSION")))
                    .size(13.0)
                    .strong()
                    .color(theme::TEXT_SECONDARY),
            );
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new("轻量游戏加速工具 - Rust + egui")
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠️").size(12.0));
                ui.label(
                    egui::RichText::new("部分功能需要管理员权限才能生效")
                        .size(10.0)
                        .color(theme::WARNING),
                );
            });
        });
    });
}

/// Checkbox with title and description
fn checkbox_with_desc(ui: &mut egui::Ui, checked: &mut bool, title: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.checkbox(checked, "");
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(13.0)
                    .strong()
                    .color(theme::TEXT_PRIMARY),
            );
            ui.label(egui::RichText::new(desc).size(10.5).color(theme::TEXT_DIM));
        });
    });
}
