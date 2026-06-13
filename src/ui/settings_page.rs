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

            checkbox_with_desc(
                ui,
                &mut app.config.kill_background_processes,
                "关闭后台进程",
                "自动关闭非必要的后台应用，释放系统资源",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.clean_memory,
                "清理内存",
                "清理系统缓存和待机内存，提供更多可用 RAM",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.enable_high_perf_power,
                "切换高性能电源计划",
                "让 CPU 全速运行，不降频省电（笔记本插电推荐）",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.set_high_priority,
                "提升游戏进程优先级",
                "让系统优先分配资源给游戏进程",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.enable_game_mode,
                "启用 Windows Game Mode",
                "使用 Windows 内置的游戏模式优化",
            );
        });

        ui.add_space(12.0);

        // Application behavior section
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

            checkbox_with_desc(
                ui,
                &mut app.config.auto_boost,
                "启动时自动加速",
                "打开应用后自动执行游戏加速优化",
            );
            ui.add_space(8.0);

            checkbox_with_desc(
                ui,
                &mut app.config.minimize_to_tray,
                "最小化到系统托盘",
                "点击最小化时隐藏到托盘而不是任务栏",
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
                app.config.save();
            }

            ui.add_space(8.0);

            if ui.add(theme::secondary_button("🔄 恢复默认")).clicked() {
                app.config = crate::config::AppConfig::default();
                app.config.save();
            }
        });

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
                egui::RichText::new("Game Accelerator v1.0.0")
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
