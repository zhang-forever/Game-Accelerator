use crate::core::command;
use crate::core::win_encoding;

/// Stop Windows Search indexing explicitly. This affects system search and is
/// not part of the automatic boost or its rollback session.
pub fn stop_windows_search_service() -> Result<(), String> {
    let output = command::run_hidden("net", &["stop", "WSearch"])?;

    if !output.status.success() {
        let detail = if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        return Err(win_encoding::friendly_error("暂停磁盘索引", detail));
    }
    Ok(())
}

/// Restart the Windows Search indexing service after it was paused.
pub fn start_windows_search_service() -> Result<(), String> {
    let output = command::run_hidden("net", &["start", "WSearch"])?;

    if !output.status.success() {
        let detail = if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        return Err(win_encoding::friendly_error("恢复磁盘索引", detail));
    }
    Ok(())
}

// Documented SERVICE_STATUS::dwCurrentState values; keep parsing usable in
// platform-independent tests without opening the service manager.
const SERVICE_STOPPED_STATE: u32 = 1;
const SERVICE_RUNNING_STATE: u32 = 4;

fn service_state_from_code(state: u32) -> Option<bool> {
    match state {
        SERVICE_RUNNING_STATE => Some(true),
        SERVICE_STOPPED_STATE => Some(false),
        _ => None,
    }
}

/// Query the actual service, rather than inferring it from an indexer process.
/// Pending/paused states are unknown; unavailable services/access return errors.
pub fn search_service_state() -> Result<Option<bool>, String> {
    #[cfg(windows)]
    {
        use std::ptr::null;
        use windows_sys::Win32::System::Services::{
            CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatus,
            SC_MANAGER_CONNECT, SERVICE_QUERY_STATUS, SERVICE_STATUS,
        };
        let name: Vec<u16> = "WSearch".encode_utf16().chain(Some(0)).collect();
        // SAFETY: Only connection/query rights are requested. SERVICE_STATUS has
        // plain DWORD fields, and both successfully opened handles are closed.
        unsafe {
            let manager = OpenSCManagerW(null(), null(), SC_MANAGER_CONNECT);
            if manager.is_null() {
                return Err(format!(
                    "无法读取服务管理器：{}",
                    std::io::Error::last_os_error()
                ));
            }
            let service = OpenServiceW(manager, name.as_ptr(), SERVICE_QUERY_STATUS);
            if service.is_null() {
                let error = std::io::Error::last_os_error();
                CloseServiceHandle(manager);
                return Err(format!("无法读取 Windows Search 服务：{error}"));
            }
            let mut status: SERVICE_STATUS = std::mem::zeroed();
            let queried = QueryServiceStatus(service, &mut status);
            let error = if queried == 0 {
                Some(std::io::Error::last_os_error())
            } else {
                None
            };
            CloseServiceHandle(service);
            CloseServiceHandle(manager);
            if let Some(error) = error {
                return Err(format!("无法读取 Windows Search 服务状态：{error}"));
            }
            Ok(service_state_from_code(status.dwCurrentState))
        }
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_transitions_are_not_reported_as_stopped() {
        assert_eq!(service_state_from_code(SERVICE_RUNNING_STATE), Some(true));
        assert_eq!(service_state_from_code(SERVICE_STOPPED_STATE), Some(false));
        for pending_or_paused in [2, 3, 5, 6, 7, 0] {
            assert_eq!(service_state_from_code(pending_or_paused), None);
        }
    }
}
