use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod http;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Http,
    Stdio,
    Sse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Request,
    Response,
    Notification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMcpEvent {
    pub event_id: String,
    pub transport: TransportType,
    pub direction: MessageDirection,
    pub request_type: String, // e.g. "mcp.tools.call"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc_id: Option<serde_json::Value>,

    pub tenant_id: String,
    pub device_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_name: Option<String>,

    pub payload: serde_json::Value,

    pub session: serde_json::Value,
    pub runtime: serde_json::Value,
}

pub trait TransportAdapter {
    fn transport_name(&self) -> &'static str;
    fn normalize_request(&self, raw: serde_json::Value) -> Result<NormalizedMcpEvent>;
    fn normalize_response(&self, raw: serde_json::Value) -> Result<NormalizedMcpEvent>;
}
