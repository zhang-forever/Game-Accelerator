/// Pin a process to specific CPU cores by setting its processor affinity mask.
/// A `core_mask` of `0xFFFFFFFF` assigns all cores; individual bits correspond
/// to core indices.
pub fn set_process_affinity(pid: u32, core_mask: u32) -> Result<(), String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "(Get-Process -Id {}).ProcessorAffinity = {}",
                pid, core_mask
            ),
        ])
        .output()
        .map_err(|e| format!("PowerShell failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to set affinity: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Disable CPU core parking via `powercfg` so that all logical processors remain
/// active even under light load. This prevents frequency throttling during
/// gaming when the OS parks idle cores.
pub fn disable_core_parking() -> Result<(), String> {
    use std::process::Command;

    let output = Command::new("powercfg")
        .args([
            "/setacvalueindex",
            "scheme_current",
            "SUB_PROCESSOR",
            "PROCTHROTTLEMIN",
            "100",
        ])
        .output()
        .map_err(|e| format!("powercfg failed: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let _ = Command::new("powercfg")
        .args(["/setactive", "scheme_current"])
        .output();

    Ok(())
}

/// Request the Windows multimedia timer resolution to be set to 1 ms via
/// `timeBeginPeriod`. A finer timer resolution reduces input latency in games,
/// though it slightly increases power consumption.
pub fn set_timer_resolution_fast() -> Result<(), String> {
    // Use PowerShell to call timeBeginPeriod via Add-Type
    let output = std::process::Command::new("powershell")
        .args([
            "-Command",
            r#"
            Add-Type -TypeDefinition '
            using System.Runtime.InteropServices;
            public class Timer {
                [DllImport("winmm.dll")]
                public static extern uint timeBeginPeriod(uint uMilliseconds);
            }
            ';
            [Timer]::timeBeginPeriod(1)
            "#,
        ])
        .output()
        .map_err(|e| format!("PowerShell failed: {}", e))?;

    // Even if it fails, it's not critical
    let _ = output;
    Ok(())
}
