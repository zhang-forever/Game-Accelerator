pub mod command;
pub mod disk_optimizer;
pub mod elevation;
pub mod game_mode;
pub mod gpu_manager;
pub mod power_manager;
pub mod process_category;
pub mod process_manager;
pub mod win_encoding;

use game_mode::RegistryValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoostResult {
    pub processes_killed: u32,
    pub memory_freed_mb: u64,
    pub power_plan_changed: bool,
    pub game_mode_enabled: bool,
    pub priority_boosted: bool,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
struct PowerChange {
    original: String,
    applied: String,
}

#[derive(Debug)]
struct GameModeChange {
    name: String,
    original: Option<RegistryValue>,
    applied: RegistryValue,
}

/// Only settings owned by this session are restored. No system operations run
/// on Drop; the UI explicitly handles restore and retry before closing.
#[derive(Debug, Default)]
pub struct BoostSession {
    power_plan: Option<PowerChange>,
    game_mode: Vec<GameModeChange>,
}

trait SessionBackend {
    fn active_power(&mut self) -> Result<String, String>;
    fn performance_power(&mut self) -> Result<String, String>;
    fn set_power(&mut self, guid: &str) -> Result<(), String>;
    fn read_game_mode(&mut self, name: &str) -> Result<Option<RegistryValue>, String>;
    fn write_game_mode(&mut self, name: &str, value: Option<&RegistryValue>) -> Result<(), String>;
}

struct WindowsBackend;

impl SessionBackend for WindowsBackend {
    fn active_power(&mut self) -> Result<String, String> {
        power_manager::get_active_power_plan_guid()
    }
    fn performance_power(&mut self) -> Result<String, String> {
        power_manager::performance_plan_guid()
    }
    fn set_power(&mut self, guid: &str) -> Result<(), String> {
        power_manager::set_active_power_plan(guid)
    }
    fn read_game_mode(&mut self, name: &str) -> Result<Option<RegistryValue>, String> {
        game_mode::read_game_mode_value(name)
    }
    fn write_game_mode(&mut self, name: &str, value: Option<&RegistryValue>) -> Result<(), String> {
        game_mode::write_game_mode_value(name, value)
    }
}

impl BoostSession {
    pub fn has_changes(&self) -> bool {
        self.power_plan.is_some() || !self.game_mode.is_empty()
    }

    /// Return only actual errors. Successfully restored settings, settings
    /// already back at their original value, and subsequent user changes are
    /// released from the session. Failed settings remain available for retry.
    pub fn restore(&mut self) -> Vec<String> {
        self.restore_with(&mut WindowsBackend)
    }

    fn restore_with(&mut self, backend: &mut impl SessionBackend) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(change) = &self.power_plan {
            let restore = (|| {
                let current = backend.active_power()?;
                // A different active scheme is now owned by the user.
                if !current.eq_ignore_ascii_case(&change.applied) {
                    return Ok(());
                }
                backend.set_power(&change.original)?;
                if !backend
                    .active_power()?
                    .eq_ignore_ascii_case(&change.original)
                {
                    return Err("恢复后电源方案与原方案不一致".to_string());
                }
                Ok(())
            })();
            match restore {
                Ok(()) => self.power_plan = None,
                Err(error) => errors.push(format!("恢复电源方案：{error}")),
            }
        }
        self.game_mode.retain(|change| {
            let restore = (|| {
                let current = backend.read_game_mode(&change.name)?;
                if current.as_ref() != Some(&change.applied) {
                    return Ok(());
                }
                backend.write_game_mode(&change.name, change.original.as_ref())?;
                if backend.read_game_mode(&change.name)? != change.original {
                    return Err("恢复后注册表设置与原值不一致".to_string());
                }
                Ok(())
            })();
            match restore {
                Ok(()) => false,
                Err(error) => {
                    errors.push(format!("恢复游戏模式 {}：{error}", change.name));
                    true
                }
            }
        });
        errors
    }

    fn enable_power(&mut self, backend: &mut impl SessionBackend, result: &mut BoostResult) {
        let operation = (|| {
            let original = backend.active_power()?;
            if original.eq_ignore_ascii_case(power_manager::HIGH_PERF_GUID)
                || original.eq_ignore_ascii_case(power_manager::ULTIMATE_GUID)
            {
                result
                    .messages
                    .push("当前已使用高性能电源方案，无需切换".to_string());
                return Ok(());
            }
            let applied = backend.performance_power()?;
            if original.eq_ignore_ascii_case(&applied) {
                return Ok(());
            }
            // Record before writing: even a failed post-write verification may
            // leave a changed scheme that the user must be able to restore.
            self.power_plan = Some(PowerChange {
                original,
                applied: applied.clone(),
            });
            backend.set_power(&applied)?;
            if !backend.active_power()?.eq_ignore_ascii_case(&applied) {
                return Err("电源方案未匹配预期值".to_string());
            }
            result.power_plan_changed = true;
            result
                .messages
                .push("已切换高性能电源方案，可在结束会话时恢复".to_string());
            Ok(())
        })();
        if let Err(error) = operation {
            result.errors.push(format!("电源方案：{error}"));
        }
    }

    fn enable_game_mode(&mut self, backend: &mut impl SessionBackend, result: &mut BoostResult) {
        // Capture every original value before the first write. Access failures
        // must never be interpreted as absent values or guessed defaults.
        let originals: Result<Vec<_>, _> = game_mode::GAME_MODE_VALUES
            .iter()
            .map(|name| {
                backend
                    .read_game_mode(name)
                    .map(|value| ((*name).to_string(), value))
            })
            .collect();
        let originals = match originals {
            Ok(values) => values,
            Err(error) => {
                result.errors.push(format!("读取原游戏模式设置：{error}"));
                return;
            }
        };
        let applied = RegistryValue::dword(1);
        let mut changed = false;
        let mut failed = false;
        for (name, original) in originals {
            if original.as_ref() == Some(&applied) {
                continue;
            }
            self.game_mode.push(GameModeChange {
                name: name.clone(),
                original,
                applied: applied.clone(),
            });
            let operation = backend
                .write_game_mode(&name, Some(&applied))
                .and_then(|_| {
                    if backend.read_game_mode(&name)?.as_ref() == Some(&applied) {
                        Ok(())
                    } else {
                        Err("写入后游戏模式值不匹配".to_string())
                    }
                });
            match operation {
                Ok(()) => changed = true,
                Err(error) => {
                    failed = true;
                    result.errors.push(format!("开启游戏模式 {name}：{error}"));
                }
            }
        }
        result.game_mode_enabled = changed && !failed;
        if !failed {
            result.messages.push(if changed {
                "已开启 Windows 游戏模式，可在结束会话时恢复".to_string()
            } else {
                "Windows 游戏模式已经开启，无需修改".to_string()
            });
        }
    }
}

/// Apply the configured trial options and return the rollback session to the UI.
/// Legacy memory/priority switches remain readable but cannot affect games.
pub fn run_boost(
    config: &crate::config::AppConfig,
    game_process: Option<&str>,
) -> (BoostResult, BoostSession) {
    let mut result = BoostResult::default();
    let mut session = BoostSession::default();
    if config.kill_background_processes {
        let mut whitelist = config.whitelist.clone();
        if let Some(game) = game_process {
            whitelist.insert(process_manager::normalize_exe_name(game));
        }
        match process_manager::kill_background_processes(&config.blacklist, &whitelist) {
            Ok(count) => {
                result.processes_killed = count;
                result.messages.push(format!(
                    "已关闭 {count} 个允许关闭的后台进程；受保护进程保持运行"
                ));
            }
            Err(error) => result.errors.push(format!("关闭后台进程：{error}")),
        }
    }
    if config.clean_memory {
        result
            .messages
            .push("已跳过全进程内存清理，避免裁剪游戏和反作弊工作集".to_string());
    }
    let mut backend = WindowsBackend;
    if config.enable_high_perf_power {
        session.enable_power(&mut backend, &mut result);
    }
    if config.enable_game_mode {
        session.enable_game_mode(&mut backend, &mut result);
    }
    if config.set_high_priority {
        result
            .messages
            .push("已跳过自动提升优先级，游戏、语音和反作弊保持系统调度".to_string());
    }
    (result, session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const ORIGINAL_GUID: &str = "01234567-89ab-cdef-0123-456789abcdef";
    const USER_GUID: &str = "11234567-89ab-cdef-0123-456789abcdef";

    struct MockBackend {
        power: String,
        values: HashMap<String, RegistryValue>,
        read_failure: Option<String>,
        write_failure: Option<String>,
        power_failure: bool,
        writes: Vec<String>,
    }

    impl Default for MockBackend {
        fn default() -> Self {
            Self {
                power: ORIGINAL_GUID.to_string(),
                values: HashMap::new(),
                read_failure: None,
                write_failure: None,
                power_failure: false,
                writes: Vec::new(),
            }
        }
    }

    impl SessionBackend for MockBackend {
        fn active_power(&mut self) -> Result<String, String> {
            if self.power_failure {
                Err("power unavailable".to_string())
            } else {
                Ok(self.power.clone())
            }
        }
        fn performance_power(&mut self) -> Result<String, String> {
            Ok(power_manager::HIGH_PERF_GUID.to_string())
        }
        fn set_power(&mut self, guid: &str) -> Result<(), String> {
            if self.power_failure {
                return Err("power write denied".to_string());
            }
            self.writes.push(format!("power:{guid}"));
            self.power = guid.to_string();
            Ok(())
        }
        fn read_game_mode(&mut self, name: &str) -> Result<Option<RegistryValue>, String> {
            if self.read_failure.as_deref() == Some(name) {
                Err("read denied".to_string())
            } else {
                Ok(self.values.get(name).cloned())
            }
        }
        fn write_game_mode(
            &mut self,
            name: &str,
            value: Option<&RegistryValue>,
        ) -> Result<(), String> {
            if self.write_failure.as_deref() == Some(name) {
                return Err("write denied".to_string());
            }
            self.writes.push(name.to_string());
            if let Some(value) = value {
                self.values.insert(name.to_string(), value.clone());
            } else {
                self.values.remove(name);
            }
            Ok(())
        }
    }

    #[test]
    fn restores_original_custom_plan_and_missing_registry_values() {
        let mut backend = MockBackend::default();
        let original_value = RegistryValue {
            kind: 1,
            bytes: vec![b'0', 0, 0, 0],
        };
        backend
            .values
            .insert("AllowAutoGameMode".to_string(), original_value.clone());
        let mut session = BoostSession::default();
        let mut result = BoostResult::default();
        session.enable_power(&mut backend, &mut result);
        session.enable_game_mode(&mut backend, &mut result);
        assert!(session.has_changes());
        assert!(result.power_plan_changed && result.game_mode_enabled);
        assert!(session.restore_with(&mut backend).is_empty());
        assert!(!session.has_changes());
        assert_eq!(backend.power, ORIGINAL_GUID);
        assert_eq!(
            backend.values.get("AllowAutoGameMode"),
            Some(&original_value)
        );
        assert!(!backend.values.contains_key("AutoGameModeEnabled"));
    }

    #[test]
    fn unreadable_snapshot_does_not_write_guessed_defaults() {
        let mut backend = MockBackend {
            power_failure: true,
            read_failure: Some("AutoGameModeEnabled".to_string()),
            ..Default::default()
        };
        let mut session = BoostSession::default();
        let mut result = BoostResult::default();
        session.enable_power(&mut backend, &mut result);
        session.enable_game_mode(&mut backend, &mut result);
        assert!(!session.has_changes());
        assert!(backend.writes.is_empty());
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn later_user_changes_are_preserved_and_released() {
        let mut backend = MockBackend::default();
        let mut session = BoostSession::default();
        let mut result = BoostResult::default();
        session.enable_power(&mut backend, &mut result);
        session.enable_game_mode(&mut backend, &mut result);
        backend.power = USER_GUID.to_string();
        backend
            .values
            .insert("AllowAutoGameMode".to_string(), RegistryValue::dword(0));
        assert!(session.restore_with(&mut backend).is_empty());
        assert!(!session.has_changes());
        assert_eq!(backend.power, USER_GUID);
        assert_eq!(
            backend.values.get("AllowAutoGameMode"),
            Some(&RegistryValue::dword(0))
        );
        assert!(!backend.values.contains_key("AutoGameModeEnabled"));
    }

    #[test]
    fn failed_restore_is_retained_for_retry() {
        let mut backend = MockBackend::default();
        let mut session = BoostSession::default();
        session.enable_power(&mut backend, &mut BoostResult::default());
        session.enable_game_mode(&mut backend, &mut BoostResult::default());
        backend.write_failure = Some("AllowAutoGameMode".to_string());
        let errors = session.restore_with(&mut backend);
        assert_eq!(errors.len(), 1);
        assert!(session.has_changes());
        assert_eq!(backend.power, ORIGINAL_GUID);
        assert_eq!(session.game_mode.len(), 1);
        backend.write_failure = None;
        assert!(session.restore_with(&mut backend).is_empty());
        assert!(!session.has_changes());
    }

    #[test]
    fn already_enabled_values_need_no_session() {
        let mut backend = MockBackend {
            power: power_manager::HIGH_PERF_GUID.to_ascii_uppercase(),
            ..Default::default()
        };
        for name in game_mode::GAME_MODE_VALUES {
            backend
                .values
                .insert(name.to_string(), RegistryValue::dword(1));
        }
        let mut session = BoostSession::default();
        let mut result = BoostResult::default();
        session.enable_power(&mut backend, &mut result);
        session.enable_game_mode(&mut backend, &mut result);
        assert!(!session.has_changes());
        assert!(backend.writes.is_empty());
        assert!(!result.power_plan_changed && !result.game_mode_enabled);
        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn partial_apply_can_restore_successful_changes() {
        let mut backend = MockBackend {
            write_failure: Some("AutoGameModeEnabled".to_string()),
            ..Default::default()
        };
        let mut session = BoostSession::default();
        let mut result = BoostResult::default();
        session.enable_game_mode(&mut backend, &mut result);
        assert!(!result.game_mode_enabled);
        assert_eq!(result.errors.len(), 1);
        assert!(session.has_changes());
        assert!(session.restore_with(&mut backend).is_empty());
        assert!(!session.has_changes());
        assert!(backend.values.is_empty());
    }

    #[test]
    fn legacy_memory_and_priority_options_cannot_affect_competitive_games() {
        let config = crate::config::AppConfig {
            kill_background_processes: false,
            enable_high_perf_power: false,
            enable_game_mode: false,
            clean_memory: true,
            set_high_priority: true,
            ..Default::default()
        };
        for game in [
            "VALORANT-Win64-Shipping.exe",
            "League of Legends.exe",
            "crossfire.exe",
        ] {
            let (result, session) = run_boost(&config, Some(game));
            assert!(result.errors.is_empty());
            assert!(!session.has_changes());
            assert!(!result.priority_boosted);
            assert_eq!(result.memory_freed_mb, 0);
            assert_eq!(result.messages.len(), 2);
        }
    }

    #[test]
    fn power_restore_read_failure_keeps_original_for_retry() {
        let mut backend = MockBackend::default();
        let mut session = BoostSession::default();
        session.enable_power(&mut backend, &mut BoostResult::default());
        backend.power_failure = true;
        assert_eq!(session.restore_with(&mut backend).len(), 1);
        assert!(session.has_changes());
        backend.power_failure = false;
        assert!(session.restore_with(&mut backend).is_empty());
        assert!(!session.has_changes());
        assert_eq!(backend.power, ORIGINAL_GUID);
    }
}
