use std::ffi::c_void;
use std::path::Path;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    DuplicateTokenEx, GetTokenInformation, SecurityImpersonation, TokenElevation, TokenPrimary,
    TOKEN_ALL_ACCESS, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows::Win32::System::Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock};
use windows::Win32::System::Threading::{
    CreateProcessWithTokenW, GetCurrentProcess, OpenProcess, OpenProcessToken,
    CREATE_UNICODE_ENVIRONMENT, LOGON_WITH_PROFILE, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetShellWindow, GetWindowThreadProcessId, SW_HIDE, SW_SHOWNORMAL,
};

/// Check whether the current process is running with elevated (administrator) privileges.
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut return_length = 0u32;
        let success = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        let _ = CloseHandle(token);

        if success.is_ok() {
            elevation.TokenIsElevated != 0
        } else {
            false
        }
    }
}

/// Retrieve and duplicate the primary access token from the desktop shell (explorer.exe).
/// This allows an elevated process to spawn non-elevated (medium integrity) user processes.
pub fn get_explorer_token() -> windows::core::Result<HANDLE> {
    unsafe {
        let shell_hwnd = GetShellWindow();
        if shell_hwnd.0.is_null() {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(-1),
                "GetShellWindow returned null; explorer may not be running",
            ));
        }

        let mut shell_pid = 0u32;
        GetWindowThreadProcessId(shell_hwnd, Some(&mut shell_pid));
        if shell_pid == 0 {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(-1),
                "Failed to get thread/process ID of explorer.exe shell window",
            ));
        }

        let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, shell_pid)?;

        let mut token_handle = HANDLE::default();
        let token_res = OpenProcessToken(
            process_handle,
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY,
            &mut token_handle,
        );
        let _ = CloseHandle(process_handle);
        token_res?;

        let mut duplicated_token = HANDLE::default();
        let dup_res = DuplicateTokenEx(
            token_handle,
            TOKEN_ALL_ACCESS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut duplicated_token,
        );
        let _ = CloseHandle(token_handle);
        dup_res?;

        Ok(duplicated_token)
    }
}

/// Spawn a process using the specified user access token (`CreateProcessWithTokenW`).
pub fn create_process_with_token(
    token: HANDLE,
    application: &str,
    args: Option<&str>,
    work_dir: Option<&Path>,
    hidden: bool,
) -> windows::core::Result<u32> {
    unsafe {
        let mut env_block: *mut c_void = std::ptr::null_mut();
        CreateEnvironmentBlock(&mut env_block, token, false)?;

        let si = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: if hidden {
                SW_HIDE.0 as u16
            } else {
                SW_SHOWNORMAL.0 as u16
            },
            ..Default::default()
        };

        let mut pi = PROCESS_INFORMATION::default();

        let cmd_string = match args {
            Some(a) if !a.trim().is_empty() => format!("\"{application}\" {a}"),
            _ => format!("\"{application}\""),
        };
        let mut cmd_wide: Vec<u16> = cmd_string.encode_utf16().chain(std::iter::once(0)).collect();

        let work_dir_wide: Option<Vec<u16>> = work_dir.map(|p| {
            p.to_string_lossy()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect()
        });

        let work_dir_ptr = match &work_dir_wide {
            Some(w) => PCWSTR(w.as_ptr()),
            None => PCWSTR::null(),
        };

        let res = CreateProcessWithTokenW(
            token,
            LOGON_WITH_PROFILE,
            PCWSTR::null(),
            PWSTR(cmd_wide.as_mut_ptr()),
            CREATE_UNICODE_ENVIRONMENT,
            Some(env_block),
            work_dir_ptr,
            &si,
            &mut pi,
        );

        let _ = DestroyEnvironmentBlock(env_block);

        if res.is_ok() {
            let pid = pi.dwProcessId;
            let _ = CloseHandle(pi.hProcess);
            let _ = CloseHandle(pi.hThread);
            Ok(pid)
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}
