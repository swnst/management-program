#[cfg(test)]
mod tests {
    use scanner::tree::VolumeIndex;
    use scanner::walk::{scan_volume_fast, ScanOptions};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    #[test]
    fn test_volume_index_aggregation() {
        let mut idx = VolumeIndex::new('T', "TestVol".into());

        // Add root children
        let dir1 = idx.add_node(0, "Dir1", 0x10, 0, 0, 1000, 1000);
        let file1 = idx.add_node(dir1, "file1.txt", 0x20, 500, 500, 1000, 1000);
        let file2 = idx.add_node(dir1, "file2.txt", 0x20, 1500, 1500, 1000, 1000);

        let dir2 = idx.add_node(0, "Dir2", 0x10, 0, 0, 1000, 1000);
        let file3 = idx.add_node(dir2, "file3.bin", 0x20, 3000, 3000, 1000, 1000);

        idx.aggregate_sizes();

        assert_eq!(idx.nodes[dir1 as usize].size_bytes, 2000);
        assert_eq!(idx.nodes[dir1 as usize].file_count, 2);

        assert_eq!(idx.nodes[dir2 as usize].size_bytes, 3000);
        assert_eq!(idx.nodes[dir2 as usize].file_count, 1);

        assert_eq!(idx.total_size_bytes, 5000);
        assert_eq!(idx.total_files_count, 3);
        assert_eq!(idx.total_dirs_count, 2);
    }

    #[test]
    fn test_cache_save_and_load() {
        let mut idx = VolumeIndex::new('X', "CacheVol".into());
        idx.add_node(0, "test_file.txt", 0x20, 1024, 1024, 500, 500);
        idx.aggregate_sizes();

        let cache = scanner::CacheManager::new().expect("Failed to create cache manager");
        cache.save_index(&idx).expect("Failed to save index");

        let loaded = cache.load_index('X');
        assert!(loaded.is_some(), "Expected cached index to be loadable");
        let l = loaded.unwrap();
        assert_eq!(l.volume_letter, 'X');
        assert_eq!(l.total_size_bytes, 1024);
        assert_eq!(l.total_files_count, 1);
    }
}
