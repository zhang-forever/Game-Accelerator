use crate::app::ProcessInfo;
use crate::core::process_manager;

/// A user-friendly category of processes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Browser,
    Chat,
    Office,
    CloudSync,
    Updater,
    Media,
    GameLauncher,
    System,
    Other,
}

impl Category {
    /// Return the Chinese display name for this category, used in the process
    /// management UI as the heading for grouped process lists.
    pub fn display_name(&self) -> &'static str {
        match self {
            Category::Browser => "浏览器",
            Category::Chat => "聊天 / 通讯",
            Category::Office => "办公软件",
            Category::CloudSync => "网盘 / 同步",
            Category::Updater => "后台更新程序",
            Category::Media => "音乐 / 视频",
            Category::GameLauncher => "游戏平台",
            Category::System => "系统进程",
            Category::Other => "其他程序",
        }
    }
}

/// Classify a process by its executable name.
pub fn classify(name: &str) -> Category {
    let n = process_manager::normalize_exe_name(name);

    if process_manager::is_protected_system_process(&n) {
        return Category::System;
    }

    // Browsers
    const BROWSERS: &[&str] = &[
        "chrome.exe",
        "msedge.exe",
        "msedgewebview2.exe",
        "firefox.exe",
        "opera.exe",
        "brave.exe",
        "iexplore.exe",
        "360se.exe",
        "360chrome.exe",
        "qqbrowser.exe",
        "sogouexplorer.exe",
        "ucbrowser.exe",
        "vivaldi.exe",
    ];
    if BROWSERS.contains(&n.as_str()) {
        return Category::Browser;
    }

    // Chat / communication
    const CHAT: &[&str] = &[
        "wechat.exe",
        "weixin.exe",
        "wechatapp.exe",
        "wechatappex.exe",
        "qq.exe",
        "qqex.exe",
        "tim.exe",
        "discord.exe",
        "telegram.exe",
        "dingtalk.exe",
        "feishu.exe",
        "lark.exe",
        "slack.exe",
        "skype.exe",
        "skypeapp.exe",
        "ms-teams.exe",
        "teams.exe",
        "whatsapp.exe",
    ];
    if CHAT.contains(&n.as_str()) {
        return Category::Chat;
    }

    // Office
    const OFFICE: &[&str] = &[
        "winword.exe",
        "excel.exe",
        "powerpnt.exe",
        "outlook.exe",
        "wps.exe",
        "et.exe",
        "wpp.exe",
        "wpscloudsvr.exe",
        "wpscenter.exe",
        "onenote.exe",
        "acrobat.exe",
        "acrord32.exe",
        "pdfreader.exe",
    ];
    if OFFICE.contains(&n.as_str()) {
        return Category::Office;
    }

    // Cloud sync
    const CLOUD: &[&str] = &[
        "onedrive.exe",
        "dropbox.exe",
        "googledrivefs.exe",
        "baidunetdisk.exe",
        "baidunetdiskhost.exe",
        "weiyun.exe",
        "nutstore.exe",
        "icloudservices.exe",
        "icloud.exe",
        "megasync.exe",
        "syncthing.exe",
    ];
    if CLOUD.contains(&n.as_str()) {
        return Category::CloudSync;
    }

    // Updaters
    const UPDATERS: &[&str] = &[
        "googleupdate.exe",
        "microsoftedgeupdate.exe",
        "adobearm.exe",
        "adobeupdateservice.exe",
        "agsservice.exe",
        "jusched.exe",
        "softwareupdate.exe",
        "updater.exe",
        "update.exe",
        "su.exe",
        "tencentdl.exe",
        "qqpcrtp.exe",
        "360tray.exe",
        "360safe.exe",
    ];
    if UPDATERS.contains(&n.as_str()) {
        return Category::Updater;
    }

    // Media
    const MEDIA: &[&str] = &[
        "spotify.exe",
        "cloudmusic.exe",
        "qqmusic.exe",
        "kugou.exe",
        "kwmusic.exe",
        "potplayer.exe",
        "potplayermini64.exe",
        "vlc.exe",
        "iqiyi.exe",
        "qiyiclient.exe",
        "youku.exe",
        "bilibili.exe",
        "itunes.exe",
        "foobar2000.exe",
    ];
    if MEDIA.contains(&n.as_str()) {
        return Category::Media;
    }

    // Game launchers
    const LAUNCHERS: &[&str] = &[
        "steam.exe",
        "steamwebhelper.exe",
        "epicgameslauncher.exe",
        "battle.net.exe",
        "origin.exe",
        "eadesktop.exe",
        "uplay.exe",
        "ubisoftconnect.exe",
        "gog.exe",
        "galaxyclient.exe",
        "wegame.exe",
        "riotclientservices.exe",
    ];
    if LAUNCHERS.contains(&n.as_str()) {
        return Category::GameLauncher;
    }

    Category::Other
}

/// Aggregated info for one category.
pub struct CategoryGroup {
    pub category: Category,
    pub process_count: usize,
    pub total_memory_mb: u64,
    pub total_cpu: f32,
    pub process_names: Vec<String>,
}

/// Group the current process list into categories, sorted by memory usage.
pub fn group_processes(processes: &[ProcessInfo]) -> Vec<CategoryGroup> {
    use std::collections::HashMap;

    let mut map: HashMap<Category, CategoryGroup> = HashMap::new();

    for p in processes {
        let cat = classify(&p.name);
        let entry = map.entry(cat).or_insert_with(|| CategoryGroup {
            category: cat,
            process_count: 0,
            total_memory_mb: 0,
            total_cpu: 0.0,
            process_names: Vec::new(),
        });
        entry.process_count += 1;
        entry.total_memory_mb += p.memory_mb;
        entry.total_cpu += p.cpu_usage;
        if !entry.process_names.contains(&p.name) {
            entry.process_names.push(p.name.clone());
        }
    }

    let mut groups: Vec<CategoryGroup> = map.into_values().collect();
    // System last, then by memory descending
    groups.sort_by(|a, b| {
        match (
            a.category == Category::System,
            b.category == Category::System,
        ) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => b.total_memory_mb.cmp(&a.total_memory_mb),
        }
    });
    groups
}
