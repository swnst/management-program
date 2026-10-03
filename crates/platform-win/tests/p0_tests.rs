#[cfg(test)]
mod tests {
    use platform_win::{list_fixed_volumes, recycle_path};
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_list_fixed_volumes_has_real_drives() {
        let volumes = list_fixed_volumes();
        assert!(!volumes.is_empty(), "Expected at least one fixed drive");

        let c_drive = volumes.iter().find(|v| v.volume_letter == 'C');
        assert!(c_drive.is_some(), "Expected drive C to be detected");

        let c = c_drive.unwrap();
        assert!(c.total_bytes > 0, "Total bytes should be > 0");
        assert!(c.free_bytes > 0, "Free bytes should be > 0");
        assert!(c.free_bytes < c.total_bytes, "Free bytes must be less than total bytes");
        assert!(c.is_system, "Drive C should be flagged as system drive");
    }

    #[test]
    fn test_recycle_path_moves_file_to_recycle_bin() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("lumen_test_recycle_{}.txt", std::process::id()));

        // Create a dummy file
        {
            let mut f = File::create(&test_file).expect("Failed to create test file");
            writeln!(f, "Lumen P0 test recycle content").expect("Write failed");
        }
        assert!(test_file.exists(), "Test file must exist before recycle");

        // Move to Recycle Bin
        let res = recycle_path(&test_file);
        assert!(res.is_ok(), "recycle_path failed: {:?}", res.err());
        assert!(!test_file.exists(), "Test file must no longer exist at original path");
    }
}
