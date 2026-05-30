use crate::config::AppConfig;
use crate::core::BoostResult;
use crate::monitor::SystemStats;
use crate::ui::{dashboard, gpu_page, process_page, settings_page, system_opt_page};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Process,
    Gpu,
    SystemOpt,
    Settings,
}

pub struct GameAcceleratorApp {
    pub current_page: Page,
    pub stats: Arc<Mutex<SystemStats>>,
    pub config: AppConfig,
    pub boost_result: Option<BoostResult>,
    pub last_boost_time: Option<String>,
    pub process_list: Vec<ProcessInfo>,
    pub process_filter: String,
    pub process_sort: ProcessSort,
    pub process_status: Option<String>,
    pub process_advanced: bool,
    pub process_hide_small: bool,
    pub gpu_status: Option<String>,
    pub sysopt_status: Option<String>,
    pub is_boosting: bool,
    pub is_admin: bool,
    /// Channel for the background boost thread to hand its result back to the UI.
    /// `Some` while a boost is in flight; the inner `Option` fills in on completion.
    pub boost_channel: Arc<Mutex<Option<BoostResult>>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    MemoryDesc,
    CpuDesc,
    NameAsc,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub is_whitelisted: bool,
    pub is_blacklisted: bool,
    pub is_protected: bool,
}

impl GameAcceleratorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let stats = Arc::new(Mutex::new(SystemStats::default()));

        let stats_clone = stats.clone();
        std::thread::spawn(move || {
            crate::monitor::run_monitor(stats_clone);
        });

        Self {
            current_page: Page::Dashboard,
            stats,
            config,
            boost_result: None,
            last_boost_time: None,
            process_list: Vec::new(),
            process_filter: String::new(),
            process_sort: ProcessSort::MemoryDesc,
            process_status: None,
            process_advanced: false,
            process_hide_small: true,
            gpu_status: None,
            sysopt_status: None,
            is_boosting: false,
            is_admin: crate::core::elevation::is_elevated(),
            boost_channel: Arc::new(Mutex::new(None)),
        }
    }

    /// Kick off the boost in a background thread so the UI never blocks on the
    /// slow external commands (ProcessIdleTasks, wmic, powercfg) it runs.
    pub fn start_boost(&mut self) {
        if self.is_boosting {
            return;
        }
        self.is_boosting = true;

        let config = self.config.clone();
        let game = self.config.selected_game.clone();
        let channel = self.boost_channel.clone();

        std::thread::spawn(move || {
            let result = crate::core::run_boost(&config, game.as_deref());
            *channel.lock() = Some(result);
        });
    }

    /// Poll the boost channel; if the background thread finished, move its result
    /// into the app state. Called once per frame.
    pub fn poll_boost(&mut self) {
        if !self.is_boosting {
            return;
        }
        let done = self.boost_channel.lock().take();
        if let Some(result) = done {
            self.boost_result = Some(result);
            self.last_boost_time = Some(format_clock_time());
            self.is_boosting = false;
        }
    }
}

/// Current wall-clock time as HH:MM:SS.
fn format_clock_time() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs = (now % 86400) as u32;
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

impl eframe::App for GameAcceleratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));

        // Pick up the result of any in-flight background boost.
        self.poll_boost();

        // Sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(170.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(10, 10, 16))
                    .inner_margin(egui::Margin::symmetric(12.0, 16.0)),
            )
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Logo
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Game")
                            .size(26.0)
                            .color(egui::Color32::from_rgb(0, 255, 136))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("Accelerator")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(140, 140, 160)),
                    );
                });

                ui.add_space(28.0);

                let pages = [
                    (Page::Dashboard, "仪表盘"),
                    (Page::Process, "进程管理"),
                    (Page::Gpu, "GPU 设置"),
                    (Page::SystemOpt, "系统优化"),
                    (Page::Settings, "设置"),
                ];

                for (page, label) in &pages {
                    let is_selected = self.current_page == *page;

                    let (bg, text_color) = if is_selected {
                        (
                            egui::Color32::from_rgb(0, 255, 136),
                            egui::Color32::from_rgb(10, 10, 16),
                        )
                    } else {
                        (
                            egui::Color32::TRANSPARENT,
                            egui::Color32::from_rgb(140, 140, 160),
                        )
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(*label).size(13.0).color(text_color),
                    )
                    .min_size(egui::vec2(146.0, 34.0))
                    .rounding(egui::Rounding::same(6.0))
                    .fill(bg);

                    if ui.add(btn).clicked() {
                        self.current_page = *page;
                    }
                    ui.add_space(3.0);
                }

                // Bottom version
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("v1.0.0")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(50, 50, 65)),
                    );
                });
            });

        // Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(13, 13, 20))
                    .inner_margin(egui::Margin::same(20.0)),
            )
            .show(ctx, |ui| {
                // Admin privilege banner - shown when not running elevated.
                // Most optimizations (registry, services) require admin rights.
                if !self.is_admin {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(60, 45, 15))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 200, 75)))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("⚠").size(18.0).color(
                                    egui::Color32::from_rgb(255, 200, 75),
                                ));
                                ui.add_space(4.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("当前未以管理员身份运行")
                                            .size(13.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(255, 200, 75)),
                                    );
                                    ui.label(
                                        egui::RichText::new(
                                            "系统优化、GPU 调度、磁盘索引等功能需要管理员权限才能生效",
                                        )
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(200, 180, 140)),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let btn = egui::Button::new(
                                            egui::RichText::new("🛡 以管理员身份重启")
                                                .size(12.0)
                                                .strong()
                                                .color(egui::Color32::from_rgb(20, 16, 8)),
                                        )
                                        .fill(egui::Color32::from_rgb(255, 200, 75))
                                        .rounding(egui::Rounding::same(6.0))
                                        .min_size(egui::vec2(150.0, 34.0));
                                        if ui.add(btn).clicked() {
                                            if crate::core::elevation::elevate_if_needed() {
                                                // A UAC-elevated instance is launching; close this one.
                                                ctx.send_viewport_cmd(
                                                    egui::ViewportCommand::Close,
                                                );
                                            }
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(12.0);
                }

                match self.current_page {
                    Page::Dashboard => dashboard::show(self, ui),
                    Page::Process => process_page::show(self, ui),
                    Page::Gpu => gpu_page::show(self, ui),
                    Page::SystemOpt => system_opt_page::show(self, ui),
                    Page::Settings => settings_page::show(self, ui),
                };
            });
    }
}
