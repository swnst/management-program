use core_model::{CleanPlan, CleanResult, JunkRuleItem, Risk};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;

pub struct JunkEngine {
    rules: Vec<JunkRuleItem>,
}

impl JunkEngine {
    pub fn new_with_builtin_rules() -> Self {
        let temp_dir = std::env::temp_dir().to_string_lossy().to_string();
        let win_dir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

        let rules = vec![
            JunkRuleItem {
                id: "system.user_temp".into(),
                category: "System".into(),
                title: "User Temporary Files".into(),
                description: "Temporary application cache and runtime files created in user local temp".into(),
                risk: Risk::Safe,
                path_patterns: vec![temp_dir],
                matched_bytes: 0,
                matched_files_count: 0,
                requires_closed_processes: vec![],
            },
            JunkRuleItem {
                id: "system.win_temp".into(),
                category: "System".into(),
                title: "Windows Temporary Files".into(),
                description: "Windows OS temporary update and installation staging files".into(),
                risk: Risk::Safe,
                path_patterns: vec![format!("{}\\Temp", win_dir)],
                matched_bytes: 0,
                matched_files_count: 0,
                requires_closed_processes: vec![],
            },
            JunkRuleItem {
                id: "system.wer_reports".into(),
                category: "System".into(),
                title: "Windows Error Reporting Dumps".into(),
                description: "Crash memory dumps and WER report queue logs".into(),
                risk: Risk::Safe,
                path_patterns: vec![
                    format!("{}\\CrashDumps", local_app_data),
                    "C:\\ProgramData\\Microsoft\\Windows\\WER\\ReportQueue".into(),
                ],
                matched_bytes: 0,
                matched_files_count: 0,
                requires_closed_processes: vec![],
            },
            JunkRuleItem {
                id: "graphics.shader_cache".into(),
                category: "Graphics".into(),
                title: "DirectX & GPU Shader Cache".into(),
                description: "Outdated and temporary compiled GPU binary shader caches".into(),
                risk: Risk::Safe,
                path_patterns: vec![
                    format!("{}\\D3DSCache", local_app_data),
                    format!("{}\\NVIDIA\\DXCache", local_app_data),
                ],
                matched_bytes: 0,
                matched_files_count: 0,
                requires_closed_processes: vec![],
            },
            JunkRuleItem {
                id: "dev.package_caches".into(),
                category: "Developer".into(),
                title: "Package Manager Caches".into(),
                description: "Download tarballs and cache files from pip and npm".into(),
                risk: Risk::Caution,
                path_patterns: vec![
                    format!("{}\\pip\\cache", local_app_data),
                    format!("{}\\npm-cache", local_app_data),
                ],
                matched_bytes: 0,
                matched_files_count: 0,
                requires_closed_processes: vec![],
            },
        ];

        Self { rules }
    }

    pub fn preview_plan(&self) -> CleanPlan {
        let mut total_bytes = 0u64;
        let mut total_files = 0u64;
        let mut safe_bytes = 0u64;
        let mut caution_bytes = 0u64;
        let mut risky_bytes = 0u64;

        let mut evaluated_items = Vec::new();

        for rule in &self.rules {
            let mut item_bytes = 0u64;
            let mut item_files = 0u64;

            for pattern in &rule.path_patterns {
                let p = Path::new(pattern);
                if p.exists() && p.is_dir() {
                    let (b, f) = count_folder_size(p);
                    item_bytes += b;
                    item_files += f;
                }
            }

            total_bytes += item_bytes;
            total_files += item_files;

            match rule.risk {
                Risk::Safe => safe_bytes += item_bytes,
                Risk::Caution => caution_bytes += item_bytes,
                Risk::Risky => risky_bytes += item_bytes,
            }

            let mut evaluated = rule.clone();
            evaluated.matched_bytes = item_bytes;
            evaluated.matched_files_count = item_files;
            evaluated_items.push(evaluated);
        }

        CleanPlan {
            plan_id: format!(
                "plan_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            items: evaluated_items,
            total_bytes,
            total_files,
            safe_bytes,
            caution_bytes,
            risky_bytes,
        }
    }

    pub fn execute_plan(&self, plan: &CleanPlan, only_safe: bool) -> CleanResult {
        let mut freed_bytes = 0u64;
        let mut deleted_files = 0u64;
        let mut errors = Vec::new();

        for item in &plan.items {
            if only_safe && item.risk != Risk::Safe {
                continue;
            }

            for pattern in &item.path_patterns {
                let p = Path::new(pattern);
                if p.exists() && p.is_dir() {
                    let (f, cnt, errs) = delete_contents(p);
                    freed_bytes += f;
                    deleted_files += cnt;
                    errors.extend(errs);
                }
            }
        }

        CleanResult {
            plan_id: plan.plan_id.clone(),
            freed_immediate_bytes: freed_bytes,
            moved_to_vault_bytes: 0,
            deleted_files_count: deleted_files,
            errors,
        }
    }
}

fn count_folder_size(dir: &Path) -> (u64, u64) {
    let mut bytes = 0u64;
    let mut count = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(m) = entry.metadata() {
                if m.is_file() {
                    bytes += m.len();
                    count += 1;
                } else if m.is_dir() && (MetadataExt::file_attributes(&m) & 0x400) == 0 {
                    let (sub_b, sub_c) = count_folder_size(&p);
                    bytes += sub_b;
                    count += sub_c;
                }
            }
        }
    }
    (bytes, count)
}

fn delete_contents(dir: &Path) -> (u64, u64, Vec<String>) {
    let mut freed = 0u64;
    let mut count = 0u64;
    let mut errors = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(m) = entry.metadata() {
                if m.is_file() {
                    let len = m.len();
                    match fs::remove_file(&p) {
                        Ok(_) => {
                            freed += len;
                            count += 1;
                        }
                        Err(e) => errors.push(format!("Cannot remove file {:?}: {}", p, e)),
                    }
                } else if m.is_dir() && (MetadataExt::file_attributes(&m) & 0x400) == 0 {
                    let (sub_f, sub_c, sub_e) = delete_contents(&p);
                    freed += sub_f;
                    count += sub_c;
                    errors.extend(sub_e);
                    let _ = fs::remove_dir(&p);
                }
            }
        }
    }
    (freed, count, errors)
}
