use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AgentAppState {
    // normally contains stores or db connections
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevel {
    Observe, // เก็บ telemetry อย่างเดียว ไม่บล็อก (default หลัง register)
    Guard,   // observe + alert + obligation เบา (เช่น require_approval งานเสี่ยง)
    Enforce, // PEP บังคับ policy เต็ม (allow/deny/redact)
    Block,   // deny ทั้งหมด (kill switch ต่อ agent)
}

#[derive(Debug, Deserialize)]
pub struct RegisterAgentReq {
    pub agent_id: String,
    pub control_level: ControlLevel,
}

pub fn router(state: Arc<AgentAppState>) -> Router {
    Router::new()
        .route("/v1/agents/bindings", get(list_bindings))
        .route("/v1/agents/fingerprints", get(list_fingerprints))
        .route("/v1/agents/register", post(register_agent))
        .with_state(state)
}

async fn list_bindings(State(_state): State<Arc<AgentAppState>>) -> Json<serde_json::Value> {
    // Mocked response for now. Will wire to dek_agent_observer::SharedBindingStore in the real app.
    Json(serde_json::json!({
        "bindings": []
    }))
}

async fn list_fingerprints(State(_state): State<Arc<AgentAppState>>) -> Json<serde_json::Value> {
    // Mocked response for now. Will wire to dek_fingerprint_defs::FingerprintService
    Json(serde_json::json!({
        "fingerprints": []
    }))
}

async fn register_agent(
    State(_state): State<Arc<AgentAppState>>,
    Json(req): Json<RegisterAgentReq>,
) -> Json<serde_json::Value> {
    // 1. insert to AgentInventory
    // 2. ถ้า req.control_level == Enforce -> deploy policy "strict_mode"
    // 3. ถ้า req.control_level == Block -> deploy policy "deny_all"
    Json(serde_json::json!({"status": "ok", "control_level": req.control_level}))
}
