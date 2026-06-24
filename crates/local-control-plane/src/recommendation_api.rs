// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde_json::Value;

use crate::{error::ApiResult, state::AppState};
use dek_recommend::Recommender;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/recommendations", get(get_recommendations))
}

async fn get_recommendations(State(_state): State<AppState>) -> ApiResult<Json<Value>> {
    // Determine local device capabilities
    let caps = dek_capability_registry::CapabilityRegistry::new("local".into(), "1.0".into()).gather();
    
    // In a real system, we'd query the SQLite/PostgreSQL store for these stats.
    // For now, return mock recent stats to surface recommendations in the dashboard.
    let recent_stats = dek_agent_observer::activity::ActivityCounts {
        total_decisions: 150,
        denied_actions: 12,
        mcp_invocations: 55,
    };

    let recs = Recommender::recommend(&caps, &recent_stats);

    Ok(Json(serde_json::json!({
        "status": "success",
        "recommendations": recs,
    })))
}
