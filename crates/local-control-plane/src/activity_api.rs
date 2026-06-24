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
        .route("/v1/activity/stream", get(stream_activity))
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
            pep_plane: Some("McpProxy".into()),
            enforced_for_real: Some(true),
            status_badge: Some("Ok".into()),
            message_th: Some("อนุญาต".into()),
        },
        dek_agent_observer::activity::ActivityItem {
            timestamp: "2026-06-24T03:00:05Z".into(),
            event_type: "network_egress".into(),
            decision: Some("deny".into()),
            resource: "example.com".into(),
            reason: "domain blocklisted".into(),
            pep_plane: Some("McpProxy".into()),
            enforced_for_real: Some(true),
            status_badge: Some("Denied".into()),
            message_th: Some("ปฏิเสธ".into()),
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

use axum::response::sse::{Event, Sse};
use std::convert::Infallible;

async fn stream_activity(
    State(_state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // In a real system, this would subscribe to a broadcast channel receiving real events
    let stream = async_stream::stream! {
        // Mock SSE heartbeat/stream for UI integration tests
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;

            let mock_item = dek_agent_observer::activity::ActivityItem {
                timestamp: chrono::Utc::now().to_rfc3339(),
                event_type: "mcp_tool_call".into(),
                decision: Some("allow".into()),
                resource: "sse.stream".into(),
                reason: "live mock event".into(),
                pep_plane: Some("McpStdio".into()),
                enforced_for_real: Some(true),
                status_badge: Some("Ok".into()),
                message_th: Some("✅ อนุญาต (mock live event)".into()),
            };

            let data = match serde_json::to_string(&mock_item) {
                Ok(d) => d,
                Err(_) => "{}".to_string(),
            };
            yield Ok(Event::default().data(data));
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}
