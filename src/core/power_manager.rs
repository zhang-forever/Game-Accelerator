use crate::core::{command, win_encoding};

pub const HIGH_PERF_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
pub const ULTIMATE_GUID: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";
const BALANCED_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

fn guid_at(bytes: &[u8]) -> Option<String> {
    if bytes.len() != 36 {
        return None;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let is_separator = matches!(index, 8 | 13 | 18 | 23);
        if (is_separator && *byte != b'-') || (!is_separator && !byte.is_ascii_hexdigit()) {
            return None;
        }
    }
    Some(String::from_utf8_lossy(bytes).to_ascii_lowercase())
}

/// Parse GUIDs without depending on the language or console code page of powercfg.
pub fn parse_power_guids(output: &[u8]) -> Vec<String> {
    let mut guids = Vec::new();
    for window in output.windows(36) {
        if let Some(guid) = guid_at(window) {
            if !guids.contains(&guid) {
                guids.push(guid);
            }
        }
    }
    guids
}

fn powercfg(args: &[&str]) -> Result<Vec<u8>, String> {
    #[cfg(not(windows))]
    {
        let _ = args;
        return Err("仅支持 Windows".to_string());
    }
    #[cfg(windows)]
    {
        let output = command::run_hidden("powercfg", args)?;
        if !output.status.success() {
            let detail = if output.stderr.is_empty() {
                &output.stdout
            } else {
                &output.stderr
            };
            return Err(format!(
                "powercfg 退出码 {:?}：{}",
                output.status.code(),
                win_encoding::decode_output(detail)
            ));
        }
        Ok(output.stdout)
    }
}

pub fn get_active_power_plan_guid() -> Result<String, String> {
    let output = powercfg(&["/getactivescheme"])?;
    let guids = parse_power_guids(&output);
    if guids.len() != 1 {
        return Err("无法识别当前电源方案 GUID，未修改电源设置".to_string());
    }
    Ok(guids[0].clone())
}

pub fn set_active_power_plan(guid: &str) -> Result<(), String> {
    let guid = guid_at(guid.as_bytes()).ok_or_else(|| "电源方案 GUID 无效".to_string())?;
    powercfg(&["/setactive", &guid])?;
    if get_active_power_plan_guid()? != guid {
        return Err("命令结束后电源方案未匹配预期值".to_string());
    }
    Ok(())
}

/// Use an existing performance scheme; never create or overwrite a user scheme.
pub fn performance_plan_guid() -> Result<String, String> {
    let guids = parse_power_guids(&powercfg(&["/list"])?);
    if guids.iter().any(|guid| guid == HIGH_PERF_GUID) {
        Ok(HIGH_PERF_GUID.to_string())
    } else if guids.iter().any(|guid| guid == ULTIMATE_GUID) {
        Ok(ULTIMATE_GUID.to_string())
    } else {
        Err("本机没有现有的高性能电源方案，保持当前方案".to_string())
    }
}

/// Direct setting for the manual settings page. The one-click boost uses a
/// session snapshot instead, so it can restore the actual previous scheme.
pub fn set_high_performance() -> Result<(), String> {
    set_active_power_plan(&performance_plan_guid()?)
}

pub fn set_balanced() -> Result<(), String> {
    set_active_power_plan(BALANCED_GUID)
}

pub fn get_active_power_plan() -> String {
    match get_active_power_plan_guid() {
        Ok(guid) if guid == HIGH_PERF_GUID => "高性能".to_string(),
        Ok(guid) if guid == ULTIMATE_GUID => "卓越性能".to_string(),
        Ok(guid) if guid == BALANCED_GUID => "平衡".to_string(),
        Ok(guid) => format!("自定义方案 ({guid})"),
        Err(_) => "无法读取电源方案".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chinese_uppercase_and_custom_power_guids() {
        let output = "电源方案 GUID: 8C5E7FDA-E8BF-4A96-9A85-A6E23A8C635C (高性能)\n电源方案 GUID: 01234567-89ab-cdef-0123-456789abcdef (我的方案)";
        assert_eq!(
            parse_power_guids(output.as_bytes()),
            vec![HIGH_PERF_GUID, "01234567-89ab-cdef-0123-456789abcdef"]
        );
    }

    #[test]
    fn parses_guid_amid_non_utf8_console_bytes() {
        let mut output = vec![0xb5, 0xe7, 0xd4, 0xb4];
        output.extend(HIGH_PERF_GUID.as_bytes());
        assert_eq!(parse_power_guids(&output), vec![HIGH_PERF_GUID]);
    }

    #[test]
    fn rejects_malformed_guids_and_deduplicates() {
        assert!(parse_power_guids(b"8c5e7fda-e8bf-4a96-9a85-a6e23a8c635z").is_empty());
        assert!(guid_at(b"scheme_current").is_none());
        let twice = format!("{HIGH_PERF_GUID}\n{HIGH_PERF_GUID}");
        assert_eq!(parse_power_guids(twice.as_bytes()).len(), 1);
    }
}
