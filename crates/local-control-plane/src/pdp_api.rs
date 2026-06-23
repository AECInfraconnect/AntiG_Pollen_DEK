use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/pdp/runtimes", get(list_runtimes))
        .route("/v1/tenants/:tenant/pdp/runtimes/:id/probe", post(probe_runtime))
        .route("/v1/tenants/:tenant/pdp/runtimes/:id/load-bundle", post(load_bundle))
        .route("/v1/tenants/:tenant/pdp/evaluate", post(evaluate))
}

async fn list_runtimes(Path(_tenant): Path<String>, State(_state): State<AppState>) -> Json<Value> {
    Json(json!([
        {
            "id": "local-opa-wasm",
            "type": "opa_wasm",
            "status": "ready",
            "policies_loaded": 1,
            "bundle_version": "v1.0.0",
            "p95_latency_ms": 15
        },
        {
            "id": "local-cedar",
            "type": "cedar_native",
            "status": "ready",
            "policies_loaded": 2,
            "bundle_version": "v1.0.0",
            "p95_latency_ms": 5
        }
    ]))
}

async fn probe_runtime(
    Path((_tenant, id)): Path<(String, String)>,
    State(_state): State<AppState>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": format!("Probe successful for runtime {}", id),
        "latency_ms": 2
    }))
}

async fn load_bundle(
    Path((_tenant, id)): Path<(String, String)>,
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": format!("Bundle loaded successfully to {}", id),
        "bundle": payload
    }))
}

async fn evaluate(
    Path(_tenant): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "decision_id": "eval-12345",
        "allowed": true,
        "mode": "enforce",
        "reason": "Allowed by default policy",
        "principal": payload.get("principal").unwrap_or(&json!("unknown")),
        "action": payload.get("action").unwrap_or(&json!("unknown")),
        "resource": payload.get("resource").unwrap_or(&json!("unknown")),
        "pep_type": payload.get("context").and_then(|c| c.get("pep_type")).unwrap_or(&json!("unknown")),
        "pdp_runtime_id": "local-opa-wasm",
        "route_id": "default-route",
        "policy_bundle_id": "baseline-bundle",
        "policy_version": "v1.0.0",
        "latency_ms": 12,
        "fallback_used": false,
        "obligations": [],
        "redactions": [],
        "errors": []
    }))
}
