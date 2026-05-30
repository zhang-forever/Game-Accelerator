use crate::core::win_encoding;
use std::process::Command;

pub fn enable_game_mode() -> Result<(), String> {
    set_reg_dword("HKCU\\Software\\Microsoft\\GameBar", "AllowAutoGameMode", 1)?;
    set_reg_dword(
        "HKCU\\Software\\Microsoft\\GameBar",
        "AutoGameModeEnabled",
        1,
    )?;
    Ok(())
}

pub fn disable_game_mode() -> Result<(), String> {
    set_reg_dword("HKCU\\Software\\Microsoft\\GameBar", "AllowAutoGameMode", 0)?;
    set_reg_dword(
        "HKCU\\Software\\Microsoft\\GameBar",
        "AutoGameModeEnabled",
        0,
    )?;
    Ok(())
}

pub fn is_game_mode_enabled() -> bool {
    get_reg_dword("HKCU\\Software\\Microsoft\\GameBar", "AllowAutoGameMode").unwrap_or(0) == 1
}

pub fn toggle_game_bar(enable: bool) -> Result<(), String> {
    set_reg_dword(
        "HKCU\\Software\\Microsoft\\GameBar",
        "UseNexusForGameBarEnabled",
        if enable { 1 } else { 0 },
    )
}

pub fn is_game_bar_enabled() -> bool {
    get_reg_dword(
        "HKCU\\Software\\Microsoft\\GameBar",
        "UseNexusForGameBarEnabled",
    )
    .unwrap_or(1)
        == 1
}

pub fn toggle_hardware_gpu_scheduling(enable: bool) -> Result<(), String> {
    set_reg_dword(
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers",
        "HwSchMode",
        if enable { 2 } else { 1 },
    )
}

pub fn is_hardware_gpu_scheduling_enabled() -> bool {
    get_reg_dword(
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\GraphicsDrivers",
        "HwSchMode",
    )
    .unwrap_or(1)
        == 2
}

fn set_reg_dword(key_path: &str, name: &str, value: u32) -> Result<(), String> {
    let output = Command::new("reg")
        .args([
            "add",
            key_path,
            "/v",
            name,
            "/t",
            "REG_DWORD",
            "/d",
            &value.to_string(),
            "/f",
        ])
        .output()
        .map_err(|e| format!("reg command failed: {}", e))?;

    if !output.status.success() {
        return Err(win_encoding::friendly_error("修改系统设置", &output.stderr));
    }
    Ok(())
}

fn get_reg_dword(key_path: &str, name: &str) -> Option<u32> {
    let output = Command::new("reg")
        .args(["query", key_path, "/v", name])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    // Parse output like: "    AllowAutoGameMode    REG_DWORD    0x1"
    for line in text.lines() {
        if line.contains(name) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(hex_str) = parts.last() {
                if let Some(stripped) = hex_str.strip_prefix("0x") {
                    return u32::from_str_radix(stripped, 16).ok();
                }
                return hex_str.parse().ok();
            }
        }
    }
    None
}
