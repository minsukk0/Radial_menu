use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use radial_core::{compute_file_hash, compute_item_hash, Item};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRecord {
    pub item_id: String,
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_hash: Option<String>,
    pub approved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalsData {
    pub version: u32,
    pub approvals: Vec<ApprovalRecord>,
}

impl Default for ApprovalsData {
    fn default() -> Self {
        Self {
            version: 1,
            approvals: Vec::new(),
        }
    }
}

/// Returns the path to `%ProgramData%\RadialMenu\approvals.json`.
pub fn get_approvals_path() -> PathBuf {
    if let Ok(custom) = std::env::var("RADIAL_MENU_APPROVALS") {
        return PathBuf::from(custom);
    }

    if let Ok(program_data) = std::env::var("ProgramData") {
        PathBuf::from(program_data)
            .join("RadialMenu")
            .join("approvals.json")
    } else {
        PathBuf::from(r"C:\ProgramData\RadialMenu\approvals.json")
    }
}

/// Read approvals list from disk.
pub fn read_approvals() -> ApprovalsData {
    let path = get_approvals_path();
    if !path.exists() {
        return ApprovalsData::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => ApprovalsData::default(),
    }
}

/// Save approvals list to disk atomically.
pub fn write_approvals(data: &ApprovalsData) -> Result<(), String> {
    let path = get_approvals_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create approvals directory: {e}"))?;
    }

    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize approvals: {e}"))?;

    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, json.as_bytes())
        .map_err(|e| format!("Failed to write temporary approvals file: {e}"))?;

    fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to finalize approvals file rename: {e}"))?;

    Ok(())
}

/// Verify if an admin item has valid approval.
pub fn check_item_approval(item: &Item) -> Result<bool, String> {
    // If not admin, no approval needed
    if !item.run_as_admin() {
        return Ok(true);
    }

    // In non-elevated mode (dev), UAC prompt serves as approval
    if !crate::win32::token::is_elevated() {
        return Ok(true);
    }

    let item_hash = compute_item_hash(item);
    let file_hash = match item {
        Item::Terminal(t) if t.mode == Some(radial_core::TerminalMode::File) => {
            if let Some(ref script_file) = t.file {
                let p = Path::new(script_file);
                if p.exists() {
                    Some(compute_file_hash(p).map_err(|e| format!("Script hash error: {e}"))?)
                } else {
                    return Ok(false);
                }
            } else {
                None
            }
        }
        _ => None,
    };

    let approvals = read_approvals();
    let record = approvals.approvals.iter().find(|a| a.item_id == item.id());

    match record {
        Some(rec) => {
            if rec.hash != item_hash {
                return Ok(false);
            }
            if rec.file_hash != file_hash {
                return Ok(false);
            }
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Add or update an approval record for an item.
pub fn approve_item(item: &Item) -> Result<(), String> {
    let item_hash = compute_item_hash(item);
    let file_hash = match item {
        Item::Terminal(t) if t.mode == Some(radial_core::TerminalMode::File) => {
            if let Some(ref script_file) = t.file {
                let p = Path::new(script_file);
                if p.exists() {
                    Some(compute_file_hash(p).map_err(|e| format!("Script hash error: {e}"))?)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    };

    let mut data = read_approvals();
    let now = Utc::now().to_rfc3339();

    if let Some(existing) = data.approvals.iter_mut().find(|a| a.item_id == item.id()) {
        existing.hash = item_hash;
        existing.file_hash = file_hash;
        existing.approved_at = now;
    } else {
        data.approvals.push(ApprovalRecord {
            item_id: item.id().to_string(),
            hash: item_hash,
            file_hash,
            approved_at: now,
        });
    }

    write_approvals(&data)
}
