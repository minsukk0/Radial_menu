use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
};

/// Get the title of the current foreground window.
pub fn get_foreground_window_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return String::new();
        }

        let length = GetWindowTextLengthW(hwnd);
        if length <= 0 {
            return String::new();
        }

        let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        if copied > 0 {
            String::from_utf16_lossy(&buffer[..copied as usize])
        } else {
            String::new()
        }
    }
}

/// Get the working area (`rcWork`) of the monitor containing the specified cursor coordinate.
pub fn get_cursor_monitor_work_area(x: i32, y: i32) -> RECT {
    unsafe {
        let pt = POINT { x, y };
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(hmon, &mut mi).as_bool() {
            mi.rcWork
        } else {
            // Fallback: standard primary display resolution
            RECT {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            }
        }
    }
}

/// Get the DPI scale factor (1.0 = 100%, 1.25 = 125%, 1.5 = 150%) for the monitor at (x, y).
pub fn get_cursor_monitor_dpi_scale(x: i32, y: i32) -> f64 {
    unsafe {
        let pt = POINT { x, y };
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut dpi_x = 96u32;
        let mut dpi_y = 96u32;

        if GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).is_ok() {
            (dpi_x as f64) / 96.0
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_metrics() {
        let wa = get_cursor_monitor_work_area(500, 500);
        let scale = get_cursor_monitor_dpi_scale(500, 500);
        println!("WorkArea: left={}, top={}, right={}, bottom={}, scale={}", wa.left, wa.top, wa.right, wa.bottom, scale);
        assert!(wa.right > wa.left);
        assert!(wa.bottom > wa.top);
    }
}
