use crate::core::{game_mode, gpu_manager, power_manager};
use parking_lot::Mutex;
use std::sync::Arc;
use sysinfo::System;

#[derive(Clone, Debug, Default)]
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

    pub process_count: usize,

    // System optimization toggle states. Each of these requires spawning an
    // external process (powercfg / reg query / nvidia-smi) to determine, so they
    // are refreshed on a slower cadence in this background thread instead of in
    // the UI render path. The UI reads these cached values every frame.
    pub flags_ready: bool,
    pub power_high_perf: bool,
    pub game_mode_on: bool,
    pub hw_gpu_sched_on: bool,
    pub game_bar_on: bool,
    pub search_indexer_running: bool,
}

/// How often the cheap in-process metrics (CPU, RAM, process count) refresh.
const FAST_INTERVAL_MS: u64 = 500;
/// Expensive metrics that shell out to external commands refresh once every
/// this many fast ticks (4 * 500ms = ~2s) to keep process spawning off the hot path.
const SLOW_EVERY_N_TICKS: u64 = 4;

pub fn run_monitor(stats: Arc<Mutex<SystemStats>>) {
    let mut sys = System::new();
    let mut tick: u64 = 0;

    loop {
        sys.refresh_memory();
        sys.refresh_cpu_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        // ----- Fast, cheap metrics (every tick) -----
        let cpu_total = sys.global_cpu_usage();
        let cpu_per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let total_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_gb = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let proc_count = sys.processes().len();
        // Reuse the already-refreshed process table instead of opening a second System.
        let indexer_running = sys.processes_by_name("SearchIndexer".as_ref()).count() > 0;

        // ----- Slow metrics that shell out (throttled) -----
        let slow = if tick % SLOW_EVERY_N_TICKS == 0 {
            Some(query_slow())
        } else {
            None
        };

        {
            let mut current = stats.lock();

            current.cpu_usage_total = cpu_total;
            current.cpu_usage_per_core = cpu_per_core;
            if current.cpu_name.is_empty() {
                current.cpu_name = sys
                    .cpus()
                    .first()
                    .map(|c| c.brand().to_string())
                    .unwrap_or_default();
                current.cpu_cores = sys.physical_core_count().unwrap_or(0);
                current.cpu_threads = sys.cpus().len();
            }

            current.ram_total_gb = (total_gb * 100.0).round() / 100.0;
            current.ram_used_gb = (used_gb * 100.0).round() / 100.0;
            current.ram_usage_percent = if total_gb > 0.0 {
                (used_gb / total_gb * 100.0) as f32
            } else {
                0.0
            };

            current.process_count = proc_count;
            current.search_indexer_running = indexer_running;

            if let Some(s) = slow {
                current.gpu_name = s.gpu_name;
                current.gpu_driver = s.gpu_driver;
                current.gpu_usage = s.gpu_usage;
                current.gpu_temp = s.gpu_temp;
                current.gpu_mem_used_mb = s.gpu_mem_used_mb;
                current.gpu_mem_total_mb = s.gpu_mem_total_mb;
                current.power_high_perf = s.power_high_perf;
                current.game_mode_on = s.game_mode_on;
                current.hw_gpu_sched_on = s.hw_gpu_sched_on;
                current.game_bar_on = s.game_bar_on;
                current.flags_ready = true;
            }
        }

        tick = tick.wrapping_add(1);
        std::thread::sleep(std::time::Duration::from_millis(FAST_INTERVAL_MS));
    }
}

/// Results of the expensive, external-command-backed queries.
struct SlowStats {
    gpu_name: String,
    gpu_driver: String,
    gpu_usage: f32,
    gpu_temp: f32,
    gpu_mem_used_mb: u64,
    gpu_mem_total_mb: u64,
    power_high_perf: bool,
    game_mode_on: bool,
    hw_gpu_sched_on: bool,
    game_bar_on: bool,
}

/// Run every external command once. Called only on slow ticks, off the UI thread.
fn query_slow() -> SlowStats {
    let gpu = gpu_manager::get_gpu_info().into_iter().next();
    let (gpu_name, gpu_driver, gpu_usage, gpu_temp, gpu_mem_used_mb, gpu_mem_total_mb) = match gpu {
        Some(g) => (
            g.name,
            g.driver_version,
            g.usage_percent,
            g.temperature,
            g.memory_used_mb,
            g.memory_total_mb,
        ),
        None => (String::new(), String::new(), 0.0, 0.0, 0, 0),
    };

    SlowStats {
        gpu_name,
        gpu_driver,
        gpu_usage,
        gpu_temp,
        gpu_mem_used_mb,
        gpu_mem_total_mb,
        power_high_perf: power_manager::is_high_performance(),
        game_mode_on: game_mode::is_game_mode_enabled(),
        hw_gpu_sched_on: game_mode::is_hardware_gpu_scheduling_enabled(),
        game_bar_on: game_mode::is_game_bar_enabled(),
    }
}
