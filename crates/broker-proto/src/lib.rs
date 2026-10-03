use serde::{Deserialize, Serialize};

pub const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\LumenBrokerPipe";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerRequest {
    Ping,
    ScanVolume { volume_letter: char },
    PurgeStandbyList,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerResponse {
    Pong,
    Success,
    Error(String),
}

// --- Wire Serialization Helpers ---

pub fn serialize_msg<T: Serialize>(val: &T) -> Result<Vec<u8>, String> {
    postcard::to_allocvec(val).map_err(|e| e.to_string())
}

pub fn deserialize_msg<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, String> {
    postcard::from_bytes(bytes).map_err(|e| e.to_string())
}
