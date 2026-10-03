use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub file_size: u64,
    pub paths: Vec<String>,
}

/// 3-Stage Duplicate Finder:
/// Stage 1: Group by exact size (filter out uniques & files < 1KB)
/// Stage 2: Quick Partial Hash (First 64KB + Last 64KB)
/// Stage 3: Full Blake3 Hash only for candidates that match Stage 2
pub fn find_duplicates(file_candidates: &[(String, u64)]) -> Vec<DuplicateGroup> {
    // Stage 1: Filter by size
    let mut size_groups: HashMap<u64, Vec<String>> = HashMap::new();
    for (path, size) in file_candidates {
        if *size >= 1024 {
            size_groups.entry(*size).or_default().push(path.clone());
        }
    }

    let mut result = Vec::new();

    for (size, paths) in size_groups {
        if paths.len() < 2 {
            continue;
        }

        // Stage 2: Partial Hash
        let mut partial_groups: HashMap<String, Vec<String>> = HashMap::new();
        for p in paths {
            if let Ok(ph) = compute_partial_hash(&p, size) {
                partial_groups.entry(ph).or_default().push(p);
            }
        }

        // Stage 3: Full Hash on collisions
        for (_, candidate_paths) in partial_groups {
            if candidate_paths.len() < 2 {
                continue;
            }

            let mut full_hash_groups: HashMap<String, Vec<String>> = HashMap::new();
            for p in candidate_paths {
                if let Ok(h) = compute_full_hash(&p) {
                    full_hash_groups.entry(h).or_default().push(p);
                }
            }

            for (h, matching_paths) in full_hash_groups {
                if matching_paths.len() > 1 {
                    result.push(DuplicateGroup {
                        hash: h,
                        file_size: size,
                        paths: matching_paths,
                    });
                }
            }
        }
    }

    result.sort_by(|a, b| (b.file_size * b.paths.len() as u64).cmp(&(a.file_size * a.paths.len() as u64)));
    result
}

fn compute_partial_hash(path: &str, file_size: u64) -> Result<String, std::io::Error> {
    let mut file = File::open(Path::new(path))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];

    // Read head
    let head_read = file.read(&mut buffer)?;
    hasher.update(&buffer[..head_read]);

    // Read tail if large enough
    if file_size > 128 * 1024 {
        let tail_pos = file_size - (64 * 1024);
        if file.seek(SeekFrom::Start(tail_pos)).is_ok() {
            let tail_read = file.read(&mut buffer)?;
            hasher.update(&buffer[..tail_read]);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn compute_full_hash(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(Path::new(path))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}
