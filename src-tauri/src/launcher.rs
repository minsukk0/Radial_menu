use std::collections::HashMap;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL};

use crate::win32::token::{create_process_with_token, get_explorer_token, is_elevated};
use crate::win32::shell::shell_execute_user;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elevation {
    Admin,
    User,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchSpec {
    pub program: String,
    pub args: Option<String>,
    pub work_dir: Option<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub hidden: bool,
}

impl LaunchSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            ..Default::default()
        }
    }

    pub fn with_args(mut self, args: impl Into<String>) -> Self {
        self.args = Some(args.into());
        self
    }

    pub fn with_work_dir(mut self, dir: PathBuf) -> Self {
        self.work_dir = Some(dir);
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), val.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("De-elevation error: {0}")]
    DeElevation(String),
    #[error("Process spawn error: {0}")]
    Spawn(String),
    #[error("Admin execution failed: {0}")]
    Admin(String),
    #[error("Win32 error: {0}")]
    Win32(#[from] windows::core::Error),
}

/// Spawns an external process with the designated elevation level.
///
/// If `Elevation::User` is requested and the host app is running elevated,
/// this function guarantees de-elevation using the explorer token or `IShellDispatch2`.
/// It will NEVER silently execute as Administrator when `Elevation::User` is requested.
pub fn spawn(spec: &LaunchSpec, elevation: Elevation) -> Result<u32, LaunchError> {
    let app_elevated = is_elevated();

    match elevation {
        Elevation::Admin => {
            if app_elevated {
                // Already elevated: spawn directly inheriting admin token
                spawn_direct(spec)
            } else {
                // Not elevated: request UAC elevation via ShellExecuteExW("runas")
                spawn_runas(spec)
            }
        }
        Elevation::User => {
            if app_elevated {
                // De-elevation path: clone explorer token
                match get_explorer_token() {
                    Ok(token) => {
                        let res = create_process_with_token(
                            token,
                            &spec.program,
                            spec.args.as_deref(),
                            spec.work_dir.as_deref(),
                            spec.hidden,
                        );
                        unsafe { let _ = CloseHandle(token); }

                        match res {
                            Ok(pid) => Ok(pid),
                            Err(e) => {
                                // Fallback to IShellDispatch2::ShellExecute
                                shell_execute_user(
                                    &spec.program,
                                    spec.args.as_deref(),
                                    spec.work_dir.as_deref(),
                                    spec.hidden,
                                )
                                .map(|_| 0)
                                .map_err(|sh_err| {
                                    LaunchError::DeElevation(format!(
                                        "Token spawn failed ({e}) and ShellExecute fallback failed ({sh_err})"
                                    ))
                                })
                            }
                        }
                    }
                    Err(token_err) => {
                        // Attempt fallback to IShellDispatch2::ShellExecute
                        shell_execute_user(
                            &spec.program,
                            spec.args.as_deref(),
                            spec.work_dir.as_deref(),
                            spec.hidden,
                        )
                        .map(|_| 0)
                        .map_err(|sh_err| {
                            LaunchError::DeElevation(format!(
                                "Cannot acquire explorer token ({token_err}) and ShellExecute failed ({sh_err})"
                            ))
                        })
                    }
                }
            } else {
                // Current process is already medium integrity; spawn directly
                spawn_direct(spec)
            }
        }
    }
}

/// Spawn process directly in the current security token environment.
fn spawn_direct(spec: &LaunchSpec) -> Result<u32, LaunchError> {
    let mut cmd = std::process::Command::new(&spec.program);

    if let Some(ref args) = spec.args {
        // Parse arguments or pass as raw args if possible
        let parts = shlex_split(args);
        cmd.args(parts);
    }

    if let Some(ref wd) = spec.work_dir {
        cmd.current_dir(wd);
    }

    for (k, v) in &spec.env_vars {
        cmd.env(k, v);
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if spec.hidden {
            // CREATE_NO_WINDOW = 0x08000000
            cmd.creation_flags(0x08000000);
        }
    }

    let child = cmd
        .spawn()
        .map_err(|e| LaunchError::Spawn(format!("Failed to spawn '{}': {e}", spec.program)))?;

    Ok(child.id())
}

/// Spawn process with UAC elevation prompt ("runas").
fn spawn_runas(spec: &LaunchSpec) -> Result<u32, LaunchError> {
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let file: Vec<u16> = spec.program.encode_utf16().chain(std::iter::once(0)).collect();

    let args_wide: Option<Vec<u16>> = spec
        .args
        .as_ref()
        .map(|a| a.encode_utf16().chain(std::iter::once(0)).collect());

    let dir_wide: Option<Vec<u16>> = spec
        .work_dir
        .as_ref()
        .map(|d| d.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect());

    let mut sei = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: HWND(std::ptr::null_mut()),
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: match &args_wide {
            Some(w) => PCWSTR(w.as_ptr()),
            None => PCWSTR::null(),
        },
        lpDirectory: match &dir_wide {
            Some(w) => PCWSTR(w.as_ptr()),
            None => PCWSTR::null(),
        },
        nShow: if spec.hidden {
            SW_HIDE.0
        } else {
            SW_SHOWNORMAL.0
        },
        ..Default::default()
    };

    unsafe {
        ShellExecuteExW(&mut sei)?;
        let h_proc = sei.hProcess;
        let pid = if !h_proc.0.is_null() {
            let pid = windows::Win32::System::Threading::GetProcessId(h_proc);
            let _ = CloseHandle(h_proc);
            pid
        } else {
            0
        };
        Ok(pid)
    }
}

/// Simple whitespace-aware command line splitter respecting quotes.
fn shlex_split(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shlex_split() {
        let input = r#"hello "world with spaces" --opt "val""#;
        let tokens = shlex_split(input);
        assert_eq!(tokens, vec!["hello", "world with spaces", "--opt", "val"]);
    }
}
