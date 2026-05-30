use crate::app::{GameAcceleratorApp, ProcessInfo, ProcessSort};
use crate::core::process_category::{self, Category};
use crate::core::process_manager;
use super::{theme, widgets};

/// Memory threshold (MB) below which a process is considered "small" and can be
/// hidden from the advanced list. Tiny processes add noise without being worth
/// closing for gaming, so they are filtered out by default.
const SMALL_PROCESS_MB: u64 = 50;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    // Header with title + mode toggle
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("进程管理")
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
            "管理系统运行的进程\n\n💡 使用方法：\n• 简单模式：按类别批量关闭进程\n• 高级模式：查看所有进程详情\n• 绿色边框 = 推荐游戏时关闭\n\n⚠️ 注意：\n• 关闭系统进程可能导致不稳定\n• 建议只关闭推荐的类别\n• 关闭前确保保存工作"
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Mode toggle pill
            let label = if app.process_advanced {
                "切换到简单模式"
            } else {
                "高级模式"
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(label)
                            .size(12.0)
                            .color(theme::TEXT_SECONDARY),
                    )
                    .fill(egui::Color32::from_rgb(32, 34, 48))
                    .rounding(egui::Rounding::same(6.0)),
                )
                .clicked()
            {
                app.process_advanced = !app.process_advanced;
            }

            if ui
                .add(
                    egui::Button::new(egui::RichText::new("🔄 刷新").size(12.0))
                        .rounding(egui::Rounding::same(6.0)),
                )
                .clicked()
            {
                refresh_process_list(app);
            }
        });
    });

    ui.add_space(6.0);

    // Auto-load on first visit
    if app.process_list.is_empty() {
        refresh_process_list(app);
    }

    // Status message
    if let Some(ref msg) = app.process_status {
        let color = if msg.starts_with("✓") {
            theme::SUCCESS
        } else {
            theme::WARNING
        };
        ui.label(egui::RichText::new(msg).size(13.0).color(color));
        ui.add_space(4.0);
    }

    if app.process_advanced {
        show_advanced(app, ui);
    } else {
        show_categories(app, ui);
    }
}

// ============ SIMPLE (CATEGORY) VIEW ============

fn show_categories(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("按用途分类，点「关闭这类」一键清理。绿色推荐项玩游戏时关掉最划算。")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(10.0);

    let groups = process_category::group_processes(&app.process_list);
    let mut close_request: Option<(Category, Vec<String>)> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for group in &groups {
            let cat = group.category;
            let is_system = cat == Category::System;

            // Card border color: highlight recommended categories
            let border = if cat.recommended_for_gaming() {
                theme::ACCENT
            } else {
                theme::CARD_BORDER
            };

            egui::Frame::none()
                .fill(theme::CARD_BG)
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(14.0))
                .stroke(egui::Stroke::new(1.0, border))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        // Icon
                        ui.label(egui::RichText::new(cat.icon()).size(26.0));
                        ui.add_space(8.0);

                        // Name + description + stats
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(cat.display_name())
                                        .size(15.0)
                                        .strong()
                                        .color(theme::TEXT_PRIMARY),
                                );
                                if cat.recommended_for_gaming() {
                                    ui.add_space(4.0);
                                    widgets::status_badge(ui, "建议关闭", theme::ACCENT);
                                }
                            });
                            ui.label(
                                egui::RichText::new(cat.description())
                                    .size(11.0)
                                    .color(theme::TEXT_DIM),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} 个程序  ·  占用 {} MB 内存",
                                    group.process_count, group.total_memory_mb
                                ))
                                .size(12.0)
                                .color(theme::TEXT_SECONDARY),
                            );
                        });

                        // Right-aligned action button
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if is_system {
                                    ui.add_enabled(
                                        false,
                                        egui::Button::new(
                                            egui::RichText::new("不可关闭")
                                                .size(12.0)
                                                .color(theme::TEXT_DIM),
                                        ),
                                    );
                                } else {
                                    let (bg, fg) = if cat.safe_to_close() {
                                        (theme::ACCENT, egui::Color32::from_rgb(8, 10, 14))
                                    } else {
                                        (
                                            egui::Color32::from_rgb(60, 40, 40),
                                            theme::WARNING,
                                        )
                                    };
                                    let btn = egui::Button::new(
                                        egui::RichText::new("关闭这类").size(13.0).strong().color(fg),
                                    )
                                    .fill(bg)
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(90.0, 32.0));
                                    if ui.add(btn).clicked() {
                                        close_request =
                                            Some((cat, group.process_names.clone()));
                                    }
                                }
                            },
                        );
                    });
                });
            ui.add_space(6.0);
        }
    });

    if let Some((cat, names)) = close_request {
        let (killed, freed) = process_category::close_category(&names);
        app.process_status = Some(format!(
            "✓ 已关闭「{}」{} 个程序，释放 {} MB 内存",
            cat.display_name(),
            killed,
            freed
        ));
        refresh_process_list(app);
    }
}

// ============ ADVANCED (FULL LIST) VIEW ============

fn show_advanced(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    // Search + sort
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("🔍").color(theme::TEXT_SECONDARY));
        ui.add(
            egui::TextEdit::singleline(&mut app.process_filter)
                .hint_text("搜索进程名")
                .desired_width(160.0),
        );
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("排序:")
                .size(12.0)
                .color(theme::TEXT_SECONDARY),
        );
        sort_button(ui, app, ProcessSort::MemoryDesc, "内存 ↓");
        sort_button(ui, app, ProcessSort::CpuDesc, "CPU ↓");
        sort_button(ui, app, ProcessSort::NameAsc, "名称 A-Z");

        ui.add_space(10.0);
        let hide_label = format!("隐藏小进程 (<{}MB)", SMALL_PROCESS_MB);
        let (bg, fg) = if app.process_hide_small {
            (theme::ACCENT, egui::Color32::from_rgb(8, 10, 14))
        } else {
            (egui::Color32::from_rgb(32, 34, 48), theme::TEXT_SECONDARY)
        };
        let hide_btn = egui::Button::new(egui::RichText::new(hide_label).size(12.0).color(fg))
            .fill(bg)
            .rounding(egui::Rounding::same(5.0));
        if ui
            .add(hide_btn)
            .on_hover_text("小于 50MB 的进程占用资源极少，关掉收益不大。\n隐藏它们让你专注于真正的内存大户。")
            .clicked()
        {
            app.process_hide_small = !app.process_hide_small;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let shown = app
                .process_list
                .iter()
                .filter(|p| !app.process_hide_small || p.memory_mb >= SMALL_PROCESS_MB)
                .count();
            let total = app.process_list.len();
            let text = if app.process_hide_small && shown < total {
                format!("显示 {} / {} 个进程", shown, total)
            } else {
                format!("{} 个进程", total)
            };
            ui.label(
                egui::RichText::new(text)
                    .size(12.0)
                    .color(theme::TEXT_DIM),
            );
        });
    });

    ui.add_space(6.0);

    let filter = app.process_filter.to_lowercase();
    sort_processes(&mut app.process_list, app.process_sort);

    let mut kill_request: Option<(u32, String)> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        let total_w = ui.available_width();

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(18, 19, 28))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    header_cell(ui, "进程名", total_w * 0.34);
                    header_cell(ui, "PID", total_w * 0.10);
                    header_cell(ui, "CPU %", total_w * 0.12);
                    header_cell(ui, "内存 MB", total_w * 0.14);
                    header_cell(ui, "状态", total_w * 0.16);
                    header_cell(ui, "操作", total_w * 0.10);
                });
            });

        ui.add_space(3.0);

        let processes: Vec<ProcessInfo> = app
            .process_list
            .iter()
            .filter(|p| filter.is_empty() || p.name.to_lowercase().contains(&filter))
            .filter(|p| !app.process_hide_small || p.memory_mb >= SMALL_PROCESS_MB)
            .take(200)
            .cloned()
            .collect();

        for proc in &processes {
            let row_bg = if proc.is_protected {
                egui::Color32::from_rgb(26, 28, 40)
            } else {
                theme::CARD_BG
            };

            egui::Frame::none()
                .fill(row_bg)
                .rounding(egui::Rounding::same(5.0))
                .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        sized_cell(ui, total_w * 0.34, |ui| {
                            ui.label(
                                egui::RichText::new(&proc.name)
                                    .size(12.0)
                                    .color(theme::TEXT_PRIMARY),
                            );
                        });
                        sized_cell(ui, total_w * 0.10, |ui| {
                            ui.label(
                                egui::RichText::new(proc.pid.to_string())
                                    .size(12.0)
                                    .color(theme::TEXT_SECONDARY),
                            );
                        });
                        sized_cell(ui, total_w * 0.12, |ui| {
                            ui.label(
                                egui::RichText::new(format!("{:.1}", proc.cpu_usage))
                                    .size(12.0)
                                    .color(if proc.cpu_usage > 30.0 {
                                        theme::WARNING
                                    } else {
                                        theme::TEXT_SECONDARY
                                    }),
                            );
                        });
                        sized_cell(ui, total_w * 0.14, |ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", proc.memory_mb))
                                    .size(12.0)
                                    .color(if proc.memory_mb > 500 {
                                        theme::WARNING
                                    } else {
                                        theme::TEXT_SECONDARY
                                    }),
                            );
                        });
                        sized_cell(ui, total_w * 0.16, |ui| {
                            if proc.is_protected {
                                widgets::status_badge(ui, "系统", theme::TEXT_SECONDARY);
                            }
                        });
                        sized_cell(ui, total_w * 0.10, |ui| {
                            if proc.is_protected {
                                ui.add_enabled(
                                    false,
                                    egui::Button::new(
                                        egui::RichText::new("受保护")
                                            .size(11.0)
                                            .color(theme::TEXT_DIM),
                                    )
                                    .small(),
                                );
                            } else {
                                let kill_btn = egui::Button::new(
                                    egui::RichText::new("结束")
                                        .size(11.0)
                                        .color(theme::DANGER),
                                )
                                .small()
                                .fill(egui::Color32::from_rgb(44, 24, 24));
                                if ui.add(kill_btn).clicked() {
                                    kill_request = Some((proc.pid, proc.name.clone()));
                                }
                            }
                        });
                    });
                });
            ui.add_space(2.0);
        }
    });

    if let Some((pid, name)) = kill_request {
        match process_manager::kill_process_by_pid(pid, &name) {
            Ok(_) => {
                app.process_status = Some(format!("✓ 已结束 {} (PID {})", name, pid));
                refresh_process_list(app);
            }
            Err(e) => {
                app.process_status = Some(format!("⚠ {}", e));
            }
        }
    }
}

// ============ HELPERS ============

fn refresh_process_list(app: &mut GameAcceleratorApp) {
    app.process_list = process_manager::get_process_list()
        .into_iter()
        .map(|p| ProcessInfo {
            is_whitelisted: app.config.whitelist.contains(&p.name),
            is_blacklisted: app.config.blacklist.contains(&p.name),
            is_protected: p.is_protected,
            name: p.name,
            pid: p.pid,
            cpu_usage: p.cpu_usage,
            memory_mb: p.memory_mb,
        })
        .collect();
}

fn sort_processes(list: &mut [ProcessInfo], sort: ProcessSort) {
    match sort {
        ProcessSort::MemoryDesc => list.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb)),
        ProcessSort::CpuDesc => list.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        ProcessSort::NameAsc => {
            list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
    }
}

fn sort_button(ui: &mut egui::Ui, app: &mut GameAcceleratorApp, sort: ProcessSort, label: &str) {
    let selected = app.process_sort == sort;
    let (bg, fg) = if selected {
        (theme::ACCENT, egui::Color32::from_rgb(8, 10, 14))
    } else {
        (egui::Color32::from_rgb(32, 34, 48), theme::TEXT_SECONDARY)
    };
    let btn = egui::Button::new(egui::RichText::new(label).size(12.0).color(fg))
        .fill(bg)
        .rounding(egui::Rounding::same(5.0));
    if ui.add(btn).clicked() {
        app.process_sort = sort;
    }
}

fn header_cell(ui: &mut egui::Ui, text: &str, width: f32) {
    sized_cell(ui, width, |ui| {
        ui.label(
            egui::RichText::new(text)
                .size(12.0)
                .strong()
                .color(theme::TEXT_SECONDARY),
        );
    });
}

fn sized_cell(ui: &mut egui::Ui, width: f32, content: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(width.max(40.0), 18.0),
        egui::Layout::left_to_right(egui::Align::Center),
        content,
    );
}
