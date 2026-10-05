use std::path::Path;
use windows::core::{Interface, BSTR, VARIANT};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    Folder2, IShellDispatch2, IShellFolderViewDual, IShellWindows, IWebBrowserApp, Shell,
    ShellWindows,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

/// Explorer context information: active folder path and list of selected file paths.
#[derive(Debug, Clone, Default)]
pub struct ExplorerContext {
    pub current_folder: Option<String>,
    pub selected_files: Vec<String>,
}

/// Retrieve the active File Explorer folder and selected files using `IShellWindows` COM interface.
pub fn get_explorer_context() -> ExplorerContext {
    let mut context = ExplorerContext::default();

    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let com_initialized = hr.is_ok();

        if let Ok(shell_windows) =
            CoCreateInstance::<_, IShellWindows>(&ShellWindows, None, CLSCTX_LOCAL_SERVER)
        {
            let fg_hwnd = GetForegroundWindow();

            let count = shell_windows.Count().unwrap_or(0);
            let mut matched_context: Option<ExplorerContext> = None;
            let mut fallback_context: Option<ExplorerContext> = None;

            for i in 0..count {
                let var_index = VARIANT::from(i);
                let dispatch = match shell_windows.Item(&var_index) {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let browser = match dispatch.cast::<IWebBrowserApp>() {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                let window_hwnd = match browser.HWND() {
                    Ok(h) => HWND(h.0 as *mut std::ffi::c_void),
                    Err(_) => HWND(std::ptr::null_mut()),
                };

                let doc = match browser.Document() {
                    Ok(d) => d,
                    Err(_) => continue,
                };

                let folder_view = match doc.cast::<IShellFolderViewDual>() {
                    Ok(fv) => fv,
                    Err(_) => continue,
                };

                let mut current_folder = None;
                if let Ok(folder) = folder_view.Folder() {
                    if let Ok(folder2) = folder.cast::<Folder2>() {
                        if let Ok(item) = folder2.Self_() {
                            if let Ok(path) = item.Path() {
                                let path_str = path.to_string();
                                if !path_str.is_empty() {
                                    current_folder = Some(path_str);
                                }
                            }
                        }
                    }
                }

                let mut selected_files = Vec::new();
                if let Ok(items) = folder_view.SelectedItems() {
                    let sel_count = items.Count().unwrap_or(0);
                    for item_idx in 0..sel_count {
                        let item_var = VARIANT::from(item_idx);
                        if let Ok(item) = items.Item(&item_var) {
                            if let Ok(path) = item.Path() {
                                let path_str = path.to_string();
                                if !path_str.is_empty() {
                                    selected_files.push(path_str);
                                }
                            }
                        }
                    }
                }

                let ctx = ExplorerContext {
                    current_folder,
                    selected_files,
                };

                if !fg_hwnd.0.is_null() && window_hwnd == fg_hwnd {
                    matched_context = Some(ctx);
                    break;
                } else if fallback_context.is_none() {
                    fallback_context = Some(ctx);
                }
            }

            if let Some(ctx) = matched_context.or(fallback_context) {
                context = ctx;
            }
        }

        if com_initialized {
            CoUninitialize();
        }
    }

    context
}

/// Fallback method to run process under normal user rights via Explorer's `IShellDispatch2::ShellExecute`.
pub fn shell_execute_user(
    application: &str,
    args: Option<&str>,
    work_dir: Option<&Path>,
    hidden: bool,
) -> Result<(), String> {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let com_initialized = hr.is_ok();

        let shell = CoCreateInstance::<_, IShellDispatch2>(&Shell, None, CLSCTX_LOCAL_SERVER)
            .map_err(|e| format!("Failed to create IShellDispatch2 instance: {e}"))?;

        let file_bstr = BSTR::from(application);
        let args_var = args
            .map(|a| VARIANT::from(BSTR::from(a)))
            .unwrap_or_else(VARIANT::default);

        let dir_var = work_dir
            .map(|d| VARIANT::from(BSTR::from(d.to_string_lossy().as_ref())))
            .unwrap_or_else(VARIANT::default);

        let op_var = VARIANT::from(BSTR::from("open"));
        let show_var = VARIANT::from(if hidden { 0i32 } else { 1i32 });

        let result = shell
            .ShellExecute(
                &file_bstr,
                &args_var,
                &dir_var,
                &op_var,
                &show_var,
            )
            .map_err(|e| format!("IShellDispatch2::ShellExecute failed: {e}"));

        if com_initialized {
            CoUninitialize();
        }
        result
    }
}
