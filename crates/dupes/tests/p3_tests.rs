#[cfg(test)]
mod tests {
    use dupes::find_duplicates;
    use leftovers::scan_uninstalled_leftovers;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_duplicate_finder_finds_exact_matches() {
        let temp = std::env::temp_dir();
        let file_a = temp.join(format!("lumen_dupe_a_{}.bin", std::process::id()));
        let file_b = temp.join(format!("lumen_dupe_b_{}.bin", std::process::id()));
        let file_c = temp.join(format!("lumen_dupe_c_{}.bin", std::process::id()));

        // Write identical content (size > 1KB)
        let payload = vec![0xAB; 2048];
        let diff_payload = vec![0xCD; 2048];

        File::create(&file_a).unwrap().write_all(&payload).unwrap();
        File::create(&file_b).unwrap().write_all(&payload).unwrap();
        File::create(&file_c).unwrap().write_all(&diff_payload).unwrap();

        let candidates = vec![
            (file_a.to_string_lossy().to_string(), 2048),
            (file_b.to_string_lossy().to_string(), 2048),
            (file_c.to_string_lossy().to_string(), 2048),
        ];

        let dupes = find_duplicates(&candidates);

        // Clean up
        let _ = std::fs::remove_file(&file_a);
        let _ = std::fs::remove_file(&file_b);
        let _ = std::fs::remove_file(&file_c);

        assert_eq!(dupes.len(), 1, "Expected exactly 1 group of duplicates");
        assert_eq!(dupes[0].paths.len(), 2, "Expected 2 matching files in dupe group");
    }

    #[test]
    fn test_leftovers_scan_runs_without_panic() {
        let leftovers = scan_uninstalled_leftovers();
        // Just verify it queries Windows registry and directories safely
        println!("Detected {} leftover candidate folders", leftovers.len());
    }
}
