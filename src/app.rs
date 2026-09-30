use crate::config::AppConfig;
use crate::core::{BoostResult, BoostSession};
use crate::monitor::{MonitorControl, SystemStats};
use crate::ui::{dashboard, gpu_page, process_page, settings_page, system_opt_page};
use parking_lot::Mutex;
use std::sync::Arc;

type BackgroundResult<T> = Arc<Mutex<Option<T>>>;
type ActionUpdate = (ActionTarget, Result<String, String>);
type ProcessUpdate = (
    Vec<crate::core::process_manager::ProcessInfo>,
    Option<String>,
);

/// Navigation pages available in the application sidebar.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Process,
    Gpu,
    SystemOpt,
    Settings,
}

/// Top-level application state, implementing [`eframe::App`] for the egui
/// immediate-mode GUI. Holds the current page, shared system stats, boost
/// state, process list, and configuration.
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
    pub process_close_request: Option<Vec<ProcessInfo>>,
    pub process_refresh_busy: bool,
    pub process_list_initialized: bool,
    pub gpu_status: Option<String>,
    pub sysopt_status: Option<String>,
    pub is_boosting: bool,
    pub is_admin: bool,
    pub settings_status: Option<String>,
    pub is_restoring: bool,
    pub action_busy: bool,
    pub boost_session: Option<BoostSession>,
    boost_channel: BackgroundResult<BoostUpdate>,
    action_channel: BackgroundResult<ActionUpdate>,
    monitor_control: Arc<MonitorControl>,
    close_after_work: bool,
    exit_ready: bool,
    smoke_test: Option<crate::smoke_test::SmokeTest>,
    process_channel: BackgroundResult<ProcessUpdate>,
}

enum BoostUpdate {
    Started(BoostResult, BoostSession),
    Restored(BoostSession, Vec<String>),
}

#[derive(Clone, Copy)]
pub enum ActionTarget {
    System,
    Gpu,
}

/// Sorting mode for the advanced process list view.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    MemoryDesc,
    CpuDesc,
    NameAsc,
}

/// UI-side process information, extending the core `ProcessInfo` with
/// whitelist/blacklist membership flags for rendering status badges.
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
    /// Create a new application instance. Loads persisted configuration,
    /// spawns the background system monitor thread, and checks for
    /// administrator privileges.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        smoke_directory: Option<std::path::PathBuf>,
    ) -> Self {
        let (config, settings_status) = match AppConfig::load() {
            Ok(config) => (config, None),
            Err(error) => (AppConfig::default(), Some(format!("⚠ {}", error))),
        };
        let stats = Arc::new(Mutex::new(SystemStats::default()));
        let monitor_control = Arc::new(MonitorControl::default());

        let stats_clone = stats.clone();
        let control_clone = monitor_control.clone();
        std::thread::spawn(move || {
            crate::monitor::run_monitor(stats_clone, control_clone);
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
            process_close_request: None,
            process_refresh_busy: false,
            process_list_initialized: false,
            gpu_status: None,
            sysopt_status: None,
            is_boosting: false,
            is_admin: crate::core::elevation::is_elevated(),
            boost_channel: Arc::new(Mutex::new(None)),
            settings_status,
            is_restoring: false,
            action_busy: false,
            boost_session: None,
            action_channel: Arc::new(Mutex::new(None)),
            monitor_control,
            close_after_work: false,
            exit_ready: false,
            smoke_test: smoke_directory.map(crate::smoke_test::SmokeTest::new),
            process_channel: Arc::new(Mutex::new(None)),
        }
    }

    /// Kick off the boost in a background thread so the UI never blocks on the
    /// external commands and system operations it runs.
    pub fn start_boost(&mut self) {
        if self.close_after_work || self.exit_ready || self.smoke_test.is_some() {
            return;
        }
        if self.is_boosting
            || self.is_restoring
            || self.action_busy
            || self.process_refresh_busy
            || self.session_active()
        {
            return;
        }
        self.is_boosting = true;

        let config = self.config.clone();
        let game = self.config.selected_game.clone();
        let channel = self.boost_channel.clone();

        std::thread::spawn(move || {
            let (result, session) = crate::core::run_boost(&config, game.as_deref());
            *channel.lock() = Some(BoostUpdate::Started(result, session));
        });
    }

    /// Poll the boost channel; if the background thread finished, move its result
    /// into the app state. Called once per frame.
    pub fn poll_boost(&mut self) {
        let done = self.boost_channel.lock().take();
        match done {
            Some(BoostUpdate::Started(result, session)) => {
                self.boost_result = Some(result);
                self.boost_session = Some(session);
                self.last_boost_time = Some(format_clock_time());
                self.is_boosting = false;
                self.monitor_control.request_refresh();
            }
            Some(BoostUpdate::Restored(session, errors)) => {
                self.boost_session = Some(session);
                self.is_restoring = false;
                self.sysopt_status = Some(if errors.is_empty() {
                    "✓ 加速会话已结束".to_string()
                } else {
                    self.close_after_work = false;
                    format!("⚠ 恢复未完成，请重试：{}", errors.join("；"))
                });
                self.monitor_control.request_refresh();
            }
            None => {}
        }
    }

    pub fn session_active(&self) -> bool {
        self.boost_session
            .as_ref()
            .is_some_and(BoostSession::has_changes)
    }

    pub fn restore_boost(&mut self) {
        if self.is_boosting || self.is_restoring || self.action_busy {
            return;
        }
        let Some(mut session) = self.boost_session.take() else {
            return;
        };
        self.is_restoring = true;
        let channel = self.boost_channel.clone();
        std::thread::spawn(move || {
            let errors = session.restore();
            *channel.lock() = Some(BoostUpdate::Restored(session, errors));
        });
    }

    pub fn start_action(
        &mut self,
        target: ActionTarget,
        action: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) {
        if self.close_after_work || self.exit_ready || self.smoke_test.is_some() {
            return;
        }
        if self.action_busy || self.is_boosting || self.is_restoring || self.process_refresh_busy {
            return;
        }
        self.action_busy = true;
        let channel = self.action_channel.clone();
        std::thread::spawn(move || {
            *channel.lock() = Some((target, action()));
        });
    }

    pub fn save_config(&mut self) {
        if self.close_after_work || self.exit_ready || self.smoke_test.is_some() {
            return;
        }
        self.settings_status = Some(match self.config.save() {
            Ok(()) => "✓ 设置已保存".to_string(),
            Err(error) => format!("⚠ {}", error),
        });
    }

    fn poll_action(&mut self) {
        let done = self.action_channel.lock().take();
        if let Some((target, result)) = done {
            let status = Some(match result {
                Ok(message) if message.starts_with('✓') => message,
                Ok(message) => format!("✓ {}", message),
                Err(error) => format!("⚠ {}", error),
            });
            match target {
                ActionTarget::System => self.sysopt_status = status,
                ActionTarget::Gpu => self.gpu_status = status,
            }
            self.action_busy = false;
            self.monitor_control.request_refresh();
        }
    }

    pub fn refresh_processes(&mut self) {
        if self.close_after_work || self.exit_ready {
            return;
        }
        if self.process_refresh_busy {
            return;
        }
        self.process_refresh_busy = true;
        let channel = self.process_channel.clone();
        std::thread::spawn(move || {
            *channel.lock() = Some((crate::core::process_manager::get_process_list(), None));
        });
    }

    pub fn close_processes(&mut self, processes: Vec<ProcessInfo>) {
        if self.close_after_work || self.exit_ready || self.smoke_test.is_some() {
            return;
        }
        if self.process_refresh_busy || self.is_boosting || self.is_restoring || self.action_busy {
            return;
        }
        self.process_refresh_busy = true;
        let channel = self.process_channel.clone();
        std::thread::spawn(move || {
            let mut killed = 0;
            let mut failures = Vec::new();
            for process in processes {
                match crate::core::process_manager::kill_process_by_pid(process.pid, &process.name)
                {
                    Ok(()) => killed += 1,
                    Err(error) => failures.push(error),
                }
            }
            let status = if failures.is_empty() {
                format!("✓ 已结束 {} 个已选择的进程", killed)
            } else {
                format!("⚠ 已结束 {} 个进程；{}", killed, failures.join("；"))
            };
            *channel.lock() = Some((
                crate::core::process_manager::get_process_list(),
                Some(status),
            ));
        });
    }

    fn poll_processes(&mut self) {
        let done = self.process_channel.lock().take();
        if let Some((processes, status)) = done {
            self.process_list = processes
                .into_iter()
                .map(|process| {
                    let contains = |names: &std::collections::HashSet<String>| {
                        names
                            .iter()
                            .any(|name| name.eq_ignore_ascii_case(&process.name))
                    };
                    ProcessInfo {
                        is_whitelisted: contains(&self.config.whitelist),
                        is_blacklisted: contains(&self.config.blacklist),
                        is_protected: process.is_protected,
                        name: process.name,
                        pid: process.pid,
                        cpu_usage: process.cpu_usage,
                        memory_mb: process.memory_mb,
                    }
                })
                .collect();
            if status.is_some() {
                self.process_status = status;
            }
            self.process_refresh_busy = false;
            self.process_list_initialized = true;
        }
    }
}

/// UTC timestamp, explicitly labelled instead of misrepresenting it as local time.
fn format_clock_time() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs = (now % 86400) as u32;
    format!(
        "{:02}:{:02}:{:02} UTC",
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
        self.poll_action();
        self.poll_processes();
        let stats = self.stats.lock().clone();
        if let Some(test) = &mut self.smoke_test {
            match test.poll(ctx, &stats) {
                Ok(Some(page)) => self.current_page = page,
                Ok(None) => {}
                Err(error) => {
                    eprintln!("{}", error);
                    std::process::exit(1);
                }
            }
        }

        // Wait for in-flight changes and restore them before a normal close.
        if !self.exit_ready
            && ctx.input(|input| input.viewport().close_requested())
            && (self.is_boosting
                || self.is_restoring
                || self.action_busy
                || self.process_refresh_busy
                || self.session_active())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_after_work = true;
            self.process_close_request = None;
        }
        if self.close_after_work
            && !self.is_boosting
            && !self.is_restoring
            && !self.action_busy
            && !self.process_refresh_busy
        {
            if self.session_active() {
                self.restore_boost();
            } else {
                self.exit_ready = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

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
                if self.smoke_test.is_some() || self.close_after_work || self.exit_ready {
                    ui.disable();
                }
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

                    let btn =
                        egui::Button::new(egui::RichText::new(*label).size(13.0).color(text_color))
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
                        egui::RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
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
                if self.smoke_test.is_some() || self.close_after_work || self.exit_ready {
                    ui.disable();
                }
                // Monitoring runs with normal privileges; elevation is optional.
                if !self.is_admin {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(60, 45, 15))
                        .rounding(egui::Rounding::same(8.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(255, 200, 75),
                        ))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("⚠")
                                        .size(18.0)
                                        .color(egui::Color32::from_rgb(255, 200, 75)),
                                );
                                ui.add_space(4.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("普通权限运行 · 可直接监控和试用")
                                            .size(13.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(255, 200, 75)),
                                    );
                                    ui.label(
                                        egui::RichText::new(
                                            "修改 GPU 调度、系统服务或遇到权限不足时，可按需提权",
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
                                        let can_restart = !self.is_boosting
                                            && !self.is_restoring
                                            && !self.action_busy
                                            && !self.process_refresh_busy
                                            && !self.session_active();
                                        if ui.add_enabled(can_restart, btn).clicked() {
                                            match crate::core::elevation::try_elevate_if_needed() {
                                                Ok(true) => ctx.send_viewport_cmd(
                                                    egui::ViewportCommand::Close,
                                                ),
                                                Ok(false) => {}
                                                Err(error) => {
                                                    self.sysopt_status =
                                                        Some(format!("⚠ {}", error))
                                                }
                                            }
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(12.0);
                }

                if let Some(status) = &self.sysopt_status {
                    if self.current_page != Page::SystemOpt {
                        ui.label(
                            egui::RichText::new(status).color(if status.starts_with('✓') {
                                crate::ui::theme::SUCCESS
                            } else {
                                crate::ui::theme::WARNING
                            }),
                        );
                        ui.add_space(6.0);
                    }
                }
                if let Some(status) = &self.settings_status {
                    if self.current_page != Page::Settings && status.starts_with('⚠') {
                        ui.label(egui::RichText::new(status).color(crate::ui::theme::WARNING));
                    }
                }

                match self.current_page {
                    Page::Dashboard => dashboard::show(self, ui),
                    Page::Process => process_page::show(self, ui),
                    Page::Gpu => gpu_page::show(self, ui),
                    Page::SystemOpt => system_opt_page::show(self, ui),
                    Page::Settings => settings_page::show(self, ui),
                };
            });
        let page_ready = self.current_page != Page::Process
            || (!self.process_refresh_busy && !self.process_list.is_empty());
        if let Some(test) = self.smoke_test.as_mut().filter(|_| page_ready) {
            test.request_capture(ctx, &stats);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.monitor_control.stop();
        if let Some(session) = &mut self.boost_session {
            for error in session.restore() {
                eprintln!("Restore: {}", error);
            }
        }
    }

    fn persist_egui_memory(&self) -> bool {
        self.smoke_test.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No monitor thread, config I/O, elevation check, or system writes.
    fn idle_app() -> GameAcceleratorApp {
        GameAcceleratorApp {
            current_page: Page::Dashboard,
            stats: Arc::new(Mutex::new(SystemStats::default())),
            config: AppConfig {
                enable_game_mode: false,
                enable_high_perf_power: false,
                ..Default::default()
            },
            boost_result: None,
            last_boost_time: None,
            process_list: Vec::new(),
            process_filter: String::new(),
            process_sort: ProcessSort::MemoryDesc,
            process_status: None,
            process_advanced: false,
            process_hide_small: true,
            process_close_request: None,
            process_refresh_busy: false,
            process_list_initialized: false,
            gpu_status: None,
            sysopt_status: None,
            is_boosting: false,
            is_admin: false,
            settings_status: None,
            is_restoring: false,
            action_busy: false,
            boost_session: None,
            boost_channel: Arc::new(Mutex::new(None)),
            action_channel: Arc::new(Mutex::new(None)),
            monitor_control: Arc::new(MonitorControl::default()),
            close_after_work: false,
            exit_ready: false,
            smoke_test: None,
            process_channel: Arc::new(Mutex::new(None)),
        }
    }

    fn assert_modifying_tasks_stay_idle(app: &mut GameAcceleratorApp) {
        app.start_boost();
        app.start_action(ActionTarget::System, || Ok("test action".to_string()));
        app.close_processes(Vec::new());
        assert!(!app.is_boosting);
        assert!(!app.action_busy);
        assert!(!app.process_refresh_busy);
        assert!(app.boost_channel.lock().is_none());
    }

    #[test]
    fn closing_does_not_start_new_work_after_restore_result_arrives() {
        let mut app = idle_app();
        app.close_after_work = true;
        app.is_restoring = true;
        *app.boost_channel.lock() =
            Some(BoostUpdate::Restored(BoostSession::default(), Vec::new()));
        app.poll_boost();
        assert!(!app.is_restoring);
        assert_modifying_tasks_stay_idle(&mut app);
    }

    #[test]
    fn final_close_frame_rejects_new_modifying_tasks() {
        let mut app = idle_app();
        app.exit_ready = true;
        assert_modifying_tasks_stay_idle(&mut app);
    }

    #[test]
    fn failed_restore_cancels_close_and_keeps_error_visible() {
        let mut app = idle_app();
        app.close_after_work = true;
        app.is_restoring = true;
        *app.boost_channel.lock() = Some(BoostUpdate::Restored(
            BoostSession::default(),
            vec!["test: access denied".to_string()],
        ));
        app.poll_boost();
        assert!(!app.close_after_work);
        assert!(!app.is_restoring);
        assert!(app.boost_session.is_some());
        assert!(app
            .sysopt_status
            .as_deref()
            .unwrap()
            .contains("access denied"));
    }
}
