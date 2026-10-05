use crate::model::{Item, MenuConfig, TerminalAction, TerminalMode};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

/// Represents a single validation error with JSON Pointer path and descriptive message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

static ID_REGEX: OnceLock<Regex> = OnceLock::new();
static HEX_COLOR_REGEX: OnceLock<Regex> = OnceLock::new();

fn id_regex() -> &'static Regex {
    ID_REGEX.get_or_init(|| Regex::new(r"^[a-z0-9-]+$").unwrap())
}

fn hex_color_regex() -> &'static Regex {
    HEX_COLOR_REGEX.get_or_init(|| Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$").unwrap())
}

const ALLOWED_PLACEHOLDERS: &[&str] = &[
    "{선택한 파일}",
    "{현재 폴더}",
    "{활성 창 제목}",
    "{클립보드}",
];

const ALLOWED_ICONS: &[&str] = &[
    "terminal", "window", "folder", "note", "globe", "activity", "timer", "play",
];

/// Validates placeholders in a string value.
fn validate_placeholders(value: &str, path: &str, errors: &mut Vec<ValidationError>) {
    let mut chars = value.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        if ch == '{' {
            let mut end_pos = None;
            for (pos, inner_ch) in chars.by_ref() {
                if inner_ch == '}' {
                    end_pos = Some(pos);
                    break;
                } else if inner_ch == '{' {
                    errors.push(ValidationError::new(
                        path,
                        "중첩되었거나 닫히지 않은 자리표시자 '{'가 있습니다.",
                    ));
                    break;
                }
            }
            if let Some(end) = end_pos {
                let placeholder = &value[start..=end];
                if !ALLOWED_PLACEHOLDERS.contains(&placeholder) {
                    errors.push(ValidationError::new(
                        path,
                        format!(
                            "'{placeholder}'는 지원되지 않는 자리표시자입니다. 지원: {ALLOWED_PLACEHOLDERS:?}"
                        ),
                    ));
                }
            } else {
                errors.push(ValidationError::new(
                    path,
                    "닫히지 않은 자리표시자 '{'가 있습니다.",
                ));
            }
        } else if ch == '}' {
            errors.push(ValidationError::new(
                path,
                "열리지 않은 자리표시자 괄호 '}'가 있습니다.",
            ));
        }
    }
}

/// Validate an entire MenuConfig.
pub fn validate_config(config: &MenuConfig) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 1. Version check
    if config.version != 1 {
        errors.push(ValidationError::new(
            "/version",
            format!("지원되지 않는 설정 버전입니다: {}. 현재 지원 버전: 1", config.version),
        ));
    }

    // 2. Accent color check
    if !hex_color_regex().is_match(&config.accent) {
        errors.push(ValidationError::new(
            "/accent",
            format!(
                "강조색 '{}'은(는) 올바른 16진수 색상 코드(#RRGGBB)가 아닙니다.",
                config.accent
            ),
        ));
    }

    let mut category_ids = HashSet::new();
    let mut all_item_ids = HashSet::new();
    let mut seen_system_fns = HashSet::new();

    // 3. Categories validation
    for (cat_idx, category) in config.categories.iter().enumerate() {
        let cat_path = format!("/categories/{cat_idx}");

        // Category ID validation
        if category.id.is_empty() {
            errors.push(ValidationError::new(
                format!("{cat_path}/id"),
                "분류 ID가 비어 있습니다.",
            ));
        } else if !id_regex().is_match(&category.id) {
            errors.push(ValidationError::new(
                format!("{cat_path}/id"),
                format!(
                    "분류 ID '{}'은(는) 영문 소문자, 숫자, 하이픈(-)만 사용할 수 있습니다.",
                    category.id
                ),
            ));
        } else if !category_ids.insert(&category.id) {
            errors.push(ValidationError::new(
                format!("{cat_path}/id"),
                format!("중복된 분류 ID입니다: '{}'", category.id),
            ));
        }

        // Category Label validation
        if category.label.trim().is_empty() {
            errors.push(ValidationError::new(
                format!("{cat_path}/label"),
                "분류 이름(label)은 비어 있을 수 없습니다.",
            ));
        }

        // Items in category
        for (item_idx, item) in category.items.iter().enumerate() {
            let item_path = format!("{cat_path}/items/{item_idx}");

            // Category kind vs item kind
            if item.kind() != category.kind {
                errors.push(ValidationError::new(
                    &item_path,
                    format!(
                        "'{}' 분류에는 '{}' 항목을 담을 수 없습니다.",
                        category.kind.as_str(),
                        item.kind().as_str()
                    ),
                ));
            }

            // Item ID validation
            let item_id = item.id();
            if item_id.is_empty() {
                errors.push(ValidationError::new(
                    format!("{item_path}/id"),
                    "항목 ID가 비어 있습니다.",
                ));
            } else if !id_regex().is_match(item_id) {
                errors.push(ValidationError::new(
                    format!("{item_path}/id"),
                    format!(
                        "항목 ID '{item_id}'은(는) 영문 소문자, 숫자, 하이픈(-)만 사용할 수 있습니다."
                    ),
                ));
            } else if !all_item_ids.insert(item_id.to_string()) {
                errors.push(ValidationError::new(
                    format!("{item_path}/id"),
                    format!("중복된 항목 ID입니다: '{item_id}'"),
                ));
            }

            // Item label
            if item.label().trim().is_empty() {
                errors.push(ValidationError::new(
                    format!("{item_path}/label"),
                    "항목 이름(label)은 비어 있을 수 없습니다.",
                ));
            }

            // Kind-specific validation
            match item {
                Item::App(app) => {
                    let path_field = format!("{item_path}/path");
                    if app.path.trim().is_empty() {
                        errors.push(ValidationError::new(&path_field, "실행 파일 경로(path)는 필수입니다."));
                    } else {
                        validate_placeholders(&app.path, &path_field, &mut errors);
                    }

                    if let Some(args) = &app.args {
                        validate_placeholders(args, &format!("{item_path}/args"), &mut errors);
                    }
                }
                Item::Terminal(term) => {
                    if let Some(icon) = &term.icon {
                        if !ALLOWED_ICONS.contains(&icon.as_str()) {
                            errors.push(ValidationError::new(
                                format!("{item_path}/icon"),
                                format!("'{icon}'은(는) 허용되지 않는 아이콘입니다. 허용값: {ALLOWED_ICONS:?}"),
                            ));
                        }
                    }

                    match term.action {
                        TerminalAction::Run => {
                            let mode = term.mode.unwrap_or(TerminalMode::Command);
                            match mode {
                                TerminalMode::Command => {
                                    if let Some(cmd) = &term.command {
                                        if cmd.trim().is_empty() {
                                            errors.push(ValidationError::new(
                                                format!("{item_path}/command"),
                                                "명령 모드에서는 command 문자열이 필수입니다.",
                                            ));
                                        } else {
                                            validate_placeholders(cmd, &format!("{item_path}/command"), &mut errors);
                                        }
                                    } else {
                                        errors.push(ValidationError::new(
                                            format!("{item_path}/command"),
                                            "명령 모드에서는 command 문자열이 필수입니다.",
                                        ));
                                    }
                                }
                                TerminalMode::File => {
                                    if let Some(file) = &term.file {
                                        if file.trim().is_empty() {
                                            errors.push(ValidationError::new(
                                                format!("{item_path}/file"),
                                                "파일 모드에서는 스크립트 파일 경로(file)가 필수입니다.",
                                            ));
                                        } else {
                                            validate_placeholders(file, &format!("{item_path}/file"), &mut errors);
                                        }
                                    } else {
                                        errors.push(ValidationError::new(
                                            format!("{item_path}/file"),
                                            "파일 모드에서는 스크립트 파일 경로(file)가 필수입니다.",
                                        ));
                                    }
                                }
                            }
                        }
                        TerminalAction::Open => {
                            // Open action validations if needed
                        }
                    }

                    if let Some(args) = &term.args {
                        validate_placeholders(args, &format!("{item_path}/args"), &mut errors);
                    }
                    if let Some(work_dir) = &term.work_dir {
                        validate_placeholders(work_dir, &format!("{item_path}/workDir"), &mut errors);
                    }
                    if let Some(start_dir) = &term.start_dir {
                        validate_placeholders(start_dir, &format!("{item_path}/startDir"), &mut errors);
                    }
                }
                Item::Folder(folder) => {
                    let target_field = format!("{item_path}/target");
                    if folder.target.trim().is_empty() {
                        errors.push(ValidationError::new(&target_field, "대상 경로(target)는 필수입니다."));
                    } else {
                        validate_placeholders(&folder.target, &target_field, &mut errors);
                    }

                    if let Some(icon) = &folder.icon {
                        if !ALLOWED_ICONS.contains(&icon.as_str()) {
                            errors.push(ValidationError::new(
                                format!("{item_path}/icon"),
                                format!("'{icon}'은(는) 허용되지 않는 아이콘입니다. 허용값: {ALLOWED_ICONS:?}"),
                            ));
                        }
                    }
                }
                Item::System(sys) => {
                    if !seen_system_fns.insert(sys.fn_name) {
                        errors.push(ValidationError::new(
                            format!("{item_path}/fn"),
                            format!("시스템 기능 '{:?}'이(가) 중복 등록되었습니다. 한 기능은 한 번만 등록할 수 있습니다.", sys.fn_name),
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
