use serde::Serialize;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct Arguments {
    pub config_dir: Option<PathBuf>,
    pub diagnose: Option<PathBuf>,
    pub smoke_test: Option<PathBuf>,
    pub help: bool,
}

impl Arguments {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let mut options = Self::default();
        while let Some(argument) = args.next() {
            let field = match argument.to_str() {
                Some("--help" | "-h") => {
                    options.help = true;
                    continue;
                }
                Some("--config-dir") => &mut options.config_dir,
                Some("--diagnose") => &mut options.diagnose,
                Some("--smoke-test") => &mut options.smoke_test,
                _ => return Err(format!("未知参数：{}", argument.to_string_lossy())),
            };
            if field.is_some() {
                return Err(format!("参数重复：{}", argument.to_string_lossy()));
            }
            let value = args
                .next()
                .ok_or_else(|| format!("{} 缺少路径", argument.to_string_lossy()))?;
            if value.is_empty() || value.to_string_lossy().starts_with("--") {
                return Err(format!("{} 需要有效路径", argument.to_string_lossy()));
            }
            *field = Some(PathBuf::from(value));
        }
        if options.diagnose.is_some() && options.smoke_test.is_some() {
            return Err("--diagnose 与 --smoke-test 只能选择一项".to_string());
        }
        Ok(options)
    }
}

#[derive(Serialize)]
struct DiagnosticReport {
    version: &'static str,
    read_only: bool,
    is_admin: bool,
    config_path: String,
    config_error: Option<String>,
    active_power_plan: String,
    stats: crate::monitor::SystemStats,
    config: crate::config::AppConfig,
}

/// Diagnostics read system state and write only the explicitly selected report.
pub fn write_diagnostics(path: &Path) -> Result<(), String> {
    let (config, config_error) = match crate::config::AppConfig::load() {
        Ok(config) => (config, None),
        Err(error) => (crate::config::AppConfig::default(), Some(error)),
    };
    let report = DiagnosticReport {
        version: env!("CARGO_PKG_VERSION"),
        read_only: true,
        is_admin: crate::core::elevation::is_elevated(),
        config_path: crate::config::AppConfig::config_path()
            .display()
            .to_string(),
        config_error,
        active_power_plan: crate::core::power_manager::get_active_power_plan(),
        stats: crate::monitor::collect_diagnostics(),
        config,
    };
    if report.stats.cpu_threads == 0 || report.stats.ram_total_gb == 0.0 {
        return Err("无法读取 CPU 或内存信息，诊断未完成".to_string());
    }
    write_report(path, &report)
}

pub fn write_report(path: &Path, report: &impl Serialize) -> Result<(), String> {
    let content = toml::to_string_pretty(report).map_err(|e| format!("无法生成报告：{}", e))?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建报告目录：{}", e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("无法保存报告（{}）：{}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_with_spaces_are_preserved() {
        let arguments = Arguments::parse([
            OsString::from("--config-dir"),
            OsString::from("C:\\My Games\\data"),
            OsString::from("--diagnose"),
            OsString::from("report.toml"),
        ])
        .unwrap();
        assert_eq!(
            arguments.config_dir,
            Some(PathBuf::from("C:\\My Games\\data"))
        );
    }

    #[test]
    fn missing_paths_and_unknown_options_fail() {
        assert!(Arguments::parse([OsString::from("--config-dir")]).is_err());
        assert!(Arguments::parse([OsString::from("--unknown")]).is_err());
        assert!(
            Arguments::parse([OsString::from("--diagnose"), OsString::from("--help")]).is_err()
        );
    }

    #[test]
    fn repeated_or_conflicting_modes_fail() {
        assert!(
            Arguments::parse(["--config-dir", "a", "--config-dir", "b"].map(OsString::from))
                .is_err()
        );
        assert!(
            Arguments::parse(["--diagnose", "a", "--smoke-test", "b"].map(OsString::from)).is_err()
        );
    }
}
