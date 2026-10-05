use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use radial_core::{
    ShellType, TerminalAction, TerminalItem, TerminalMode, TerminalWindow,
};

use crate::context::ContextValues;
use crate::launcher::{spawn, Elevation, LaunchSpec};

/// Build the command arguments and binary for the chosen shell environment.
fn build_shell_command(
    item: &TerminalItem,
    ctx: &ContextValues,
    work_dir: &Path,
) -> (String, Option<String>) {
    let shell = item.shell.unwrap_or(ShellType::Powershell);
    let window_mode = item.window.unwrap_or(TerminalWindow::Hidden);

    match item.action {
        TerminalAction::Open => match shell {
            ShellType::Powershell => {
                let dir_str = work_dir.to_string_lossy();
                let args = format!(
                    "-NoExit -Command \"Set-Location -LiteralPath '{}'\"",
                    dir_str.replace('\'', "''")
                );
                ("powershell.exe".to_string(), Some(args))
            }
            ShellType::Cmd => {
                let dir_str = work_dir.to_string_lossy();
                let args = format!("/k \"cd /d {}\"", dir_str);
                ("cmd.exe".to_string(), Some(args))
            }
            ShellType::Wsl => {
                let dir_str = work_dir.to_string_lossy();
                let args = format!("--cd \"{}\"", dir_str);
                ("wsl.exe".to_string(), Some(args))
            }
            ShellType::Gitbash => {
                let dir_str = work_dir.to_string_lossy();
                let git_bash = find_git_bash();
                let args = format!("--cd=\"{}\" --login -i", dir_str);
                (git_bash, Some(args))
            }
        },

        TerminalAction::Run => {
            let cmd_text = match item.mode.unwrap_or(TerminalMode::Command) {
                TerminalMode::Command => {
                    let raw = item.command.as_deref().unwrap_or_default();
                    ctx.substitute_shell_command(raw, shell)
                }
                TerminalMode::File => {
                    let file_raw = item.file.as_deref().unwrap_or_default();
                    let file_sub = ctx.substitute_raw_text(file_raw);
                    let args_raw = item.args.as_deref().unwrap_or_default();
                    let args_sub = ctx.substitute_shell_command(args_raw, shell);

                    match shell {
                        ShellType::Powershell => {
                            format!("& '{}' {}", file_sub.replace('\'', "''"), args_sub)
                        }
                        ShellType::Cmd => format!("\"{}\" {}", file_sub, args_sub),
                        ShellType::Wsl | ShellType::Gitbash => {
                            format!("\"{}\" {}", file_sub, args_sub)
                        }
                    }
                }
            };

            match shell {
                ShellType::Powershell => {
                    let flag = if window_mode == TerminalWindow::Keep {
                        "-NoExit -Command"
                    } else {
                        "-Command"
                    };
                    let dir_str = work_dir.to_string_lossy();
                    let full_cmd = format!(
                        "Set-Location -LiteralPath '{}'; {}",
                        dir_str.replace('\'', "''"),
                        cmd_text
                    );
                    (
                        "powershell.exe".to_string(),
                        Some(format!("{} \"{}\"", flag, full_cmd.replace('"', "\\\""))),
                    )
                }
                ShellType::Cmd => {
                    let flag = if window_mode == TerminalWindow::Keep {
                        "/k"
                    } else {
                        "/c"
                    };
                    let dir_str = work_dir.to_string_lossy();
                    let full_cmd = format!("cd /d \"{}\" && {}", dir_str, cmd_text);
                    ("cmd.exe".to_string(), Some(format!("{} \"{}\"", flag, full_cmd)))
                }
                ShellType::Wsl => {
                    let dir_str = work_dir.to_string_lossy();
                    (
                        "wsl.exe".to_string(),
                        Some(format!("--cd \"{}\" -e sh -c \"{}\"", dir_str, cmd_text)),
                    )
                }
                ShellType::Gitbash => {
                    let dir_str = work_dir.to_string_lossy();
                    let git_bash = find_git_bash();
                    (
                        git_bash,
                        Some(format!("--cd=\"{}\" -c \"{}\"", dir_str, cmd_text)),
                    )
                }
            }
        }
    }
}

/// Locate git bash executable on Windows.
fn find_git_bash() -> String {
    let candidate = PathBuf::from(r"C:\Program Files\Git\bin\bash.exe");
    if candidate.exists() {
        return candidate.to_string_lossy().to_string();
    }
    let candidate_x86 = PathBuf::from(r"C:\Program Files (x86)\Git\bin\bash.exe");
    if candidate_x86.exists() {
        return candidate_x86.to_string_lossy().to_string();
    }
    "bash.exe".to_string()
}

/// Execute a terminal item (open or run).
pub fn execute_terminal(item: &TerminalItem, ctx: &ContextValues) -> Result<(), String> {
    let is_admin = item.run_as_admin.unwrap_or(false);
    if is_admin {
        let term_item = radial_core::Item::Terminal(item.clone());
        let _ = crate::security::approve_item(&term_item);
    }

    let elevation = if is_admin {
        Elevation::Admin
    } else {
        Elevation::User
    };

    let work_dir = ctx.resolve_work_dir(item.working_directory());
    let (program, args) = build_shell_command(item, ctx, &work_dir);

    let is_hidden = match item.action {
        TerminalAction::Open => false,
        TerminalAction::Run => {
            matches!(
                item.window.unwrap_or(TerminalWindow::Hidden),
                TerminalWindow::Hidden
            )
        }
    };

    let mut spec = LaunchSpec::new(program)
        .with_work_dir(work_dir)
        .with_hidden(is_hidden);

    if let Some(a) = args {
        spec = spec.with_args(a);
    }

    for (k, v) in ctx.to_env_vars() {
        spec = spec.with_env(k, v);
    }

    // If copy_output or notify_on_done is requested in Run action:
    let copy_output = item.copy_output.unwrap_or(false);
    let notify_on_done = item.notify_on_done.unwrap_or(false);

    if item.action == TerminalAction::Run && (copy_output || notify_on_done) {
        let label = item.label.clone();
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new(&spec.program);
            if let Some(ref a) = spec.args {
                cmd.raw_arg(a);
            }
            if let Some(ref wd) = spec.work_dir {
                cmd.current_dir(wd);
            }
            for (k, v) in &spec.env_vars {
                cmd.env(k, v);
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                if spec.hidden {
                    cmd.creation_flags(0x08000000);
                }
            }

            if let Ok(output) = cmd.output() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if copy_output && !stdout.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(stdout);
                    }
                }
                if notify_on_done {
                    // Windows toast/beep or notification
                    eprintln!("Radial Menu: '{}' 완료됨", label);
                }
            }
        });
        return Ok(());
    }

    spawn(&spec, elevation).map_err(|e| format!("터미널 실행 실패: {e}"))?;
    Ok(())
}
