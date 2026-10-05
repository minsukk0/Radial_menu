use crate::model::{ensure_ids, MenuConfig};
use crate::validate::{validate_config, ValidationError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O 오류: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 직렬화/역직렬화 오류: {0}")]
    Json(#[from] serde_json::Error),

    #[error("설정 유효성 검사 실패: {0:?}")]
    Validation(Vec<ValidationError>),

    #[error("동시성 충돌: {0}")]
    Conflict(String),

    #[error("잠금 획득 실패: {0}")]
    Lock(String),

    #[error("설정 파일을 찾을 수 없습니다: {0}")]
    NotFound(String),

    #[error("수정 작업 오류: {0}")]
    Custom(String),
}

/// Resolves the active menu.json configuration path.
///
/// Priority:
/// 1. `RADIAL_MENU_CONFIG` environment variable
/// 2. `%APPDATA%\RadialMenu\menu.json` (on Windows) or config directory fallback
pub fn get_default_config_path() -> PathBuf {
    if let Ok(path_str) = std::env::var("RADIAL_MENU_CONFIG") {
        if !path_str.trim().is_empty() {
            return PathBuf::from(path_str);
        }
    }

    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("RadialMenu").join("menu.json")
    } else {
        PathBuf::from("menu.json")
    }
}

/// Helper to get lock file path for a config path.
pub fn get_lock_path(config_path: &Path) -> PathBuf {
    config_path.with_extension("json.lock")
}

/// Helper to get backup file path for a config path.
pub fn get_backup_path(config_path: &Path) -> PathBuf {
    config_path.with_extension("json.bak")
}

/// RAII file lock guard.
pub struct LockGuard {
    file: File,
    #[allow(dead_code)]
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

/// Acquires an exclusive lock on the configuration lock file with a timeout.
pub fn acquire_lock(config_path: &Path, timeout: Duration) -> Result<LockGuard, StoreError> {
    let lock_path = get_lock_path(config_path);
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| StoreError::Lock(format!("잠금 파일 열기 실패 ({}): {e}", lock_path.display())))?;

    let start = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => {
                return Ok(LockGuard {
                    file,
                    path: lock_path,
                });
            }
            Err(_) if start.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(StoreError::Lock(format!(
                    "잠금 파일 획득 시간 초과 ({}): {e}",
                    lock_path.display()
                )));
            }
        }
    }
}

/// Reads the menu configuration from disk, returning config and last modified time.
pub fn read_config(path: &Path) -> Result<(MenuConfig, Option<SystemTime>), StoreError> {
    if !path.exists() {
        return Ok((MenuConfig::default(), None));
    }

    let mtime = std::fs::metadata(path).ok().and_then(|m| m.modified().ok());
    let content = std::fs::read_to_string(path)?;
    let content = content.trim_start_matches('\u{feff}');
    let config: MenuConfig = serde_json::from_str(content)?;
    Ok((config, mtime))
}

/// Atomically writes the configuration to disk with backup and disk freshness check.
pub fn write_config_atomic(
    path: &Path,
    config: &mut MenuConfig,
    expected_mtime: Option<SystemTime>,
) -> Result<(), StoreError> {
    // 1. Verify disk freshness if expected_mtime is provided
    if let Some(expected) = expected_mtime {
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(current_mtime) = metadata.modified() {
                    // Give a tiny tolerance for filesystem timestamp granularity
                    if current_mtime > expected
                        && current_mtime.duration_since(expected).unwrap_or_default()
                            > Duration::from_millis(10)
                    {
                        return Err(StoreError::Conflict(
                            "디스크의 설정 파일이 다른 프로세스에 의해 변경되었습니다.".to_string(),
                        ));
                    }
                }
            }
        }
    }

    // 2. Ensure IDs and validate
    ensure_ids(config);
    if let Err(errors) = validate_config(config) {
        return Err(StoreError::Validation(errors));
    }

    // 3. Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 4. Write to temp file
    let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));
    {
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        let json_bytes = serde_json::to_vec_pretty(config)?;
        temp_file.write_all(&json_bytes)?;
        temp_file.write_all(b"\n")?;
        temp_file.flush()?;
        temp_file.sync_all()?;
    }

    // 5. Backup previous file if it exists
    let backup_path = get_backup_path(path);
    if path.exists() {
        let _ = std::fs::copy(path, &backup_path);
    }

    // 6. Rename temp file to target path
    if let Err(err) = std::fs::rename(&temp_path, path) {
        // Retry rename once on Windows in case of temporary file handle contention
        std::thread::sleep(Duration::from_millis(20));
        if let Err(retry_err) = std::fs::rename(&temp_path, path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("원자적 파일 교체 실패 ({err}, 재시도: {retry_err})"),
            )));
        }
    }

    Ok(())
}

/// High-level function to lock, read, modify, validate, and atomically save configuration.
pub fn update_config<F>(
    custom_path: Option<&Path>,
    update_fn: F,
) -> Result<MenuConfig, StoreError>
where
    F: FnOnce(&mut MenuConfig) -> Result<(), StoreError>,
{
    let path_buf = custom_path
        .map(PathBuf::from)
        .unwrap_or_else(get_default_config_path);
    let path = path_buf.as_path();

    // 1. Acquire lock
    let _lock = acquire_lock(path, Duration::from_secs(5))?;

    // 2. Read latest config from disk
    let (mut config, mtime) = read_config(path)?;

    // 3. Apply modification
    update_fn(&mut config)?;

    // 4. Atomically write updated config
    write_config_atomic(path, &mut config, mtime)?;

    Ok(config)
}
