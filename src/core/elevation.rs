/// Inspect this process's access token, independent of Windows services.
pub fn is_elevated() -> bool {
    token_is_elevated().unwrap_or(false)
}

fn token_is_elevated() -> Result<bool, String> {
    #[cfg(windows)]
    {
        use std::mem::size_of;
        use std::ptr::null_mut;
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        // SAFETY: The token buffer and size match TOKEN_ELEVATION, and the token
        // handle is closed whether the query succeeds or fails.
        unsafe {
            let mut token = null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return Err(format!(
                    "无法读取当前权限：{}",
                    std::io::Error::last_os_error()
                ));
            }
            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut returned = 0;
            let succeeded = GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned,
            );
            let error = if succeeded == 0 {
                Some(std::io::Error::last_os_error())
            } else {
                None
            };
            CloseHandle(token);
            if let Some(error) = error {
                return Err(format!("无法核实当前权限：{error}"));
            }
            Ok(elevation.TokenIsElevated != 0)
        }
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

/// Windows command-line argument quoting (no command shell is involved).
fn quote_argument(argument: &str) -> String {
    let mut quoted = String::from("\"");
    let mut backslashes = 0;
    for character in argument.chars() {
        match character {
            '\\' => backslashes += 1,
            '"' => {
                quoted.extend(std::iter::repeat_n('\\', backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                quoted.extend(std::iter::repeat_n('\\', backslashes));
                quoted.push(character);
                backslashes = 0;
            }
        }
    }
    quoted.extend(std::iter::repeat_n('\\', backslashes * 2));
    quoted.push('"');
    quoted
}

/// Explicit user-triggered UAC relaunch. Forward the original arguments,
/// including the config directory, and report cancellation/failure accurately.
pub fn try_elevate_if_needed() -> Result<bool, String> {
    if token_is_elevated()? {
        return Ok(false);
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ptr::{null, null_mut};
        use windows_sys::Win32::Foundation::{GetLastError, ERROR_CANCELLED};
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let executable =
            std::env::current_exe().map_err(|error| format!("无法定位程序：{error}"))?;
        let executable: Vec<u16> = executable
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let parameters = std::env::args_os()
            .skip(1)
            .map(|argument| quote_argument(&argument.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(" ");
        let parameters: Vec<u16> = parameters.encode_utf16().chain(Some(0)).collect();
        let verb: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();
        // SAFETY: All UTF-16 arguments are terminated; ShellExecuteW returns a
        // value greater than 32 only when it accepted the launch request.
        unsafe {
            let launched = ShellExecuteW(
                null_mut(),
                verb.as_ptr(),
                executable.as_ptr(),
                parameters.as_ptr(),
                null(),
                SW_SHOWNORMAL,
            ) as isize;
            if launched > 32 {
                return Ok(true);
            }
            let code = GetLastError();
            if code == ERROR_CANCELLED {
                Err("已取消管理员权限请求，当前窗口保持打开".to_string())
            } else {
                Err(format!(
                    "管理员启动失败 (ShellExecute={launched})：{}",
                    std::io::Error::from_raw_os_error(code as i32)
                ))
            }
        }
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_empty_spaces_quotes_and_trailing_backslash() {
        assert_eq!(quote_argument(""), "\"\"");
        assert_eq!(quote_argument("--config-dir"), "\"--config-dir\"");
        assert_eq!(quote_argument(r"C:\Game Trial\"), r#""C:\Game Trial\\""#);
        assert_eq!(quote_argument("a\"b"), r#""a\"b""#);
    }

    #[test]
    fn metacharacters_remain_one_literal_argument() {
        assert_eq!(
            quote_argument("路径 & name; value"),
            "\"路径 & name; value\""
        );
    }
}
