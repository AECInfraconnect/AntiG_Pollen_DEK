// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dek_deployment_planner::SuggestedPolicy;
use dek_domain_schema::feasibility::{PolicyFeasibilityRequest, PolicyFeasibilityResult};

use crate::error::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/local/scan", post(scan))
        .route(
            "/v1/local/capability-snapshot/latest",
            get(get_latest_snapshot),
        )
        .route("/v1/policy-suggestions", get(get_policy_suggestions))
        .route("/v1/policies/feasibility", post(evaluate_feasibility))
        .route("/v1/deployment-sessions", post(create_deployment_session))
        .route(
            "/v1/deployment-sessions/:id/actions/:action_id/approve",
            post(approve_action),
        )
}

async fn scan(State(_st): State<AppState>) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // Stub
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "scanned"})),
    ))
}

async fn get_latest_snapshot(
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // Stub
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "snapshot"})),
    ))
}

async fn get_policy_suggestions(
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<Vec<SuggestedPolicy>>)> {
    // Stub
    Ok((StatusCode::OK, Json(vec![])))
}

async fn evaluate_feasibility(
    State(_st): State<AppState>,
    Json(_req): Json<PolicyFeasibilityRequest>,
) -> ApiResult<(StatusCode, Json<Vec<PolicyFeasibilityResult>>)> {
    // Stub
    Ok((StatusCode::OK, Json(vec![])))
}

async fn create_deployment_session(
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // Stub
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "session_created"})),
    ))
}

async fn approve_action(
    Path((_session_id, _action_id)): Path<(String, String)>,
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // Stub
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "approved"})),
    ))
}
