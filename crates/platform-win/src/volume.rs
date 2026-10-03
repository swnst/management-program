use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, GENERIC_READ, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::FSCTL_GET_NTFS_VOLUME_DATA;
use windows::Win32::System::IO::DeviceIoControl;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct NtfsVolumeDataBuffer {
    pub volume_serial_number: i64,
    pub number_sectors: i64,
    pub total_clusters: i64,
    pub free_clusters: i64,
    pub total_reserved: i64,
    pub bytes_per_sector: u32,
    pub bytes_per_cluster: u32,
    pub bytes_per_file_record_segment: u32,
    pub clusters_per_file_record_segment: u32,
    pub mft_valid_data_length: i64,
    pub mft_start_lcn: i64,
    pub mft2_start_lcn: i64,
    pub mft_zone_start: i64,
    pub mft_zone_end: i64,
}

pub fn open_volume_handle(drive_letter: char) -> Result<HANDLE, String> {
    let path = format!(r"\\.\{}:", drive_letter);
    let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();

    unsafe {
        let handle = CreateFileW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            HANDLE::default(),
        );

        match handle {
            Ok(h) if !h.is_invalid() => Ok(h),
            _ => Err(format!(
                "Failed to open volume handle for {}: {:?}",
                drive_letter,
                GetLastError()
            )),
        }
    }
}

pub fn get_ntfs_volume_data(handle: HANDLE) -> Result<NtfsVolumeDataBuffer, String> {
    let mut data = NtfsVolumeDataBuffer::default();
    let mut bytes_returned = 0u32;

    unsafe {
        let res = DeviceIoControl(
            handle,
            FSCTL_GET_NTFS_VOLUME_DATA,
            None,
            0,
            Some(&mut data as *mut _ as *mut _),
            std::mem::size_of::<NtfsVolumeDataBuffer>() as u32,
            Some(&mut bytes_returned),
            None,
        );

        if res.is_ok() {
            Ok(data)
        } else {
            Err(format!(
                "DeviceIoControl FSCTL_GET_NTFS_VOLUME_DATA failed: {:?}",
                GetLastError()
            ))
        }
    }
}

pub fn safe_close_handle(handle: HANDLE) {
    unsafe {
        if !handle.is_invalid() {
            let _ = CloseHandle(handle);
        }
    }
}

pub fn list_fixed_volumes() -> Vec<core_model::DiskSummary> {
    use windows::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW,
    };

    let mut drives = Vec::new();
    let mut buffer = [0u16; 512];

    unsafe {
        let len = GetLogicalDriveStringsW(Some(&mut buffer));
        if len == 0 || len > buffer.len() as u32 {
            return drives;
        }

        let sys_drive_letter = std::env::var("SystemDrive")
            .unwrap_or_else(|_| "C:".to_string())
            .chars()
            .next()
            .unwrap_or('C')
            .to_ascii_uppercase();

        let mut start = 0;
        while start < len as usize {
            let mut end = start;
            while end < len as usize && buffer[end] != 0 {
                end += 1;
            }

            if end > start {
                let drive_str_utf16 = &buffer[start..=end]; // includes null terminator
                let drive_type = GetDriveTypeW(PCWSTR::from_raw(drive_str_utf16.as_ptr()));

                // DRIVE_FIXED has constant value 3 in Win32
                if drive_type == 3 {
                    let drive_char = buffer[start] as u8 as char;
                    let letter = drive_char.to_ascii_uppercase();

                    let mut free_bytes_available = 0u64;
                    let mut total_number_of_bytes = 0u64;
                    let mut total_number_of_free_bytes = 0u64;

                    let space_ok = GetDiskFreeSpaceExW(
                        PCWSTR::from_raw(drive_str_utf16.as_ptr()),
                        Some(&mut free_bytes_available),
                        Some(&mut total_number_of_bytes),
                        Some(&mut total_number_of_free_bytes),
                    )
                    .is_ok();

                    let mut volume_name_buf = [0u16; 261];
                    let mut fs_name_buf = [0u16; 261];
                    let mut serial_number = 0u32;
                    let mut max_component_len = 0u32;
                    let mut file_system_flags = 0u32;

                    let _ = GetVolumeInformationW(
                        PCWSTR::from_raw(drive_str_utf16.as_ptr()),
                        Some(&mut volume_name_buf),
                        Some(&mut serial_number),
                        Some(&mut max_component_len),
                        Some(&mut file_system_flags),
                        Some(&mut fs_name_buf),
                    );

                    let vol_name_raw = String::from_utf16_lossy(&volume_name_buf)
                        .trim_matches('\0')
                        .to_string();
                    let vol_name = if vol_name_raw.is_empty() {
                        format!("Local Disk ({}:)", letter)
                    } else {
                        vol_name_raw
                    };

                    let fs_name = String::from_utf16_lossy(&fs_name_buf)
                        .trim_matches('\0')
                        .to_string();

                    if space_ok {
                        drives.push(core_model::DiskSummary {
                            volume_letter: letter,
                            volume_name: vol_name,
                            total_bytes: total_number_of_bytes,
                            free_bytes: free_bytes_available,
                            is_ntfs: fs_name.eq_ignore_ascii_case("NTFS"),
                            is_system: letter == sys_drive_letter,
                        });
                    }
                }
            }

            start = end + 1;
        }
    }

    drives
}
