use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, Manager,
};
use windows::core::HSTRING;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

pub const TRAY_ID: &str = "radial_tray";

/// Setup system tray icon and context menu.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let autostart_enabled = crate::autostart::is_autostart_enabled();

    let manage_item = MenuItemBuilder::with_id("manage", "항목 관리...").build(app)?;
    let manage_categories_item = MenuItemBuilder::with_id("manage_categories", "상위 메뉴(분류) 설정...").build(app)?;
    let open_config_item = MenuItemBuilder::with_id("open_config", "설정 파일(menu.json) 열기").build(app)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let autostart_item = CheckMenuItemBuilder::with_id("autostart", "Windows 시작 시 자동 실행")
        .checked(autostart_enabled)
        .build(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "종료").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&manage_item)
        .item(&manage_categories_item)
        .item(&open_config_item)
        .item(&separator1)
        .item(&autostart_item)
        .item(&separator2)
        .item(&quit_item)
        .build()?;

    let icon = if let Some(icon) = app.default_window_icon() {
        icon.clone()
    } else if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")) {
        icon
    } else if let Ok(icon) = tauri::image::Image::from_app_icon_resource(32) {
        icon
    } else {
        tauri::image::Image::new_owned(vec![0u8; 16 * 16 * 4], 16, 16)
    };

    let autostart_item_for_event = autostart_item.clone();
    app.listen("autostart:changed", move |event| {
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
                let _ = autostart_item_for_event.set_checked(enabled);
            }
        }
    });

    let autostart_item_clone = autostart_item.clone();

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Radial Menu")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            let id = event.id().as_ref();
            crate::log_msg(&format!("Tray menu event: {id}"));

            match id {
                "manage" => {
                    if let Some(win) = app_handle.get_webview_window("add-item") {
                        let _ = win.center();
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
                "manage_categories" => {
                    if let Some(win) = app_handle.get_webview_window("category-settings") {
                        let _ = win.center();
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
                "open_config" => {
                    let config_path = if let Ok(custom) = std::env::var("RADIAL_MENU_CONFIG") {
                        std::path::PathBuf::from(custom)
                    } else {
                        radial_core::get_default_config_path()
                    };

                    if !config_path.exists() {
                        let mut template = radial_core::MenuConfig::default_template();
                        let _ = radial_core::write_config_atomic(&config_path, &mut template, None);
                    }

                    let path_h = HSTRING::from(config_path.to_string_lossy().as_ref());
                    unsafe {
                        ShellExecuteW(
                            None,
                            windows::core::w!("open"),
                            windows::core::w!("notepad.exe"),
                            &path_h,
                            None,
                            SW_SHOWNORMAL,
                        );
                    }
                }
                "autostart" => {
                    let current = crate::autostart::is_autostart_enabled();
                    let new_state = !current;
                    match crate::autostart::set_autostart(new_state) {
                        Ok(()) => {
                            let _ = autostart_item_clone.set_checked(new_state);
                            let _ = app_handle.emit("autostart:changed", serde_json::json!({ "enabled": new_state }));
                            crate::log_msg(&format!("Autostart toggled to {new_state} via tray"));
                        }
                        Err(e) => {
                            let _ = autostart_item_clone.set_checked(current);
                            crate::log_msg(&format!("Failed to toggle autostart via tray: {e}"));
                        }
                    }
                }
                "quit" => {
                    crate::log_msg("Radial Menu exiting cleanly via tray...");
                    crate::input::hook::stop_input_hook();
                    app_handle.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app_handle = tray.app_handle();
                if let Some(win) = app_handle.get_webview_window("add-item") {
                    let _ = win.center();
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;

    crate::log_msg("System tray setup completed successfully.");
    Ok(())
}
