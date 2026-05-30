use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub whitelist: HashSet<String>,
    pub blacklist: HashSet<String>,
    pub auto_boost: bool,
    pub minimize_to_tray: bool,
    pub selected_game: Option<String>,
    pub enable_game_mode: bool,
    pub enable_high_perf_power: bool,
    pub kill_background_processes: bool,
    pub clean_memory: bool,
    pub set_high_priority: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut whitelist = HashSet::new();
        // System-critical processes that must never be killed
        for name in &[
            "System",
            "Registry",
            "smss.exe",
            "csrss.exe",
            "wininit.exe",
            "winlogon.exe",
            "services.exe",
            "lsass.exe",
            "svchost.exe",
            "dwm.exe",
            "explorer.exe",
            "fontdrvhost.exe",
            "SearchHost.exe",
            "sihost.exe",
            "taskhostw.exe",
            "RuntimeBroker.exe",
            "game-accelerator.exe",
            "Game Accelerator",
        ] {
            whitelist.insert(name.to_string());
        }

        let mut blacklist = HashSet::new();
        // Common background processes to kill on boost
        for name in &[
            "OneDrive.exe",
            "Dropbox.exe",
            "GoogleDriveFS.exe",
            "AdobeARM.exe",
            "AdobeUpdateService.exe",
            "AGSService.exe",
            "GoogleUpdate.exe",
            "MicrosoftEdgeUpdate.exe",
            "Teams.exe",
            "ms-teams.exe",
            "SearchApp.exe",
            "SearchIndexer.exe",
            "Cortana.exe",
            "YourPhone.exe",
            "PhoneExperienceHost.exe",
            "Widgets.exe",
            "WidgetService.exe",
            "SecurityHealthService.exe",
            "MpCmdRun.exe",
            "iTunesHelper.exe",
            "iCloudServices.exe",
            "Spotify.exe",
            "Discord.exe",
            "SkypeApp.exe",
            "SkypeBackgroundHost.exe",
            "GameBarPresenceWriter.exe",
            "gamingservices.exe",
            "gamingservicesnet.exe",
        ] {
            blacklist.insert(name.to_string());
        }

        Self {
            whitelist,
            blacklist,
            auto_boost: false,
            minimize_to_tray: true,
            selected_game: None,
            enable_game_mode: true,
            enable_high_perf_power: true,
            kill_background_processes: true,
            clean_memory: true,
            set_high_priority: true,
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let mut path = dirs_next::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("GameAccelerator");
        std::fs::create_dir_all(&path).ok();
        path.push("config.toml");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(content) = toml::to_string_pretty(self) {
            std::fs::write(path, content).ok();
        }
    }
}
