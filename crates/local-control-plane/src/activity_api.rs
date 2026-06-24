// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::{error::ApiResult, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/activity", get(get_activity))
        .route("/v1/tenants/:tenant/activity", get(get_activity_tenant))
}

async fn get_activity(State(_state): State<AppState>) -> ApiResult<Json<Value>> {
    // Return mock activity grouped into sets for UI
    let raw_events = vec![
        dek_agent_observer::activity::ActivityItem {
            timestamp: "2026-06-24T03:00:00Z".into(),
            event_type: "mcp_tool_call".into(),
            decision: Some("allow".into()),
            resource: "safe.echo".into(),
            reason: "allowed by policy".into(),
        },
        dek_agent_observer::activity::ActivityItem {
            timestamp: "2026-06-24T03:00:05Z".into(),
            event_type: "network_egress".into(),
            decision: Some("deny".into()),
            resource: "example.com".into(),
            reason: "domain blocklisted".into(),
        },
    ];

    let sets = dek_agent_observer::activity::group_into_sets(raw_events, 300);

    Ok(Json(json!({
        "status": "success",
        "activity_sets": sets,
    })))
}

async fn get_activity_tenant(
    Path(_tenant): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<Value>> {
    get_activity(State(state)).await
}
