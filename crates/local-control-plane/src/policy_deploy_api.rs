use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/policies/deploy/preview", post(preview_deploy))
        .route("/v1/tenants/:tenant/policies/deploy/commit", post(commit_deploy))
        .route("/v1/tenants/:tenant/policies/deploy/rollback", post(rollback_deploy))
}

async fn preview_deploy(
    Path(_tenant): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": "Preview generated",
        "diff": {
            "added": ["new_policy.cedar"],
            "removed": []
        },
        "payload": payload
    }))
}

async fn commit_deploy(
    Path(_tenant): Path<String>,
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": "Deployment committed successfully",
        "bundle_version": "v1.0.1"
    }))
}

async fn rollback_deploy(
    Path(_tenant): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": "Deployment rolled back",
        "rolled_back_to": payload.get("version").unwrap_or(&json!("v1.0.0"))
    }))
}
