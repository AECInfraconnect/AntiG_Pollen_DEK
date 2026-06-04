use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage<T> {
    pub version: String,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    HealthCheck,
    ReloadConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    HealthStatus {
        status: String,
        core_version: String,
    },
    ReloadStatus {
        status: String,
    },
    Error(String),
}
