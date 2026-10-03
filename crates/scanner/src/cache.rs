use crate::tree::VolumeIndex;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new() -> Result<Self, String> {
        let local_app = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
        let cache_dir = Path::new(&local_app).join("Lumen").join("cache");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
        }

        Ok(Self { cache_dir })
    }

    pub fn save_index(&self, index: &VolumeIndex) -> Result<(), String> {
        let file_path = self
            .cache_dir
            .join(format!("vol_{}.bin", index.volume_letter));
        let bytes = postcard::to_allocvec(index).map_err(|e| e.to_string())?;
        fs::write(file_path, bytes).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_index(&self, volume_letter: char) -> Option<VolumeIndex> {
        let file_path = self.cache_dir.join(format!("vol_{}.bin", volume_letter));
        if !file_path.exists() {
            return None;
        }

        let bytes = fs::read(file_path).ok()?;
        postcard::from_bytes(&bytes).ok()
    }
}
