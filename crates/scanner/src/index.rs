use core_model::AgeEstimate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub allocated_bytes: u64,
    pub is_directory: bool,
    pub is_hidden: bool,
    pub is_system: bool,
    pub is_reparse_point: bool,
    pub age: AgeEstimate,
}

#[derive(Default, Debug, Clone)]
pub struct FileIndex {
    pub files: Vec<ScannedFile>,
    pub total_size_bytes: u64,
    pub total_files_count: u64,
    pub total_dirs_count: u64,
}

impl FileIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, file: ScannedFile) {
        if !file.is_directory {
            self.total_size_bytes += file.size_bytes;
            self.total_files_count += 1;
        } else {
            self.total_dirs_count += 1;
        }
        self.files.push(file);
    }
}
