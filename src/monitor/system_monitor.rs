use crate::core::{disk_optimizer, game_mode, gpu_manager, power_manager};
use parking_lot::{Condvar, Mutex};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;

/// CPU and RAM are refreshed separately from expensive external queries.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SystemStats {
    pub cpu_usage_total: f32,
    pub cpu_usage_per_core: Vec<f32>,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_threads: usize,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_usage_percent: f32,
    pub gpu_name: String,
    pub gpu_driver: String,
    pub gpu_usage: f32,
    pub gpu_temp: f32,
    pub gpu_mem_used_mb: u64,
    pub gpu_mem_total_mb: u64,
    pub gpu_available: bool,
    pub process_count: usize,
    pub flags_ready: bool,
    pub power_high_perf: Option<bool>,
    pub game_mode_on: Option<bool>,
    pub hw_gpu_sched_on: Option<bool>,
    pub game_bar_on: Option<bool>,
    pub search_indexer_running: Option<bool>,
    pub settings_query_errors: Vec<String>,
}

#[derive(Default)]
pub struct MonitorControl {
    stopped: AtomicBool,
    refresh: AtomicBool,
    wake_lock: Mutex<()>,
    wake: Condvar,
}

impl MonitorControl {
    pub fn stop(&self) {
        let _guard = self.wake_lock.lock();
        self.stopped.store(true, Ordering::Relaxed);
        self.wake.notify_all();
    }
    pub fn request_refresh(&self) {
        let _guard = self.wake_lock.lock();
        self.refresh.store(true, Ordering::Relaxed);
        self.wake.notify_all();
    }

    /// Only the settings worker consumes refresh requests. Holding wake_lock
    /// around the predicate and notification avoids a lost wake during sleep.
    fn wait_until(&self, deadline: Instant, refresh_on_request: bool) -> bool {
        let mut guard = self.wake_lock.lock();
        loop {
            if self.stopped.load(Ordering::Relaxed) {
                return false;
            }
            let refresh = refresh_on_request && self.refresh.swap(false, Ordering::Relaxed);
            let now = Instant::now();
            if refresh || now >= deadline {
                return true;
            }
            self.wake.wait_for(&mut guard, deadline - now);
        }
    }
}

const FAST_INTERVAL: Duration = Duration::from_millis(500);
const GPU_INTERVAL: Duration = Duration::from_secs(2);
const FLAGS_INTERVAL: Duration = Duration::from_secs(15);

/// One long-lived worker per source bounds in-flight queries. Slow samples are
/// collected without the shared lock, then publish only their owned fields.
fn run_worker(
    stats: &Mutex<SystemStats>,
    control: &MonitorControl,
    interval: Duration,
    refresh_on_request: bool,
    mut sample: impl FnMut() -> SystemStats,
    publish: impl Fn(&mut SystemStats, SystemStats),
) {
    let mut deadline = Instant::now();
    while control.wait_until(deadline, refresh_on_request) {
        let started = Instant::now();
        let next = sample();
        if control.stopped.load(Ordering::Relaxed) {
            return;
        }
        publish(&mut stats.lock(), next);
        deadline = started + interval;
        // Skip missed periods instead of running a burst of catch-up queries.
        if deadline <= Instant::now() {
            deadline = Instant::now() + interval;
        }
    }
}

/// Native metrics refresh twice per second independently of GPU/process and
/// settings queries. Shutdown wakes sleeping workers; an active external query
/// remains bounded by command::run_hidden's timeout.
pub fn run_monitor(stats: Arc<Mutex<SystemStats>>, control: Arc<MonitorControl>) {
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let mut sys = System::new();
            run_worker(
                &stats,
                &control,
                GPU_INTERVAL,
                false,
                || {
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
                    let mut next = SystemStats {
                        process_count: sys.processes().len(),
                        ..Default::default()
                    };
                    update_gpu(&mut next);
                    next
                },
                publish_gpu,
            );
        });
        scope.spawn(|| {
            run_worker(
                &stats,
                &control,
                FLAGS_INTERVAL,
                true,
                || {
                    let mut next = SystemStats::default();
                    update_flags(&mut next);
                    next
                },
                publish_flags,
            );
        });

        let mut sys = System::new();
        // CPU usage is a delta; the first counter read is not a valid sample.
        sys.refresh_cpu_all();
        if !control.wait_until(Instant::now() + sysinfo::MINIMUM_CPU_UPDATE_INTERVAL, false) {
            return;
        }
        let mut native = SystemStats::default();
        run_worker(
            &stats,
            &control,
            FAST_INTERVAL,
            false,
            || {
                sys.refresh_memory();
                sys.refresh_cpu_all();
                update_native_stats(&mut native, &sys);
                native.clone()
            },
            publish_native,
        );
    });
}

fn publish_native(stats: &mut SystemStats, next: SystemStats) {
    stats.cpu_usage_total = next.cpu_usage_total;
    stats.cpu_usage_per_core = next.cpu_usage_per_core;
    stats.cpu_name = next.cpu_name;
    stats.cpu_cores = next.cpu_cores;
    stats.cpu_threads = next.cpu_threads;
    stats.ram_used_gb = next.ram_used_gb;
    stats.ram_total_gb = next.ram_total_gb;
    stats.ram_usage_percent = next.ram_usage_percent;
}

fn publish_gpu(stats: &mut SystemStats, next: SystemStats) {
    stats.process_count = next.process_count;
    stats.gpu_available = next.gpu_available;
    stats.gpu_name = next.gpu_name;
    stats.gpu_driver = next.gpu_driver;
    stats.gpu_usage = next.gpu_usage;
    stats.gpu_temp = next.gpu_temp;
    stats.gpu_mem_used_mb = next.gpu_mem_used_mb;
    stats.gpu_mem_total_mb = next.gpu_mem_total_mb;
}

fn publish_flags(stats: &mut SystemStats, next: SystemStats) {
    stats.power_high_perf = next.power_high_perf;
    stats.game_mode_on = next.game_mode_on;
    stats.hw_gpu_sched_on = next.hw_gpu_sched_on;
    stats.game_bar_on = next.game_bar_on;
    stats.search_indexer_running = next.search_indexer_running;
    stats.settings_query_errors = next.settings_query_errors;
    stats.flags_ready = next.flags_ready;
}

fn update_native_stats(stats: &mut SystemStats, sys: &System) {
    stats.cpu_usage_total = sys.global_cpu_usage();
    stats.cpu_usage_per_core = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    if stats.cpu_name.is_empty() {
        stats.cpu_name = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_default();
        stats.cpu_cores = sys.physical_core_count().unwrap_or(0);
        stats.cpu_threads = sys.cpus().len();
    }
    stats.ram_total_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    stats.ram_used_gb = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    stats.ram_usage_percent = if sys.total_memory() > 0 {
        (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as f32
    } else {
        0.0
    };
}

fn update_gpu(stats: &mut SystemStats) {
    let gpu = gpu_manager::get_gpu_info().into_iter().next();
    stats.gpu_available = gpu.is_some();
    if let Some(gpu) = gpu {
        stats.gpu_name = gpu.name;
        stats.gpu_driver = gpu.driver_version;
        stats.gpu_usage = gpu.usage_percent;
        stats.gpu_temp = gpu.temperature;
        stats.gpu_mem_used_mb = gpu.memory_used_mb;
        stats.gpu_mem_total_mb = gpu.memory_total_mb;
    }
}

fn update_flags(stats: &mut SystemStats) {
    let mut errors = Vec::new();
    let mut query = |name: &str, result: Result<Option<bool>, String>| match result {
        Ok(value) => value,
        Err(error) => {
            errors.push(format!("{}：{}", name, error));
            None
        }
    };
    stats.power_high_perf = query(
        "电源方案",
        power_manager::get_active_power_plan_guid().map(|guid| {
            Some(guid == power_manager::HIGH_PERF_GUID || guid == power_manager::ULTIMATE_GUID)
        }),
    );
    stats.game_mode_on = query("游戏模式", game_mode::game_mode_state());
    stats.hw_gpu_sched_on = query("GPU 调度", game_mode::hardware_gpu_scheduling_state());
    stats.game_bar_on = query("Game Bar", game_mode::game_bar_state());
    stats.search_indexer_running = query("Windows Search", disk_optimizer::search_service_state());
    stats.settings_query_errors = errors;
    stats.flags_ready = true;
}

/// A single read-only sample for diagnostics; no configuration writes or boost.
pub fn collect_diagnostics() -> SystemStats {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();
    let mut stats = SystemStats::default();
    update_native_stats(&mut stats, &sys);
    stats.process_count = sys.processes().len();
    update_gpu(&mut stats);
    update_flags(&mut stats);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    struct StopOnDrop<'a>(&'a MonitorControl);
    impl Drop for StopOnDrop<'_> {
        fn drop(&mut self) {
            self.0.stop();
        }
    }

    #[test]
    fn slow_gpu_and_settings_do_not_block_native_publication() {
        let stats = Arc::new(Mutex::new(SystemStats::default()));
        let control = Arc::new(MonitorControl::default());
        let (started_tx, started_rx) = mpsc::channel();
        let (gpu_release_tx, gpu_release_rx) = mpsc::channel();
        let (flags_release_tx, flags_release_rx) = mpsc::channel();
        let (native_tx, native_rx) = mpsc::channel();
        std::thread::scope(|scope| {
            let _stop = StopOnDrop(&control);
            for (release, refresh, publish) in [
                (
                    gpu_release_rx,
                    false,
                    publish_gpu as fn(&mut SystemStats, SystemStats),
                ),
                (
                    flags_release_rx,
                    true,
                    publish_flags as fn(&mut SystemStats, SystemStats),
                ),
            ] {
                let started = started_tx.clone();
                let stats = &stats;
                let control = &control;
                scope.spawn(move || {
                    run_worker(
                        stats,
                        control,
                        Duration::from_secs(30),
                        refresh,
                        || {
                            started.send(()).unwrap();
                            release.recv_timeout(Duration::from_secs(5)).unwrap();
                            SystemStats {
                                gpu_available: true,
                                flags_ready: true,
                                ..Default::default()
                            }
                        },
                        publish,
                    );
                });
            }
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            scope.spawn(|| {
                let mut count = 0;
                run_worker(
                    &stats,
                    &control,
                    Duration::from_millis(5),
                    false,
                    || {
                        count += 1;
                        SystemStats {
                            cpu_usage_total: count as f32,
                            ..Default::default()
                        }
                    },
                    |stats, next| {
                        publish_native(stats, next);
                        native_tx.send(()).unwrap();
                    },
                );
            });
            for _ in 0..3 {
                native_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            }
            assert!(stats.lock().cpu_usage_total >= 3.0);
            control.stop();
            gpu_release_tx.send(()).unwrap();
            flags_release_tx.send(()).unwrap();
        });
        // Results completed after stop are discarded, including slow sources.
        let stats = stats.lock();
        assert!(stats.cpu_usage_total >= 3.0);
        assert!(!stats.gpu_available);
        assert!(!stats.flags_ready);
    }

    #[test]
    fn source_publication_preserves_other_fields_and_clears_missing_gpu() {
        let mut stats = SystemStats {
            cpu_usage_total: 21.0,
            gpu_available: true,
            gpu_name: "old adapter".to_string(),
            gpu_temp: 75.0,
            game_mode_on: Some(true),
            ..Default::default()
        };
        publish_gpu(&mut stats, SystemStats::default());
        assert!(!stats.gpu_available);
        assert!(stats.gpu_name.is_empty());
        assert_eq!(stats.gpu_temp, 0.0);
        assert_eq!(stats.cpu_usage_total, 21.0);
        assert_eq!(stats.game_mode_on, Some(true));
        publish_flags(
            &mut stats,
            SystemStats {
                flags_ready: true,
                settings_query_errors: vec!["query failed".to_string()],
                ..Default::default()
            },
        );
        assert_eq!(stats.cpu_usage_total, 21.0);
        assert_eq!(stats.game_mode_on, None);
        assert_eq!(stats.settings_query_errors, ["query failed"]);
        publish_native(
            &mut stats,
            SystemStats {
                cpu_usage_total: 42.0,
                ..Default::default()
            },
        );
        assert!(stats.flags_ready);
        assert_eq!(stats.settings_query_errors, ["query failed"]);
        assert_eq!(stats.cpu_usage_total, 42.0);
    }

    #[test]
    fn refresh_during_query_is_retained_without_overlapping_workers() {
        let stats = Mutex::new(SystemStats::default());
        let control = MonitorControl::default();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (published_tx, published_rx) = mpsc::channel();
        std::thread::scope(|scope| {
            let _stop = StopOnDrop(&control);
            let stats = &stats;
            let control = &control;
            scope.spawn(move || {
                run_worker(
                    stats,
                    control,
                    Duration::from_secs(30),
                    true,
                    || {
                        started_tx.send(()).unwrap();
                        release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                        SystemStats::default()
                    },
                    |stats, next| {
                        publish_flags(stats, next);
                        published_tx.send(()).unwrap();
                    },
                );
            });
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            control.request_refresh();
            control.request_refresh();
            assert!(matches!(
                started_rx.try_recv(),
                Err(mpsc::TryRecvError::Empty)
            ));
            release_tx.send(()).unwrap();
            published_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            release_tx.send(()).unwrap();
            published_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            control.stop();
        });
        assert!(started_rx.try_recv().is_err());
    }

    #[test]
    fn stop_wakes_idle_worker_and_refresh_is_consumed_once() {
        let control = MonitorControl::default();
        control.request_refresh();
        assert!(control.wait_until(Instant::now(), true));
        assert!(!control.refresh.load(Ordering::Relaxed));
        std::thread::scope(|scope| {
            let _stop = StopOnDrop(&control);
            let (tx, rx) = mpsc::channel();
            let control = &control;
            scope.spawn(move || {
                tx.send(control.wait_until(Instant::now() + Duration::from_secs(30), false))
                    .unwrap();
            });
            control.stop();
            assert!(!rx.recv_timeout(Duration::from_secs(2)).unwrap());
        });
    }
}
