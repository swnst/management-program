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
