use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use dek_control_plane_api::policy::*;
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant_id/policies",
            get(list_policies).post(create_policy),
        )
        .route(
            "/v1/tenants/:tenant_id/policies/:policy_id",
            get(get_policy).patch(patch_policy).delete(delete_policy),
        )
}

async fn list_policies(
    Path(_tenant_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // TODO: implement with policy store
    (StatusCode::OK, Json(json!([])))
}

async fn create_policy(
    Path(_tenant_id): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<PolicyDraft>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!(payload)))
}

async fn get_policy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn patch_policy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn delete_policy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}
