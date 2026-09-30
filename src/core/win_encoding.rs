/// Decode UTF-8 first, then the OEM code page used by Windows console tools.
pub fn decode_output(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.trim().to_string();
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Globalization::{
            MultiByteToWideChar, CP_OEMCP, MB_ERR_INVALID_CHARS,
        };
        if let Ok(length) = i32::try_from(bytes.len()) {
            // SAFETY: Input length and output capacity are passed exactly, and
            // a failed conversion is never interpreted as a permission error.
            unsafe {
                let size = MultiByteToWideChar(
                    CP_OEMCP,
                    MB_ERR_INVALID_CHARS,
                    bytes.as_ptr(),
                    length,
                    std::ptr::null_mut(),
                    0,
                );
                if size > 0 {
                    let mut wide = vec![0u16; size as usize];
                    let written = MultiByteToWideChar(
                        CP_OEMCP,
                        MB_ERR_INVALID_CHARS,
                        bytes.as_ptr(),
                        length,
                        wide.as_mut_ptr(),
                        size,
                    );
                    if written > 0 {
                        return String::from_utf16_lossy(&wide[..written as usize])
                            .trim()
                            .to_string();
                    }
                }
            }
        }
    }
    "系统命令返回了无法解码的输出".to_string()
}

pub fn friendly_error(action: &str, output: &[u8]) -> String {
    let detail = decode_output(output);
    if detail.is_empty() {
        format!("{action}失败，系统命令没有提供错误详情")
    } else {
        format!("{action}失败：{detail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_without_inventing_permission_errors() {
        assert_eq!(
            decode_output("错误：找不到任务\r\n".as_bytes()),
            "错误：找不到任务"
        );
        assert_eq!(decode_output(b"  access denied\r\n"), "access denied");
        assert_eq!(decode_output(b""), "");
        assert_eq!(
            friendly_error("修改设置", b""),
            "修改设置失败，系统命令没有提供错误详情"
        );
    }
}
