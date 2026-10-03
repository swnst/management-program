use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum Risk {
    Safe,
    Caution,
    Risky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum AgeConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum UsageSource {
    LastAccess,
    Prefetch,
    ModifiedOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AgeEstimate {
    pub on_disk_since_secs: u64,
    pub content_dated_secs: u64,
    pub last_used_secs: Option<u64>,
    pub source: UsageSource,
    pub confidence: AgeConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DiskSummary {
    pub volume_letter: char,
    pub volume_name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub is_ntfs: bool,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ScanProgress {
    pub files_scanned: u64,
    pub bytes_scanned: u64,
    pub current_directory: String,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct JunkRuleItem {
    pub id: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub risk: Risk,
    pub path_patterns: Vec<String>,
    pub matched_bytes: u64,
    pub matched_files_count: u64,
    pub requires_closed_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CleanPlan {
    pub plan_id: String,
    pub items: Vec<JunkRuleItem>,
    pub total_bytes: u64,
    pub total_files: u64,
    pub safe_bytes: u64,
    pub caution_bytes: u64,
    pub risky_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CleanResult {
    pub plan_id: String,
    pub freed_immediate_bytes: u64,
    pub moved_to_vault_bytes: u64,
    pub deleted_files_count: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VaultItem {
    pub id: i64,
    pub original_path: String,
    pub vault_path: String,
    pub size_bytes: u64,
    pub deleted_timestamp_secs: u64,
    pub expiry_timestamp_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MemoryInsight {
    pub total_ram_bytes: u64,
    pub used_ram_bytes: u64,
    pub free_ram_bytes: u64,
    pub standby_ram_bytes: u64,
    pub modified_ram_bytes: u64,
    pub top_processes: Vec<ProcessMemoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ProcessMemoryItem {
    pub pid: u32,
    pub name: String,
    pub private_working_set_bytes: u64,
    pub working_set_bytes: u64,
    pub cpu_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StartupProgram {
    pub id: String,
    pub name: String,
    pub command: String,
    pub location: String,
    pub is_enabled: bool,
    pub publisher: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LiveTelemetry {
    pub cpu_percent: f32,
    pub ram_percent: f32,
    pub disk_read_bytes_sec: u64,
    pub disk_write_bytes_sec: u64,
    pub network_in_bytes_sec: u64,
    pub network_out_bytes_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FileRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_year: i32,
    pub age_label: String,
    pub is_dir: bool,
    pub label: String,
    pub reason: String,
}
