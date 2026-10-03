use core_model::{CleanPlan, CleanResult, DiskSummary, MemoryInsight, StartupProgram};
use junk_rules::JunkEngine;
use platform_win::{list_fixed_volumes, recycle_path};
use scanner::walk::scan_directory_walk;
use std::path::Path;
use sysinsight::{get_memory_insight, list_startup_programs};

// --- Tauri Commands ---

#[tauri::command]
fn get_disk_summary() -> Vec<DiskSummary> {
    list_fixed_volumes()
}

#[tauri::command]
fn scan_folder(path: String) -> Result<serde_json::Value, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("Directory does not exist".into());
    }

    let index = scan_directory_walk(p, None)?;
    Ok(serde_json::json!({
        "total_files": index.total_files_count,
        "total_dirs": index.total_dirs_count,
        "total_bytes": index.total_size_bytes,
        "sample_files": index.files.into_iter().take(200).collect::<Vec<_>>()
    }))
}

#[tauri::command]
fn preview_junk_clean() -> CleanPlan {
    let engine = JunkEngine::new_with_builtin_rules();
    engine.preview_plan()
}

#[tauri::command]
fn execute_junk_clean(only_safe: bool) -> CleanResult {
    let engine = JunkEngine::new_with_builtin_rules();
    let plan = engine.preview_plan();
    engine.execute_plan(&plan, only_safe)
}

#[tauri::command]
fn get_system_memory() -> MemoryInsight {
    get_memory_insight()
}

#[tauri::command]
fn get_startup_apps() -> Vec<StartupProgram> {
    list_startup_programs()
}

#[tauri::command]
fn recycle_file_or_folder(path: String) -> Result<(), String> {
    recycle_path(path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_disk_summary,
            scan_folder,
            preview_junk_clean,
            execute_junk_clean,
            get_system_memory,
            get_startup_apps,
            recycle_file_or_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
