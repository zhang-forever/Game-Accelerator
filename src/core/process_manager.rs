use std::collections::HashSet;
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Hard protection takes precedence over every user blacklist and UI action.
const SYSTEM_PROCESSES: &[&str] = &[
    "system",
    "system idle process",
    "registry",
    "memory compression",
    "secure system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "lsm.exe",
    "svchost.exe",
    "fontdrvhost.exe",
    "wudfhost.exe",
    "dwm.exe",
    "explorer.exe",
    "ctfmon.exe",
    "sihost.exe",
    "taskhostw.exe",
    "runtimebroker.exe",
    "shellexperiencehost.exe",
    "startmenuexperiencehost.exe",
    "searchhost.exe",
    "applicationframehost.exe",
    "textinputhost.exe",
    "dllhost.exe",
    "conhost.exe",
    "audiodg.exe",
    "spoolsv.exe",
    "game-accelerator.exe",
    "msmpeng.exe",
    "mssense.exe",
    "nissrv.exe",
    "msmpengcp.exe",
    "mpcmdrun.exe",
    "securityhealthservice.exe",
    "securityhealthsystray.exe",
    "sgrmbroker.exe",
];

const COMPETITIVE_GAME_PROCESSES: &[&str] = &[
    "valorant.exe",
    "valorant-win64-shipping.exe",
    "league of legends.exe",
    "leagueclient.exe",
    "leagueclientux.exe",
    "leagueclientuxrender.exe",
    "lol.exe",
    "lolclient.exe",
    "crossfire.exe",
    "crossfire_x64.exe",
    "cf.exe",
    "cf64.exe",
    "client.exe",
    "client64.exe",
    "tcls.exe",
    "tclsclient.exe",
];

const GAME_SUPPORT_PROCESSES: &[&str] = &[
    "riotclientservices.exe",
    "riotclientux.exe",
    "riotclientuxrender.exe",
    "riotclientcrashhandler.exe",
    "riotcrashhandler.exe",
    "vgc.exe",
    "vgtray.exe",
    "vanguard.exe",
    "wegame.exe",
    "wegamehelper.exe",
    "wegameservice.exe",
    "tgp.exe",
    "tgp_daemon.exe",
    "tencentgames.exe",
    "qqlogin.exe",
    "tensafe.exe",
    "tensafe_1.exe",
    "tensafe_2.exe",
    "tensafe64.exe",
    "tensafe_64.exe",
    "tenprotect.exe",
    "tphelper.exe",
    "tesvc.exe",
    "ace-service.exe",
    "aceservice.exe",
    "ace-guard.exe",
    "aceguard.exe",
    "ace-tray.exe",
    "anticheatexpert.exe",
    "easyanticheat.exe",
    "easyanticheat_eos.exe",
    "beservice.exe",
    "beservice_x64.exe",
    "eac_launcher.exe",
    "gamemon.exe",
    "gamemon64.exe",
    "steam.exe",
    "steamwebhelper.exe",
    "steamservice.exe",
    "epicgameslauncher.exe",
    "epicwebhelper.exe",
    "battle.net.exe",
    "agent.exe",
    "eadesktop.exe",
    "origin.exe",
    "ubisoftconnect.exe",
    "uplay.exe",
];

const VOICE_PROCESSES: &[&str] = &[
    "discord.exe",
    "qq.exe",
    "qqex.exe",
    "qqnt.exe",
    "tim.exe",
    "wechat.exe",
    "weixin.exe",
    "wechatapp.exe",
    "wechatappex.exe",
    "yy.exe",
    "yyclient.exe",
    "yylauncher.exe",
    "yysv.exe",
    "kook.exe",
    "kaiheila.exe",
    "teamspeak.exe",
    "ts3client_win64.exe",
    "ts3client_win32.exe",
    "mumble.exe",
    "overwolf.exe",
    "overwolfbrowser.exe",
];

/// Accept a process basename or an executable path, preserving embedded spaces.
pub fn normalize_exe_name(name: &str) -> String {
    name.trim()
        .trim_matches('"')
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

fn listed(name: &str, names: &[&str]) -> bool {
    let normalized = normalize_exe_name(name);
    names.iter().any(|candidate| {
        normalized == *candidate
            || (!normalized.contains('.')
                && candidate.strip_suffix(".exe") == Some(normalized.as_str()))
    })
}

/// Only OS/security processes, for category labels that still distinguish games
/// and voice tools from the Windows system category.
pub fn is_protected_system_process(name: &str) -> bool {
    listed(name, SYSTEM_PROCESSES)
}

pub fn is_competitive_game_process(name: &str) -> bool {
    listed(name, COMPETITIVE_GAME_PROCESSES)
}

pub fn is_protected_process(name: &str) -> bool {
    is_protected_system_process(name)
        || is_competitive_game_process(name)
        || listed(name, GAME_SUPPORT_PROCESSES)
        || listed(name, VOICE_PROCESSES)
        // Service suffixes differ among Tencent game/anti-cheat releases.
        || normalize_exe_name(name).starts_with("tensafe")
        || normalize_exe_name(name).starts_with("ace-")
}

fn validate_process_identity(expected: &str, current: &str) -> Result<(), String> {
    if is_protected_process(expected) || is_protected_process(current) {
        return Err(format!(
            "{current} 属于受保护的系统、游戏、反作弊或语音进程，已阻止操作"
        ));
    }
    if normalize_exe_name(expected) != normalize_exe_name(current) {
        return Err("进程身份已改变，请刷新进程列表后重试".to_string());
    }
    Ok(())
}

/// All batch actions use the same PID/name-checked close path as a single close.
pub fn kill_background_processes(
    blacklist: &HashSet<String>,
    whitelist: &HashSet<String>,
) -> Result<u32, String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);
    let blacklist: HashSet<String> = blacklist.iter().map(|n| normalize_exe_name(n)).collect();
    let whitelist: HashSet<String> = whitelist.iter().map(|n| normalize_exe_name(n)).collect();
    let mut killed = 0;
    let mut failed = 0;
    for (pid, process) in sys.processes() {
        let name = normalize_exe_name(&process.name().to_string_lossy());
        if !is_protected_process(&name) && blacklist.contains(&name) && !whitelist.contains(&name) {
            if kill_process_by_pid(pid.as_u32(), &name).is_ok() {
                killed += 1;
            } else {
                failed += 1;
            }
        }
    }
    if failed > 0 {
        return Err(format!(
            "已关闭 {killed} 个进程；{failed} 个进程未能关闭或身份已改变"
        ));
    }
    Ok(killed)
}

/// Resolve the PID again, then verify the image on the same native handle used
/// to terminate it. A stale UI row must never authorize killing a reused PID.
pub fn kill_process_by_pid(pid: u32, name: &str) -> Result<(), String> {
    if pid == 0 || pid == std::process::id() || is_protected_process(name) {
        return Err(format!("{name} 是受保护进程，已阻止关闭"));
    }
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]));
    let process = sys
        .process(Pid::from_u32(pid))
        .ok_or_else(|| format!("进程 {name} (PID {pid}) 已不存在"))?;
    validate_process_identity(name, &process.name().to_string_lossy())?;

    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
        };
        // SAFETY: Query and terminate refer to the same handle; every opened
        // handle is closed, including when the image identity check fails.
        unsafe {
            let handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE,
                0,
                pid,
            );
            if handle.is_null() {
                return Err(format!(
                    "无法打开进程 {name}：{}",
                    std::io::Error::last_os_error()
                ));
            }
            let result = process_image_name(handle)
                .and_then(|current| validate_process_identity(name, &current))
                .and_then(|_| {
                    if TerminateProcess(handle, 1) != 0 {
                        Ok(())
                    } else {
                        Err(format!(
                            "无法关闭 {name}：{}",
                            std::io::Error::last_os_error()
                        ))
                    }
                });
            CloseHandle(handle);
            result
        }
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows".to_string())
    }
}

#[cfg(windows)]
unsafe fn process_image_name(
    handle: windows_sys::Win32::Foundation::HANDLE,
) -> Result<String, String> {
    use windows_sys::Win32::System::Threading::QueryFullProcessImageNameW;
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    if QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) == 0 {
        return Err(format!(
            "无法核实进程身份：{}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(normalize_exe_name(&String::from_utf16_lossy(
        &buffer[..length as usize],
    )))
}

pub fn get_process_list() -> Vec<ProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);
    // Process CPU counters need two samples; the UI calls this on a worker.
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_processes(ProcessesToUpdate::All);
    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let name = process.name().to_string_lossy().into_owned();
            ProcessInfo {
                is_protected: is_protected_process(&name) || pid.as_u32() == std::process::id(),
                name,
                pid: pid.as_u32(),
                cpu_usage: process.cpu_usage(),
                memory_mb: process.memory() / 1024 / 1024,
            }
        })
        .collect();
    processes.sort_by_key(|process| std::cmp::Reverse(process.memory_mb));
    processes
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub is_protected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_accept_paths_case_and_spaces() {
        assert_eq!(
            normalize_exe_name(r#" "C:\Games\League of Legends.exe" "#),
            "league of legends.exe"
        );
        assert!(is_competitive_game_process("VALORANT-Win64-Shipping.EXE"));
        assert!(is_protected_system_process("MsMpEng"));
    }

    #[test]
    fn game_security_and_voice_are_hard_protected() {
        for name in [
            "system",
            "MsMpEng.exe",
            "LeagueClientUxRender.exe",
            "CrossFire.exe",
            "vgc.exe",
            "TenSafe_2.exe",
            "ACE-Service64.exe",
            "Discord.exe",
            "QQ.exe",
            "YY.exe",
            "WeGame.exe",
        ] {
            assert!(is_protected_process(name), "{name}");
        }
        assert!(!is_protected_process("chrome.exe"));
    }

    #[test]
    fn stale_row_cannot_authorize_different_or_protected_image() {
        assert!(validate_process_identity("chrome.exe", "msmpeng.exe").is_err());
        assert!(validate_process_identity("chrome.exe", "notepad.exe").is_err());
        assert!(validate_process_identity("VALORANT.exe", "valorant.exe").is_err());
        assert!(validate_process_identity("chrome.exe", r"C:\Browser\CHROME.EXE").is_ok());
    }
}
