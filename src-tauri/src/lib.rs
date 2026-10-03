use core_model::{CleanPlan, CleanResult, DiskSummary, MemoryInsight, StartupProgram};
use junk_rules::JunkEngine;
use platform_win::{list_fixed_volumes, recycle_path};
use scanner::{scan_volume_fast, CacheManager, ScanOptions};
use sysinsight::{get_memory_insight, list_startup_programs};

// --- Tauri Commands ---

#[tauri::command]
fn get_disk_summary() -> Vec<DiskSummary> {
    list_fixed_volumes()
}

#[tauri::command]
fn scan_volume(drive_letter: char) -> Result<serde_json::Value, String> {
    let cache = CacheManager::new().ok();
    if let Some(ref c) = cache {
        if let Some(cached_index) = c.load_index(drive_letter) {
            return Ok(serde_json::json!({
                "volume_letter": cached_index.volume_letter,
                "total_files": cached_index.total_files_count,
                "total_dirs": cached_index.total_dirs_count,
                "total_bytes": cached_index.total_size_bytes,
                "is_cached": true,
            }));
        }
    }

    let options = ScanOptions::default();
    let index = scan_volume_fast(drive_letter, format!("Drive {}", drive_letter), &options, None)?;

    if let Some(ref c) = cache {
        let _ = c.save_index(&index);
    }

    Ok(serde_json::json!({
        "volume_letter": index.volume_letter,
        "total_files": index.total_files_count,
        "total_dirs": index.total_dirs_count,
        "total_bytes": index.total_size_bytes,
        "is_cached": false,
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

#[tauri::command]
fn purge_standby_memory() -> Result<String, String> {
    // Attempt elevated Broker via named pipe first
    if let Ok(client) = broker::BrokerClient::connect() {
        match client.send_request(&broker_proto::BrokerRequest::PurgeStandbyList) {
            Ok(broker_proto::BrokerResponse::Success) => {
                return Ok("Standby list purged successfully via Privileged Broker.".into());
            }
            Ok(broker_proto::BrokerResponse::Error(err)) => {
                return Err(format!("Broker error: {}", err));
            }
            _ => {}
        }
    }

    // Fallback: in-process if already elevated
    platform_win::purge_standby_list()
        .map(|_| "Standby list purged successfully in current process.".into())
        .map_err(|e| format!("Purge failed (run Broker as Admin or elevate): {}", e))
}

#[tauri::command]
fn check_broker_status() -> bool {
    if let Ok(client) = broker::BrokerClient::connect() {
        matches!(
            client.send_request(&broker_proto::BrokerRequest::Ping),
            Ok(broker_proto::BrokerResponse::Pong)
        )
    } else {
        false
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_disk_summary,
            scan_volume,
            preview_junk_clean,
            execute_junk_clean,
            get_system_memory,
            get_startup_apps,
            recycle_file_or_folder,
            purge_standby_memory,
            check_broker_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
