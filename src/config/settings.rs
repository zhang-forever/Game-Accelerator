use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CONFIG_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
const CONFIG_VERSION: u32 = 2;

/// A command-line override isolates portable installations and smoke tests.
pub fn set_config_directory(path: PathBuf) -> Result<(), String> {
    CONFIG_DIRECTORY
        .set(path)
        .map_err(|_| "配置目录已经初始化".to_string())
}

/// Missing fields inherit conservative defaults, including older configurations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(default)]
    pub config_version: u32,
    pub whitelist: HashSet<String>,
    pub blacklist: HashSet<String>,
    pub selected_game: Option<String>,
    pub enable_game_mode: bool,
    pub enable_high_perf_power: bool,
    pub kill_background_processes: bool,
    pub clean_memory: bool,
    pub set_high_priority: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_version: CONFIG_VERSION,
            whitelist: [
                "System",
                "Registry",
                "svchost.exe",
                "explorer.exe",
                "game-accelerator.exe",
                "MsMpEng.exe",
                "Discord.exe",
                "WeChat.exe",
                "QQ.exe",
                "OneDrive.exe",
                "Dropbox.exe",
                "GoogleDriveFS.exe",
                "VALORANT-Win64-Shipping.exe",
                "League of Legends.exe",
                "crossfire.exe",
                "RiotClientServices.exe",
                "vgc.exe",
                "vgtray.exe",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            // Only a user-selected list may be closed.
            blacklist: HashSet::new(),
            selected_game: None,
            enable_game_mode: true,
            enable_high_perf_power: true,
            kill_background_processes: false,
            clean_memory: false,
            set_high_priority: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let directory = CONFIG_DIRECTORY.get().cloned().unwrap_or_else(|| {
            dirs_next::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("GameAccelerator")
        });
        directory.join("config.toml")
    }

    pub fn load() -> Result<Self, String> {
        Self::load_from(&Self::config_path())
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::config_path())
    }

    fn load_from(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let mut config: Self = toml::from_str(&content).map_err(|e| {
                    format!("配置格式错误，原文件已保留（{}）：{}", path.display(), e)
                })?;
                if config.config_version < CONFIG_VERSION {
                    // v1 stored aggressive defaults without a schema version.
                    // Do not treat these defaults as an explicit choice to close apps.
                    config.kill_background_processes = false;
                    config.clean_memory = false;
                    config.set_high_priority = false;
                    config.config_version = CONFIG_VERSION;
                }
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("无法读取配置（{}）：{}", path.display(), e)),
        }
    }

    fn save_to(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self).map_err(|e| format!("无法生成配置：{}", e))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建配置目录（{}）：{}", parent.display(), e))?;
        }
        std::fs::write(path, content)
            .map_err(|e| format!("无法保存配置（{}）：{}", path.display(), e))
    }

    /// Competitive presets never opt into process manipulation.
    pub fn select_competitive_game(&mut self, exe: &str) {
        self.selected_game = Some(exe.to_string());
        self.kill_background_processes = false;
        self.clean_memory = false;
        self.set_high_priority = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_run_does_not_manipulate_processes() {
        let config = AppConfig::default();
        assert!(!config.kill_background_processes);
        assert!(!config.clean_memory);
        assert!(!config.set_high_priority);
        assert!(config.blacklist.is_empty());
    }

    #[test]
    fn partial_config_inherits_safe_defaults() {
        let config: AppConfig = toml::from_str("selected_game = 'crossfire.exe'").unwrap();
        assert_eq!(config.selected_game.as_deref(), Some("crossfire.exe"));
        assert!(!config.clean_memory);
        assert!(!config.kill_background_processes);
    }

    #[test]
    fn competitive_preset_disables_legacy_process_options() {
        let mut config = AppConfig {
            kill_background_processes: true,
            clean_memory: true,
            set_high_priority: true,
            ..Default::default()
        };
        config.select_competitive_game("VALORANT-Win64-Shipping.exe");
        assert!(!config.kill_background_processes);
        assert!(!config.clean_memory);
        assert!(!config.set_high_priority);
    }

    #[test]
    fn config_round_trip_keeps_settings() {
        let mut config = AppConfig::default();
        config.select_competitive_game("League of Legends.exe");
        let saved = toml::to_string_pretty(&config).unwrap();
        assert_eq!(toml::from_str::<AppConfig>(&saved).unwrap(), config);
    }

    #[test]
    fn malformed_config_is_not_overwritten() {
        let path = std::env::temp_dir().join(format!(
            "game-accelerator-invalid-config-{}.toml",
            std::process::id()
        ));
        std::fs::write(&path, "this is not toml").unwrap();
        assert!(AppConfig::load_from(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "this is not toml");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn legacy_config_does_not_silently_enable_process_manipulation() {
        let path = std::env::temp_dir().join(format!(
            "game-accelerator-legacy-config-{}.toml",
            std::process::id()
        ));
        let old = "selected_game = 'crossfire.exe'\nblacklist = ['OneDrive.exe']\nkill_background_processes = true\nclean_memory = true\nset_high_priority = true\n";
        std::fs::write(&path, old).unwrap();
        let config = AppConfig::load_from(&path).unwrap();
        assert!(!config.kill_background_processes);
        assert!(!config.clean_memory);
        assert!(!config.set_high_priority);
        assert_eq!(config.selected_game.as_deref(), Some("crossfire.exe"));
        assert!(config.blacklist.contains("OneDrive.exe"));
        assert_eq!(config.config_version, CONFIG_VERSION);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), old);
        std::fs::remove_file(path).unwrap();
    }
}
