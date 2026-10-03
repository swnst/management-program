use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerRequest {
    Ping,
    ScanVolume { volume_letter: char },
    PurgeStandbyList,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerResponse {
    Pong,
    Success,
    Error(String),
}
