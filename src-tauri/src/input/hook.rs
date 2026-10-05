use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use crossbeam_channel::{Receiver, Sender};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_CONTROL, VK_ESCAPE, VK_LCONTROL, VK_RCONTROL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_MBUTTONDOWN,
    WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::gesture::{GestureAction, GestureEvent, GestureStateMachine};

struct HookState {
    sm: GestureStateMachine,
    sender: Sender<GestureAction>,
    is_ctrl_down: bool,
    last_cursor_pos: (i32, i32),
}

static HOOK_STATE: Mutex<Option<HookState>> = Mutex::new(None);
static MOUSE_HHOOK: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static KEYBOARD_HHOOK: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static IS_RUNNING: AtomicBool = AtomicBool::new(false);
pub static IS_MENU_OPEN_ATOMIC: AtomicBool = AtomicBool::new(false);

/// Start the global mouse and keyboard hook on a dedicated thread.
/// Returns a receiver for `GestureAction`s.
pub fn start_input_hook() -> Result<Receiver<GestureAction>, String> {
    if IS_RUNNING.load(Ordering::SeqCst) {
        return Err("Input hook is already running".to_string());
    }

    let (tx, rx) = crossbeam_channel::unbounded();

    {
        let mut state = HOOK_STATE.lock().unwrap();
        *state = Some(HookState {
            sm: GestureStateMachine::new(),
            sender: tx,
            is_ctrl_down: false,
            last_cursor_pos: (0, 0),
        });
    }

    let (init_tx, init_rx) = crossbeam_channel::bounded::<Result<(), String>>(1);

    std::thread::Builder::new()
        .name("radial-input-hook".into())
        .spawn(move || {
            let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
            HOOK_THREAD_ID.store(thread_id, Ordering::SeqCst);

            let hinstance = unsafe {
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                    .map(|h| HINSTANCE(h.0))
                    .unwrap_or(HINSTANCE(std::ptr::null_mut()))
            };
            crate::log_msg(&format!("Hook: GetModuleHandleW returned {:?}", hinstance));

            let mouse_hook = match unsafe {
                SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0)
            } {
                Ok(h) => {
                    crate::log_msg(&format!("Hook: SetWindowsHookExW(WH_MOUSE_LL) succeeded: {:?}", h));
                    h
                }
                Err(e) => {
                    crate::log_msg(&format!("Hook ERROR: SetWindowsHookExW(WH_MOUSE_LL) failed: {e}"));
                    let _ = init_tx.send(Err(format!("SetWindowsHookExW(WH_MOUSE_LL) failed: {e}")));
                    return;
                }
            };

            let keyboard_hook = match unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), hinstance, 0)
            } {
                Ok(h) => {
                    crate::log_msg(&format!("Hook: SetWindowsHookExW(WH_KEYBOARD_LL) succeeded: {:?}", h));
                    h
                }
                Err(e) => {
                    crate::log_msg(&format!("Hook ERROR: SetWindowsHookExW(WH_KEYBOARD_LL) failed: {e}"));
                    unsafe { let _ = UnhookWindowsHookEx(mouse_hook); }
                    let _ = init_tx.send(Err(format!(
                        "SetWindowsHookExW(WH_KEYBOARD_LL) failed: {e}"
                    )));
                    return;
                }
            };

            MOUSE_HHOOK.store(mouse_hook.0 as isize, Ordering::SeqCst);
            KEYBOARD_HHOOK.store(keyboard_hook.0 as isize, Ordering::SeqCst);

            IS_RUNNING.store(true, Ordering::SeqCst);
            let _ = init_tx.send(Ok(()));

            // Standard Win32 Message Loop
            let mut msg = MSG::default();
            unsafe {
                while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            // Cleanup hooks on thread exit
            unsafe {
                let m = MOUSE_HHOOK.swap(0, Ordering::SeqCst);
                if m != 0 {
                    let _ = UnhookWindowsHookEx(HHOOK(m as *mut std::ffi::c_void));
                }
                let k = KEYBOARD_HHOOK.swap(0, Ordering::SeqCst);
                if k != 0 {
                    let _ = UnhookWindowsHookEx(HHOOK(k as *mut std::ffi::c_void));
                }
            }

            IS_RUNNING.store(false, Ordering::SeqCst);
        })
        .map_err(|e| format!("Failed to spawn hook thread: {e}"))?;

    init_rx
        .recv()
        .map_err(|e| format!("Failed to receive hook init signal: {e}"))?
        .map(|_| rx)
}

/// Stop the global hook thread.
pub fn stop_input_hook() {
    let thread_id = HOOK_THREAD_ID.load(Ordering::SeqCst);
    if thread_id != 0 {
        unsafe {
            let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}

/// Provide an external response (e.g. commit result from frontend) into the state machine.
pub fn dispatch_commit_response(target: Option<String>) {
    if let Ok(mut state_guard) = HOOK_STATE.lock() {
        if let Some(state) = state_guard.as_mut() {
            let (_, actions) = state.sm.handle_event(GestureEvent::CommitResponse(target));
            IS_MENU_OPEN_ATOMIC.store(state.sm.is_open(), Ordering::Relaxed);
            for act in actions {
                let _ = state.sender.send(act);
            }
        }
    }
}

/// Dispatch timeout for commit
pub fn dispatch_timeout() {
    if let Ok(mut state_guard) = HOOK_STATE.lock() {
        if let Some(state) = state_guard.as_mut() {
            let (_, actions) = state.sm.handle_event(GestureEvent::Timeout);
            IS_MENU_OPEN_ATOMIC.store(state.sm.is_open(), Ordering::Relaxed);
            for act in actions {
                let _ = state.sender.send(act);
            }
        }
    }
}

/// Dispatch cancellation/close from external trigger.
pub fn dispatch_close() {
    if let Ok(mut state_guard) = HOOK_STATE.lock() {
        if let Some(state) = state_guard.as_mut() {
            let (_, actions) = state.sm.handle_event(GestureEvent::EscapeDown);
            IS_MENU_OPEN_ATOMIC.store(state.sm.is_open(), Ordering::Relaxed);
            for act in actions {
                let _ = state.sender.send(act);
            }
        }
    }
}

/// Check if the radial menu gesture state is currently open or committing.
pub fn is_menu_open() -> bool {
    IS_MENU_OPEN_ATOMIC.load(Ordering::Relaxed)
}

/// Query hardware state directly via Win32 GetAsyncKeyState for Ctrl key.
pub fn is_ctrl_physically_down() -> bool {
    unsafe {
        (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0
            || (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000) != 0
            || (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000) != 0
    }
}

/// Dispatch CtrlUp event from the watchdog timer or external check.
pub fn dispatch_ctrl_up() {
    if let Ok(mut state_guard) = HOOK_STATE.lock() {
        if let Some(state) = state_guard.as_mut() {
            state.is_ctrl_down = false;
            if state.sm.is_open() {
                let (_, actions) = state.sm.handle_event(GestureEvent::CtrlUp);
                IS_MENU_OPEN_ATOMIC.store(state.sm.is_open(), Ordering::Relaxed);
                for act in actions {
                    let _ = state.sender.send(act);
                }
            }
        }
    }
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let ms = *(lparam.0 as *const MSLLHOOKSTRUCT);
        let msg = wparam.0 as u32;

        if let Ok(mut state_guard) = HOOK_STATE.lock() {
            if let Some(state) = state_guard.as_mut() {
                let event = match msg {
                    WM_MBUTTONDOWN => {
                        let is_ctrl = state.is_ctrl_down || is_ctrl_physically_down();
                        state.is_ctrl_down = is_ctrl;

                        Some(GestureEvent::MButtonDown {
                            ctrl_down: is_ctrl,
                            x: ms.pt.x,
                            y: ms.pt.y,
                        })
                    }
                    WM_MBUTTONUP => Some(GestureEvent::MButtonUp),
                    WM_LBUTTONDOWN => Some(GestureEvent::LButtonDown {
                        x: ms.pt.x,
                        y: ms.pt.y,
                    }),
                    WM_RBUTTONDOWN => Some(GestureEvent::RButtonDown),
                    WM_RBUTTONUP => Some(GestureEvent::RButtonUp),
                    WM_MOUSEWHEEL => {
                        let delta = ((ms.mouseData >> 16) & 0xffff) as i16 as i32;
                        Some(GestureEvent::MouseWheel { delta })
                    }
                    WM_MOUSEMOVE => {
                        if state.sm.is_open() && (state.last_cursor_pos != (ms.pt.x, ms.pt.y)) {
                            state.last_cursor_pos = (ms.pt.x, ms.pt.y);
                            let _ = state.sender.send(GestureAction::CursorMove {
                                x: ms.pt.x,
                                y: ms.pt.y,
                            });
                        }
                        None
                    }
                    _ => None,
                };

                if let Some(ev) = event {
                    let (swallow, actions) = state.sm.handle_event(ev);
                    IS_MENU_OPEN_ATOMIC.store(state.sm.is_open(), Ordering::Relaxed);
                    for act in actions {
                        let _ = state.sender.send(act);
                    }
                    if swallow {
                        return LRESULT(1);
                    }
                }
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let msg = wparam.0 as u32;
        let vk = kb.vkCode as u16;

        let is_ctrl_key = vk == VK_CONTROL.0 || vk == VK_LCONTROL.0 || vk == VK_RCONTROL.0;

        // Zero-latency bypass: if menu is NOT open and not a Ctrl key, immediately forward!
        if !is_ctrl_key && !IS_MENU_OPEN_ATOMIC.load(Ordering::Relaxed) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let is_key_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_key_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        let mut event_to_fire = None;

        if let Ok(mut state) = HOOK_STATE.lock() {
            if let Some(state) = state.as_mut() {
                if is_ctrl_key {
                    if is_key_down {
                        // Ignore repeated keydowns while holding Ctrl
                        if !state.is_ctrl_down {
                            state.is_ctrl_down = true;
                        }
                    } else if is_key_up {
                        state.is_ctrl_down = false;
                        if state.sm.is_open() {
                            event_to_fire = Some(GestureEvent::CtrlUp);
                        }
                    }
                } else if state.sm.is_open() {
                    if vk == VK_ESCAPE.0 {
                        if is_key_down {
                            event_to_fire = Some(GestureEvent::EscapeDown);
                        } else if is_key_up {
                            event_to_fire = Some(GestureEvent::EscapeUp);
                        }
                    } else {
                        if is_key_down {
                            event_to_fire = Some(GestureEvent::OtherKeyDown);
                        } else if is_key_up {
                            event_to_fire = Some(GestureEvent::OtherKeyUp);
                        }
                    }
                }

                if let Some(ev) = event_to_fire {
                    let (swallow, actions) = state.sm.handle_event(ev);
                    IS_MENU_OPEN_ATOMIC.store(state.sm.is_open(), Ordering::Relaxed);
                    for act in actions {
                        let _ = state.sender.send(act);
                    }
                    if swallow {
                        return LRESULT(1);
                    }
                }
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_initialization() {
        let res = start_input_hook();
        assert!(res.is_ok(), "start_input_hook failed: {:?}", res.err());
        stop_input_hook();
    }
}
