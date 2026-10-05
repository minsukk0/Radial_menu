use std::path::Path;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow, ShowWindow,
    SW_RESTORE,
};
use radial_core::{AppItem, WhenRunning};

use crate::context::ContextValues;
use crate::launcher::{spawn, Elevation, LaunchSpec};

struct FocusSearch {
    exe_name: String,
    found: bool,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let search = &mut *(lparam.0 as *mut FocusSearch);

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 {
        return BOOL(1);
    }

    if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        if QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
        {
            let full_path = String::from_utf16_lossy(&buf[..size as usize]);
            let file_name = Path::new(&full_path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();

            if file_name.eq_ignore_ascii_case(&search.exe_name) {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                let _ = SetForegroundWindow(hwnd);
                search.found = true;
                return BOOL(0); // Stop enumeration
            }
        }
    }

    BOOL(1)
}

/// Focus existing window of the application if running.
fn try_focus_app(path: &str) -> bool {
    let exe_name = Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    if exe_name.is_empty() {
        return false;
    }

    let mut search = FocusSearch {
        exe_name,
        found: false,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut search as *mut FocusSearch as isize),
        );
    }

    search.found
}

/// Execute an application item or bring its window to foreground.
pub fn execute_app(item: &AppItem, ctx: &ContextValues) -> Result<(), String> {
    let raw_path = ctx.substitute_raw_text(&item.path);

    let is_url = raw_path.starts_with("http://")
        || raw_path.starts_with("https://")
        || raw_path.starts_with("ftp://");

    if is_url {
        let spec = LaunchSpec::new("explorer.exe").with_args(format!("\"{raw_path}\""));
        spawn(&spec, Elevation::User).map_err(|e| format!("웹 앱 열기 실패: {e}"))?;
        return Ok(());
    }

    if item.when_running.unwrap_or(WhenRunning::Focus) == WhenRunning::Focus {
        if try_focus_app(&raw_path) {
            return Ok(());
        }
    }

    let is_admin = item.run_as_admin.unwrap_or(false);
    if is_admin {
        let app_item = radial_core::Item::App(item.clone());
        let _ = crate::security::approve_item(&app_item);
    }

    let elevation = if is_admin {
        Elevation::Admin
    } else {
        Elevation::User
    };

    let args = item.args.as_ref().map(|a| ctx.substitute_raw_text(a));
    let work_dir = Path::new(&raw_path)
        .parent()
        .map(|p| p.to_path_buf())
        .or_else(|| dirs::home_dir());

    let mut spec = LaunchSpec::new(raw_path);
    if let Some(a) = args {
        spec = spec.with_args(a);
    }
    if let Some(wd) = work_dir {
        spec = spec.with_work_dir(wd);
    }

    for (k, v) in ctx.to_env_vars() {
        spec = spec.with_env(k, v);
    }

    spawn(&spec, elevation).map_err(|e| format!("앱 실행 실패: {e}"))?;
    Ok(())
}
