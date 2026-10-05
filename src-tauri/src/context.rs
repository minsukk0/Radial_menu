use std::collections::HashMap;
use std::path::PathBuf;
use radial_core::ShellType;

use crate::win32::shell::get_explorer_context;
use crate::win32::window::get_foreground_window_title;

/// Collected context values captured at the moment the radial menu was opened.
#[derive(Debug, Clone, Default)]
pub struct ContextValues {
    pub clipboard: String,
    pub selected_files: String,
    pub current_folder: String,
    pub active_window_title: String,
}

impl ContextValues {
    /// Collect current environment and window state.
    pub fn collect() -> Self {
        let active_window_title = get_foreground_window_title();
        let explorer = get_explorer_context();

        let current_folder = explorer.current_folder.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| r"C:\".to_string())
        });

        let selected_files = if explorer.selected_files.is_empty() {
            String::new()
        } else {
            explorer
                .selected_files
                .iter()
                .map(|f| format!("\"{f}\""))
                .collect::<Vec<_>>()
                .join(" ")
        };

        let clipboard = match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
            Ok(text) => text,
            Err(_) => String::new(),
        };

        Self {
            clipboard,
            selected_files,
            current_folder,
            active_window_title,
        }
    }

    /// Build environment variables map for child process execution.
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("RM_CLIPBOARD".to_string(), self.clipboard.clone());
        env.insert("RM_SELECTED".to_string(), self.selected_files.clone());
        env.insert("RM_CWD".to_string(), self.current_folder.clone());
        env.insert("RM_WINDOW_TITLE".to_string(), self.active_window_title.clone());
        env
    }

    /// Substitute placeholders for shell execution by mapping them to environment variable references.
    ///
    /// Prevents command injection vulnerabilities (especially in admin commands).
    pub fn substitute_shell_command(&self, command: &str, shell: ShellType) -> String {
        let (clip_ref, sel_ref, cwd_ref, title_ref) = match shell {
            ShellType::Powershell => (
                "$env:RM_CLIPBOARD",
                "$env:RM_SELECTED",
                "$env:RM_CWD",
                "$env:RM_WINDOW_TITLE",
            ),
            ShellType::Cmd => (
                "!RM_CLIPBOARD!",
                "!RM_SELECTED!",
                "!RM_CWD!",
                "!RM_WINDOW_TITLE!",
            ),
            ShellType::Wsl | ShellType::Gitbash => (
                "\"$RM_CLIPBOARD\"",
                "\"$RM_SELECTED\"",
                "\"$RM_CWD\"",
                "\"$RM_WINDOW_TITLE\"",
            ),
        };

        command
            .replace("{클립보드}", clip_ref)
            .replace("{선택한 파일}", sel_ref)
            .replace("{현재 폴더}", cwd_ref)
            .replace("{활성 창 제목}", title_ref)
    }

    /// Substitute placeholders in non-shell parameters (e.g. app args, working directories, folder targets).
    pub fn substitute_raw_text(&self, text: &str) -> String {
        let mut res = text.to_string();

        // If the user correctly wrapped in quotes: "{선택한 파일}", replace the whole thing
        // because selected_files already includes quotes for each file.
        res = res.replace("\"{선택한 파일}\"", &self.selected_files);

        // Fallback for unquoted and other placeholders
        res = res.replace("{클립보드}", &self.clipboard);
        res = res.replace("{선택한 파일}", &self.selected_files);
        res = res.replace("{현재 폴더}", &self.current_folder);
        res = res.replace("{활성 창 제목}", &self.active_window_title);

        // Expand environment variables like %USERPROFILE% or %APPDATA%
        if res.contains('%') {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                res = res.replace("%USERPROFILE%", &userprofile);
            }
            if let Ok(appdata) = std::env::var("APPDATA") {
                res = res.replace("%APPDATA%", &appdata);
            }
            if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                res = res.replace("%LOCALAPPDATA%", &localappdata);
            }
        }

        res
    }

    /// Resolve a working directory string with fallback to current folder.
    pub fn resolve_work_dir(&self, dir_opt: Option<&str>) -> PathBuf {
        match dir_opt {
            Some(d) if !d.trim().is_empty() => {
                let substituted = self.substitute_raw_text(d);
                PathBuf::from(substituted)
            }
            _ => PathBuf::from(&self.current_folder),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_placeholder_substitution() {
        let ctx = ContextValues {
            clipboard: "abc".into(),
            selected_files: "\"foo.txt\"".into(),
            current_folder: r"C:\Work".into(),
            active_window_title: "Code".into(),
        };

        let cmd = "python script.py \"{선택한 파일}\" --dir \"{현재 폴더}\" --clip {클립보드} --title \"{활성 창 제목}\"";

        let ps_res = ctx.substitute_shell_command(cmd, ShellType::Powershell);
        assert_eq!(
            ps_res,
            "python script.py \"$env:RM_SELECTED\" --dir \"$env:RM_CWD\" --clip $env:RM_CLIPBOARD --title \"$env:RM_WINDOW_TITLE\""
        );

        let cmd_res = ctx.substitute_shell_command(cmd, ShellType::Cmd);
        assert_eq!(
            cmd_res,
            "python script.py \"!RM_SELECTED!\" --dir \"!RM_CWD!\" --clip !RM_CLIPBOARD! --title \"!RM_WINDOW_TITLE!\""
        );

        let wsl_res = ctx.substitute_shell_command(cmd, ShellType::Wsl);
        assert_eq!(
            wsl_res,
            "python script.py \"\"$RM_SELECTED\"\" --dir \"\"$RM_CWD\"\" --clip \"$RM_CLIPBOARD\" --title \"\"$RM_WINDOW_TITLE\"\""
        );
    }

    #[test]
    fn test_raw_placeholder_substitution() {
        let ctx = ContextValues {
            clipboard: "clip_text".into(),
            selected_files: "\"a.png\"".into(),
            current_folder: r"C:\Folder".into(),
            active_window_title: "Chrome".into(),
        };

        let raw = "Open {현재 폴더} with {활성 창 제목}";
        assert_eq!(ctx.substitute_raw_text(raw), r"Open C:\Folder with Chrome");
    }
}
