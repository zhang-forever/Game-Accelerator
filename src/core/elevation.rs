use std::process::Command;

/// Returns true if the current process has administrator privileges.
/// Detected by attempting to read a HKLM key that requires elevation to write,
/// but more reliably by checking via `net session` which only works as admin.
pub fn is_elevated() -> bool {
    // `net session` returns exit code 0 only when run as administrator.
    Command::new("net")
        .arg("session")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Relaunch the current executable with administrator privileges via UAC.
/// Returns Ok(true) if a relaunch was triggered (caller should exit),
/// Ok(false) if already elevated (caller should continue normally).
pub fn elevate_if_needed() -> bool {
    if is_elevated() {
        return false;
    }

    if let Ok(exe) = std::env::current_exe() {
        let exe_path = exe.to_string_lossy();
        // Use PowerShell Start-Process with RunAs verb to trigger the UAC prompt.
        let result = Command::new("powershell")
            .args([
                "-WindowStyle",
                "Hidden",
                "-Command",
                &format!(
                    "Start-Process -FilePath '{}' -Verb RunAs",
                    exe_path.replace('\'', "''")
                ),
            ])
            .spawn();
        return result.is_ok();
    }
    false
}
