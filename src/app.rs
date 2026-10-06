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

impl ProcessInfo {
    fn refresh_membership(&mut self, config: &AppConfig) {
        use crate::core::process_manager::matches_process_name;
        self.is_whitelisted = matches_process_name(&self.name, &config.whitelist);
        self.is_blacklisted = matches_process_name(&self.name, &config.blacklist);
    }

    pub fn can_close(&self, config: &AppConfig) -> bool {
        !self.is_protected
            && self.pid != 0
            && self.pid != std::process::id()
            && !crate::core::process_manager::is_protected_process(&self.name)
            && !crate::core::process_manager::matches_process_name(&self.name, &config.whitelist)
    }
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
                // A completed session remains active even when the requested
                // settings were already enabled. Otherwise another click can
                // repeat irreversible process closures. A fully failed start
                // with nothing to restore may be retried immediately.
                let active = session.has_changes()
                    || result.errors.is_empty()
                    || result.processes_killed > 0;
                self.boost_result = Some(result);
                self.boost_session = active.then_some(session);
                self.sysopt_status = None;
                self.last_boost_time = Some(format_clock_time());
                self.is_boosting = false;
                self.monitor_control.request_refresh();
            }
            Some(BoostUpdate::Restored(session, errors)) => {
                self.boost_session = if errors.is_empty() {
                    None
                } else {
                    Some(session)
                };
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
        self.boost_session.is_some()
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
        let whitelist = self.config.whitelist.clone();
        // Recheck the live configuration even if a caller supplies stale rows.
        let processes: Vec<_> = processes
            .into_iter()
            .filter(|process| process.can_close(&self.config))
            .map(|process| (process.pid, process.name))
            .collect();
        std::thread::spawn(move || {
            let report =
                crate::core::process_manager::close_selected_processes(processes, &whitelist);
            let killed = report.closed;
            let failures = report.failures;
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

    pub fn refresh_process_membership(&mut self) {
        for process in &mut self.process_list {
            process.refresh_membership(&self.config);
        }
    }

    fn poll_processes(&mut self) {
        let done = self.process_channel.lock().take();
        if let Some((processes, status)) = done {
            self.process_list = processes
                .into_iter()
                .map(|process| ProcessInfo {
                    is_whitelisted: crate::core::process_manager::matches_process_name(
                        &process.name,
                        &self.config.whitelist,
                    ),
                    is_blacklisted: crate::core::process_manager::matches_process_name(
                        &process.name,
                        &self.config.blacklist,
                    ),
                    is_protected: process.is_protected,
                    name: process.name,
                    pid: process.pid,
                    cpu_usage: process.cpu_usage,
                    memory_mb: process.memory_mb,
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
                Ok(Some(page)) => {
                    self.current_page = page;
                    if page == Page::Process {
                        self.process_advanced = test.process_list_view();
                    }
                }
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

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(196.0)
            .frame(
                egui::Frame::none()
                    .fill(crate::ui::theme::SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(14.0, 24.0)),
            )
            .show(ctx, |ui| {
                if self.close_after_work || self.exit_ready {
                    ui.disable();
                }
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::same(10.0),
                        crate::ui::theme::ACCENT_BG,
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "G",
                        egui::FontId::proportional(23.0),
                        crate::ui::theme::ACCENT,
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("游戏加速器")
                                .size(16.0)
                                .strong()
                                .color(crate::ui::theme::TEXT_PRIMARY),
                        );
                        ui.label(
                            egui::RichText::new("GAME ACCELERATOR")
                                .size(9.5)
                                .color(crate::ui::theme::TEXT_DIM),
                        );
                    });
                });
                ui.add_space(30.0);
                ui.label(
                    egui::RichText::new("工作台")
                        .size(11.0)
                        .color(crate::ui::theme::TEXT_DIM),
                );
                ui.add_space(8.0);
                for (page, label, icon) in [
                    (
                        Page::Dashboard,
                        "性能概览",
                        crate::ui::icons::Icon::Overview,
                    ),
                    (Page::Process, "进程管理", crate::ui::icons::Icon::Processes),
                    (Page::Gpu, "显卡设置", crate::ui::icons::Icon::Gpu),
                    (Page::SystemOpt, "系统优化", crate::ui::icons::Icon::Sliders),
                    (Page::Settings, "偏好设置", crate::ui::icons::Icon::Settings),
                ] {
                    if navigation_row(ui, label, icon, self.current_page == page).clicked() {
                        self.current_page = page;
                    }
                    ui.add_space(5.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(concat!("Version ", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(crate::ui::theme::TEXT_DIM),
                    );
                    ui.add_space(10.0);
                    egui::Frame::none()
                        .fill(crate::ui::theme::CARD_BG)
                        .rounding(egui::Rounding::same(10.0))
                        .inner_margin(egui::Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.set_min_width(140.0);
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(7.0, 7.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    rect.center(),
                                    3.0,
                                    crate::ui::theme::SUCCESS,
                                );
                                ui.label(
                                    egui::RichText::new("资源监控运行中")
                                        .size(11.5)
                                        .color(crate::ui::theme::TEXT_SECONDARY),
                                );
                            });
                            ui.label(
                                egui::RichText::new("手动启动 · 按需优化")
                                    .size(10.5)
                                    .color(crate::ui::theme::TEXT_DIM),
                            );
                        });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(crate::ui::theme::BG).inner_margin(egui::Margin::symmetric(24.0,20.0)))
            .show(ctx, |ui| {
                if self.close_after_work || self.exit_ready { ui.disable(); }
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("工作台").size(12.0).color(crate::ui::theme::TEXT_DIM));
                    ui.label(egui::RichText::new("/").size(12.0).color(crate::ui::theme::TEXT_DIM));
                    let label=match self.current_page {
                        Page::Dashboard=>"性能概览", Page::Process=>"进程管理",Page::Gpu=>"显卡设置",
                        Page::SystemOpt=>"系统优化",Page::Settings=>"偏好设置",
                    };
                    ui.label(egui::RichText::new(label).size(12.0).color(crate::ui::theme::TEXT_SECONDARY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {
                        if !self.is_admin {
                            let can_restart=!self.is_boosting&&!self.is_restoring&&!self.action_busy&&!self.process_refresh_busy&&!self.session_active();
                            if ui.add_enabled(can_restart,crate::ui::theme::secondary_button("管理员权限").min_size(egui::vec2(100.0,30.0)))
                                .on_hover_text("监控和游戏模式可使用普通权限；修改系统服务、GPU 调度或遇到权限不足时再提权。").clicked() {
                                match crate::core::elevation::try_elevate_if_needed() {
                                    Ok(true)=>ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                                    Ok(false)=>{},
                                    Err(error)=>self.sysopt_status=Some(format!("⚠ {error}")),
                                }
                            }
                        }
                        crate::ui::widgets::status_badge(ui,if self.is_admin{"管理员模式"}else{"普通权限"},crate::ui::theme::TEXT_SECONDARY);
                    });
                });
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(18.0);
                if let Some(status)=&self.sysopt_status {
                    if self.current_page!=Page::SystemOpt {
                        crate::ui::widgets::notice(ui,status,if status.starts_with('✓'){crate::ui::theme::SUCCESS}else{crate::ui::theme::WARNING});
                    }
                }
                if let Some(status)=&self.settings_status {
                    if self.current_page!=Page::Settings && status.starts_with('⚠') {crate::ui::widgets::notice(ui,status,crate::ui::theme::WARNING);}
                }
                match self.current_page {
                    Page::Dashboard=>dashboard::show(self,ui),
                    Page::Process=>process_page::show(self,ui),
                    Page::Gpu=>gpu_page::show(self,ui),
                    Page::SystemOpt=>system_opt_page::show(self,ui),
                    Page::Settings=>settings_page::show(self,ui),
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

    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if self.smoke_test.is_some() {
            // Keep the live visual style while preventing input from triggering actions.
            input
                .events
                .retain(|event| matches!(event, egui::Event::Screenshot { .. }));
        }
    }
    fn persist_egui_memory(&self) -> bool {
        self.smoke_test.is_none()
    }
}

fn navigation_row(
    ui: &mut egui::Ui,
    label: &str,
    icon: crate::ui::icons::Icon,
    selected: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
    let background = if selected {
        crate::ui::theme::ACCENT_BG
    } else if response.hovered() {
        crate::ui::theme::SURFACE
    } else {
        egui::Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, egui::Rounding::same(10.0), background);
    let color = if selected {
        crate::ui::theme::ACCENT
    } else {
        crate::ui::theme::TEXT_SECONDARY
    };
    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_center_size(
                egui::pos2(rect.left() + 2.0, rect.center().y),
                egui::vec2(3.0, 18.0),
            ),
            egui::Rounding::same(2.0),
            crate::ui::theme::ACCENT,
        );
    }
    crate::ui::icons::paint(
        ui.painter(),
        egui::Rect::from_center_size(
            egui::pos2(rect.left() + 24.0, rect.center().y),
            egui::vec2(18.0, 18.0),
        ),
        icon,
        color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 44.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        color,
    );
    response
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

    #[test]
    fn process_poll_matches_full_path_user_lists() {
        let mut app = idle_app();
        app.config
            .whitelist
            .insert(r#" "C:\Browser\CHROME.EXE" "#.to_string());
        app.config.blacklist.insert("/apps/chrome.exe".to_string());
        *app.process_channel.lock() = Some((
            vec![crate::core::process_manager::ProcessInfo {
                name: "chrome.exe".to_string(),
                pid: 10,
                cpu_usage: 0.0,
                memory_mb: 100,
                is_protected: false,
            }],
            None,
        ));
        app.poll_processes();
        assert!(app.process_list[0].is_whitelisted);
        assert!(app.process_list[0].is_blacklisted);
    }

    #[test]
    fn successful_noop_session_stays_active_and_rejects_duplicate_start() {
        let mut app = idle_app();
        *app.boost_channel.lock() = Some(BoostUpdate::Started(
            BoostResult::default(),
            BoostSession::default(),
        ));
        app.poll_boost();
        assert!(
            app.session_active(),
            "a completed session need not own changed settings"
        );
        app.start_boost();
        assert!(
            !app.is_boosting,
            "a duplicate click must not rerun process closures"
        );
        assert!(app.boost_channel.lock().is_none());
    }

    #[test]
    fn successful_stop_releases_session() {
        let mut app = idle_app();
        app.boost_session = Some(BoostSession::default());
        app.is_restoring = true;
        *app.boost_channel.lock() =
            Some(BoostUpdate::Restored(BoostSession::default(), Vec::new()));
        app.poll_boost();
        assert!(app.boost_session.is_none());
        assert!(!app.session_active());
        assert!(!app.is_restoring);
        assert!(app.sysopt_status.as_deref().unwrap().starts_with('✓'));
    }

    #[test]
    fn new_session_clears_previous_stop_status() {
        let mut app = idle_app();
        app.sysopt_status = Some("✓ 加速会话已结束".to_string());
        *app.boost_channel.lock() = Some(BoostUpdate::Started(
            BoostResult::default(),
            BoostSession::default(),
        ));
        app.poll_boost();
        assert!(app.session_active());
        assert!(app.sysopt_status.is_none());
    }

    #[test]
    fn completely_failed_start_leaves_no_active_session() {
        let mut app = idle_app();
        *app.boost_channel.lock() = Some(BoostUpdate::Started(
            BoostResult {
                errors: vec!["power unavailable".to_string()],
                ..Default::default()
            },
            BoostSession::default(),
        ));
        app.poll_boost();
        assert!(app.boost_session.is_none());
        assert!(!app.session_active());
        assert!(!app.is_boosting);
        assert_eq!(
            app.boost_result.as_ref().unwrap().errors,
            ["power unavailable"]
        );
    }

    #[test]
    fn partial_process_success_keeps_session_active_without_reversible_changes() {
        let mut app = idle_app();
        *app.boost_channel.lock() = Some(BoostUpdate::Started(
            BoostResult {
                processes_killed: 1,
                errors: vec!["one process could not close".to_string()],
                ..Default::default()
            },
            BoostSession::default(),
        ));
        app.poll_boost();
        assert!(app.session_active());
        assert_eq!(app.boost_result.as_ref().unwrap().processes_killed, 1);
    }

    #[test]
    fn smoke_capture_keeps_pixels_but_blocks_interactive_events() {
        use eframe::App;
        let mut app = idle_app();
        app.smoke_test = Some(crate::smoke_test::SmokeTest::new(std::env::temp_dir()));
        let mut input = egui::RawInput::default();
        input
            .events
            .push(egui::Event::PointerMoved(egui::pos2(100.0, 100.0)));
        input.events.push(egui::Event::Text("typing".to_string()));
        input.events.push(egui::Event::Screenshot {
            viewport_id: egui::ViewportId::ROOT,
            image: Arc::new(egui::ColorImage::new([1, 1], egui::Color32::WHITE)),
        });
        app.raw_input_hook(&egui::Context::default(), &mut input);
        assert_eq!(input.events.len(), 1);
        assert!(matches!(input.events[0], egui::Event::Screenshot { .. }));
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
        assert!(app.session_active());
        assert!(app
            .sysopt_status
            .as_deref()
            .unwrap()
            .contains("access denied"));
    }
}

#[cfg(test)]
mod process_policy_tests {
    use super::*;

    fn row() -> ProcessInfo {
        ProcessInfo {
            name: "chrome.exe".to_string(),
            pid: 10,
            cpu_usage: 0.0,
            memory_mb: 100,
            is_whitelisted: false,
            is_blacklisted: false,
            is_protected: false,
        }
    }

    #[test]
    fn stale_dialog_and_badges_follow_current_config_without_process_refresh() {
        let mut config = AppConfig::default();
        config.whitelist.clear();
        let mut process = row();
        assert!(process.can_close(&config));
        config
            .whitelist
            .insert(r#" "C:\Browser\CHROME.EXE" "#.to_string());
        config.blacklist.insert("/apps/chrome.exe".to_string());
        assert!(!process.can_close(&config));
        process.refresh_membership(&config);
        assert!(process.is_whitelisted && process.is_blacklisted);
        config.whitelist.clear();
        process.refresh_membership(&config);
        assert!(!process.is_whitelisted);
        assert!(process.can_close(&config));
    }

    #[test]
    fn reset_defaults_protects_pending_rows_and_hard_protection_survives_edits() {
        let mut config = AppConfig::default();
        config.whitelist.clear();
        let mut process = row();
        process.name = "OneDrive.exe".to_string();
        assert!(process.can_close(&config));
        process.refresh_membership(&AppConfig::default());
        assert!(process.is_whitelisted);
        assert!(!process.can_close(&AppConfig::default()));
        process.name = "MsMpEng.exe".to_string();
        assert!(!process.can_close(&config));
        process.name = "chrome.exe".to_string();
        process.is_protected = true;
        assert!(!process.can_close(&config));
    }
}
