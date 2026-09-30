use super::{theme, widgets};
use crate::app::{GameAcceleratorApp, ProcessInfo, ProcessSort};
use crate::core::process_category::{self, Category};

/// Memory threshold below which the advanced list may hide small processes.
const SMALL_PROCESS_MB: u64 = 50;
const PROCESS_LIST_LIMIT: usize = 200;
const TABLE_ROW_HEIGHT: f32 = 40.0;
const TABLE_ROW_PADDING: f32 = 8.0;

pub fn show(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    widgets::page_header(
        ui,
        "进程管理",
        "查看正在运行的程序和资源占用，按需管理后台任务。",
    );

    ui.horizontal(|ui| {
        if ui
            .add(view_button("分类概览", !app.process_advanced))
            .clicked()
        {
            app.process_advanced = false;
        }
        if ui
            .add(view_button("进程列表", app.process_advanced))
            .clicked()
        {
            app.process_advanced = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    !app.process_refresh_busy,
                    theme::secondary_button(if app.process_refresh_busy {
                        "读取中…"
                    } else {
                        "刷新列表"
                    })
                    .min_size(egui::vec2(100.0, 36.0)),
                )
                .clicked()
            {
                refresh_process_list(app);
            }
            widgets::status_badge(ui, "关键进程已保护", theme::ACCENT);
        });
    });
    ui.add_space(18.0);

    // Keep the initial load and refresh on the app's existing background worker.
    if !app.process_list_initialized && !app.process_refresh_busy {
        refresh_process_list(app);
    }
    if app.process_refresh_busy {
        widgets::notice(ui, "正在读取进程列表…", theme::ACCENT);
        ui.add_space(12.0);
    } else if app.process_list_initialized && app.process_list.is_empty() {
        widgets::notice(ui, "暂无进程数据，请点击刷新重试。", theme::TEXT_SECONDARY);
        ui.add_space(12.0);
    }

    if let Some(ref message) = app.process_status {
        let color = if message.starts_with('✓') {
            theme::SUCCESS
        } else {
            theme::WARNING
        };
        widgets::notice(ui, message, color);
        ui.add_space(12.0);
    }

    if app.process_advanced {
        show_advanced(app, ui);
    } else {
        show_categories(app, ui);
    }
    show_close_confirmation(app, ui);
}

fn show_categories(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    let groups = process_category::group_processes(&app.process_list);
    let mut close_request: Option<(Category, Vec<String>)> = None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("按用途分组")
                .size(14.0)
                .strong()
                .color(theme::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} 类程序 · {} 个进程",
                    groups.len(),
                    app.process_list.len()
                ))
                .size(12.0)
                .color(theme::TEXT_DIM),
            );
        });
    });
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("关闭前会显示程序名单，请先保存工作。")
            .size(12.0)
            .color(theme::TEXT_SECONDARY),
    );
    ui.add_space(14.0);

    egui::ScrollArea::vertical()
        .id_salt("process_categories")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for group in &groups {
                let category = group.category;
                let is_system = category == Category::System;

                theme::card_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.x = 12.0;
                    ui.horizontal(|ui| {
                        let details_width = (ui.available_width() - 176.0).max(120.0);
                        category_marker(ui, category);
                        ui.allocate_ui_with_layout(
                            egui::vec2(details_width, 64.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_max_width(details_width);
                                ui.spacing_mut().item_spacing.y = 3.0;
                                ui.label(
                                    egui::RichText::new(category.display_name())
                                        .size(16.0)
                                        .strong()
                                        .color(theme::TEXT_PRIMARY),
                                );
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(category_description(category))
                                            .size(12.0)
                                            .color(theme::TEXT_DIM),
                                    )
                                    .truncate(),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} 个进程   ·   {} MB 内存",
                                        group.process_count, group.total_memory_mb
                                    ))
                                    .size(12.0)
                                    .color(theme::TEXT_SECONDARY),
                                );
                            },
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let button = theme::secondary_button(if is_system {
                                "受保护"
                            } else {
                                "查看并关闭"
                            })
                            .min_size(egui::vec2(108.0, 36.0));
                            if ui
                                .add_enabled(!is_system, button)
                                .on_hover_text(if is_system {
                                    "系统与安全进程会保留。"
                                } else {
                                    "先查看程序名单，再确认结束。"
                                })
                                .clicked()
                            {
                                close_request = Some((category, group.process_names.clone()));
                            }
                        });
                    });
                });
                ui.add_space(12.0);
            }
        });

    if let Some((_category, names)) = close_request {
        app.process_close_request = Some(
            app.process_list
                .iter()
                .filter(|process| !process.is_protected && !process.is_whitelisted)
                .filter(|process| {
                    names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&process.name))
                })
                .cloned()
                .collect(),
        );
    }
}

fn show_advanced(app: &mut GameAcceleratorApp, ui: &mut egui::Ui) {
    let filter = app.process_filter.to_lowercase();
    let matching_count = app
        .process_list
        .iter()
        .filter(|process| filter.is_empty() || process.name.to_lowercase().contains(&filter))
        .filter(|process| !app.process_hide_small || process.memory_mb >= SMALL_PROCESS_MB)
        .count();

    theme::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let search_width = (ui.available_width() - 174.0).max(160.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.process_filter)
                    .font(egui::FontId::proportional(14.0))
                    .hint_text("搜索进程名称…")
                    .desired_width(search_width),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} / {} 个进程",
                        matching_count.min(PROCESS_LIST_LIMIT),
                        app.process_list.len()
                    ))
                    .size(12.0)
                    .color(theme::TEXT_DIM),
                );
            });
        });
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("排序")
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            );
            sort_button(ui, app, ProcessSort::MemoryDesc, "内存 ↓");
            sort_button(ui, app, ProcessSort::CpuDesc, "CPU ↓");
            sort_button(ui, app, ProcessSort::NameAsc, "名称 A-Z");
            ui.add_space(8.0);
            ui.checkbox(
                &mut app.process_hide_small,
                egui::RichText::new(format!("隐藏小进程（< {} MB）", SMALL_PROCESS_MB))
                    .size(12.0)
                    .color(theme::TEXT_SECONDARY),
            )
            .on_hover_text("仅隐藏列表中内存占用不足 50 MB 的进程，不会关闭它们。");
        });
    });
    ui.add_space(14.0);

    // Sorting and display limits retain the existing list behavior.
    let filter = app.process_filter.to_lowercase();
    sort_processes(&mut app.process_list, app.process_sort);
    let processes: Vec<ProcessInfo> = app
        .process_list
        .iter()
        .filter(|process| filter.is_empty() || process.name.to_lowercase().contains(&filter))
        .filter(|process| !app.process_hide_small || process.memory_mb >= SMALL_PROCESS_MB)
        .take(PROCESS_LIST_LIMIT)
        .cloned()
        .collect();
    let mut kill_request: Option<(u32, String)> = None;

    egui::Frame::none()
        .fill(theme::CARD_BG)
        .rounding(egui::Rounding::same(16.0))
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(1.0, theme::CARD_BORDER))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .id_salt("process_table")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let row_width = ui.available_width();
                    let widths = table_widths(row_width - TABLE_ROW_PADDING * 2.0);
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(TABLE_ROW_PADDING, 4.0))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.horizontal(|ui| {
                                header_cell(ui, "进程名称", widths[0], false);
                                header_cell(ui, "PID", widths[1], true);
                                header_cell(ui, "CPU %", widths[2], true);
                                header_cell(ui, "内存 MB", widths[3], true);
                                header_cell(ui, "状态", widths[4], false);
                                header_cell(ui, "操作", widths[5], true);
                            });
                        });
                    ui.separator();
                    ui.add_space(4.0);

                    if processes.is_empty() {
                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("没有匹配的进程")
                                    .size(14.0)
                                    .color(theme::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new("尝试其他名称，或关闭“隐藏小进程”。")
                                    .size(12.0)
                                    .color(theme::TEXT_DIM),
                            );
                        });
                        ui.add_space(20.0);
                    }

                    for process in &processes {
                        let row_rect = egui::Rect::from_min_size(
                            ui.next_widget_position(),
                            egui::vec2(row_width, TABLE_ROW_HEIGHT + 8.0),
                        );
                        let hovered = ui
                            .input(|input| input.pointer.hover_pos())
                            .is_some_and(|position| row_rect.contains(position));
                        egui::Frame::none()
                            .fill(if hovered {
                                theme::SURFACE_HOVER
                            } else {
                                egui::Color32::TRANSPARENT
                            })
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::symmetric(TABLE_ROW_PADDING, 4.0))
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                ui.horizontal(|ui| {
                                    table_cell(ui, widths[0], TABLE_ROW_HEIGHT, false, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&process.name)
                                                    .size(14.0)
                                                    .color(theme::TEXT_PRIMARY),
                                            )
                                            .truncate(),
                                        )
                                        .on_hover_text(&process.name);
                                    });
                                    table_cell(ui, widths[1], TABLE_ROW_HEIGHT, true, |ui| {
                                        number_label(ui, &process.pid.to_string(), theme::TEXT_DIM);
                                    });
                                    table_cell(ui, widths[2], TABLE_ROW_HEIGHT, true, |ui| {
                                        number_label(
                                            ui,
                                            &format!("{:.1}", process.cpu_usage),
                                            if process.cpu_usage > 30.0 {
                                                theme::WARNING
                                            } else {
                                                theme::TEXT_SECONDARY
                                            },
                                        );
                                    });
                                    table_cell(ui, widths[3], TABLE_ROW_HEIGHT, true, |ui| {
                                        number_label(
                                            ui,
                                            &process.memory_mb.to_string(),
                                            if process.memory_mb > 500 {
                                                theme::WARNING
                                            } else {
                                                theme::TEXT_SECONDARY
                                            },
                                        );
                                    });
                                    table_cell(ui, widths[4], TABLE_ROW_HEIGHT, false, |ui| {
                                        ui.add_space(16.0);
                                        if process.is_protected {
                                            widgets::status_badge(ui, "受保护", theme::ACCENT);
                                        } else if process.is_whitelisted {
                                            widgets::status_badge(ui, "白名单", theme::ACCENT);
                                        } else if process.is_blacklisted {
                                            widgets::status_badge(ui, "清理名单", theme::WARNING);
                                        } else {
                                            widgets::status_badge(ui, "运行中", theme::TEXT_DIM);
                                        }
                                    });
                                    table_cell(ui, widths[5], TABLE_ROW_HEIGHT, true, |ui| {
                                        let button = egui::Button::new(
                                            egui::RichText::new("结束")
                                                .size(12.0)
                                                .color(theme::DANGER),
                                        )
                                        .fill(theme::DANGER.linear_multiply(0.08))
                                        .rounding(egui::Rounding::same(10.0))
                                        .min_size(egui::vec2(62.0, 30.0));
                                        if ui
                                            .add_enabled(
                                                !process.is_protected && !process.is_whitelisted,
                                                button,
                                            )
                                            .on_hover_text(
                                                if process.is_protected || process.is_whitelisted {
                                                    "受保护和白名单进程不会被关闭。"
                                                } else {
                                                    "查看确认窗口后结束此进程。"
                                                },
                                            )
                                            .clicked()
                                        {
                                            kill_request =
                                                Some((process.pid, process.name.clone()));
                                        }
                                    });
                                });
                            });
                        ui.add_space(2.0);
                    }
                    if matching_count > PROCESS_LIST_LIMIT {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "当前展示前 {} 项，可用搜索缩小范围。",
                                PROCESS_LIST_LIMIT
                            ))
                            .size(12.0)
                            .color(theme::TEXT_DIM),
                        );
                    }
                });
        });

    if let Some((pid, _name)) = kill_request {
        app.process_close_request = Some(
            app.process_list
                .iter()
                .filter(|process| {
                    process.pid == pid && !process.is_protected && !process.is_whitelisted
                })
                .cloned()
                .collect(),
        );
    }
}

fn refresh_process_list(app: &mut GameAcceleratorApp) {
    app.refresh_processes();
}

fn show_close_confirmation(app: &mut GameAcceleratorApp, ui: &egui::Ui) {
    let Some(processes) = app.process_close_request.clone() else {
        return;
    };
    let mut open = true;
    let mut confirm = false;
    let mut cancel = false;
    egui::Window::new("确认关闭进程")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(440.0)
        .show(ui.ctx(), |ui| {
            widgets::notice(
                ui,
                "将强制结束下面的程序，未保存的内容可能丢失。",
                theme::WARNING,
            );
            ui.add_space(14.0);
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for process in &processes {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&process.name)
                                        .size(14.0)
                                        .color(theme::TEXT_PRIMARY),
                                )
                                .truncate(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("PID {}", process.pid))
                                            .size(12.0)
                                            .monospace()
                                            .color(theme::TEXT_DIM),
                                    );
                                },
                            );
                        });
                    }
                });
            if processes.is_empty() {
                ui.label(
                    egui::RichText::new("没有可关闭的进程。受保护和白名单程序已保留。")
                        .size(12.0)
                        .color(theme::TEXT_SECONDARY),
                );
            }
            ui.add_space(18.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                confirm = ui
                    .add_enabled(
                        !processes.is_empty()
                            && !app.process_refresh_busy
                            && !app.is_boosting
                            && !app.is_restoring
                            && !app.action_busy,
                        theme::primary_button("确认结束"),
                    )
                    .clicked();
                cancel = ui.add(theme::secondary_button("取消")).clicked();
            });
        });
    if confirm {
        app.close_processes(processes);
    }
    if confirm || cancel || !open {
        app.process_close_request = None;
    }
}

fn sort_processes(list: &mut [ProcessInfo], sort: ProcessSort) {
    match sort {
        ProcessSort::MemoryDesc => list.sort_by_key(|process| std::cmp::Reverse(process.memory_mb)),
        ProcessSort::CpuDesc => list.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        ProcessSort::NameAsc => list.sort_by_key(|process| process.name.to_lowercase()),
    }
}

fn view_button(label: &str, selected: bool) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(label).size(14.0).color(if selected {
        theme::ACCENT
    } else {
        theme::TEXT_SECONDARY
    }))
    .fill(if selected {
        theme::ACCENT_BG
    } else {
        theme::SURFACE
    })
    .stroke(egui::Stroke::new(
        1.0,
        if selected {
            theme::ACCENT.linear_multiply(0.4)
        } else {
            theme::CARD_BORDER
        },
    ))
    .rounding(egui::Rounding::same(10.0))
    .min_size(egui::vec2(104.0, 36.0))
}

fn sort_button(ui: &mut egui::Ui, app: &mut GameAcceleratorApp, sort: ProcessSort, label: &str) {
    let selected = app.process_sort == sort;
    if ui
        .add(view_button(label, selected).min_size(egui::vec2(76.0, 30.0)))
        .clicked()
    {
        app.process_sort = sort;
    }
}

fn category_marker(ui: &mut egui::Ui, category: Category) {
    let label = match category {
        Category::Browser => "WEB",
        Category::Chat => "IM",
        Category::Office => "DOC",
        Category::CloudSync => "SYNC",
        Category::Updater => "UPD",
        Category::Media => "PLAY",
        Category::GameLauncher => "GAME",
        Category::System => "SYS",
        Category::Other => "APP",
    };
    let color = if category == Category::System {
        theme::TEXT_SECONDARY
    } else {
        theme::ACCENT
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::Rounding::same(14.0),
        color.linear_multiply(0.10),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.0),
        color,
    );
}

fn category_description(category: Category) -> &'static str {
    match category {
        Category::Browser => "网页浏览与网页应用",
        Category::Chat => "即时通讯与语音聊天",
        Category::Office => "文档、表格与阅读器",
        Category::CloudSync => "文件备份与同步",
        Category::Updater => "安装与自动更新任务",
        Category::Media => "音乐与视频播放",
        Category::GameLauncher => "游戏下载与平台客户端",
        Category::System => "Windows 系统、安全与关键服务",
        Category::Other => "未分类的桌面与后台应用",
    }
}

/// Fixed data columns keep changing numeric values from shifting the table.
fn table_widths(total_width: f32) -> [f32; 6] {
    let total_width = total_width.max(0.0);
    let name_width = (total_width - 380.0).max(120.0).min(total_width);
    let scale = ((total_width - name_width) / 380.0).clamp(0.0, 1.0);
    [
        name_width,
        56.0 * scale,
        70.0 * scale,
        88.0 * scale,
        92.0 * scale,
        74.0 * scale,
    ]
}

fn header_cell(ui: &mut egui::Ui, text: &str, width: f32, right_aligned: bool) {
    table_cell(ui, width, 28.0, right_aligned, |ui| {
        if text == "状态" {
            ui.add_space(16.0);
        }
        ui.add(
            egui::Label::new(egui::RichText::new(text).size(12.0).color(theme::TEXT_DIM))
                .truncate(),
        );
    });
}

fn number_label(ui: &mut egui::Ui, value: &str, color: egui::Color32) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(value)
                .size(12.0)
                .monospace()
                .color(color),
        )
        .truncate(),
    );
}

fn table_cell(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    right_aligned: bool,
    content: impl FnOnce(&mut egui::Ui),
) {
    let layout = if right_aligned {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    ui.allocate_ui_with_layout(egui::vec2(width, height), layout, |ui| {
        ui.set_min_width(width);
        ui.set_max_width(width);
        content(ui);
    });
}
