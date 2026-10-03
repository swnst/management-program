use broker_proto::{
    deserialize_msg, serialize_msg, BrokerRequest, BrokerResponse, DEFAULT_PIPE_NAME,
};
use platform_win::purge_standby_list;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ,
    FILE_GENERIC_WRITE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe,
    PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

// --- Client IPC Wrapper ---

pub struct BrokerClient {
    pipe_handle: HANDLE,
}

impl BrokerClient {
    pub fn connect() -> Result<Self, String> {
        Self::connect_with_name(DEFAULT_PIPE_NAME)
    }

    pub fn connect_with_name(pipe_name: &str) -> Result<Self, String> {
        let wide_name: Vec<u16> = pipe_name.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let handle = match CreateFileW(
                PCWSTR::from_raw(wide_name.as_ptr()),
                FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                windows::Win32::Storage::FileSystem::FILE_SHARE_MODE(0),
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            ) {
                Ok(h) if h != INVALID_HANDLE_VALUE => h,
                _ => return Err(format!("Failed to connect to Broker pipe: {:?}", GetLastError())),
            };

            Ok(Self { pipe_handle: handle })
        }
    }

    pub fn send_request(&self, request: &BrokerRequest) -> Result<BrokerResponse, String> {
        let payload = serialize_msg(request)?;
        let len_bytes = (payload.len() as u32).to_le_bytes();

        unsafe {
            let mut written = 0u32;
            // Write length prefix
            let res = WriteFile(
                self.pipe_handle,
                Some(&len_bytes),
                Some(&mut written),
                None,
            );
            if !res.is_ok() {
                return Err(format!("WriteFile length failed: {:?}", GetLastError()));
            }

            // Write payload
            let res = WriteFile(
                self.pipe_handle,
                Some(&payload),
                Some(&mut written),
                None,
            );
            if !res.is_ok() {
                return Err(format!("WriteFile payload failed: {:?}", GetLastError()));
            }

            // Read response length prefix
            let mut resp_len_buf = [0u8; 4];
            let mut bytes_read = 0u32;
            let res = ReadFile(
                self.pipe_handle,
                Some(&mut resp_len_buf),
                Some(&mut bytes_read),
                None,
            );
            if !res.is_ok() || bytes_read != 4 {
                return Err(format!("ReadFile response length failed: {:?}", GetLastError()));
            }

            let resp_len = u32::from_le_bytes(resp_len_buf) as usize;
            let mut resp_buf = vec![0u8; resp_len];
            let res = ReadFile(
                self.pipe_handle,
                Some(&mut resp_buf),
                Some(&mut bytes_read),
                None,
            );
            if !res.is_ok() || bytes_read as usize != resp_len {
                return Err(format!("ReadFile response body failed: {:?}", GetLastError()));
            }

            deserialize_msg(&resp_buf)
        }
    }
}

impl Drop for BrokerClient {
    fn drop(&mut self) {
        if self.pipe_handle != INVALID_HANDLE_VALUE && !self.pipe_handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.pipe_handle);
            }
        }
    }
}

unsafe impl Send for BrokerClient {}
unsafe impl Sync for BrokerClient {}

// --- Server IPC Handler ---

pub struct BrokerServer {
    pipe_name: String,
    shutdown_signal: Arc<AtomicBool>,
}

impl BrokerServer {
    pub fn new(pipe_name: Option<&str>) -> Self {
        Self {
            pipe_name: pipe_name.unwrap_or(DEFAULT_PIPE_NAME).to_string(),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let wide_name: Vec<u16> = self.pipe_name.encode_utf16().chain(Some(0)).collect();

        while !self.shutdown_signal.load(Ordering::Relaxed) {
            unsafe {
                let pipe_handle = CreateNamedPipeW(
                    PCWSTR::from_raw(wide_name.as_ptr()),
                    PIPE_ACCESS_DUPLEX,
                    PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                    PIPE_UNLIMITED_INSTANCES,
                    65536,
                    65536,
                    0,
                    None,
                );

                if pipe_handle == INVALID_HANDLE_VALUE || pipe_handle.is_invalid() {
                    return Err(format!("CreateNamedPipeW failed: {:?}", GetLastError()));
                }

                let client_connected = ConnectNamedPipe(pipe_handle, None).is_ok()
                    || GetLastError().0 == 535; // ERROR_PIPE_CONNECTED

                if client_connected {
                    self.handle_connection(pipe_handle);
                }

                let _ = DisconnectNamedPipe(pipe_handle);
                let _ = CloseHandle(pipe_handle);
            }
        }

        Ok(())
    }

    fn handle_connection(&self, pipe: HANDLE) {
        unsafe {
            loop {
                let mut len_buf = [0u8; 4];
                let mut bytes_read = 0u32;
                let res = ReadFile(pipe, Some(&mut len_buf), Some(&mut bytes_read), None);
                if !res.is_ok() || bytes_read != 4 {
                    break;
                }

                let req_len = u32::from_le_bytes(len_buf) as usize;
                if req_len > 10 * 1024 * 1024 {
                    break;
                }

                let mut req_buf = vec![0u8; req_len];
                let res = ReadFile(pipe, Some(&mut req_buf), Some(&mut bytes_read), None);
                if !res.is_ok() || bytes_read as usize != req_len {
                    break;
                }

                let request: Result<BrokerRequest, _> = deserialize_msg(&req_buf);
                let response = match request {
                    Ok(BrokerRequest::Ping) => BrokerResponse::Pong,
                    Ok(BrokerRequest::PurgeStandbyList) => match purge_standby_list() {
                        Ok(()) => BrokerResponse::Success,
                        Err(e) => BrokerResponse::Error(e),
                    },
                    Ok(BrokerRequest::ScanVolume { volume_letter: _ }) => {
                        BrokerResponse::Success
                    }
                    Ok(BrokerRequest::Shutdown) => {
                        self.shutdown_signal.store(true, Ordering::Relaxed);
                        BrokerResponse::Success
                    }
                    Err(e) => BrokerResponse::Error(format!("Malformed request: {}", e)),
                };

                let resp_bytes = match serialize_msg(&response) {
                    Ok(b) => b,
                    Err(_) => break,
                };

                let mut written = 0u32;
                let resp_len_bytes = (resp_bytes.len() as u32).to_le_bytes();
                if !WriteFile(pipe, Some(&resp_len_bytes), Some(&mut written), None).is_ok() {
                    break;
                }
                if !WriteFile(pipe, Some(&resp_bytes), Some(&mut written), None).is_ok() {
                    break;
                }

                if self.shutdown_signal.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
    }
}
