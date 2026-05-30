/// Decode Windows console command output.
///
/// Windows CLI tools (`reg`, `net`, `sc`, etc.) emit text in the OEM code page
/// (GBK/GB2312 on Chinese systems), not UTF-8. Decoding those bytes as UTF-8
/// produces replacement characters (U+FFFD), so this helper detects that case
/// and returns a clean, user-friendly message instead of mojibake.
pub fn decode_output(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    // Try UTF-8 first - clean ASCII output decodes correctly.
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.trim().to_string();
    }

    // Non-UTF-8 bytes: almost always a localized error in the OEM code page.
    // We can't decode GBK without a dependency, so surface a clear message.
    "操作失败，通常是因为权限不足（请用管理员身份运行）".to_string()
}

/// Build a friendly error string from a failed command's stderr.
///
/// `action` describes what was attempted, e.g. "暂停磁盘索引".
pub fn friendly_error(action: &str, stderr: &[u8]) -> String {
    let detail = decode_output(stderr);
    if detail.is_empty() {
        format!("{}失败，可能需要管理员权限", action)
    } else {
        format!("{}失败：{}", action, detail)
    }
}
