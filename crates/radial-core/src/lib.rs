pub mod hash;
pub mod model;
pub mod store;
pub mod validate;

pub use hash::{compute_file_hash, compute_item_hash};
pub use model::{
    deserialize_item_for_kind, ensure_ids, generate_unique_id, AppItem, Category, CategoryKind,
    FolderItem, Item, MenuConfig, OpenWith, ShellType, SystemFn, SystemItem, TerminalAction,
    TerminalItem, TerminalMode, TerminalWindow, WhenRunning,
};
pub use store::{
    acquire_lock, get_backup_path, get_default_config_path, get_lock_path, read_config,
    update_config, write_config_atomic, LockGuard, StoreError,
};
pub use validate::{validate_config, ValidationError};

/// Generates the JSON Schema for the `MenuConfig` struct.
pub fn generate_schema() -> schemars::schema::RootSchema {
    schemars::schema_for!(MenuConfig)
}

/// Generates pretty-printed JSON Schema string.
pub fn generate_schema_json() -> String {
    serde_json::to_string_pretty(&generate_schema()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::tempdir;

    #[test]
    fn test_generate_schema_file() {
        let schema_str = generate_schema_json();
        let schema_path = std::path::Path::new("../../schema/menu.schema.json");
        if let Some(parent) = schema_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(schema_path, format!("{schema_str}\n")).unwrap();
    }

    #[test]
    fn test_menu_config_default_and_serialization() {
        let mut config = MenuConfig::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.accent, "#1E9BFF");

        let mut cat = Category::new("apps", CategoryKind::App, "앱");
        cat.items.push(Item::App(AppItem {
            id: "notepad".into(),
            label: "메모장".into(),
            path: "C:\\Windows\\System32\\notepad.exe".into(),
            args: None,
            when_running: Some(WhenRunning::Focus),
            run_as_admin: Some(false),
        }));
        config.categories.push(cat);

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"notepad\""));

        let deserialized: MenuConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.categories.len(), 1);
        assert_eq!(deserialized.categories[0].items.len(), 1);
        assert_eq!(deserialized.categories[0].items[0].id(), "notepad");
    }

    #[test]
    fn test_item_update_from_json() {
        let mut item = Item::App(AppItem {
            id: "calc".into(),
            label: "계산기".into(),
            path: "calc.exe".into(),
            args: None,
            when_running: None,
            run_as_admin: None,
        });

        let patch = serde_json::json!({
            "label": "새 계산기",
            "runAsAdmin": true
        });

        item.update_from_json(&patch).unwrap();
        assert_eq!(item.label(), "새 계산기");
        assert!(item.run_as_admin());
    }

    #[test]
    fn test_generate_unique_id() {
        let mut existing = HashSet::new();
        existing.insert("vs-code".into());

        let id1 = generate_unique_id("VS Code", &existing);
        assert_eq!(id1, "vs-code-2");

        let id2 = generate_unique_id("새 터미널", &existing);
        assert_eq!(id2, "item");
    }

    #[test]
    fn test_validation_success() {
        let mut config = MenuConfig::default();
        let mut cat = Category::new("term", CategoryKind::Terminal, "터미널");
        cat.items.push(Item::Terminal(TerminalItem {
            id: "ps".into(),
            label: "PowerShell".into(),
            icon: Some("terminal".into()),
            action: TerminalAction::Run,
            mode: Some(TerminalMode::Command),
            command: Some("Get-Process \"{선택한 파일}\"".into()),
            file: None,
            args: None,
            shell: Some(ShellType::Powershell),
            work_dir: Some("{현재 폴더}".into()),
            start_dir: None,
            window: Some(TerminalWindow::Hidden),
            run_as_admin: Some(false),
            notify_on_done: Some(true),
            copy_output: None,
            confirm_first: None,
            reuse_tab: None,
        }));
        config.categories.push(cat);

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validation_errors() {
        let mut config = MenuConfig::default();
        config.version = 2; // invalid
        config.accent = "invalid_color".into(); // invalid

        let mut cat = Category::new("app_cat", CategoryKind::App, "앱");
        // Invalid item kind in App category
        cat.items.push(Item::Terminal(TerminalItem {
            id: "term-item".into(),
            label: "터미널".into(),
            icon: None,
            action: TerminalAction::Open,
            mode: None,
            command: None,
            file: None,
            args: None,
            shell: None,
            work_dir: None,
            start_dir: None,
            window: None,
            run_as_admin: None,
            notify_on_done: None,
            copy_output: None,
            confirm_first: None,
            reuse_tab: None,
        }));
        config.categories.push(cat);

        let res = validate_config(&config);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.path == "/version"));
        assert!(errors.iter().any(|e| e.path == "/accent"));
        assert!(errors.iter().any(|e| e.path == "/categories/0/items/0"));
    }

    #[test]
    fn test_placeholder_validation() {
        let mut config = MenuConfig::default();
        let mut cat = Category::new("app", CategoryKind::App, "앱");
        cat.items.push(Item::App(AppItem {
            id: "test-app".into(),
            label: "테스트".into(),
            path: "cmd.exe".into(),
            args: Some("--file {잘못된 자리표시자}".into()),
            when_running: None,
            run_as_admin: None,
        }));
        config.categories.push(cat);

        let errs = validate_config(&config).unwrap_err();
        assert!(errs.iter().any(|e| e.path == "/categories/0/items/0/args"));
    }

    #[test]
    fn test_duplicate_system_fn() {
        let mut config = MenuConfig::default();
        let mut cat = Category::new("sys", CategoryKind::System, "시스템");
        cat.items.push(Item::System(SystemItem {
            id: "lock1".into(),
            label: "잠금1".into(),
            icon: None,
            fn_name: SystemFn::Lock,
        }));
        cat.items.push(Item::System(SystemItem {
            id: "lock2".into(),
            label: "잠금2".into(),
            icon: None,
            fn_name: SystemFn::Lock,
        }));
        config.categories.push(cat);

        let errs = validate_config(&config).unwrap_err();
        assert!(errs.iter().any(|e| e.path == "/categories/0/items/1/fn"));
    }

    #[test]
    fn test_store_atomic_write_and_backup() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("menu.json");

        let mut config = MenuConfig::default();
        let mut cat = Category::new("apps", CategoryKind::App, "앱");
        cat.items.push(Item::App(AppItem {
            id: "notepad".into(),
            label: "메모장".into(),
            path: "notepad.exe".into(),
            args: None,
            when_running: None,
            run_as_admin: None,
        }));
        config.categories.push(cat);

        // First write
        write_config_atomic(&config_path, &mut config, None).unwrap();
        assert!(config_path.exists());

        // Update with update_config
        let updated = update_config(Some(&config_path), |cfg| {
            cfg.accent = "#3D7BFF".into();
            Ok(())
        })
        .unwrap();

        assert_eq!(updated.accent, "#3D7BFF");
        let backup_path = get_backup_path(&config_path);
        assert!(backup_path.exists());

        let (read_back, _) = read_config(&config_path).unwrap();
        assert_eq!(read_back.accent, "#3D7BFF");
    }

    #[test]
    fn test_hash_calculation_and_invariance() {
        let item1 = Item::Terminal(TerminalItem {
            id: "item1".into(),
            label: "DNS 초기화".into(),
            icon: Some("activity".into()),
            action: TerminalAction::Run,
            mode: Some(TerminalMode::Command),
            command: Some("ipconfig /flushdns".into()),
            file: None,
            args: None,
            shell: Some(ShellType::Powershell),
            work_dir: None,
            start_dir: None,
            window: Some(TerminalWindow::Hidden),
            run_as_admin: Some(true),
            notify_on_done: Some(true),
            copy_output: None,
            confirm_first: None,
            reuse_tab: None,
        });

        // Same execution parameters, different label and icon and id
        let item2 = Item::Terminal(TerminalItem {
            id: "different-id".into(),
            label: "DNS Flush (Renamed)".into(),
            icon: Some("play".into()),
            action: TerminalAction::Run,
            mode: Some(TerminalMode::Command),
            command: Some("ipconfig /flushdns".into()),
            file: None,
            args: None,
            shell: Some(ShellType::Powershell),
            work_dir: None,
            start_dir: None,
            window: Some(TerminalWindow::Hidden),
            run_as_admin: Some(true),
            notify_on_done: Some(true),
            copy_output: None,
            confirm_first: None,
            reuse_tab: None,
        });

        // Item with changed command
        let item3 = Item::Terminal(TerminalItem {
            id: "item1".into(),
            label: "DNS 초기화".into(),
            icon: Some("activity".into()),
            action: TerminalAction::Run,
            mode: Some(TerminalMode::Command),
            command: Some("ipconfig /release".into()), // Different command!
            file: None,
            args: None,
            shell: Some(ShellType::Powershell),
            work_dir: None,
            start_dir: None,
            window: Some(TerminalWindow::Hidden),
            run_as_admin: Some(true),
            notify_on_done: Some(true),
            copy_output: None,
            confirm_first: None,
            reuse_tab: None,
        });

        let hash1 = compute_item_hash(&item1);
        let hash2 = compute_item_hash(&item2);
        let hash3 = compute_item_hash(&item3);

        assert!(hash1.starts_with("sha256:"));
        assert_eq!(hash1, hash2, "Label and icon changes must NOT affect hash");
        assert_ne!(hash1, hash3, "Command change MUST affect hash");
    }
}
