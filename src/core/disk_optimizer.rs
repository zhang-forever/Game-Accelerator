use crate::core::win_encoding;
use std::process::Command;

pub fn stop_windows_search_service() -> Result<(), String> {
    let output = Command::new("net")
        .args(["stop", "WSearch"])
        .output()
        .map_err(|e| format!("Failed to stop WSearch: {}", e))?;

    if !output.status.success() {
        return Err(win_encoding::friendly_error("暂停磁盘索引", &output.stderr));
    }
    Ok(())
}

pub fn start_windows_search_service() -> Result<(), String> {
    let output = Command::new("net")
        .args(["start", "WSearch"])
        .output()
        .map_err(|e| format!("Failed to start WSearch: {}", e))?;

    if !output.status.success() {
        return Err(win_encoding::friendly_error("恢复磁盘索引", &output.stderr));
    }
    Ok(())
}

pub fn flush_file_cache() -> Result<(), String> {
    // Flush file system cache via PowerShell
    let _ = Command::new("powershell")
        .args([
            "-Command",
            "[System.GC]::Collect(); [System.GC]::WaitForPendingFinalizers()",
        ])
        .output();
    Ok(())
}

pub fn is_search_indexer_running() -> bool {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);
    sys.processes_by_name("SearchIndexer".as_ref()).count() > 0
}
