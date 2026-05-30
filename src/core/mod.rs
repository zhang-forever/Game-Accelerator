pub mod process_manager;
pub mod process_category;
pub mod elevation;
pub mod memory_cleaner;
pub mod power_manager;
pub mod gpu_manager;
pub mod game_mode;
pub mod cpu_optimizer;
pub mod disk_optimizer;
pub mod win_encoding;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoostResult {
    pub processes_killed: u32,
    pub memory_freed_mb: u64,
    pub power_plan_changed: bool,
    pub game_mode_enabled: bool,
    pub priority_boosted: bool,
    pub errors: Vec<String>,
}

pub fn run_boost(
    config: &crate::config::AppConfig,
    game_process: Option<&str>,
) -> BoostResult {
    let mut result = BoostResult::default();

    // 1. Kill background processes
    if config.kill_background_processes {
        match process_manager::kill_background_processes(&config.blacklist, &config.whitelist) {
            Ok(count) => result.processes_killed = count,
            Err(e) => result.errors.push(format!("Process kill: {}", e)),
        }
    }

    // 2. Clean memory
    if config.clean_memory {
        match memory_cleaner::clean_all_process_memory() {
            Ok(freed) => result.memory_freed_mb = freed,
            Err(e) => result.errors.push(format!("Memory clean: {}", e)),
        }
    }

    // 3. Set power plan
    if config.enable_high_perf_power {
        match power_manager::set_high_performance() {
            Ok(_) => result.power_plan_changed = true,
            Err(e) => result.errors.push(format!("Power plan: {}", e)),
        }
    }

    // 4. Enable Game Mode
    if config.enable_game_mode {
        match game_mode::enable_game_mode() {
            Ok(_) => result.game_mode_enabled = true,
            Err(e) => result.errors.push(format!("Game mode: {}", e)),
        }
    }

    // 5. Set game priority
    if let Some(game) = game_process {
        if config.set_high_priority {
            match process_manager::set_process_high_priority(game) {
                Ok(_) => result.priority_boosted = true,
                Err(e) => result.errors.push(format!("Priority: {}", e)),
            }
        }
    }

    result
}
