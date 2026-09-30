use crate::core::{disk_optimizer, game_mode, gpu_manager, power_manager};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
}

impl MonitorControl {
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
    }
    pub fn request_refresh(&self) {
        self.refresh.store(true, Ordering::Relaxed);
    }
}

const FAST_INTERVAL_MS: u64 = 500;
const GPU_EVERY_N_TICKS: u64 = 4;
const FLAGS_EVERY_N_TICKS: u64 = 30;

/// Native metrics refresh twice per second; GPU/process queries every two
/// seconds; system settings every fifteen seconds or after a user action.
pub fn run_monitor(stats: Arc<Mutex<SystemStats>>, control: Arc<MonitorControl>) {
    let mut sys = System::new();
    // CPU usage is a delta; the first counter read alone is not a valid sample.
    sys.refresh_cpu_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    let mut tick = 0u64;

    while !control.stopped.load(Ordering::Relaxed) {
        sys.refresh_memory();
        sys.refresh_cpu_all();
        let mut next = stats.lock().clone();
        update_native_stats(&mut next, &sys);

        if tick.is_multiple_of(GPU_EVERY_N_TICKS) {
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
            next.process_count = sys.processes().len();
            update_gpu(&mut next);
        }

        if tick.is_multiple_of(FLAGS_EVERY_N_TICKS)
            || control.refresh.swap(false, Ordering::Relaxed)
        {
            update_flags(&mut next);
        }

        *stats.lock() = next;
        tick = tick.wrapping_add(1);
        std::thread::sleep(std::time::Duration::from_millis(FAST_INTERVAL_MS));
    }
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
