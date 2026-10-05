use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use radial_core::{read_config, validate_config, MenuConfig};
use tauri::{AppHandle, Emitter};

/// Start watching the `menu.json` configuration file on disk.
/// Automatically reloads and notifies frontends when valid, maintaining fallback on invalid.
pub fn start_config_watcher(
    app: AppHandle,
    config_state: Arc<RwLock<MenuConfig>>,
    config_path: PathBuf,
) -> Result<RecommendedWatcher, String> {
    let watch_path = config_path.clone();
    let state_clone = config_state.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let is_modify = matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                );

                if is_modify {
                    // Small sleep for editor flush debouncing
                    std::thread::sleep(Duration::from_millis(50));

                    match read_config(&watch_path) {
                        Ok((new_config, _)) => match validate_config(&new_config) {
                            Ok(_) => {
                                {
                                    let mut lock = state_clone.write().unwrap();
                                    *lock = new_config.clone();
                                }
                                let _ = app.emit("menu:config_changed", serde_json::json!({ "config": new_config }));
                            }
                            Err(val_errs) => {
                                eprintln!(
                                    "Radial Menu: 설정 검증 실패 (이전 정상 설정 유지됨): {:?}",
                                    val_errs
                                );
                            }
                        },
                        Err(e) => {
                            eprintln!(
                                "Radial Menu: 설정 파일 읽기 실패 (이전 정상 설정 유지됨): {e}"
                            );
                        }
                    }
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to initialize file watcher: {e}"))?;

    let parent_dir = config_path
        .parent()
        .ok_or_else(|| "Invalid config path parent".to_string())?;

    let _ = std::fs::create_dir_all(parent_dir);

    watcher
        .watch(parent_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch config directory: {e}"))?;

    Ok(watcher)
}
