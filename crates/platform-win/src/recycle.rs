use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::{
    SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FO_DELETE,
    SHFILEOPSTRUCTW,
};

/// Safely moves the specified file or directory to the Windows Recycle Bin.
/// Returns Ok(()) on success, or an Err with the Windows error code.
pub fn recycle_path<P: AsRef<Path>>(target: P) -> Result<(), String> {
    let path = target.as_ref();
    if !path.exists() {
        return Err("Target path does not exist".into());
    }

    let full_path = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let path_str = full_path.to_string_lossy().to_string();
    // Strip Windows extended-length prefix \\?\ if present as SHFileOperation doesn't support it well
    let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);

    // SHFileOperation expects a double-null-terminated string (\0\0)
    let mut wide_buf: Vec<u16> = clean_path.encode_utf16().collect();
    wide_buf.push(0);
    wide_buf.push(0);

    unsafe {
        let flags = (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT).0 as u16;
        let mut op = SHFILEOPSTRUCTW {
            hwnd: Default::default(),
            wFunc: FO_DELETE,
            pFrom: PCWSTR::from_raw(wide_buf.as_ptr()),
            pTo: PCWSTR::null(),
            fFlags: flags,
            fAnyOperationsAborted: false.into(),
            hNameMappings: std::ptr::null_mut(),
            lpszProgressTitle: PCWSTR::null(),
        };

        let res = SHFileOperationW(&mut op);
        if res != 0 {
            return Err(format!("SHFileOperationW failed with code {}", res));
        }

        if op.fAnyOperationsAborted.as_bool() {
            return Err("File deletion was aborted".into());
        }

        Ok(())
    }
}
