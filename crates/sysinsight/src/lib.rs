use core_model::{MemoryInsight, ProcessMemoryItem, StartupProgram};
use sysinfo::System;
use windows::core::PCWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumValueW, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA};

pub fn get_memory_insight() -> MemoryInsight {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = sys.free_memory();

    let mut procs = Vec::new();
    for (pid, process) in sys.processes() {
        let mem = process.memory();
        if mem > 10 * 1024 * 1024 {
            // Include processes using more than 10MB
            procs.push(ProcessMemoryItem {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                private_working_set_bytes: mem,
                working_set_bytes: mem,
                cpu_percent: process.cpu_usage(),
            });
        }
    }

    procs.sort_by(|a, b| b.private_working_set_bytes.cmp(&a.private_working_set_bytes));
    procs.truncate(20);

    MemoryInsight {
        total_ram_bytes: total,
        used_ram_bytes: used,
        free_ram_bytes: free,
        standby_ram_bytes: 0,
        modified_ram_bytes: 0,
        top_processes: procs,
    }
}

pub fn trim_working_sets() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_all();
    let before_used = sys.used_memory();

    for (pid, _) in sys.processes() {
        let raw_pid = pid.as_u32();
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_SET_QUOTA, false, raw_pid) {
                let _ = EmptyWorkingSet(handle);
                let _ = CloseHandle(handle);
            }
        }
    }

    let mut sys_after = System::new_all();
    sys_after.refresh_all();
    let after_used = sys_after.used_memory();

    if before_used > after_used {
        before_used - after_used
    } else {
        0
    }
}

pub fn list_startup_programs() -> Vec<StartupProgram> {
    let mut items = Vec::new();

    // Scan HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    scan_registry_run_key(HKEY_CURRENT_USER, "HKCU\\Run", &mut items);
    // Scan HKLM\Software\Microsoft\Windows\CurrentVersion\Run
    scan_registry_run_key(HKEY_LOCAL_MACHINE, "HKLM\\Run", &mut items);

    items
}

fn scan_registry_run_key(root: HKEY, location_label: &str, items: &mut Vec<StartupProgram>) {
    let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0";
    let wide_subkey: Vec<u16> = subkey.encode_utf16().collect();

    unsafe {
        let mut hkey: HKEY = HKEY::default();
        if RegOpenKeyExW(
            root,
            PCWSTR::from_raw(wide_subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_ok()
        {
            let mut index = 0u32;
            let mut val_name = [0u16; 260];
            let mut val_data = [0u8; 1024];

            loop {
                let mut name_len = val_name.len() as u32;
                let mut data_len = val_data.len() as u32;
                let mut val_type = 0u32;

                let status = RegEnumValueW(
                    hkey,
                    index,
                    windows::core::PWSTR(val_name.as_mut_ptr()),
                    &mut name_len,
                    None,
                    Some(&mut val_type),
                    Some(val_data.as_mut_ptr()),
                    Some(&mut data_len),
                );

                if !status.is_ok() {
                    break;
                }

                let name = String::from_utf16_lossy(&val_name[..name_len as usize]);
                let data_u16: &[u16] = std::slice::from_raw_parts(
                    val_data.as_ptr() as *const u16,
                    (data_len / 2) as usize,
                );
                let command = String::from_utf16_lossy(data_u16)
                    .trim_matches('\0')
                    .to_string();

                items.push(StartupProgram {
                    id: format!("{}_{}", location_label, name),
                    name,
                    command,
                    location: location_label.to_string(),
                    is_enabled: true,
                    publisher: None,
                });

                index += 1;
            }

            let _ = RegCloseKey(hkey);
        }
    }
}
