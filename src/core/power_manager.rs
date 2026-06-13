/// High Performance power plan GUID
const HIGH_PERF_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
/// Ultimate Performance power plan GUID
const ULTIMATE_GUID: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

/// Balanced power plan GUID
const BALANCED_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

/// Activate the High Performance or Ultimate Performance power plan. Prefers
/// Ultimate Performance if available on the system, otherwise falls back to
/// High Performance. Requires administrator privileges.
pub fn set_high_performance() -> Result<(), String> {
    use std::process::Command;

    // Try Ultimate Performance first, fall back to High Performance
    let guid = if power_plan_exists(ULTIMATE_GUID) {
        ULTIMATE_GUID
    } else {
        HIGH_PERF_GUID
    };

    let output = Command::new("powercfg")
        .args(["/setactive", guid])
        .output()
        .map_err(|e| format!("Failed to run powercfg: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "powercfg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Revert the active power plan to the Windows Balanced plan. Useful for
/// restoring normal power behavior after a gaming session.
pub fn set_balanced() -> Result<(), String> {
    use std::process::Command;
    let output = Command::new("powercfg")
        .args(["/setactive", BALANCED_GUID])
        .output()
        .map_err(|e| format!("Failed to run powercfg: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "powercfg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// True if currently on a High or Ultimate Performance plan.
pub fn is_high_performance() -> bool {
    let plan = get_active_power_plan();
    plan.contains("High") || plan.contains("Ultimate")
}

/// Query the currently active power plan and return a human-readable name
/// such as "High Performance", "Ultimate Performance", or "Balanced".
pub fn get_active_power_plan() -> String {
    use std::process::Command;
    if let Ok(output) = Command::new("powercfg").args(["/getactivescheme"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        if text.contains(HIGH_PERF_GUID) || text.contains("High performance") {
            return "High Performance".to_string();
        } else if text.contains(ULTIMATE_GUID) || text.contains("Ultimate") {
            return "Ultimate Performance".to_string();
        } else if text.contains("Balanced") || text.contains("平衡") {
            return "Balanced".to_string();
        }
        return text.trim().to_string();
    }
    "Unknown".to_string()
}

fn power_plan_exists(guid: &str) -> bool {
    use std::process::Command;
    if let Ok(output) = Command::new("powercfg").args(["/list"]).output() {
        let text = String::from_utf8_lossy(&output.stdout);
        text.contains(guid)
    } else {
        false
    }
}
