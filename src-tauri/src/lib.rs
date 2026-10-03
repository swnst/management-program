use core_model::{CleanPlan, CleanResult, DiskSummary, MemoryInsight, StartupProgram, VaultItem};
use junk_rules::JunkEngine;
use scanner::walk::scan_directory_walk;
use std::path::Path;
use sysinsight::{get_memory_insight, list_startup_programs};
use vault::SafetyVault;

// --- Tauri Commands ---

#[tauri::command]
fn get_disk_summary() -> Vec<DiskSummary> {
    vec![DiskSummary {
        volume_letter: 'C',
        volume_name: "Local Disk".into(),
        total_bytes: 512 * 1024 * 1024 * 1024,
        free_bytes: 220 * 1024 * 1024 * 1024,
        is_ntfs: true,
        is_system: true,
    }]
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
fn list_vault_quarantine() -> Result<Vec<VaultItem>, String> {
    let vault_path = std::env::temp_dir().join(".lumen_vault");
    let vault = SafetyVault::new(vault_path)?;
    vault.list_items()
}

#[tauri::command]
fn restore_vault_item(item_id: i64) -> Result<(), String> {
    let vault_path = std::env::temp_dir().join(".lumen_vault");
    let vault = SafetyVault::new(vault_path)?;
    vault.restore_file(item_id)
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
            list_vault_quarantine,
            restore_vault_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
