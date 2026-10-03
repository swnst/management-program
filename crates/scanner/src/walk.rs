use crate::index::{FileIndex, ScannedFile};
use core_model::{AgeConfidence, AgeEstimate, UsageSource};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn scan_directory_walk(
    dir_path: &Path,
    on_progress: Option<&dyn Fn(u64, &str)>,
) -> Result<FileIndex, String> {
    let mut index = FileIndex::new();
    let mut scanned_count = 0u64;

    scan_recursive(dir_path, &mut index, &mut scanned_count, on_progress)?;
    Ok(index)
}

fn scan_recursive(
    dir: &Path,
    index: &mut FileIndex,
    scanned_count: &mut u64,
    on_progress: Option<&dyn Fn(u64, &str)>,
) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // Skip directories we do not have permission to read
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_attrs = metadata.file_attributes();
        let is_dir = metadata.is_dir();
        let is_hidden = (file_attrs & 0x2) != 0;
        let is_system = (file_attrs & 0x4) != 0;
        let is_reparse_point = (file_attrs & 0x400) != 0;

        let size = if is_dir { 0 } else { metadata.len() };

        let created_secs = metadata
            .created()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let modified_secs = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let accessed_secs = metadata
            .accessed()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Calculate age estimate and confidence
        let (source, confidence, last_used) = if accessed_secs > 0 && accessed_secs != modified_secs {
            (UsageSource::LastAccess, AgeConfidence::Medium, Some(accessed_secs))
        } else {
            (UsageSource::ModifiedOnly, AgeConfidence::Low, Some(modified_secs))
        };

        let age = AgeEstimate {
            on_disk_since_secs: created_secs,
            content_dated_secs: modified_secs,
            last_used_secs: last_used,
            source,
            confidence,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();
        let full_path_str = path.to_string_lossy().to_string();

        index.add(ScannedFile {
            path: full_path_str.clone(),
            name: file_name,
            size_bytes: size,
            allocated_bytes: size,
            is_directory: is_dir,
            is_hidden,
            is_system,
            is_reparse_point,
            age,
        });

        *scanned_count += 1;
        if *scanned_count % 1000 == 0 {
            if let Some(cb) = on_progress {
                cb(*scanned_count, &full_path_str);
            }
        }

        // Do not recurse into reparse points (junctions/symlinks) to prevent infinite loops
        if is_dir && !is_reparse_point {
            let _ = scan_recursive(&path, index, scanned_count, on_progress);
        }
    }

    Ok(())
}
