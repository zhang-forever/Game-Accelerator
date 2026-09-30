const GAME_BAR_KEY: &str = r"Software\Microsoft\GameBar";
const GRAPHICS_KEY: &str = r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers";
pub(crate) const GAME_MODE_VALUES: &[&str] = &["AllowAutoGameMode", "AutoGameModeEnabled"];

/// Capture the original type and bytes, including non-DWORD values. Missing and
/// unreadable registry values must remain distinguishable during rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegistryValue {
    pub kind: u32,
    pub bytes: Vec<u8>,
}

impl RegistryValue {
    pub fn dword(value: u32) -> Self {
        Self {
            kind: 4,
            bytes: value.to_le_bytes().to_vec(),
        }
    }

    fn as_dword(&self) -> Option<u32> {
        if self.kind != 4 || self.bytes.len() != 4 {
            return None;
        }
        Some(u32::from_le_bytes(self.bytes.as_slice().try_into().ok()?))
    }
}

pub(crate) fn read_game_mode_value(name: &str) -> Result<Option<RegistryValue>, String> {
    registry_read(false, GAME_BAR_KEY, name)
}

pub(crate) fn write_game_mode_value(
    name: &str,
    value: Option<&RegistryValue>,
) -> Result<(), String> {
    registry_write(false, GAME_BAR_KEY, name, value)
}

pub fn enable_game_mode() -> Result<(), String> {
    for name in GAME_MODE_VALUES {
        write_game_mode_value(name, Some(&RegistryValue::dword(1)))?;
    }
    Ok(())
}

pub fn disable_game_mode() -> Result<(), String> {
    for name in GAME_MODE_VALUES {
        write_game_mode_value(name, Some(&RegistryValue::dword(0)))?;
    }
    Ok(())
}

fn preference_state(
    value: Option<RegistryValue>,
    name: &str,
    enabled: u32,
    disabled: u32,
) -> Result<Option<bool>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let dword = value
        .as_dword()
        .ok_or_else(|| format!("{name} 不是有效的 REG_DWORD 值，无法判断当前偏好"))?;
    if dword == enabled {
        Ok(Some(true))
    } else if dword == disabled {
        Ok(Some(false))
    } else {
        Err(format!("{name} 包含未识别的值 {dword}，无法判断当前偏好"))
    }
}

fn game_mode_state_with(
    mut read: impl FnMut(&str) -> Result<Option<RegistryValue>, String>,
) -> Result<Option<bool>, String> {
    let primary = preference_state(read("AutoGameModeEnabled")?, "AutoGameModeEnabled", 1, 0)?;
    let secondary = preference_state(read("AllowAutoGameMode")?, "AllowAutoGameMode", 1, 0)?;
    match (primary, secondary) {
        (Some(primary), Some(secondary)) if primary != secondary => {
            Err("游戏模式设置 AutoGameModeEnabled 与 AllowAutoGameMode 存在冲突".to_string())
        }
        (primary, _) => Ok(primary),
    }
}

/// The Windows Settings preference is AutoGameModeEnabled. A missing preference
/// means system default, not disabled; unreadable/conflicting values are errors.
pub fn game_mode_state() -> Result<Option<bool>, String> {
    game_mode_state_with(read_game_mode_value)
}

/// This only controls opening Game Bar with a controller's Nexus/Xbox button.
/// It does not control Win+G, recording, or whether the Game Bar app is installed.
pub fn toggle_game_bar(enable: bool) -> Result<(), String> {
    registry_write(
        false,
        GAME_BAR_KEY,
        "UseNexusForGameBarEnabled",
        Some(&RegistryValue::dword(u32::from(enable))),
    )
}

/// Query the controller-button preference, preserving system default/unknown.
pub fn game_bar_state() -> Result<Option<bool>, String> {
    preference_state(
        registry_read(false, GAME_BAR_KEY, "UseNexusForGameBarEnabled")?,
        "UseNexusForGameBarEnabled",
        1,
        0,
    )
}

/// This machine setting requires supported hardware and a reboot.
pub fn toggle_hardware_gpu_scheduling(enable: bool) -> Result<(), String> {
    registry_write(
        true,
        GRAPHICS_KEY,
        "HwSchMode",
        Some(&RegistryValue::dword(if enable { 2 } else { 1 })),
    )
}

/// Read the configured preference; this does not prove driver support or that a
/// reboot-required change has taken effect in the current Windows session.
pub fn hardware_gpu_scheduling_state() -> Result<Option<bool>, String> {
    preference_state(
        registry_read(true, GRAPHICS_KEY, "HwSchMode")?,
        "HwSchMode",
        2,
        1,
    )
}

#[cfg(windows)]
fn registry_read(machine: bool, path: &str, name: &str) -> Result<Option<RegistryValue>, String> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
    };
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
        KEY_QUERY_VALUE,
    };
    let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    // SAFETY: Buffers are null-terminated, capacities match the native lengths,
    // and the successfully opened key is closed on every return path.
    unsafe {
        let mut key = null_mut();
        let opened = RegOpenKeyExW(
            if machine {
                HKEY_LOCAL_MACHINE
            } else {
                HKEY_CURRENT_USER
            },
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        );
        if opened == ERROR_FILE_NOT_FOUND || opened == ERROR_PATH_NOT_FOUND {
            return Ok(None);
        }
        if opened != ERROR_SUCCESS {
            return Err(registry_error("读取注册表", opened));
        }
        let result = (|| {
            let mut kind = 0;
            let mut length = 0;
            let queried = RegQueryValueExW(
                key,
                name.as_ptr(),
                null(),
                &mut kind,
                null_mut(),
                &mut length,
            );
            if queried == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if queried != ERROR_SUCCESS {
                return Err(registry_error("读取注册表值", queried));
            }
            // Only these small settings are needed; reject corrupt/oversized values.
            if length > 1024 * 1024 {
                return Err("注册表设置过大，未修改".to_string());
            }
            let mut bytes = vec![0u8; length as usize];
            let queried = RegQueryValueExW(
                key,
                name.as_ptr(),
                null(),
                &mut kind,
                bytes.as_mut_ptr(),
                &mut length,
            );
            if queried != ERROR_SUCCESS {
                return Err(registry_error("捕获注册表原值", queried));
            }
            bytes.truncate(length as usize);
            Ok(Some(RegistryValue { kind, bytes }))
        })();
        RegCloseKey(key);
        result
    }
}

#[cfg(windows)]
fn registry_write(
    machine: bool,
    path: &str,
    name: &str,
    value: Option<&RegistryValue>,
) -> Result<(), String> {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{
        ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
    };
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE,
    };
    let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    // SAFETY: Only the indicated value is changed; the rest of the key and any
    // other user settings remain intact. The handle is closed after use.
    unsafe {
        let mut key = null_mut();
        let opened = if value.is_some() {
            RegCreateKeyExW(
                if machine {
                    HKEY_LOCAL_MACHINE
                } else {
                    HKEY_CURRENT_USER
                },
                path.as_ptr(),
                0,
                null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                null(),
                &mut key,
                null_mut(),
            )
        } else {
            RegOpenKeyExW(
                if machine {
                    HKEY_LOCAL_MACHINE
                } else {
                    HKEY_CURRENT_USER
                },
                path.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut key,
            )
        };
        if value.is_none() && (opened == ERROR_FILE_NOT_FOUND || opened == ERROR_PATH_NOT_FOUND) {
            return Ok(());
        }
        if opened != ERROR_SUCCESS {
            return Err(registry_error("打开注册表设置", opened));
        }
        let written = if let Some(value) = value {
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                value.kind,
                value.bytes.as_ptr(),
                value.bytes.len() as u32,
            )
        } else {
            RegDeleteValueW(key, name.as_ptr())
        };
        RegCloseKey(key);
        if written == ERROR_SUCCESS || (value.is_none() && written == ERROR_FILE_NOT_FOUND) {
            Ok(())
        } else {
            Err(registry_error("修改注册表设置", written))
        }
    }
}

#[cfg(windows)]
fn registry_error(action: &str, code: u32) -> String {
    format!(
        "{action}失败：{}",
        std::io::Error::from_raw_os_error(code as i32)
    )
}

#[cfg(not(windows))]
fn registry_read(
    _machine: bool,
    _path: &str,
    _name: &str,
) -> Result<Option<RegistryValue>, String> {
    Err("仅支持 Windows".to_string())
}

#[cfg(not(windows))]
fn registry_write(
    _machine: bool,
    _path: &str,
    _name: &str,
    _value: Option<&RegistryValue>,
) -> Result<(), String> {
    Err("仅支持 Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_dword_requires_exact_type_and_length() {
        assert_eq!(RegistryValue::dword(1).as_dword(), Some(1));
        assert_eq!(
            RegistryValue {
                kind: 1,
                bytes: vec![1, 0, 0, 0]
            }
            .as_dword(),
            None
        );
        assert_eq!(
            RegistryValue {
                kind: 4,
                bytes: vec![1]
            }
            .as_dword(),
            None
        );
    }

    #[test]
    fn preference_absence_and_invalid_types_do_not_mean_disabled() {
        assert_eq!(preference_state(None, "example", 1, 0).unwrap(), None);
        assert_eq!(
            preference_state(Some(RegistryValue::dword(0)), "example", 1, 0).unwrap(),
            Some(false)
        );
        assert_eq!(
            preference_state(Some(RegistryValue::dword(1)), "example", 1, 0).unwrap(),
            Some(true)
        );
        assert!(preference_state(
            Some(RegistryValue {
                kind: 1,
                bytes: vec![1, 0, 0, 0]
            }),
            "example",
            1,
            0
        )
        .is_err());
        assert!(preference_state(Some(RegistryValue::dword(2)), "example", 1, 0).is_err());
    }

    #[test]
    fn game_mode_uses_primary_and_rejects_conflicting_values() {
        let primary_only = game_mode_state_with(|name| {
            Ok(if name == "AutoGameModeEnabled" {
                Some(RegistryValue::dword(1))
            } else {
                None
            })
        });
        assert_eq!(primary_only.unwrap(), Some(true));
        let legacy_only = game_mode_state_with(|name| {
            Ok(if name == "AllowAutoGameMode" {
                Some(RegistryValue::dword(1))
            } else {
                None
            })
        });
        assert_eq!(legacy_only.unwrap(), None);
        let conflicting = game_mode_state_with(|name| {
            Ok(Some(RegistryValue::dword(u32::from(
                name == "AutoGameModeEnabled",
            ))))
        });
        assert!(conflicting.is_err());
    }

    #[test]
    fn game_mode_propagates_read_errors_instead_of_guessing() {
        let failure = game_mode_state_with(|_| Err("read denied".to_string()));
        assert_eq!(failure.unwrap_err(), "read denied");
        assert_eq!(game_mode_state_with(|_| Ok(None)).unwrap(), None);
        assert_eq!(
            game_mode_state_with(|_| Ok(Some(RegistryValue::dword(0)))).unwrap(),
            Some(false)
        );
    }

    #[test]
    fn gpu_scheduling_uses_only_explicit_supported_preference_values() {
        assert_eq!(preference_state(None, "HwSchMode", 2, 1).unwrap(), None);
        assert_eq!(
            preference_state(Some(RegistryValue::dword(2)), "HwSchMode", 2, 1).unwrap(),
            Some(true)
        );
        assert_eq!(
            preference_state(Some(RegistryValue::dword(1)), "HwSchMode", 2, 1).unwrap(),
            Some(false)
        );
        assert!(preference_state(Some(RegistryValue::dword(0)), "HwSchMode", 2, 1).is_err());
    }
}
