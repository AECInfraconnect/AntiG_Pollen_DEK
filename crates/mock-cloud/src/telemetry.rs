#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::state::{AppState, LogEntry};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::Utc;
use serde_json::{json, Value};
use tracing::info;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/telemetry",
            post(ingest_telemetry),
        )
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/decision-logs",
            post(ingest_decision_logs),
        )
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/health",
            post(report_health),
        )
        // More endpoints for phase 2
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/telemetry/security-events",
            post(ingest_security_events),
        )
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/telemetry/runtime-metrics",
            post(ingest_runtime_metrics),
        )
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/telemetry/ebpf-events",
            post(ingest_ebpf_events),
        )
}

fn verify_device_tenant(
    state: &AppState,
    tenant_id: &str,
    device_id: &str,
) -> Result<(), &'static str> {
    let devices = state.devices.lock().unwrap();
    if let Some(dev) = devices.get(device_id) {
        if dev.tenant_id == tenant_id {
            return Ok(());
        }
        return Err("Tenant mismatch");
    }
    Err("Device not found")
}

async fn ingest_telemetry(
    Path((tenant_id, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    if verify_device_tenant(&state, &tenant_id, &device_id).is_err() {
        return Json(json!({"error": "tenant mismatch"}));
    }
    info!(
        "CLOUD RECEIVED TELEMETRY from {}/{}: {}",
        tenant_id, device_id, payload
    );
    Json(json!({ "status": "ingested" }))
}

async fn ingest_decision_logs(
    Path((tenant_id, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    if verify_device_tenant(&state, &tenant_id, &device_id).is_err() {
        return Json(json!({"error": "tenant mismatch"}));
    }
    info!(
        "CLOUD RECEIVED DECISION LOGS from {}/{}: {}",
        tenant_id, device_id, payload
    );

    let mut logs = state.decision_logs.lock().unwrap();
    if let Some(events) = payload.as_array() {
        for ev in events {
            let action = ev
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let decision = ev
                .get("decision")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let ts = ev
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            logs.push_front(LogEntry {
                device_id: device_id.clone(),
                timestamp: ts,
                action,
                decision,
            });

            if logs.len() > 1000 {
                logs.pop_back();
            }
        }
    }

    Json(json!({ "status": "ingested" }))
}

async fn report_health(
    Path((_tenant_id, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    info!(
        "CLOUD RECEIVED HEALTH REPORT from {}: {}",
        device_id, payload
    );
    let mut devices = state.devices.lock().unwrap();
    if let Some(dev) = devices.get_mut(&device_id) {
        dev.last_health = Utc::now().to_rfc3339();
    }
    Json(json!({ "status": "ok" }))
}

async fn ingest_security_events(
    Path((tenant_id, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if verify_device_tenant(&state, &tenant_id, &device_id).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "tenant mismatch"})),
        );
    }
    info!(
        "CLOUD RECEIVED SECURITY EVENTS from {}/{}: {}",
        tenant_id, device_id, payload
    );
    (StatusCode::OK, Json(json!({ "status": "ingested" })))
}

async fn ingest_runtime_metrics(
    Path((tenant_id, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if verify_device_tenant(&state, &tenant_id, &device_id).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "tenant mismatch"})),
        );
    }
    info!(
        "CLOUD RECEIVED RUNTIME METRICS from {}/{}: {}",
        tenant_id, device_id, payload
    );
    (StatusCode::OK, Json(json!({ "status": "ingested" })))
}

async fn ingest_ebpf_events(
    Path((tenant_id, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    if verify_device_tenant(&state, &tenant_id, &device_id).is_err() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "tenant mismatch"})),
        );
    }
    info!(
        "CLOUD RECEIVED EBPF EVENTS from {}/{}: {}",
        tenant_id, device_id, payload
    );
    (StatusCode::OK, Json(json!({ "status": "ingested" })))
}
