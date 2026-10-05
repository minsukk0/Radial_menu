use serde_json::Value;
use std::process::Command;
use tempfile::tempdir;

fn run_rmctl(args: &[&str], config_path: Option<&std::path::Path>) -> (i32, Value) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rmctl"));
    cmd.args(args);
    if let Some(path) = config_path {
        cmd.env("RADIAL_MENU_CONFIG", path);
    }
    let output = cmd.output().expect("failed to execute rmctl");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: Value = serde_json::from_str(&stdout).unwrap_or_else(|_| {
        serde_json::json!({
            "raw_stdout": stdout,
            "raw_stderr": String::from_utf8_lossy(&output.stderr)
        })
    });
    (code, val)
}

#[test]
fn test_cli_path() {
    let (code, val) = run_rmctl(&["path"], None);
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert!(val["path"].is_string());
}

#[test]
fn test_cli_schema() {
    let (code, val) = run_rmctl(&["schema"], None);
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert!(val["schema"]["properties"]["categories"].is_object());
}

fn get_example_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("menu.example.json")
}

#[test]
fn test_cli_validate() {
    let dir = tempdir().unwrap();
    let valid_file = dir.path().join("valid.json");
    std::fs::copy(get_example_path(), &valid_file).unwrap();

    let (code, val) = run_rmctl(&["validate", "--file", valid_file.to_str().unwrap()], None);
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);

    let invalid_file = dir.path().join("invalid.json");
    std::fs::write(&invalid_file, r#"{"version": 99, "accent": "not_hex", "categories": []}"#).unwrap();

    let (code, val) = run_rmctl(&["validate", "--file", invalid_file.to_str().unwrap()], None);
    assert_eq!(code, 1);
    assert_eq!(val["ok"], false);
    assert!(val["errors"].is_array());
}

#[test]
fn test_cli_category_and_item_lifecycle() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_menu.json");
    std::fs::copy(get_example_path(), &config_path).unwrap();

    // 1. Category add
    let (code, val) = run_rmctl(
        &["category", "add", "--kind", "terminal", "--label", "추가 터미널", "--id", "custom-term"],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["id"], "custom-term");

    // 2. Item add with --json
    let item_json = r#"{"label":"새 터미널","action":"open","shell":"powershell"}"#;
    let (code, val) = run_rmctl(
        &["item", "add", "--category", "custom-term", "--json", item_json],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["needsApproval"], false);
    let item_id = val["id"].as_str().unwrap().to_string();

    // 3. Item add with admin terminal
    let admin_item_json = r#"{"label":"관리자 세션","action":"open","shell":"powershell","runAsAdmin":true}"#;
    let (code, val) = run_rmctl(
        &["item", "add", "--category", "custom-term", "--json", admin_item_json],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["needsApproval"], true);

    // 4. Item update
    let patch_json = r#"{"label":"수정된 터미널"}"#;
    let (code, val) = run_rmctl(
        &["item", "update", &item_id, "--json", patch_json],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["item"]["label"], "수정된 터미널");

    // 5. Item move
    let (code, val) = run_rmctl(
        &["item", "move", &item_id, "--to", "0"],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["newIndex"], 0);

    // 6. List category
    let (code, val) = run_rmctl(
        &["list", "--category", "custom-term"],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["items"].as_array().unwrap().len(), 2);

    // 7. Item remove
    let (code, val) = run_rmctl(
        &["item", "remove", &item_id],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["removedId"], item_id);

    // 8. Category remove
    let (code, val) = run_rmctl(
        &["category", "remove", "custom-term"],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
    assert_eq!(val["removedId"], "custom-term");
}

#[test]
fn test_cli_item_add_create_category() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("empty_menu.json");
    std::fs::write(&config_path, r##"{"version": 1, "accent": "#1E9BFF", "categories": []}"##).unwrap();

    let item_json = r#"{"label":"내 폴더","target":"C:\\"}"#;

    // Fail without --create-category
    let (code, val) = run_rmctl(
        &["item", "add", "--category", "folder", "--json", item_json],
        Some(&config_path),
    );
    assert_eq!(code, 2);
    assert_eq!(val["ok"], false);

    // Succeed with --create-category
    let (code, val) = run_rmctl(
        &["item", "add", "--category", "folder", "--create-category", "--json", item_json],
        Some(&config_path),
    );
    assert_eq!(code, 0);
    assert_eq!(val["ok"], true);
}
