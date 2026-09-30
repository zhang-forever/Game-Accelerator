use super::{theme, widgets};
use crate::app::{ActionTarget, GameAcceleratorApp};
use crate::core::gpu_manager;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("GPU 设置")
            .size(22.0)
            .strong()
            .color(theme::TEXT_PRIMARY),
    );
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("查看 NVIDIA 实时读数，或为指定游戏设置 Windows 显卡偏好。")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(14.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_header(ui, "GPU 信息");
            let snap = app.stats.lock().clone();
            if !snap.gpu_available {
                ui.label(egui::RichText::new("暂无 NVIDIA 监控读数。AMD / Intel 或缺少 nvidia-smi 时，CPU 与内存监控仍可使用。")
                    .size(12.0).color(theme::TEXT_SECONDARY));
            } else {
                ui.label(egui::RichText::new(&snap.gpu_name).size(15.0).strong().color(theme::ACCENT));
                ui.add_space(6.0);
                widgets::gauge_row(ui, "使用", snap.gpu_usage);
                if snap.gpu_mem_total_mb > 0 {
                    widgets::gauge_row(ui, "显存", snap.gpu_mem_used_mb as f32 / snap.gpu_mem_total_mb as f32 * 100.0);
                    ui.label(format!("{} / {} MB", snap.gpu_mem_used_mb, snap.gpu_mem_total_mb));
                }
                if snap.gpu_temp > 0.0 { ui.label(format!("温度 {}°C", snap.gpu_temp)); }
                ui.label(format!("驱动 {}", snap.gpu_driver));
            }
        });
        ui.add_space(10.0);

        if let Some(message) = &app.gpu_status {
            ui.label(egui::RichText::new(message).color(if message.starts_with('✓') {
                theme::SUCCESS
            } else { theme::WARNING }));
        }
        if app.action_busy { ui.label("正在应用设置，请稍候…"); }
        ui.add_space(10.0);

        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_header(ui, "游戏显卡偏好");
            ui.label(egui::RichText::new("填写实际存在的游戏 EXE 完整路径，优先使用 Windows 识别的高性能显卡。")
                .size(11.0).color(theme::TEXT_SECONDARY));
            ui.add_space(8.0);
            let mut game = app.config.selected_game.clone().unwrap_or_default();
            if ui.add(egui::TextEdit::singleline(&mut game)
                .hint_text("例如 C:\\Games\\game.exe").desired_width(ui.available_width())).changed() {
                app.config.selected_game = if game.trim().is_empty() { None } else { Some(game) };
            }
            ui.add_space(6.0);
            ui.label(egui::RichText::new("游戏预设中的文件名用于识别进程；显卡偏好必须改填完整路径，设置后重新启动游戏。")
                .size(10.5).color(theme::TEXT_DIM));
            ui.add_space(8.0);
            if ui.add_enabled(!app.action_busy && !app.is_boosting && !app.is_restoring,
                theme::primary_button("设为高性能显卡")).clicked() {
                let game = app.config.selected_game.clone().unwrap_or_default();
                app.save_config();
                app.start_action(ActionTarget::Gpu, move || gpu_manager::force_discrete_gpu_for_game(&game));
            }
        });
        ui.add_space(10.0);
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_header(ui, "驱动设置");
            ui.label(egui::RichText::new("NVIDIA 电源管理模式请在 NVIDIA 控制面板的「管理 3D 设置 → 程序设置」中按游戏调整。")
                .size(12.0).color(theme::TEXT_SECONDARY));
            ui.label(egui::RichText::new("驱动选项和实际支持取决于显卡与驱动版本；本工具不写入未验证的驱动注册表项。")
                .size(11.0).color(theme::TEXT_DIM));
        });
    });
}
