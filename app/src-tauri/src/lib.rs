use cleaner_core::{CleanReport, FullScanResult};
use std::path::PathBuf;

#[tauri::command]
async fn run_scan() -> Result<FullScanResult, String> {
    // Scanning can take a while, so keep it off the UI thread.
    tauri::async_runtime::spawn_blocking(cleaner_core::run_full_scan)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean_items(paths: Vec<PathBuf>) -> Result<CleanReport, String> {
    // Validation happens in the core; only cache/trash paths are accepted.
    tauri::async_runtime::spawn_blocking(move || cleaner_core::clean_paths(&paths))
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run_scan, clean_items])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
