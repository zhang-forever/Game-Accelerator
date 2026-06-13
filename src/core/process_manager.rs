use std::collections::HashSet;
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Critical Windows processes that must NEVER be killed, regardless of config.
/// Killing any of these can freeze, crash, or BSOD the system.
const PROTECTED_PROCESSES: &[&str] = &[
    // Core kernel / session
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
    // Shell / desktop
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
    // Our own app
    "game-accelerator.exe",
];

/// Returns true if the process name is a protected Windows system process.
pub fn is_protected_system_process(name: &str) -> bool {
    let lower = name.to_lowercase();
    PROTECTED_PROCESSES.contains(&lower.as_str())
}

/// Kill every process whose name is in the `blacklist`, skipping any process
/// that is in the `whitelist` or is a protected system process. Returns the
/// number of processes successfully terminated.
pub fn kill_background_processes(
    blacklist: &HashSet<String>,
    whitelist: &HashSet<String>,
) -> Result<u32, String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);

    let mut killed = 0u32;
    let blacklist_lower: HashSet<String> = blacklist.iter().map(|n| n.to_lowercase()).collect();
    let whitelist_lower: HashSet<String> = whitelist.iter().map(|n| n.to_lowercase()).collect();

    for (_pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();

        // Hard protection: never touch system-critical processes
        if is_protected_system_process(&name) {
            continue;
        }

        if whitelist_lower.contains(&name) {
            continue;
        }

        if blacklist_lower.contains(&name) {
            if process.kill() {
                killed += 1;
            }
        }
    }

    Ok(killed)
}

/// Kill a single process by PID. Refuses to kill protected system processes.
pub fn kill_process_by_pid(pid: u32, name: &str) -> Result<(), String> {
    if is_protected_system_process(name) {
        return Err(format!("{} 是系统关键进程，已阻止关闭", name));
    }

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);

    match sys.process(Pid::from_u32(pid)) {
        Some(process) => {
            if process.kill() {
                Ok(())
            } else {
                Err(format!(
                    "无法关闭 {} (PID {})，可能需要管理员权限",
                    name, pid
                ))
            }
        }
        None => Err(format!("进程 {} (PID {}) 已不存在", name, pid)),
    }
}

/// Return a list of all running processes sorted by memory usage (descending).
/// Each entry includes the process name, PID, CPU usage, memory footprint, and
/// whether it is a protected system process.
pub fn get_process_list() -> Vec<ProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);
    sys.refresh_memory();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, p)| {
            let name = p.name().to_string_lossy().to_string();
            let is_protected = is_protected_system_process(&name);
            ProcessInfo {
                name,
                pid: pid.as_u32(),
                cpu_usage: p.cpu_usage(),
                memory_mb: p.memory() / 1024 / 1024,
                is_protected,
            }
        })
        .collect();

    processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
    processes
}

/// Information about a single running process, used by the process management
/// UI to display stats and per-process kill actions.
#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub is_protected: bool,
}

/// Set the priority class of all running processes matching `process_name` to
/// `HIGH_PRIORITY_CLASS`. This is useful for boosting the game executable so
/// the OS allocates CPU time to it preferentially.
pub fn set_process_high_priority(process_name: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, SetPriorityClass, HIGH_PRIORITY_CLASS, PROCESS_SET_INFORMATION,
        };

        let target = process_name.to_lowercase();
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All);

        let mut found = false;
        let mut set_any = false;

        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_lowercase();
            if name != target {
                continue;
            }
            found = true;

            // SAFETY: We open the process with only PROCESS_SET_INFORMATION, set its
            // priority class, and always close the handle. A null handle means the
            // process could not be opened (e.g. insufficient rights) and is skipped.
            unsafe {
                let handle = OpenProcess(PROCESS_SET_INFORMATION, 0, pid.as_u32());
                if handle.is_null() {
                    continue;
                }
                if SetPriorityClass(handle, HIGH_PRIORITY_CLASS) != 0 {
                    set_any = true;
                }
                let _ = CloseHandle(handle);
            }
        }

        if !found {
            return Err(format!("未找到进程 {}（游戏可能还没启动）", process_name));
        }
        if !set_any {
            return Err(format!(
                "无法提升 {} 优先级，可能需要管理员权限",
                process_name
            ));
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = process_name;
        Err("仅支持 Windows".to_string())
    }
}
