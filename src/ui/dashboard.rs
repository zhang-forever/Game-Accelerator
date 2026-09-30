use super::{
    icons::{self, Icon},
    theme, widgets,
};
use crate::app::{GameAcceleratorApp, Page};

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    widgets::page_header(
        ui,
        "性能概览",
        "为下一场对局做好准备，随时查看电脑的资源状态。",
    );
    egui::ScrollArea::vertical().id_salt("overview_content").auto_shrink([false,false]).show(ui, |ui| {
        session_card(app,ui);
        ui.add_space(18.0);
        let stats=app.stats.lock().clone();
        ui.columns(4, |cols| {
            widgets::metric_card(&mut cols[0],"CPU", &format!("{:.0}%",stats.cpu_usage_total),&format!("{} 核心 · {} 线程",stats.cpu_cores,stats.cpu_threads),Some(stats.cpu_usage_total),Icon::Memory);
            widgets::metric_card(&mut cols[1],"内存",&format!("{:.1} GB",stats.ram_used_gb),&format!("共 {:.0} GB · {:.0}% 已用",stats.ram_total_gb,stats.ram_usage_percent),Some(stats.ram_usage_percent),Icon::Memory);
            widgets::metric_card(&mut cols[2],"显卡",&if stats.gpu_available{format!("{:.0}%",stats.gpu_usage)}else{"—".to_string()},&if stats.gpu_available && stats.gpu_temp > 0.0{format!("{:.0}°C · NVIDIA",stats.gpu_temp)}else{"暂无温度读数".to_string()},stats.gpu_available.then_some(stats.gpu_usage),Icon::Gpu);
            widgets::metric_card(&mut cols[3],"运行进程",&stats.process_count.to_string(),"可在进程管理中查看",None,Icon::Processes);
        });
        ui.add_space(18.0);
        ui.columns(2, |cols| {
            let width=cols[0].available_width();
            theme::card_frame().show(&mut cols[0], |ui| {
                ui.set_min_width((width-36.0).max(10.0));
                widgets::section_header(ui,"CPU 线程负载");
                ui.label(egui::RichText::new(&stats.cpu_name).size(11.5).color(theme::TEXT_DIM));
                ui.add_space(10.0);
                if stats.cpu_usage_per_core.is_empty(){ui.label("正在采集资源信息…");}
                egui::ScrollArea::vertical().id_salt("thread_loads").max_height(205.0).show(ui, |ui| {
                    for (row, usages) in stats.cpu_usage_per_core.chunks(2).enumerate() {
                        ui.columns(2, |columns| {
                            for (column, percent) in usages.iter().enumerate() {
                                let ui = &mut columns[column];
                                let width = ui.available_width();
                                egui::Frame::none().fill(theme::SURFACE).rounding(egui::Rounding::same(8.0))
                                    .inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
                                        ui.set_min_width((width - 16.0).max(30.0));
                                        ui.spacing_mut().item_spacing.y = 4.0;
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("线程 {:02}", row * 2 + column + 1)).size(11.0).color(theme::TEXT_SECONDARY));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(egui::RichText::new(format!("{percent:.0}%")).size(12.0).strong().color(theme::gauge_color(*percent)));
                                            });
                                        });
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 3.0), egui::Sense::hover());
                                        ui.painter().rect_filled(rect, egui::Rounding::same(2.0), theme::CARD_BG);
                                        ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * (*percent / 100.0).clamp(0.0, 1.0), 3.0)), egui::Rounding::same(2.0), theme::gauge_color(*percent));
                                    });
                            }
                        });
                        ui.add_space(4.0);
                    }
                });
            });
            let width=cols[1].available_width();
            theme::card_frame().show(&mut cols[1], |ui| {
                ui.set_min_width((width-36.0).max(10.0));
                widgets::section_header(ui,"内存与显卡");
                ui.label(egui::RichText::new(if stats.gpu_name.is_empty(){"等待显卡信息"}else{&stats.gpu_name}).size(11.5).color(theme::TEXT_DIM));
                ui.add_space(12.0);
                widgets::gauge_row(ui,"内存",stats.ram_usage_percent);
                ui.label(egui::RichText::new(format!("{:.1} / {:.1} GB",stats.ram_used_gb,stats.ram_total_gb)).size(11.0).color(theme::TEXT_DIM));
                ui.add_space(10.0);
                if stats.gpu_available {
                    widgets::gauge_row(ui,"显存",stats.gpu_mem_used_mb as f32/stats.gpu_mem_total_mb as f32*100.0);
                    ui.label(egui::RichText::new(format!("{} / {} MB",stats.gpu_mem_used_mb,stats.gpu_mem_total_mb)).size(11.0).color(theme::TEXT_DIM));
                } else {ui.label(egui::RichText::new("当前没有 GPU 监控读数").size(12.0).color(theme::TEXT_SECONDARY));}
                ui.add_space(12.0);
                if ui.add(theme::secondary_button("查看进程管理")).clicked(){app.current_page=Page::Process;}
            });
        });
    });
}

fn session_card(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    let width = ui.available_width();
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(24, 39, 62))
        .rounding(egui::Rounding::same(18.0))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 77, 111)))
        .inner_margin(egui::Margin::same(20.0))
        .show(ui, |ui| {
            ui.set_min_width((width - 40.0).max(10.0));
            ui.spacing_mut().item_spacing.y = 6.0;
            let active = app.session_active();
            let busy =
                app.is_boosting || app.is_restoring || app.action_busy || app.process_refresh_busy;
            ui.horizontal(|ui| {
                icons::show(ui, Icon::Shield, 18.0, theme::ACCENT);
                widgets::status_badge(
                    ui,
                    if active {
                        "会话已开启"
                    } else {
                        "游戏准备"
                    },
                    if active {
                        theme::SUCCESS
                    } else {
                        theme::ACCENT
                    },
                );
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let text_width = (ui.available_width() - 200.0).max(160.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(text_width, 80.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_min_width(text_width);
                        ui.label(
                            egui::RichText::new(if app.is_restoring {
                                "正在恢复原设置"
                            } else if app.is_boosting {
                                "正在准备游戏会话"
                            } else if active {
                                "专注你的下一场对局"
                            } else {
                                "准备好，开始游戏"
                            })
                            .size(23.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new("调整电源和游戏模式，保留游戏、反作弊及语音进程。")
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                        );
                        ui.add_space(8.0);
                        ui.horizontal_wrapped(|ui| {
                            for (name, exe) in [
                                ("无畏契约", "VALORANT-Win64-Shipping.exe"),
                                ("英雄联盟", "League of Legends.exe"),
                                ("穿越火线", "crossfire.exe"),
                            ] {
                                let selected =
                                    app.config.selected_game.as_deref().is_some_and(|game| {
                                        crate::core::process_manager::normalize_exe_name(game)
                                            == crate::core::process_manager::normalize_exe_name(exe)
                                    });
                                let button = egui::Button::new(
                                    egui::RichText::new(name).size(12.0).color(if selected {
                                        theme::ACCENT
                                    } else {
                                        theme::TEXT_SECONDARY
                                    }),
                                )
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
                                .rounding(egui::Rounding::same(7.0))
                                .min_size(egui::vec2(84.0, 30.0));
                                if ui.add_enabled(!busy && !active, button).clicked() {
                                    app.config.select_competitive_game(exe);
                                    app.save_config();
                                }
                            }
                        });
                    },
                );
                ui.vertical(|ui| {
                    ui.set_min_width(180.0);
                    let title = if app.is_restoring {
                        "恢复中…"
                    } else if app.is_boosting {
                        "准备中…"
                    } else if active {
                        "加速已开启"
                    } else {
                        "启动加速"
                    };
                    if ui
                        .add_enabled(
                            !busy && !active,
                            theme::primary_button(title).min_size(egui::vec2(180.0, 44.0)),
                        )
                        .clicked()
                    {
                        app.start_boost();
                    }
                    if active {
                        if ui
                            .add_enabled(
                                !busy,
                                theme::secondary_button("停止并恢复原设置")
                                    .min_size(egui::vec2(180.0, 36.0)),
                            )
                            .clicked()
                        {
                            app.restore_boost();
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("游戏结束后停止并恢复")
                                .size(11.0)
                                .color(theme::TEXT_DIM),
                        );
                    }
                });
            });
            if let Some(result) = &app.boost_result {
                ui.add_space(8.0);
                for message in &result.messages {
                    ui.label(
                        egui::RichText::new(message)
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    );
                }
                for error in &result.errors {
                    ui.label(egui::RichText::new(error).size(12.0).color(theme::WARNING));
                }
                if let Some(time) = &app.last_boost_time {
                    ui.label(
                        egui::RichText::new(format!("上次操作 · {time}"))
                            .size(11.0)
                            .color(theme::TEXT_DIM),
                    );
                }
            }
        });
}
