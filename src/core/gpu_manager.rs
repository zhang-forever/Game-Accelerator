pub struct GpuInfo {
    pub name: String,
    pub temperature: f32,
    pub usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub driver_version: String,
}

pub fn get_gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let parts: Vec<&str> = line.split(", ").collect();
            if parts.len() >= 6 {
                gpus.push(GpuInfo {
                    name: parts[0].trim().to_string(),
                    temperature: parts[1].trim().parse().unwrap_or(0.0),
                    usage_percent: parts[2].trim().parse().unwrap_or(0.0),
                    memory_used_mb: parts[3].trim().parse::<u64>().unwrap_or(0),
                    memory_total_mb: parts[4].trim().parse::<u64>().unwrap_or(0),
                    driver_version: parts[5].trim().to_string(),
                });
            }
        }
    }

    gpus
}

/// Whether nvidia-smi (and thus an NVIDIA GPU) is available.
pub fn nvidia_available() -> bool {
    std::process::Command::new("nvidia-smi")
        .arg("-L")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Force the NVIDIA GPU into maximum performance mode.
/// On consumer GeForce cards `nvidia-smi -pm/-ac` is unsupported, so we use the
/// PowerMizer registry keys, which take effect after a reboot.
pub fn set_nvidia_max_performance() -> Result<String, String> {
    if !nvidia_available() {
        return Err("未检测到 NVIDIA 显卡，无法设置".to_string());
    }

    let subkeys = find_nvidia_class_subkeys();
    if subkeys.is_empty() {
        return Err("找不到 NVIDIA 驱动注册表项，可能需要管理员权限".to_string());
    }

    let mut applied = 0;
    for sub in &subkeys {
        // PowerMizer: force maximum performance on both AC and battery
        let settings = [
            ("PowerMizerEnable", "1"),
            ("PowerMizerLevel", "1"),
            ("PowerMizerLevelAC", "1"),
            ("PerfLevelSrc", "8738"), // 0x2222 = prefer max perf
        ];
        let mut ok = true;
        for (name, value) in &settings {
            let res = std::process::Command::new("reg")
                .args(["add", sub, "/v", name, "/t", "REG_DWORD", "/d", value, "/f"])
                .output();
            if res.map(|o| !o.status.success()).unwrap_or(true) {
                ok = false;
            }
        }
        if ok {
            applied += 1;
        }
    }

    if applied > 0 {
        Ok(format!(
            "✓ 已设置最大性能模式（重启电脑后生效）"
        ))
    } else {
        Err("设置失败，请用管理员身份运行本程序".to_string())
    }
}

/// Disable NVIDIA telemetry scheduled tasks (reduces background overhead).
pub fn disable_nvidia_telemetry() -> Result<String, String> {
    let tasks = [
        "NvTmRep_CrashReport1_{B2FE1952-0186-46C3-BAEC-A80AA35AC5B8}",
        "NvTmRep_CrashReport2_{B2FE1952-0186-46C3-BAEC-A80AA35AC5B8}",
        "NvTmRep_CrashReport3_{B2FE1952-0186-46C3-BAEC-A80AA35AC5B8}",
        "NvTmRep_CrashReport4_{B2FE1952-0186-46C3-BAEC-A80AA35AC5B8}",
        "NvTmMon_{B2FE1952-0186-46C3-BAEC-A80AA35AC5B8}",
        "NvTmRepOnLogon_{B2FE1952-0186-46C3-BAEC-A80AA35AC5B8}",
    ];

    let mut disabled = 0;
    for task in &tasks {
        let res = std::process::Command::new("schtasks")
            .args(["/Change", "/TN", task, "/Disable"])
            .output();
        if res.map(|o| o.status.success()).unwrap_or(false) {
            disabled += 1;
        }
    }

    Ok(format!("✓ 已禁用 {} 个 NVIDIA 后台遥测任务", disabled))
}

/// Force a specific game executable to use the high-performance (discrete) GPU.
/// This writes to Windows Graphics Settings and works on all GPUs.
pub fn force_discrete_gpu_for_game(game_exe: &str) -> Result<String, String> {
    let exe = game_exe.trim();
    if exe.is_empty() {
        return Err("请先填写游戏 EXE 名称或完整路径".to_string());
    }

    let output = std::process::Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\DirectX\\UserGpuPreferences",
            "/v",
            exe,
            "/t",
            "REG_SZ",
            "/d",
            "GpuPreference=2;",
            "/f",
        ])
        .output()
        .map_err(|e| format!("执行失败: {}", e))?;

    if output.status.success() {
        Ok(format!("✓ 已设置「{}」优先使用独立显卡", exe))
    } else {
        Err(format!(
            "设置失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

/// Find NVIDIA display-adapter subkeys under the GPU class GUID.
fn find_nvidia_class_subkeys() -> Vec<String> {
    const CLASS_KEY: &str =
        "HKLM\\SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";

    let mut result = Vec::new();
    // Display adapters are numbered 0000, 0001, ...
    for i in 0..16 {
        let sub = format!("{}\\{:04}", CLASS_KEY, i);
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", &sub, "/v", "ProviderName"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if text.to_uppercase().contains("NVIDIA") {
                    result.push(sub);
                }
            }
        }
    }
    result
}
