use crate::model::{
    Item, OpenWith, ShellType, SystemFn, TerminalAction, TerminalMode, TerminalWindow, WhenRunning,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// Compute normalized SHA-256 hash for an item, taking into account only execution-influencing fields.
/// Display fields such as `label`, `icon`, and `id` are excluded.
pub fn compute_item_hash(item: &Item) -> String {
    let mut map: BTreeMap<&str, serde_json::Value> = BTreeMap::new();

    match item {
        Item::App(app) => {
            map.insert("kind", serde_json::Value::String("app".into()));
            map.insert("path", serde_json::Value::String(app.path.trim().into()));
            map.insert(
                "args",
                serde_json::Value::String(app.args.as_deref().unwrap_or("").trim().into()),
            );
            let when_running = match app.when_running.unwrap_or(WhenRunning::Focus) {
                WhenRunning::Focus => "focus",
                WhenRunning::New => "new",
            };
            map.insert("whenRunning", serde_json::Value::String(when_running.into()));
            map.insert(
                "runAsAdmin",
                serde_json::Value::Bool(app.run_as_admin.unwrap_or(false)),
            );
        }
        Item::Terminal(term) => {
            map.insert("kind", serde_json::Value::String("terminal".into()));
            let action = match term.action {
                TerminalAction::Open => "open",
                TerminalAction::Run => "run",
            };
            map.insert("action", serde_json::Value::String(action.into()));

            let mode = match term.mode.unwrap_or(TerminalMode::Command) {
                TerminalMode::Command => "command",
                TerminalMode::File => "file",
            };
            map.insert("mode", serde_json::Value::String(mode.into()));

            map.insert(
                "command",
                serde_json::Value::String(term.command.as_deref().unwrap_or("").trim().into()),
            );
            map.insert(
                "file",
                serde_json::Value::String(term.file.as_deref().unwrap_or("").trim().into()),
            );
            map.insert(
                "args",
                serde_json::Value::String(term.args.as_deref().unwrap_or("").trim().into()),
            );

            let shell = match term.shell.unwrap_or(ShellType::Powershell) {
                ShellType::Powershell => "powershell",
                ShellType::Cmd => "cmd",
                ShellType::Wsl => "wsl",
                ShellType::Gitbash => "gitbash",
            };
            map.insert("shell", serde_json::Value::String(shell.into()));

            map.insert(
                "workDir",
                serde_json::Value::String(term.working_directory().unwrap_or("").trim().into()),
            );

            let window = match term.window.unwrap_or(TerminalWindow::Hidden) {
                TerminalWindow::Hidden => "hidden",
                TerminalWindow::Visible => "visible",
                TerminalWindow::Keep => "keep",
            };
            map.insert("window", serde_json::Value::String(window.into()));

            map.insert(
                "runAsAdmin",
                serde_json::Value::Bool(term.run_as_admin.unwrap_or(false)),
            );
            map.insert(
                "reuseTab",
                serde_json::Value::Bool(term.reuse_tab.unwrap_or(true)),
            );
            map.insert(
                "notifyOnDone",
                serde_json::Value::Bool(term.notify_on_done.unwrap_or(false)),
            );
            map.insert(
                "copyOutput",
                serde_json::Value::Bool(term.copy_output.unwrap_or(false)),
            );
            map.insert(
                "confirmFirst",
                serde_json::Value::Bool(term.confirm_first.unwrap_or(false)),
            );
        }
        Item::Folder(folder) => {
            map.insert("kind", serde_json::Value::String("folder".into()));
            map.insert("target", serde_json::Value::String(folder.target.trim().into()));
            let open_with = match folder.open_target() {
                OpenWith::Default => "default",
                OpenWith::Explorer => "explorer",
                OpenWith::Browser => "browser",
                OpenWith::Current => "current",
            };
            map.insert("openWith", serde_json::Value::String(open_with.into()));
            map.insert(
                "selectTarget",
                serde_json::Value::Bool(folder.select_target.unwrap_or(false)),
            );
        }
        Item::System(sys) => {
            map.insert("kind", serde_json::Value::String("system".into()));
            let fn_str = match sys.fn_name {
                SystemFn::Lock => "lock",
                SystemFn::Sleep => "sleep",
                SystemFn::Screensaver => "screensaver",
                SystemFn::DisplayOff => "displayOff",
                SystemFn::VolumeMute => "volumeMute",
                SystemFn::RadialSettings => "radialSettings",
                SystemFn::Taskmgr => "taskmgr",
                SystemFn::Brightness => "brightness",
                SystemFn::EmptyBin => "emptybin",
            };
            map.insert("fn", serde_json::Value::String(fn_str.into()));
        }
    }

    let canonical_json = serde_json::to_string(&map).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical_json.as_bytes());
    let digest = hasher.finalize();
    format!("sha256:{}", hex::encode(digest))
}

/// Compute SHA-256 hash of a file's content on disk.
pub fn compute_file_hash(path: &Path) -> std::io::Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    Ok(format!("sha256:{}", hex::encode(digest)))
}
