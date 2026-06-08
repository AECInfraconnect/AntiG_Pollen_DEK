use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde_json::json;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/scenarios/poisoned-bundle", post(poisoned_bundle))
        .route("/v1/scenarios/replay-old-bundle", post(replay_old_bundle))
        .route("/v1/scenarios/wrong-tenant-bundle", post(wrong_tenant_bundle))
        .route("/v1/scenarios/expired-bundle", post(expired_bundle))
        .route("/v1/scenarios/mitm-config", post(mitm_config))
        .route("/v1/scenarios/telemetry-backpressure", post(telemetry_backpressure))
        .route("/v1/scenarios/ebpf-map-poisoning", post(ebpf_map_poisoning))
        .route("/v1/scenarios/ebpf-map-exhaustion", post(ebpf_map_exhaustion))
        .route("/v1/scenarios/spiffe-bundle-rotation", post(spiffe_bundle_rotation))
        .route("/v1/scenarios/device-revoked", post(device_revoked))
}

async fn poisoned_bundle(State(_state): State<AppState>) -> impl IntoResponse {
    // This endpoint triggers mock-cloud to start serving a bundle with a bad signature
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "poisoned-bundle"})))
}

async fn replay_old_bundle(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "replay-old-bundle"})))
}

async fn wrong_tenant_bundle(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "wrong-tenant-bundle"})))
}

async fn expired_bundle(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "expired-bundle"})))
}

async fn mitm_config(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "mitm-config"})))
}

async fn telemetry_backpressure(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "telemetry-backpressure"})))
}

async fn ebpf_map_poisoning(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "ebpf-map-poisoning"})))
}

async fn ebpf_map_exhaustion(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "ebpf-map-exhaustion"})))
}

async fn spiffe_bundle_rotation(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "spiffe-bundle-rotation"})))
}

async fn device_revoked(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "scenario_activated", "scenario": "device-revoked"})))
}
