pub mod commands;
pub mod config_watch;
pub mod context;
pub mod exec;
pub mod input;
pub mod launcher;
pub mod menu_window;
pub mod security;
pub mod win32;

use std::sync::{Arc, RwLock};
use std::time::Duration;
use tauri::{Emitter, Listener, Manager};
use radial_core::{get_default_config_path, read_config, MenuConfig};

use input::gesture::GestureAction;
use input::hook::{dispatch_commit_response, start_input_hook};
use menu_window::{hide_menu_window, setup_menu_window_styles, show_menu_at_cursor};
use windows::core::w;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcessId, OpenProcess, TerminateProcess, WaitForSingleObject,
    PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

static SINGLE_INSTANCE_MUTEX: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Forcibly terminate any previously running RadialMenu processes to cleanly reclaim low-level hooks.
pub fn kill_other_instances() {
    unsafe {
        let current_pid = GetCurrentProcessId();
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(s) => s,
            Err(e) => {
                log_msg(&format!("kill_other_instances: CreateToolhelp32Snapshot failed: {e}"));
                return;
            }
        };

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let nul_pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let exe_name = String::from_utf16_lossy(&entry.szExeFile[..nul_pos]);

                if (exe_name.eq_ignore_ascii_case("RadialMenu.exe") || exe_name.to_lowercase().starts_with("radialmenu."))
                    && entry.th32ProcessID != current_pid
                {
                    log_msg(&format!("kill_other_instances: Found older instance '{}' (PID {}). Terminating...", exe_name, entry.th32ProcessID));
                    if let Ok(handle) = OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, entry.th32ProcessID) {
                        let _ = TerminateProcess(handle, 1);
                        let _ = WaitForSingleObject(handle, 1000);
                        let _ = windows::Win32::Foundation::CloseHandle(handle);
                        log_msg(&format!("kill_other_instances: Successfully terminated PID {}.", entry.th32ProcessID));
                    } else {
                        log_msg(&format!("kill_other_instances: OpenProcess failed for PID {} (requires admin rights).", entry.th32ProcessID));
                    }
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = windows::Win32::Foundation::CloseHandle(snapshot);
    }
}

pub fn log_msg(msg: &str) {
    let line = format!("[{}] {}\n", chrono::Local::now().format("%H:%M:%S%.3f"), msg);
    eprint!("{}", line);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("D:\\Python Workspace\\Radial_menu\\radial.log")
    {
        use std::io::Write;
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
}

pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        log_msg(&format!("PANIC OCCURRED: {:?}", info));
    }));

    // Forcibly terminate older/stale RadialMenu instances so hooks and ports are clean
    kill_other_instances();
    std::thread::sleep(Duration::from_millis(100));

    // Single-instance protection
    unsafe {
        match CreateMutexW(None, true, w!("Global\\RadialMenu_SingleInstance_Mutex")) {
            Ok(h) => {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    log_msg("RadialMenu instance already running. Exiting duplicate instance.");
                    eprintln!("RadialMenu is already running.");
                    std::process::exit(0);
                }
                SINGLE_INSTANCE_MUTEX.store(h.0 as isize, std::sync::atomic::Ordering::SeqCst);
            }
            Err(e) => {
                log_msg(&format!("Single-instance mutex warning: {e}"));
            }
        }
    }

    log_msg("=== RadialMenu Starting ===");
    let config_path = if let Ok(custom) = std::env::var("RADIAL_MENU_CONFIG") {
        std::path::PathBuf::from(custom)
    } else {
        get_default_config_path()
    };
    log_msg(&format!("Config path: {:?}", config_path));

    // Load initial configuration or initialize from default template
    let initial_config = if config_path.exists() {
        read_config(&config_path)
            .map(|(cfg, _)| cfg)
            .unwrap_or_default()
    } else {
        let mut template = MenuConfig::default_template();
        let _ = radial_core::write_config_atomic(&config_path, &mut template, None);
        template
    };
    let config_state = Arc::new(RwLock::new(initial_config));

    let app = tauri::Builder::default()
        .manage(config_state.clone())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "menu" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::frontend_log,
            commands::get_menu_config,
            commands::commit_menu_action,
            commands::open_dialog,
            commands::approve_admin_item,
            commands::close_menu_window,
            commands::close_dialog_window,
            commands::add_menu_item,
            commands::remove_menu_item,
            commands::add_menu_category,
            commands::browse_file,
            commands::browse_folder,
        ])
        .setup(move |app| {
            log_msg("Tauri setup hook executing...");
            let app_handle = app.handle().clone();

            // 1. Prepare menu window Win32 styles
            if let Some(menu_win) = app_handle.get_webview_window("menu") {
                log_msg(&format!("Found 'menu' window, setting up styles... HWND: {:?}", menu_win.hwnd()));
                if let Err(e) = setup_menu_window_styles(&menu_win) {
                    log_msg(&format!("Failed to configure menu window styles: {e}"));
                }
            } else {
                log_msg("ERROR: 'menu' window NOT FOUND in app_handle!");
            }

            // 2. Start config watcher and keep it alive in app state
            let config_path_for_open = config_path.clone();
            match config_watch::start_config_watcher(
                app_handle.clone(),
                config_state.clone(),
                config_path,
            ) {
                Ok(watcher) => {
                    app_handle.manage(watcher);
                }
                Err(e) => {
                    log_msg(&format!("Config watcher error: {e}"));
                }
            }

            // 3. Listen to frontend commit response
            app_handle.listen("menu:commit_result", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    let target = payload.get("target").cloned();
                    dispatch_commit_response(target.as_ref().map(|t| t.to_string()));
                }
            });

            // 4. Listen to frontend menu closed
            let app_for_closed = app_handle.clone();
            app_handle.listen("menu:closed", move |_| {
                if let Some(menu_win) = app_for_closed.get_webview_window("menu") {
                    let _ = hide_menu_window(&menu_win);
                }
            });

            // 5. Start Windows low-level input hooks and spawn action processor
            match start_input_hook() {
                Ok(receiver) => {
                    let hook_app = app_handle.clone();
                    let hook_state = config_state.clone();
                    let cfg_path = config_path_for_open.clone();

                    use std::sync::Mutex;
                    static MENU_ORIGIN: Mutex<(i32, i32, f64)> = Mutex::new((0, 0, 1.0));

                    std::thread::Builder::new()
                        .name("radial-action-processor".into())
                        .spawn(move || {
                            log_msg("radial-action-processor thread started");
                            while let Ok(action) = receiver.recv() {
                                let app = hook_app.clone();
                                let state = hook_state.clone();

                                match action {
                                    GestureAction::OpenMenu { x, y } => {
                                        log_msg(&format!("Processor: GestureAction::OpenMenu at ({}, {})", x, y));
                                        if let Some(menu_win) = app.get_webview_window("menu") {
                                            // Always sync with the latest disk configuration so external edits / rmctl take effect immediately
                                            if let Ok((fresh_cfg, _)) = radial_core::read_config(&cfg_path) {
                                                if radial_core::validate_config(&fresh_cfg).is_ok() {
                                                    let mut lock = state.write().unwrap();
                                                    *lock = fresh_cfg;
                                                }
                                            }

                                            match show_menu_at_cursor(&menu_win, x, y) {
                                                Ok((cx, cy)) => {
                                                    let scale = crate::win32::window::get_cursor_monitor_dpi_scale(x, y);
                                                    log_msg(&format!("Processor: show_menu_at_cursor returned clamped ({}, {}), scale={}", cx, cy, scale));
                                                    *MENU_ORIGIN.lock().unwrap() = (cx, cy, scale);
                                                    let cfg = state.read().unwrap().clone();
                                                    let emit_res = menu_win.emit(
                                                        "menu:open",
                                                        serde_json::json!({
                                                            "x": 340,
                                                            "y": 340,
                                                            "config": cfg,
                                                        }),
                                                    );
                                                    log_msg(&format!("Processor: menu:open emitted, result: {:?}", emit_res));

                                                    // Spawn Ctrl release watchdog thread while menu is open
                                                    std::thread::spawn(|| {
                                                        while crate::input::hook::is_menu_open() {
                                                            std::thread::sleep(Duration::from_millis(16));
                                                            if !crate::input::hook::is_ctrl_physically_down() {
                                                                crate::log_msg("Watchdog: Ctrl release detected via GetAsyncKeyState! Dispatching CtrlUp.");
                                                                crate::input::hook::dispatch_ctrl_up();
                                                                break;
                                                            }
                                                        }
                                                    });
                                                }
                                                Err(e) => {
                                                    log_msg(&format!("Processor ERROR: show_menu_at_cursor failed: {e}"));
                                                }
                                            }
                                        } else {
                                            log_msg("Processor ERROR: 'menu' window not found!");
                                        }
                                    }

                                    GestureAction::CloseMenu => {
                                        if let Some(menu_win) = app.get_webview_window("menu") {
                                            let _ = hide_menu_window(&menu_win);
                                            let _ = menu_win.emit("menu:close", ());
                                        }
                                    }

                                    GestureAction::RequestCommit => {
                                        if let Some(menu_win) = app.get_webview_window("menu") {
                                            let _ = menu_win.emit("menu:commit", ());

                                            // 200ms timeout guard
                                            std::thread::spawn(move || {
                                                std::thread::sleep(Duration::from_millis(200));
                                                // If still committing, close
                                                crate::input::hook::dispatch_timeout();
                                            });
                                        }
                                    }

                                    GestureAction::CursorMove { x, y } => {
                                        if let Some(menu_win) = app.get_webview_window("menu") {
                                            let (ox, oy, scale) = *MENU_ORIGIN.lock().unwrap();
                                            let s = if scale > 0.0 { scale } else { 1.0 };
                                            let local_x = 340.0 + (x - ox) as f64 / s;
                                            let local_y = 340.0 + (y - oy) as f64 / s;
                                            let _ = menu_win.emit(
                                                "menu:cursor_move",
                                                serde_json::json!({
                                                    "x": local_x.round() as i32,
                                                    "y": local_y.round() as i32
                                                }),
                                            );
                                        }
                                    }

                                    GestureAction::ExecuteTarget(target_str) => {
                                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&target_str) {
                                            let st = app.state::<Arc<RwLock<MenuConfig>>>();
                                            let _ = commands::commit_menu_action(app.clone(), st, val);
                                        }
                                    }
                                }
                            }
                        })
                        .expect("Failed to spawn action processor thread");
                }
                Err(e) => {
                    eprintln!("Failed to start input hooks: {e}");
                }
            }

            log_msg("Tauri setup completed successfully");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Error while building Radial Menu application");

    log_msg("Calling app.run()...");
    app.run(|_app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                log_msg("Tauri RunEvent: ExitRequested intercepted! Calling api.prevent_exit()");
                api.prevent_exit();
            }
            tauri::RunEvent::Ready => {
                log_msg("Tauri RunEvent: Ready");
            }
            _ => {}
        }
    });
}

