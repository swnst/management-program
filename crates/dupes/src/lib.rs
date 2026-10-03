use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub file_size: u64,
    pub paths: Vec<String>,
}

pub fn find_duplicates(file_candidates: &[(String, u64)]) -> Vec<DuplicateGroup> {
    // Stage 1: Group by size
    let mut size_groups: HashMap<u64, Vec<String>> = HashMap::new();
    for (path, size) in file_candidates {
        if *size > 0 {
            size_groups.entry(*size).or_default().push(path.clone());
        }
    }

    let mut result = Vec::new();

    // Stage 2: Fast partial hash on files with same size
    for (size, paths) in size_groups {
        if paths.len() < 2 {
            continue;
        }

        let mut hash_groups: HashMap<String, Vec<String>> = HashMap::new();
        for p in paths {
            if let Ok(h) = compute_file_hash(&p) {
                hash_groups.entry(h).or_default().push(p);
            }
        }

        for (h, matching_paths) in hash_groups {
            if matching_paths.len() > 1 {
                result.push(DuplicateGroup {
                    hash: h,
                    file_size: size,
                    paths: matching_paths,
                });
            }
        }
    }

    result
}

fn compute_file_hash(path: &str) -> Result<String, std::io::Error> {
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
