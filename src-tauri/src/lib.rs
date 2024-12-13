use serde::Serialize;
use std::ffi::OsStr;
use std::path::Path;
use walkdir::WalkDir;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use chrono::{DateTime, Local};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::time::{timeout, Duration};
use tokio::sync::watch;
use once_cell::sync::Lazy;

const SEARCH_TIMEOUT_SECS: u64 = 30;

#[derive(Clone)]
struct SearchState {
    tx: watch::Sender<bool>,
    rx: watch::Receiver<bool>,
}

impl Default for SearchState {
    fn default() -> Self {
        let (tx, rx) = watch::channel(false);
        Self { tx, rx }
    }
}

static SEARCH_STATE: Lazy<Arc<Mutex<SearchState>>> = 
    Lazy::new(|| Arc::new(Mutex::new(SearchState::default())));

#[tauri::command]
async fn cancel_search() -> Result<(), String> {
    if let Ok(mut state) = SEARCH_STATE.lock() {
        let _ = state.tx.send(true);
        Ok(())
    } else {
        Err("Failed to cancel search".to_string())
    }
}

#[tauri::command]
async fn search(
    search_term: String,
    path: String,
    level: String,
    target: String,
    file_extensions: String,
) -> Result<Vec<SearchResult>, String> {
    // println!("Searching for: {}", search_term);
    // println!("In path: {}", path);
    
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    // Reset cancellation state
    if let Ok(mut state) = SEARCH_STATE.lock() {
        let (tx, rx) = watch::channel(false);
        *state = SearchState { tx, rx };
    }

    let search_result = timeout(
        Duration::from_secs(SEARCH_TIMEOUT_SECS),
        perform_parallel_search(
            path,
            &search_term,
            &level,
            &target,
            &file_extensions,
            SEARCH_STATE.clone(),
        ),
    )
    .await;

    match search_result {
        Ok(result) => result,
        Err(_) => Err("Search operation timed out".to_string()),
    }
}

async fn perform_parallel_search(
    path: &Path,
    search_term: &str,
    level: &str,
    target: &str,
    file_extensions: &str,
    search_state: Arc<Mutex<SearchState>>,
) -> Result<Vec<SearchResult>, String> {
    // println!("Search parameters:");
    // println!("  - Path: {:?}", path);
    // println!("  - Search term: {}", search_term);
    // println!("  - Level: {}", level);
    // println!("  - Target: {}", target);
    // println!("  - File extensions: {}", file_extensions);

    let max_depth = if level == "top" { 1 } else { std::usize::MAX };
    let walker = WalkDir::new(path).max_depth(max_depth);
    
    let results = Arc::new(Mutex::new(Vec::new()));
    let search_term = search_term.to_lowercase();
    

    walker.into_iter()
        .par_bridge() // 转换为并行迭代器
        .try_for_each(|e| {
            if let Ok(entry) = e {
                // 检查取消状态
                if let Ok(state) = search_state.lock() {
                    if *state.rx.borrow() {
                        return Err("Search cancelled".to_string());
                    }
                }

                let is_dir = entry.file_type().is_dir();
                // 目标类型过滤
                match target {
                    "folders" if !is_dir => return Ok(()),
                    "files" if is_dir => return Ok(()),
                    _ => {}
                }
                let path_str = entry.path().to_string_lossy().to_string();
                let name = entry.file_name().to_string_lossy().to_string();

                // 搜索匹配逻辑
                let name_lower = name.to_lowercase();
                
                // let path_components: Vec<&str> = path_str_lower.split(['\\', '/']).collect();
                // let path_matches = path_components.iter().any(|component| 
                //     component.contains(&search_term_clone));
                let name_matches = name_lower.contains(&search_term);
                
                if !name_matches {
                    return Ok(());
                }


                // 文件扩展名过滤
                let file_extensions: Vec<&str> = if file_extensions.trim().is_empty() {
                    Vec::new()
                } else {
                    file_extensions.split(',').collect()
                };
                if !is_dir && !file_extensions.is_empty() {
                   
                    let ext = entry.path()
                        .extension()
                        .and_then(OsStr::to_str)
                        .unwrap_or("");
                    if !file_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                        return Ok(());
                    }
                }

                let size = if is_dir {
                    0
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };

                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let time: DateTime<Local> = modified.into();
                        if let Ok(mut results) = results.lock() {
                            results.push(SearchResult {
                                name,
                                path: path_str,
                                is_dir,
                                size,
                                modified_at: time.format("%Y-%m-%d %H:%M:%S").to_string(),
                            });
                        }
                    }
                }
            }
            Ok(())
        })?;

    // Get final results and sort
    let mut final_results = results.lock()
        .map_err(|e| e.to_string())?
        .clone();
    
    final_results.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(final_results)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        // println!("settings menu item was clicked");
                        // if settings window is already created, show and focus it
                        // if settings window is not created yet, create it and show and focus it
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            let window = WebviewWindowBuilder::new(
                                app,
                                "settings",
                                WebviewUrl::App("settings".into()),
                            )
                            .title("Settings")
                            .center()
                            .inner_size(800.0, 600.0)
                            .decorations(false)
                            .always_on_top(false)
                            .resizable(false)
                            .build()
                            .unwrap();
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "quit" => {
                        // println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        // println!("menu item {:?} not handled", event.id);
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        // println!("left click pressed and released");
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            println!("showing main window");
                            let _ = window.show();
                            let _ = window.set_focus();
                        }else{
                            println!("creating main window");
                            let window = WebviewWindowBuilder::new(
                                app,
                                "main",
                                WebviewUrl::App("index.html".into()),
                            )
                            .title("Finder")
                            .center()
                            .inner_size(400.0, 400.0)
                            .decorations(false)
                            .always_on_top(false)
                            .resizable(false)
                            .build()
                            .unwrap();
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {
                        // println!("unhandled event {event:?}");
                    }
                })
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![search, cancel_search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Serialize,Clone)]
pub struct SearchResult {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified_at: String
}
