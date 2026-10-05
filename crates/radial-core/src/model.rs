use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Root configuration for Radial Menu (`menu.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfig {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(default = "default_version")]
    pub version: u32,

    #[serde(default = "default_accent")]
    pub accent: String,

    #[serde(default)]
    pub categories: Vec<Category>,
}

fn default_version() -> u32 {
    1
}

fn default_accent() -> String {
    "#1E9BFF".to_string()
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            schema: Some("https://radial-menu.local/schema/menu.schema.json".to_string()),
            version: 1,
            accent: default_accent(),
            categories: Vec::new(),
        }
    }
}

impl MenuConfig {
    /// Returns the standard default template with 4 starter categories.
    pub fn default_template() -> Self {
        Self {
            schema: Some("https://radial-menu.local/schema/menu.schema.json".to_string()),
            version: 1,
            accent: default_accent(),
            categories: vec![
                Category {
                    id: "apps".to_string(),
                    kind: CategoryKind::App,
                    label: "앱".to_string(),
                    icon: Some("apps".to_string()),
                    items: vec![
                        Item::App(AppItem {
                            id: "chatgpt".to_string(),
                            label: "ChatGPT".to_string(),
                            icon: Some("gpt".to_string()),
                            path: "https://chatgpt.com".to_string(),
                            args: None,
                            when_running: Some(WhenRunning::New),
                            run_as_admin: Some(false),
                        }),
                        Item::App(AppItem {
                            id: "claude".to_string(),
                            label: "Claude".to_string(),
                            icon: Some("claude".to_string()),
                            path: "https://claude.ai".to_string(),
                            args: None,
                            when_running: Some(WhenRunning::New),
                            run_as_admin: Some(false),
                        }),
                        Item::App(AppItem {
                            id: "antigravity".to_string(),
                            label: "Antigravity".to_string(),
                            icon: Some("antigravity".to_string()),
                            path: r"C:\Users\minsu\AppData\Local\Programs\antigravity\Antigravity.exe".to_string(),
                            args: None,
                            when_running: Some(WhenRunning::Focus),
                            run_as_admin: Some(false),
                        }),
                        Item::App(AppItem {
                            id: "notepad".to_string(),
                            label: "메모장".to_string(),
                            icon: None,
                            path: r"C:\Windows\System32\notepad.exe".to_string(),
                            args: None,
                            when_running: Some(WhenRunning::Focus),
                            run_as_admin: Some(false),
                        }),
                    ],
                },
                Category {
                    id: "terminal".to_string(),
                    kind: CategoryKind::Terminal,
                    label: "터미널".to_string(),
                    icon: Some("terminal".to_string()),
                    items: vec![
                        Item::Terminal(TerminalItem {
                            id: "ps-here".to_string(),
                            label: "새 창".to_string(),
                            icon: Some("window".to_string()),
                            action: TerminalAction::Open,
                            mode: Some(TerminalMode::Command),
                            command: None,
                            file: None,
                            args: None,
                            shell: Some(ShellType::Powershell),
                            work_dir: None,
                            start_dir: Some("{현재 폴더}".to_string()),
                            window: None,
                            run_as_admin: Some(false),
                            notify_on_done: Some(false),
                            copy_output: Some(false),
                            confirm_first: Some(false),
                            reuse_tab: Some(true),
                        }),
                    ],
                },
                Category {
                    id: "folders".to_string(),
                    kind: CategoryKind::Folder,
                    label: "폴더·주소".to_string(),
                    icon: Some("folder".to_string()),
                    items: vec![
                        Item::Folder(FolderItem {
                            id: "downloads".to_string(),
                            label: "다운로드".to_string(),
                            icon: None,
                            target: r"%USERPROFILE%\Downloads".to_string(),
                            open_in: Some(OpenWith::Explorer),
                            select_target: Some(false),
                        }),
                    ],
                },
                Category {
                    id: "system".to_string(),
                    kind: CategoryKind::System,
                    label: "시스템".to_string(),
                    icon: Some("power".to_string()),
                    items: vec![
                        Item::System(SystemItem {
                            id: "lock".to_string(),
                            label: "화면 잠금".to_string(),
                            icon: None,
                            fn_name: SystemFn::Lock,
                        }),
                    ],
                },
            ],
        }
    }
}

/// Category kind indicating what type of items this category holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CategoryKind {
    App,
    Terminal,
    Folder,
    System,
    #[serde(rename = "empty")]
    Empty,
}

impl CategoryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryKind::App => "app",
            CategoryKind::Terminal => "terminal",
            CategoryKind::Folder => "folder",
            CategoryKind::System => "system",
            CategoryKind::Empty => "empty",
        }
    }

    pub fn default_label(&self) -> &'static str {
        match self {
            CategoryKind::App => "앱",
            CategoryKind::Terminal => "터미널",
            CategoryKind::Folder => "폴더·주소",
            CategoryKind::System => "시스템",
            CategoryKind::Empty => "빈칸",
        }
    }
}

impl std::str::FromStr for CategoryKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "app" => Ok(CategoryKind::App),
            "terminal" => Ok(CategoryKind::Terminal),
            "folder" => Ok(CategoryKind::Folder),
            "system" => Ok(CategoryKind::System),
            "empty" => Ok(CategoryKind::Empty),
            other => Err(format!(
                "지원되지 않는 분류 종류: '{other}'. 허용값: app, terminal, folder, system, empty"
            )),
        }
    }
}

/// A category grouping items in the outer/inner radial ring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[serde(default)]
    pub id: String,

    pub kind: CategoryKind,

    pub label: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[serde(default)]
    pub items: Vec<Item>,
}

impl Category {
    pub fn new(id: impl Into<String>, kind: CategoryKind, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            icon: None,
            items: Vec::new(),
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Item definition, untagged union representing App, Terminal, Folder, or System.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Item {
    App(AppItem),
    Terminal(TerminalItem),
    Folder(FolderItem),
    System(SystemItem),
}

impl Item {
    pub fn id(&self) -> &str {
        match self {
            Item::App(i) => &i.id,
            Item::Terminal(i) => &i.id,
            Item::Folder(i) => &i.id,
            Item::System(i) => &i.id,
        }
    }

    pub fn set_id(&mut self, id: String) {
        match self {
            Item::App(i) => i.id = id,
            Item::Terminal(i) => i.id = id,
            Item::Folder(i) => i.id = id,
            Item::System(i) => i.id = id,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Item::App(i) => &i.label,
            Item::Terminal(i) => &i.label,
            Item::Folder(i) => &i.label,
            Item::System(i) => &i.label,
        }
    }

    pub fn set_label(&mut self, label: String) {
        match self {
            Item::App(i) => i.label = label,
            Item::Terminal(i) => i.label = label,
            Item::Folder(i) => i.label = label,
            Item::System(i) => i.label = label,
        }
    }

    pub fn kind(&self) -> CategoryKind {
        match self {
            Item::App(_) => CategoryKind::App,
            Item::Terminal(_) => CategoryKind::Terminal,
            Item::Folder(_) => CategoryKind::Folder,
            Item::System(_) => CategoryKind::System,
        }
    }

    pub fn run_as_admin(&self) -> bool {
        match self {
            Item::App(i) => i.run_as_admin.unwrap_or(false),
            Item::Terminal(i) => i.run_as_admin.unwrap_or(false),
            Item::Folder(_) => false,
            Item::System(_) => false,
        }
    }

    pub fn icon(&self) -> Option<&str> {
        match self {
            Item::App(i) => i.icon.as_deref(),
            Item::Terminal(i) => i.icon.as_deref(),
            Item::Folder(i) => i.icon.as_deref(),
            Item::System(i) => i.icon.as_deref(),
        }
    }

    /// Update this item by shallow-merging fields from a JSON patch object.
    pub fn update_from_json(&mut self, patch: &serde_json::Value) -> Result<(), String> {
        let patch_obj = patch
            .as_object()
            .ok_or_else(|| "업데이트할 JSON은 객체 형식이어야 합니다.".to_string())?;

        let mut current_val = serde_json::to_value(&self)
            .map_err(|e| format!("현재 항목 직렬화 실패: {e}"))?;

        if let Some(target_obj) = current_val.as_object_mut() {
            for (k, v) in patch_obj {
                target_obj.insert(k.clone(), v.clone());
            }
        }

        let updated_item = match self.kind() {
            CategoryKind::App => {
                let app: AppItem = serde_json::from_value(current_val)
                    .map_err(|e| format!("앱 항목 업데이트 실패: {e}"))?;
                Item::App(app)
            }
            CategoryKind::Terminal => {
                let term: TerminalItem = serde_json::from_value(current_val)
                    .map_err(|e| format!("터미널 항목 업데이트 실패: {e}"))?;
                Item::Terminal(term)
            }
            CategoryKind::Folder => {
                let folder: FolderItem = serde_json::from_value(current_val)
                    .map_err(|e| format!("폴더 항목 업데이트 실패: {e}"))?;
                Item::Folder(folder)
            }
            CategoryKind::System => {
                let sys: SystemItem = serde_json::from_value(current_val)
                    .map_err(|e| format!("시스템 항목 업데이트 실패: {e}"))?;
                Item::System(sys)
            }
            CategoryKind::Empty => unreachable!("Item cannot be of kind Empty"),
        };

        *self = updated_item;
        Ok(())
    }
}

/// Behavior when the application is already running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum WhenRunning {
    Focus,
    New,
}

impl Default for WhenRunning {
    fn default() -> Self {
        WhenRunning::Focus
    }
}

/// Application item specification (`kind: "app"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppItem {
    #[serde(default)]
    pub id: String,

    pub label: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    pub path: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when_running: Option<WhenRunning>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as_admin: Option<bool>,
}

impl AppItem {
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// Terminal action: open interactive terminal or run script/command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TerminalAction {
    Open,
    Run,
}

/// Terminal run mode: inline command string or script file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TerminalMode {
    Command,
    File,
}

impl Default for TerminalMode {
    fn default() -> Self {
        TerminalMode::Command
    }
}

/// Supported shell environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Powershell,
    Cmd,
    Wsl,
    Gitbash,
}

impl Default for ShellType {
    fn default() -> Self {
        ShellType::Powershell
    }
}

/// Window visibility for terminal command execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TerminalWindow {
    Hidden,
    Visible,
    Keep,
}

impl Default for TerminalWindow {
    fn default() -> Self {
        TerminalWindow::Hidden
    }
}

/// Terminal item specification (`kind: "terminal"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TerminalItem {
    #[serde(default)]
    pub id: String,

    pub label: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    pub action: TerminalAction,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<TerminalMode>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shell: Option<ShellType>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_dir: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<TerminalWindow>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as_admin: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify_on_done: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_output: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm_first: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reuse_tab: Option<bool>,
}

impl TerminalItem {
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn working_directory(&self) -> Option<&str> {
        self.work_dir
            .as_deref()
            .or(self.start_dir.as_deref())
    }
}

/// Target application to open folder or URL with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OpenWith {
    Default,
    Explorer,
    Browser,
    Current,
}

impl Default for OpenWith {
    fn default() -> Self {
        OpenWith::Default
    }
}

/// Folder or URL item specification (`kind: "folder"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderItem {
    #[serde(default)]
    pub id: String,

    pub label: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    pub target: String,

    #[serde(default, skip_serializing_if = "Option::is_none", alias = "openWith")]
    pub open_in: Option<OpenWith>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select_target: Option<bool>,
}

impl FolderItem {
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn open_target(&self) -> OpenWith {
        self.open_in.unwrap_or(OpenWith::Default)
    }
}

/// Supported system action functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum SystemFn {
    #[serde(rename = "lock")]
    Lock,
    #[serde(rename = "sleep")]
    Sleep,
    #[serde(rename = "screensaver")]
    Screensaver,
    #[serde(
        rename = "displayOff",
        alias = "screenoff",
        alias = "screenOff",
        alias = "display_off"
    )]
    DisplayOff,
    #[serde(rename = "volumeMute", alias = "mute", alias = "volume_mute")]
    VolumeMute,
    #[serde(
        rename = "radialSettings",
        alias = "settings",
        alias = "radial_settings"
    )]
    RadialSettings,
    #[serde(rename = "taskmgr", alias = "taskMgr", alias = "task_mgr")]
    Taskmgr,
    #[serde(rename = "brightness")]
    Brightness,
    #[serde(rename = "emptybin", alias = "emptyBin", alias = "empty_bin")]
    EmptyBin,
}

/// System action item specification (`kind: "system"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemItem {
    #[serde(default)]
    pub id: String,

    pub label: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[serde(rename = "fn")]
    pub fn_name: SystemFn,
}

impl SystemItem {
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// Deserialize item JSON specifically for a known category kind.
pub fn deserialize_item_for_kind(
    kind: CategoryKind,
    value: &serde_json::Value,
) -> Result<Item, serde_json::Error> {
    match kind {
        CategoryKind::App => {
            let item: AppItem = serde_json::from_value(value.clone())?;
            Ok(Item::App(item))
        }
        CategoryKind::Terminal => {
            let item: TerminalItem = serde_json::from_value(value.clone())?;
            Ok(Item::Terminal(item))
        }
        CategoryKind::Folder => {
            let item: FolderItem = serde_json::from_value(value.clone())?;
            Ok(Item::Folder(item))
        }
        CategoryKind::System => {
            let item: SystemItem = serde_json::from_value(value.clone())?;
            Ok(Item::System(item))
        }
        CategoryKind::Empty => {
            Err(serde::de::Error::custom("빈칸 분류에는 항목을 추가할 수 없습니다."))
        }
    }
}

/// Generate a unique identifier from a human-readable label or fallback prefix.
pub fn generate_unique_id(label: &str, existing: &HashSet<String>) -> String {
    let mut base = String::new();
    for c in label.chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '_' || c == '-') && !base.ends_with('-') {
            base.push('-');
        }
    }
    let base = base.trim_matches('-');
    let prefix = if base.is_empty() { "item" } else { base };

    if !existing.contains(prefix) {
        return prefix.to_string();
    }

    let mut counter = 2;
    loop {
        let candidate = format!("{prefix}-{counter}");
        if !existing.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// Auto-generate missing IDs in categories and items.
pub fn ensure_ids(config: &mut MenuConfig) {
    let mut existing: HashSet<String> = HashSet::new();

    for cat in &config.categories {
        if !cat.id.is_empty() {
            existing.insert(cat.id.clone());
        }
        for item in &cat.items {
            if !item.id().is_empty() {
                existing.insert(item.id().to_string());
            }
        }
    }

    for cat in &mut config.categories {
        if cat.id.is_empty() {
            let id = generate_unique_id(&cat.label, &existing);
            existing.insert(id.clone());
            cat.id = id;
        }
        for item in &mut cat.items {
            if item.id().is_empty() {
                let id = generate_unique_id(item.label(), &existing);
                existing.insert(id.clone());
                item.set_id(id);
            }
        }
    }
}
