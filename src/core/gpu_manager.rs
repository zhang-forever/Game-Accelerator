use crate::core::{command, win_encoding};
use std::path::{Path, PathBuf};

pub struct GpuInfo {
    pub name: String,
    pub temperature: f32,
    pub usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub driver_version: String,
}

fn parse_gpu_info(line: &str) -> Option<GpuInfo> {
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() != 6 || parts[0].is_empty() || parts[5].is_empty() {
        return None;
    }
    let gpu = GpuInfo {
        name: parts[0].to_string(),
        temperature: parts[1].parse().ok()?,
        usage_percent: parts[2].parse().ok()?,
        memory_used_mb: parts[3].parse().ok()?,
        memory_total_mb: parts[4].parse().ok()?,
        driver_version: parts[5].to_string(),
    };
    if !gpu.temperature.is_finite() || !gpu.usage_percent.is_finite() || gpu.memory_total_mb == 0 {
        return None;
    }
    Some(gpu)
}

/// Unsupported/missing measurements are omitted instead of being reported as 0.
pub fn get_gpu_info() -> Vec<GpuInfo> {
    let output = command::run_hidden("nvidia-smi", &[
        "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total,driver_version",
        "--format=csv,noheader,nounits",
    ]);
    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(parse_gpu_info)
            .collect(),
        _ => Vec::new(),
    }
}

/// Graphics preferences use the full executable path as the value name.
pub fn validate_game_exe_path(game_exe: &str) -> Result<PathBuf, String> {
    let trimmed = game_exe.trim();
    let trimmed = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);
    let path = Path::new(trimmed);
    if !path.is_absolute()
        || !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
    {
        return Err("请选择现有游戏 .exe 文件的完整绝对路径".to_string());
    }
    if !path.is_file() {
        return Err("游戏 EXE 文件不存在，请确认安装路径".to_string());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("无法确认游戏路径：{error}"))?;
    #[cfg(windows)]
    {
        // Windows Graphics Settings expects ordinary drive/UNC paths, whereas
        // canonicalize returns the extended path prefix.
        let text = canonical.to_string_lossy();
        if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
            return Ok(PathBuf::from(format!(r"\\{unc}")));
        }
        if let Some(drive) = text.strip_prefix(r"\\?\") {
            return Ok(PathBuf::from(drive));
        }
    }
    Ok(canonical)
}

pub fn force_discrete_gpu_for_game(game_exe: &str) -> Result<String, String> {
    let executable = validate_game_exe_path(game_exe)?;
    let executable = executable
        .to_str()
        .ok_or_else(|| "游戏路径无法编码".to_string())?;
    let output = command::run_hidden(
        "reg",
        &[
            "add",
            r"HKCU\Software\Microsoft\DirectX\UserGpuPreferences",
            "/v",
            executable,
            "/t",
            "REG_SZ",
            "/d",
            "GpuPreference=2;",
            "/f",
        ],
    )?;
    if output.status.success() {
        Ok(format!(
            "已设置「{executable}」优先使用高性能 GPU，重新启动游戏后使用"
        ))
    } else {
        let detail = if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        Err(win_encoding::friendly_error("设置游戏 GPU 首选项", detail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_csv_requires_available_measurements() {
        let gpu = parse_gpu_info("NVIDIA Test, 50, 0, 100, 8192, 600.1").unwrap();
        assert_eq!(gpu.usage_percent, 0.0);
        assert!(parse_gpu_info("NVIDIA Test, N/A, N/A, 100, 8192, 600.1").is_none());
        assert!(parse_gpu_info("NVIDIA Test, NaN, 0, 100, 8192, 600.1").is_none());
        assert!(parse_gpu_info("error").is_none());
    }

    #[test]
    fn gpu_preference_rejects_bare_names_and_missing_executables() {
        assert!(validate_game_exe_path("VALORANT-Win64-Shipping.exe").is_err());
        assert!(validate_game_exe_path("").is_err());
        let missing = std::env::temp_dir().join(format!(
            "missing-game-accelerator-{}.exe",
            std::process::id()
        ));
        assert!(validate_game_exe_path(&missing.to_string_lossy()).is_err());
    }

    #[test]
    fn gpu_preference_accepts_existing_absolute_exe_only() {
        let executable =
            std::env::temp_dir().join(format!("game-accelerator-test-{}.EXE", std::process::id()));
        std::fs::write(&executable, b"test marker").unwrap();
        let result = validate_game_exe_path(&executable.to_string_lossy());
        std::fs::remove_file(&executable).unwrap();
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }
}
