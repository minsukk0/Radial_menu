use std::sync::atomic::{AtomicIsize, Ordering};
use tauri::WebviewWindow;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongW, IsWindow, SetForegroundWindow, SetWindowLongW,
    SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE,
    SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNOACTIVATE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};

use crate::win32::window::{get_cursor_monitor_dpi_scale, get_cursor_monitor_work_area};

static PREV_FOREGROUND_WINDOW: AtomicIsize = AtomicIsize::new(0);

/// Configure Win32 styles so the menu window never steals focus.
pub fn setup_menu_window_styles(window: &WebviewWindow) -> Result<(), String> {
    let raw_hwnd = window
        .hwnd()
        .map_err(|e| format!("Failed to get window HWND: {e}"))?
        .0;
    let hwnd = HWND(raw_hwnd);

    unsafe {
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let target_style = ex_style
            | (WS_EX_NOACTIVATE.0 as i32)
            | (WS_EX_TOOLWINDOW.0 as i32)
            | (WS_EX_TOPMOST.0 as i32);

        SetWindowLongW(hwnd, GWL_EXSTYLE, target_style);
        crate::log_msg(&format!(
            "setup_menu_window_styles: hwnd={:?}, old_ex=0x{:X}, new_ex=0x{:X}",
            raw_hwnd, ex_style, target_style
        ));
    }

    Ok(())
}

/// Calculate clamped position and display the menu window at cursor coordinates.
/// Returns the actual clamped center coordinates `(clamped_cx, clamped_cy)`.
pub fn show_menu_at_cursor(
    window: &WebviewWindow,
    cursor_x: i32,
    cursor_y: i32,
) -> Result<(i32, i32), String> {
    let raw_hwnd = window
        .hwnd()
        .map_err(|e| format!("Failed to get window HWND: {e}"))?
        .0;
    let hwnd = HWND(raw_hwnd);

    // Save the previous foreground window so we can restore focus when closing
    unsafe {
        let fg = GetForegroundWindow();
        if fg != hwnd && !fg.0.is_null() {
            PREV_FOREGROUND_WINDOW.store(fg.0 as isize, Ordering::SeqCst);
        }
    }

    let work_area = get_cursor_monitor_work_area(cursor_x, cursor_y);
    let scale = get_cursor_monitor_dpi_scale(cursor_x, cursor_y);

    let outer_radius = (278.0 * scale).round() as i32;
    let half_w = (340.0 * scale).round() as i32;
    let half_h = (340.0 * scale).round() as i32;
    let win_w = (680.0 * scale).round() as i32;
    let win_h = (680.0 * scale).round() as i32;

    // Clamp center so the outer ring (radius 278) fits entirely within monitor work area
    let clamped_cx = cursor_x.clamp(
        work_area.left + outer_radius,
        work_area.right - outer_radius,
    );
    let clamped_cy = cursor_y.clamp(
        work_area.top + outer_radius,
        work_area.bottom - outer_radius,
    );

    let win_x = clamped_cx - half_w;
    let win_y = clamped_cy - half_h;

    let show_res = window.show();
    crate::log_msg(&format!("window.show() result: {:?}", show_res));

    unsafe {
        let swp_res = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            win_x,
            win_y,
            win_w,
            win_h,
            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        );
        let sw_res = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let is_vis = windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd).as_bool();
        crate::log_msg(&format!(
            "show_menu_at_cursor: hwnd={:?}, win=({}, {}, {}, {}), swp={:?}, sw={:?}, is_visible={}",
            raw_hwnd, win_x, win_y, win_w, win_h, swp_res, sw_res, is_vis
        ));
    }

    Ok((clamped_cx, clamped_cy))
}

/// Hide the radial menu window without losing its state, and restore focus to the previous active window.
pub fn hide_menu_window(window: &WebviewWindow) -> Result<(), String> {
    let raw_hwnd = window
        .hwnd()
        .map_err(|e| format!("Failed to get window HWND: {e}"))?
        .0;
    let hwnd = HWND(raw_hwnd);

    let _ = window.hide();
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
        let prev_raw = PREV_FOREGROUND_WINDOW.swap(0, Ordering::SeqCst);
        if prev_raw != 0 {
            let prev = HWND(prev_raw as *mut std::ffi::c_void);
            if IsWindow(prev).as_bool() {
                let _ = SetForegroundWindow(prev);
            }
        }
    }

    Ok(())
}
