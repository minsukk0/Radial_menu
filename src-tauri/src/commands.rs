use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use radial_core::{Item, MenuConfig};

use crate::context::ContextValues;
use crate::exec::execute_item;
use crate::menu_window::hide_menu_window;
use crate::security::approve_item as approve_item_security;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogOptions {
    pub category_id: Option<String>,
    pub item_id: Option<String>,
}

#[tauri::command]
pub fn frontend_log(msg: String) {
    crate::log_msg(&format!("[JS] {}", msg));
}

#[tauri::command]
pub fn get_menu_config(config_state: State<'_, Arc<RwLock<MenuConfig>>>) -> Result<MenuConfig, String> {
    let lock = config_state
        .read()
        .map_err(|e| format!("Failed to acquire config read lock: {e}"))?;
    Ok(lock.clone())
}

#[tauri::command]
pub fn commit_menu_action(
    app: AppHandle,
    config_state: State<'_, Arc<RwLock<MenuConfig>>>,
    target: Value,
) -> Result<(), String> {
    // Hide radial menu window upon committing
    if let Some(menu_win) = app.get_webview_window("menu") {
        let _ = hide_menu_window(&menu_win);
    }

    let target_type = target
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("none");

    match target_type {
        "item" => {
            // Retrieve item either directly from target payload or index
            let item_value = target.get("item");
            let item: Option<Item> = if let Some(val) = item_value {
                serde_json::from_value(val.clone()).ok()
            } else {
                let cat_idx = target.get("categoryIndex").and_then(|v| v.as_u64());
                let item_idx = target.get("itemIndex").and_then(|v| v.as_u64());
                if let (Some(c_i), Some(i_i)) = (cat_idx, item_idx) {
                    let lock = config_state.read().map_err(|e| e.to_string())?;
                    lock.categories
                        .get(c_i as usize)
                        .and_then(|c| c.items.get(i_i as usize).cloned())
                } else {
                    None
                }
            };

            if let Some(it) = item {
                let ctx = ContextValues::collect();
                if let Err(e) = execute_item(&it, &ctx) {
                    eprintln!("실행 실패: {e}");
                    // If elevation approval is required, open approve window
                    if e.contains("승인이 필요합니다") {
                        let _ = open_dialog(app, "approve".to_string(), None);
                    }
                    return Err(e);
                }
            }
        }

        "add_item" => {
            let cat_id = target
                .get("category")
                .and_then(|c| c.get("id"))
                .and_then(|id| id.as_str())
                .map(|s| s.to_string());

            let _ = open_dialog(
                app,
                "add-item".to_string(),
                Some(DialogOptions {
                    category_id: cat_id,
                    item_id: None,
                }),
            );
        }

        "add_category" => {
            let _ = open_dialog(
                app,
                "category-settings".to_string(),
                None,
            );
        }

        "close" | "none" => {}

        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub fn open_dialog(
    app: AppHandle,
    dialog_label: String,
    options: Option<DialogOptions>,
) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&dialog_label) {
        if let Some(opts) = options {
            let _ = win.emit("dialog:init", opts);
        }
        let _ = win.center();
        let _ = win.show();
        let _ = win.set_focus();
        Ok(())
    } else {
        Err(format!("Dialog window '{}' not found", dialog_label))
    }
}

#[tauri::command]
pub fn approve_admin_item(
    config_state: State<'_, Arc<RwLock<MenuConfig>>>,
    item_id: Option<String>,
    item_data: Option<Value>,
) -> Result<(), String> {
    let item = if let Some(data) = item_data {
        serde_json::from_value::<Item>(data).ok()
    } else if let Some(id) = item_id {
        let lock = config_state.read().map_err(|e| e.to_string())?;
        lock.categories
            .iter()
            .flat_map(|c| &c.items)
            .find(|i| i.id() == id)
            .cloned()
    } else {
        None
    };

    if let Some(it) = item {
        approve_item_security(&it)?;
        let ctx = ContextValues::collect();
        let _ = execute_item(&it, &ctx);
        Ok(())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn close_menu_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("menu") {
        hide_menu_window(&win)?;
    }
    Ok(())
}

#[tauri::command]
pub fn close_dialog_window(app: AppHandle, dialog_label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&dialog_label) {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn add_menu_item(
    app: AppHandle,
    config_state: State<'_, Arc<RwLock<MenuConfig>>>,
    category_id: String,
    item: Value,
) -> Result<(), String> {
    let config_path = if let Ok(custom) = std::env::var("RADIAL_MENU_CONFIG") {
        std::path::PathBuf::from(custom)
    } else {
        radial_core::get_default_config_path()
    };

    let mut lock = config_state
        .write()
        .map_err(|e| format!("설정 락 획득 실패: {e}"))?;

    let cat_idx = lock
        .categories
        .iter()
        .position(|c| {
            c.id == category_id
                || c.id.eq_ignore_ascii_case(&category_id)
                || category_id.starts_with(c.kind.as_str())
                || c.id.starts_with(&category_id)
        })
        .ok_or_else(|| format!("분류 '{}'를 찾을 수 없습니다.", category_id))?;

    let kind = lock.categories[cat_idx].kind;
    let mut item_obj = radial_core::deserialize_item_for_kind(kind, &item)
        .map_err(|e| format!("유효하지 않은 항목 형식: {e}"))?;

    if item_obj.id().trim().is_empty() {
        let mut existing: std::collections::HashSet<String> = std::collections::HashSet::new();
        for c in &lock.categories {
            for it in &c.items {
                existing.insert(it.id().to_string());
            }
        }
        let new_id = radial_core::generate_unique_id(item_obj.label(), &existing);
        item_obj.set_id(new_id);
    }

    lock.categories[cat_idx].items.push(item_obj);

    radial_core::write_config_atomic(&config_path, &mut lock, None)
        .map_err(|e| format!("설정 저장 실패: {e}"))?;

    let updated_config = lock.clone();
    let _ = app.emit("menu:config_changed", serde_json::json!({ "config": updated_config }));

    crate::log_msg(&format!("add_menu_item: Successfully added item to category '{}'", category_id));
    Ok(())
}

#[tauri::command]
pub fn remove_menu_item(
    app: AppHandle,
    config_state: State<'_, Arc<RwLock<MenuConfig>>>,
    item_id: String,
) -> Result<(), String> {
    let config_path = if let Ok(custom) = std::env::var("RADIAL_MENU_CONFIG") {
        std::path::PathBuf::from(custom)
    } else {
        radial_core::get_default_config_path()
    };

    let mut lock = config_state
        .write()
        .map_err(|e| format!("설정 락 획득 실패: {e}"))?;

    let mut found = false;
    for cat in &mut lock.categories {
        if let Some(pos) = cat.items.iter().position(|it| it.id() == item_id) {
            cat.items.remove(pos);
            found = true;
            break;
        }
    }

    if !found {
        return Err(format!("항목 '{}'을(를) 찾을 수 없습니다.", item_id));
    }

    radial_core::write_config_atomic(&config_path, &mut lock, None)
        .map_err(|e| format!("설정 저장 실패: {e}"))?;

    let updated_config = lock.clone();
    let _ = app.emit("menu:config_changed", serde_json::json!({ "config": updated_config }));

    crate::log_msg(&format!("remove_menu_item: Successfully removed item '{}'", item_id));
    Ok(())
}

#[tauri::command]
pub fn add_menu_category(
    app: AppHandle,
    config_state: State<'_, Arc<RwLock<MenuConfig>>>,
    kind: String,
    label: String,
) -> Result<(), String> {
    use std::str::FromStr;
    let category_kind = radial_core::CategoryKind::from_str(&kind)
        .map_err(|e| format!("유효하지 않은 분류 종류: {e}"))?;

    let config_path = if let Ok(custom) = std::env::var("RADIAL_MENU_CONFIG") {
        std::path::PathBuf::from(custom)
    } else {
        radial_core::get_default_config_path()
    };

    let mut lock = config_state
        .write()
        .map_err(|e| format!("설정 락 획득 실패: {e}"))?;

    let mut existing: std::collections::HashSet<String> = std::collections::HashSet::new();
    for c in &lock.categories {
        existing.insert(c.id.clone());
    }
    let cat_id = radial_core::generate_unique_id(&label, &existing);
    let new_cat = radial_core::Category::new(cat_id, category_kind, label);
    lock.categories.push(new_cat);

    radial_core::write_config_atomic(&config_path, &mut lock, None)
        .map_err(|e| format!("설정 저장 실패: {e}"))?;

    let updated_config = lock.clone();
    let _ = app.emit("menu:config_changed", serde_json::json!({ "config": updated_config }));

    crate::log_msg("add_menu_category: Successfully added category");
    Ok(())
}

#[tauri::command]
pub fn browse_file() -> Result<Option<String>, String> {
    std::thread::spawn(|| {
        use windows::Win32::System::Com::*;
        use windows::Win32::UI::Shell::*;
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let dialog: IFileOpenDialog = match CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL) {
                Ok(d) => d,
                Err(e) => return Err(format!("CoCreateInstance failed: {e}")),
            };
            if dialog.Show(None).is_ok() {
                if let Ok(item) = dialog.GetResult() {
                    if let Ok(name) = item.GetDisplayName(SIGDN_FILESYSPATH) {
                        return Ok(Some(name.to_string().unwrap_or_default()));
                    }
                }
            }
            Ok(None)
        }
    })
    .join()
    .map_err(|_| "Browse file thread panicked".to_string())?
}

#[tauri::command]
pub fn browse_folder() -> Result<Option<String>, String> {
    std::thread::spawn(|| {
        use windows::Win32::System::Com::*;
        use windows::Win32::UI::Shell::*;
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let dialog: IFileOpenDialog = match CoCreateInstance(&FileOpenDialog, None, CLSCTX_ALL) {
                Ok(d) => d,
                Err(e) => return Err(format!("CoCreateInstance failed: {e}")),
            };
            let _ = dialog.SetOptions(FOS_PICKFOLDERS);
            if dialog.Show(None).is_ok() {
                if let Ok(item) = dialog.GetResult() {
                    if let Ok(name) = item.GetDisplayName(SIGDN_FILESYSPATH) {
                        return Ok(Some(name.to_string().unwrap_or_default()));
                    }
                }
            }
            Ok(None)
        }
    })
    .join()
    .map_err(|_| "Browse folder thread panicked".to_string())?
}

#[tauri::command]
pub fn get_autostart_status() -> Result<bool, String> {
    Ok(crate::autostart::is_autostart_enabled())
}

#[tauri::command]
pub fn set_autostart_status(app: AppHandle, enabled: bool) -> Result<bool, String> {
    crate::autostart::set_autostart(enabled)?;
    let status = crate::autostart::is_autostart_enabled();
    let _ = app.emit("autostart:changed", serde_json::json!({ "enabled": status }));
    Ok(status)
}

#[tauri::command]
pub fn save_categories(
    app: AppHandle,
    config_state: State<'_, Arc<RwLock<MenuConfig>>>,
    categories: Vec<radial_core::Category>,
) -> Result<(), String> {
    let config_path = if let Ok(custom) = std::env::var("RADIAL_MENU_CONFIG") {
        std::path::PathBuf::from(custom)
    } else {
        radial_core::get_default_config_path()
    };

    let mut lock = config_state
        .write()
        .map_err(|e| format!("설정 락 획득 실패: {e}"))?;

    let mut temp_config = lock.clone();
    temp_config.categories = categories.clone();

    radial_core::validate_config(&temp_config).map_err(|errs| {
        let err_msgs: Vec<String> = errs
            .into_iter()
            .map(|e| format!("{}: {}", e.path, e.message))
            .collect();
        format!("설정 유효성 검사 실패:\n{}", err_msgs.join("\n"))
    })?;

    lock.categories = categories;

    radial_core::write_config_atomic(&config_path, &mut lock, None)
        .map_err(|e| format!("설정 저장 실패: {e}"))?;

    let updated_config = lock.clone();
    let _ = app.emit("menu:config_changed", serde_json::json!({ "config": updated_config }));

    crate::log_msg("save_categories: Successfully updated and saved categories");
    Ok(())
}

