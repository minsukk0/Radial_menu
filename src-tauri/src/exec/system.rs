use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Power::SetSuspendState;
use windows::Win32::System::Shutdown::LockWorkStation;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_VOLUME_MUTE,
};
use windows::Win32::UI::Shell::SHEmptyRecycleBinW;
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageW, HWND_BROADCAST, SC_MONITORPOWER, WM_SYSCOMMAND,
};
use radial_core::{SystemFn, SystemItem};

use crate::launcher::{spawn, Elevation, LaunchSpec};

const SC_SCREENSAVE: u32 = 0xF140;

/// Empty recycle bin flags (SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI)
const SHERB_NOCONFIRMATION: u32 = 0x00000001;
const SHERB_NOPROGRESSUI: u32 = 0x00000002;

/// Execute a system action item.
pub fn execute_system(item: &SystemItem) -> Result<(), String> {
    match item.fn_name {
        SystemFn::Lock => unsafe {
            LockWorkStation().map_err(|e| format!("화면 잠금 실패: {e}"))?;
            Ok(())
        },

        SystemFn::Sleep => unsafe {
            // SetSuspendState(hibernate = false, force = false, disable_wake = false)
            let res = SetSuspendState(false, false, false);
            if res.0 == 0 {
                Err("절전 모드 전환 실패".to_string())
            } else {
                Ok(())
            }
        },

        SystemFn::Screensaver => unsafe {
            let _ = SendMessageW(
                HWND_BROADCAST,
                WM_SYSCOMMAND,
                WPARAM(SC_SCREENSAVE as usize),
                LPARAM(0),
            );
            Ok(())
        },

        SystemFn::DisplayOff => unsafe {
            // SC_MONITORPOWER with parameter 2 turns the display off
            let _ = SendMessageW(
                HWND_BROADCAST,
                WM_SYSCOMMAND,
                WPARAM(SC_MONITORPOWER as usize),
                LPARAM(2),
            );
            Ok(())
        },

        SystemFn::VolumeMute => unsafe {
            // Toggle volume mute via simulated media key
            keybd_event(VK_VOLUME_MUTE.0 as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
            keybd_event(
                VK_VOLUME_MUTE.0 as u8,
                0,
                KEYEVENTF_KEYUP | KEYEVENTF_EXTENDEDKEY,
                0,
            );
            Ok(())
        },

        SystemFn::RadialSettings => {
            let spec = LaunchSpec::new("explorer.exe").with_args("ms-settings:");
            spawn(&spec, Elevation::User).map_err(|e| format!("설정 실행 실패: {e}"))?;
            Ok(())
        },

        SystemFn::Taskmgr => {
            let spec = LaunchSpec::new("taskmgr.exe");
            spawn(&spec, Elevation::User).map_err(|e| format!("작업 관리자 실행 실패: {e}"))?;
            Ok(())
        },

        SystemFn::Brightness => {
            let spec = LaunchSpec::new("explorer.exe").with_args("ms-settings:display");
            spawn(&spec, Elevation::User).map_err(|e| format!("디스플레이 설정 실행 실패: {e}"))?;
            Ok(())
        },

        SystemFn::EmptyBin => unsafe {
            SHEmptyRecycleBinW(
                HWND(std::ptr::null_mut()),
                PCWSTR::null(),
                SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI,
            )
            .map_err(|e| format!("휴지통 비우기 실패: {e}"))?;
            Ok(())
        },
    }
}
