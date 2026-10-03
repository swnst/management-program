use std::ffi::c_void;
use std::mem::size_of;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

pub fn enable_privilege(privilege_name: &str) -> Result<(), String> {
    unsafe {
        let mut token: HANDLE = HANDLE::default();
        let open_res = OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        );
        if !open_res.is_ok() {
            return Err(format!("OpenProcessToken failed: {:?}", GetLastError()));
        }

        let wide_name: Vec<u16> = privilege_name.encode_utf16().chain(Some(0)).collect();
        let mut luid = LUID::default();
        let lookup_res = LookupPrivilegeValueW(
            PCWSTR::null(),
            PCWSTR::from_raw(wide_name.as_ptr()),
            &mut luid,
        );
        if !lookup_res.is_ok() {
            let _ = CloseHandle(token);
            return Err(format!("LookupPrivilegeValueW failed: {:?}", GetLastError()));
        }

        let mut tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: windows::Win32::Security::TOKEN_PRIVILEGES_ATTRIBUTES(0x00000002), // SE_PRIVILEGE_ENABLED
            }],
        };

        let adjust_res = AdjustTokenPrivileges(
            token,
            false,
            Some(&mut tp),
            size_of::<TOKEN_PRIVILEGES>() as u32,
            None,
            None,
        );
        let _ = CloseHandle(token);

        if !adjust_res.is_ok() {
            return Err(format!("AdjustTokenPrivileges failed: {:?}", GetLastError()));
        }
        Ok(())
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = 0u32;
        let mut ret_len = 0u32;
        let res = windows::Win32::Security::GetTokenInformation(
            token,
            windows::Win32::Security::TokenElevation,
            Some(&mut elevation as *mut _ as *mut c_void),
            size_of::<u32>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);
        res.is_ok() && elevation != 0
    }
}
