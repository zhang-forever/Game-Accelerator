use sysinfo::{ProcessesToUpdate, System};

#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::EmptyWorkingSet;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
};

/// Empty the working set of every accessible process to release physical RAM
/// back to the system. Returns an estimate of the megabytes freed.
///
/// This replaces the old `rundll32 ...ProcessIdleTasks` call, which was slow and
/// triggered unrelated system maintenance (defrag, prefetch, etc.). Trimming each
/// working set directly is near-instant per process and only moves resident pages
/// to the standby list, so it is safe to apply broadly.
pub fn clean_all_process_memory() -> Result<u64, String> {
    let mut sys = System::new();
    sys.refresh_memory();
    let used_before = sys.used_memory();

    #[cfg(windows)]
    {
        sys.refresh_processes(ProcessesToUpdate::All);
        for pid in sys.processes().keys() {
            trim_working_set(pid.as_u32());
        }
        // Let the OS reclaim the trimmed pages before measuring the difference.
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    sys.refresh_memory();
    let used_after = sys.used_memory();

    let freed_mb = used_before.saturating_sub(used_after) / 1024 / 1024;
    Ok(freed_mb)
}

/// Trim a single process's working set. Failures (access denied on protected
/// processes) are expected and ignored.
#[cfg(windows)]
fn trim_working_set(pid: u32) {
    if pid == 0 {
        return;
    }
    // SAFETY: We open a handle with only the rights EmptyWorkingSet requires and
    // always close it. EmptyWorkingSet trims the target's resident pages only; it
    // cannot affect our own address space. A null handle means access was denied,
    // which we skip.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA, 0, pid);
        if handle.is_null() {
            return;
        }
        let _ = EmptyWorkingSet(handle);
        let _ = CloseHandle(handle);
    }
}
