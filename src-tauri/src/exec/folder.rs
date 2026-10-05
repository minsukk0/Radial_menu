use std::path::Path;
use radial_core::{FolderItem, OpenWith};

use crate::context::ContextValues;
use crate::launcher::{spawn, Elevation, LaunchSpec};

/// Execute a folder, file, or URL item. Always runs with non-elevated user privileges.
pub fn execute_folder(item: &FolderItem, ctx: &ContextValues) -> Result<(), String> {
    let target = ctx.substitute_raw_text(&item.target);
    let open_mode = item.open_target();

    let is_url = target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("ftp://");

    let spec = match open_mode {
        OpenWith::Explorer => {
            if item.select_target.unwrap_or(false) && Path::new(&target).exists() {
                LaunchSpec::new("explorer.exe").with_args(format!("/select,\"{target}\""))
            } else {
                LaunchSpec::new("explorer.exe").with_args(format!("\"{target}\""))
            }
        }
        OpenWith::Browser => {
            LaunchSpec::new("explorer.exe").with_args(format!("\"{target}\""))
        }
        OpenWith::Default | OpenWith::Current => {
            if is_url {
                LaunchSpec::new("explorer.exe").with_args(format!("\"{target}\""))
            } else {
                LaunchSpec::new("explorer.exe").with_args(format!("\"{target}\""))
            }
        }
    };

    spawn(&spec, Elevation::User).map_err(|e| format!("폴더/주소 열기 실패: {e}"))?;
    Ok(())
}
