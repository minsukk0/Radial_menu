use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;
pub const TASK_NAME: &str = "RadialMenu";

/// Build argument vector for creating the scheduled task.
pub fn build_create_args(task_name: &str, exe_path: &str) -> Vec<String> {
    vec![
        "/Create".to_string(),
        "/TN".to_string(),
        task_name.to_string(),
        "/TR".to_string(),
        format!("\"{}\"", exe_path),
        "/SC".to_string(),
        "ONLOGON".to_string(),
        "/RL".to_string(),
        "HIGHEST".to_string(),
        "/F".to_string(),
    ]
}

/// Build argument vector for deleting the scheduled task.
pub fn build_delete_args(task_name: &str) -> Vec<String> {
    vec![
        "/Delete".to_string(),
        "/TN".to_string(),
        task_name.to_string(),
        "/F".to_string(),
    ]
}

/// Check whether the RadialMenu auto-start task is enabled in Windows Task Scheduler.
pub fn is_autostart_enabled() -> bool {
    let output = Command::new("schtasks")
        .args(["/Query", "/TN", TASK_NAME])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(e) => {
            crate::log_msg(&format!("autostart::is_autostart_enabled query error: {e}"));
            false
        }
    }
}

/// Enable or disable auto-start for RadialMenu via Windows Task Scheduler.
///
/// When enabled, creates an ONLOGON task with HIGHEST run level so it launches
/// elevated upon user login without showing a UAC prompt every boot.
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("실행 파일 경로를 가져올 수 없습니다: {e}"))?;
        let exe_path_str = current_exe.to_string_lossy();
        let args = build_create_args(TASK_NAME, &exe_path_str);

        let output = Command::new("schtasks")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("schtasks 실행 실패: {e}"))?;

        if output.status.success() {
            crate::log_msg("autostart::set_autostart: Successfully registered RadialMenu in Task Scheduler.");
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            let out_msg = String::from_utf8_lossy(&output.stdout);
            let detail = if !err_msg.trim().is_empty() {
                err_msg.trim()
            } else {
                out_msg.trim()
            };
            if detail.contains("액세스가 거부되었습니다") || detail.contains("Access is denied") {
                Err("자동 실행을 설정하려면 관리자 권한으로 실행해야 합니다.".to_string())
            } else {
                Err(format!("작업 스케줄러 등록 실패: {detail}"))
            }
        }
    } else {
        let args = build_delete_args(TASK_NAME);

        let output = Command::new("schtasks")
            .args(&args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("schtasks 실행 실패: {e}"))?;

        if output.status.success() {
            crate::log_msg("autostart::set_autostart: Successfully removed RadialMenu from Task Scheduler.");
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            let out_msg = String::from_utf8_lossy(&output.stdout);
            let detail = if !err_msg.trim().is_empty() {
                err_msg.trim()
            } else {
                out_msg.trim()
            };

            // If task was already not found, consider it successfully disabled
            if detail.contains("find") || detail.contains("찾을 수 없습니다") || detail.contains("존재하지") {
                crate::log_msg("autostart::set_autostart: Task did not exist, considered disabled.");
                return Ok(());
            }

            Err(format!("작업 스케줄러 삭제 실패: {detail}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_name_constant() {
        assert_eq!(TASK_NAME, "RadialMenu");
    }

    #[test]
    fn test_build_create_args() {
        let args = build_create_args("RadialMenu", "C:\\Program Files\\RadialMenu\\RadialMenu.exe");
        assert_eq!(
            args,
            vec![
                "/Create",
                "/TN",
                "RadialMenu",
                "/TR",
                "\"C:\\Program Files\\RadialMenu\\RadialMenu.exe\"",
                "/SC",
                "ONLOGON",
                "/RL",
                "HIGHEST",
                "/F"
            ]
        );
    }

    #[test]
    fn test_build_delete_args() {
        let args = build_delete_args("RadialMenu");
        assert_eq!(args, vec!["/Delete", "/TN", "RadialMenu", "/F"]);
    }

    #[test]
    fn test_query_does_not_panic() {
        // Querying should gracefully return a bool without crashing or hanging
        let _ = is_autostart_enabled();
    }
}
