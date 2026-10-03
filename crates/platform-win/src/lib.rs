pub mod privilege;
pub mod volume;

pub use privilege::{enable_privilege, is_elevated};
pub use volume::{get_ntfs_volume_data, open_volume_handle, safe_close_handle, NtfsVolumeDataBuffer};
