use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverCandidate {
    pub program_hint: String,
    pub path: String,
    pub size_bytes: u64,
    pub confidence: String,
}

pub fn scan_uninstalled_leftovers() -> Vec<LeftoverCandidate> {
    // Scaffold leftover detector looking for orphaned folders in AppData/Local
    let mut results = Vec::new();
    if let Ok(local_app) = std::env::var("LOCALAPPDATA") {
        let path = std::path::Path::new(&local_app);
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("Temp") || name.starts_with("Crash") {
                    if let Ok(m) = entry.metadata() {
                        results.push(LeftoverCandidate {
                            program_hint: name,
                            path: p.to_string_lossy().to_string(),
                            size_bytes: m.len(),
                            confidence: "Medium".into(),
                        });
                    }
                }
            }
        }
    }
    results
}
