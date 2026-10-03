use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverCandidate {
    pub program_hint: String,
    pub path: String,
    pub size_bytes: u64,
    pub files_count: u64,
    pub confidence: String,
}

pub fn scan_uninstalled_leftovers() -> Vec<LeftoverCandidate> {
    let installed_apps = get_installed_program_names();
    let mut candidates = Vec::new();

    let mut search_roots = Vec::new();
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        search_roots.push(PathBuf::from(local));
    }
    if let Ok(roaming) = std::env::var("APPDATA") {
        search_roots.push(PathBuf::from(roaming));
    }
    if let Ok(prog_data) = std::env::var("ProgramData") {
        search_roots.push(PathBuf::from(prog_data));
    }

    // Common system vendors to never flag
    let safe_vendors: HashSet<&str> = [
        "microsoft", "windows", "nvidia", "intel", "amd", "google", "apple", "adobe",
        "mozilla", "realtek", "system32", "temp",
    ].iter().cloned().collect();

    for root in search_roots {
        if !root.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let lower_name = folder_name.to_lowercase();

                    if safe_vendors.contains(lower_name.as_str()) {
                        continue;
                    }

                    // Check if folder name matches any currently installed program
                    let is_matched = installed_apps.iter().any(|app| {
                        let app_lower = app.to_lowercase();
                        app_lower.contains(&lower_name) || lower_name.contains(&app_lower)
                    });

                    // If not matched to any active installation, inspect directory
                    if !is_matched {
                        let (bytes, count) = get_dir_size_and_count(&path);
                        if bytes > 5 * 1024 * 1024 {
                            // > 5MB leftover folder
                            candidates.push(LeftoverCandidate {
                                program_hint: folder_name,
                                path: path.to_string_lossy().to_string(),
                                size_bytes: bytes,
                                files_count: count,
                                confidence: "Medium".into(),
                            });
                        }
                    }
                }
            }
        }
    }

    candidates
}

fn get_dir_size_and_count(dir: &Path) -> (u64, u64) {
    let mut bytes = 0u64;
    let mut count = 0u64;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Ok(m) = entry.metadata() {
                if m.is_file() {
                    bytes += m.len();
                    count += 1;
                } else if m.is_dir() && (MetadataExt::file_attributes(&m) & 0x400) == 0 {
                    let (sub_b, sub_c) = get_dir_size_and_count(&p);
                    bytes += sub_b;
                    count += sub_c;
                }
            }
        }
    }

    (bytes, count)
}

fn get_installed_program_names() -> HashSet<String> {
    let mut names = HashSet::new();

    // HKLM 64-bit & 32-bit
    scan_uninstall_key(HKEY_LOCAL_MACHINE, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", &mut names);
    scan_uninstall_key(HKEY_LOCAL_MACHINE, r"Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall", &mut names);
    // HKCU
    scan_uninstall_key(HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Uninstall", &mut names);

    names
}

fn scan_uninstall_key(root: HKEY, subkey: &str, names: &mut HashSet<String>) {
    let wide_key: Vec<u16> = subkey.encode_utf16().chain(Some(0)).collect();

    unsafe {
        let mut hkey: HKEY = HKEY::default();
        if RegOpenKeyExW(root, PCWSTR::from_raw(wide_key.as_ptr()), 0, KEY_READ, &mut hkey).is_ok() {
            let mut index = 0u32;
            let mut subkey_name = [0u16; 256];

            loop {
                let mut name_len = subkey_name.len() as u32;
                if RegEnumKeyExW(
                    hkey,
                    index,
                    windows::core::PWSTR(subkey_name.as_mut_ptr()),
                    &mut name_len,
                    None,
                    windows::core::PWSTR::null(),
                    None,
                    None,
                )
                .is_err()
                {
                    break;
                }

                let sub_entry = String::from_utf16_lossy(&subkey_name[..name_len as usize]);
                let full_sub = format!("{}\\{}", subkey, sub_entry);
                let wide_full: Vec<u16> = full_sub.encode_utf16().chain(Some(0)).collect();

                let mut item_key = HKEY::default();
                if RegOpenKeyExW(root, PCWSTR::from_raw(wide_full.as_ptr()), 0, KEY_READ, &mut item_key).is_ok() {
                    let display_val = "DisplayName\0".encode_utf16().collect::<Vec<u16>>();
                    let mut data = [0u8; 512];
                    let mut data_len = data.len() as u32;
                    let mut val_type = 0u32;

                    if RegQueryValueExW(
                        item_key,
                        PCWSTR::from_raw(display_val.as_ptr()),
                        None,
                        None,
                        Some(data.as_mut_ptr()),
                        Some(&mut data_len),
                    )
                    .is_ok()
                    {
                        let slice_u16 = std::slice::from_raw_parts(data.as_ptr() as *const u16, (data_len / 2) as usize);
                        let name = String::from_utf16_lossy(slice_u16).trim_matches('\0').to_string();
                        if !name.is_empty() {
                            names.insert(name);
                        }
                    }
                    let _ = RegCloseKey(item_key);
                }

                index += 1;
            }
            let _ = RegCloseKey(hkey);
        }
    }
}
