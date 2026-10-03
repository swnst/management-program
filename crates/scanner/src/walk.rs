use crate::tree::VolumeIndex;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileExW, FindNextFileW, FindExInfoBasic, FIND_FIRST_EX_LARGE_FETCH,
    WIN32_FIND_DATAW, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
};

pub struct ScanOptions {
    pub cancel_flag: Arc<AtomicBool>,
    pub max_depth: Option<u32>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            max_depth: None,
        }
    }
}

pub fn scan_volume_fast(
    drive_letter: char,
    volume_name: String,
    options: &ScanOptions,
    on_progress: Option<&dyn Fn(u64, u64, &str)>,
) -> Result<VolumeIndex, String> {
    let mut index = VolumeIndex::new(drive_letter, volume_name);
    let root_path_str = format!("{}:\\", drive_letter);
    let root_path = Path::new(&root_path_str);

    let mut scanned_files = 0u64;
    let mut scanned_bytes = 0u64;

    walk_directory_fast(
        root_path,
        0, // root node index
        &mut index,
        0,
        options,
        &mut scanned_files,
        &mut scanned_bytes,
        on_progress,
    )?;

    index.aggregate_sizes();
    Ok(index)
}

fn walk_directory_fast(
    dir_path: &Path,
    parent_node_idx: u32,
    index: &mut VolumeIndex,
    current_depth: u32,
    options: &ScanOptions,
    scanned_files: &mut u64,
    scanned_bytes: &mut u64,
    on_progress: Option<&dyn Fn(u64, u64, &str)>,
) -> Result<(), String> {
    if options.cancel_flag.load(Ordering::Relaxed) {
        return Ok(());
    }

    if let Some(max_d) = options.max_depth {
        if current_depth > max_d {
            return Ok(());
        }
    }

    let search_pattern = format!("{}\\*", dir_path.to_string_lossy());
    let wide_pattern: Vec<u16> = search_pattern.encode_utf16().chain(Some(0)).collect();

    let mut find_data = WIN32_FIND_DATAW::default();

    unsafe {
        let handle = FindFirstFileExW(
            PCWSTR::from_raw(wide_pattern.as_ptr()),
            FindExInfoBasic,
            &mut find_data as *mut _ as *mut _,
            windows::Win32::Storage::FileSystem::FindExSearchNameMatch,
            None,
            FIND_FIRST_EX_LARGE_FETCH,
        );

        let handle: HANDLE = match handle {
            Ok(h) if !h.is_invalid() => h,
            _ => return Ok(()), // Access denied or directory empty
        };

        let mut subdirectories = Vec::new();

        loop {
            let name_len = find_data
                .cFileName
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(find_data.cFileName.len());
            let name = String::from_utf16_lossy(&find_data.cFileName[..name_len]);

            if name != "." && name != ".." {
                let attrs = find_data.dwFileAttributes;
                let is_dir = (attrs & FILE_ATTRIBUTE_DIRECTORY.0) != 0;
                let is_reparse = (attrs & FILE_ATTRIBUTE_REPARSE_POINT.0) != 0;

                let size = if is_dir {
                    0
                } else {
                    ((find_data.nFileSizeHigh as u64) << 32) | (find_data.nFileSizeLow as u64)
                };

                let created_secs = filetime_to_unix(
                    find_data.ftCreationTime.dwHighDateTime,
                    find_data.ftCreationTime.dwLowDateTime,
                );
                let modified_secs = filetime_to_unix(
                    find_data.ftLastWriteTime.dwHighDateTime,
                    find_data.ftLastWriteTime.dwLowDateTime,
                );

                let node_idx = index.add_node(
                    parent_node_idx,
                    &name,
                    attrs,
                    size,
                    size, // initial allocated estimation
                    created_secs,
                    modified_secs,
                );

                *scanned_files += 1;
                *scanned_bytes += size;

                if is_dir && !is_reparse {
                    let sub_path = dir_path.join(&name);
                    subdirectories.push((sub_path, node_idx));
                }

                if *scanned_files % 2000 == 0 {
                    if let Some(cb) = on_progress {
                        cb(*scanned_files, *scanned_bytes, &name);
                    }
                }
            }

            if FindNextFileW(handle, &mut find_data).is_err() {
                break;
            }
        }

        let _ = FindClose(handle);

        // Recurse into subdirectories
        for (sub_path, sub_node_idx) in subdirectories {
            let _ = walk_directory_fast(
                &sub_path,
                sub_node_idx,
                index,
                current_depth + 1,
                options,
                scanned_files,
                scanned_bytes,
                on_progress,
            );
        }
    }

    Ok(())
}

fn filetime_to_unix(high: u32, low: u32) -> i64 {
    let ft = ((high as u64) << 32) | (low as u64);
    // 100-nanosecond intervals between Jan 1, 1601 and Jan 1, 1970 is 116,444,736,000,000,000
    if ft < 116_444_736_000_000_000 {
        return 0;
    }
    ((ft - 116_444_736_000_000_000) / 10_000_000) as i64
}
