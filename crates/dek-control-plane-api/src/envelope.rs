use crate::identity::ControlPlaneIdentity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    pub api_version: String,
    pub request_id: String,
    pub identity: ControlPlaneIdentity,
    pub data: T,
    pub warnings: Vec<ApiWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub api_version: String,
    pub request_id: String,
    pub identity: Option<ControlPlaneIdentity>,
    pub error: ApiErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: serde_json::Value,
    pub retryable: bool,
}
