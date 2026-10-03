use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::Foundation::STATUS_SUCCESS;

// --- Native NT Definitions for Memory Purge ---

const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;
const MEMORY_PURGE_STANDBY_LIST: u32 = 4;

type NtSetSystemInformationFn = unsafe extern "system" fn(
    system_information_class: u32,
    system_information: *const c_void,
    system_information_length: u32,
) -> i32;

pub fn purge_standby_list() -> Result<(), String> {
    crate::enable_privilege("SeProfileSingleProcessPrivilege")?;

    unsafe {
        let ntdll = windows::Win32::System::LibraryLoader::GetModuleHandleW(windows::core::w!("ntdll.dll"))
            .map_err(|e| format!("GetModuleHandleW ntdll.dll failed: {:?}", e))?;

        let proc_addr = windows::Win32::System::LibraryLoader::GetProcAddress(
            ntdll,
            windows::core::s!("NtSetSystemInformation"),
        );

        let func: NtSetSystemInformationFn = match proc_addr {
            Some(sym) => std::mem::transmute(sym),
            None => return Err("NtSetSystemInformation symbol not found in ntdll.dll".into()),
        };

        let command: u32 = MEMORY_PURGE_STANDBY_LIST;
        let status = func(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );

        if status == STATUS_SUCCESS.0 {
            Ok(())
        } else {
            Err(format!("NtSetSystemInformation returned NTSTATUS 0x{:08X}", status as u32))
        }
    }
}
