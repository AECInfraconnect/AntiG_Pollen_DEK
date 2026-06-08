use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use dek_domain_schema::TelemetryEvent;

pub fn router() -> Router<AppState> {
    Router::new()
        // Generic bulk ingest
        .route(
            "/v1/tenants/:tenant_id/telemetry/events",
            post(ingest_telemetry_events),
        )
}

#[derive(serde::Deserialize)]
pub struct TelemetryPayload {
    pub events: Vec<TelemetryEvent>,
}

async fn ingest_telemetry_events(
    Path(_tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<TelemetryPayload>,
) -> impl IntoResponse {
    let mut logs = state.telemetry_events.lock().unwrap();

    let mut redaction_failed = false;

    for event in payload.events {
        // Redaction Validation:
        // Mock-cloud asserts that no raw credentials leak into telemetry
        if let TelemetryEvent::Decision { reason, .. } = &event {
            if reason.to_lowercase().contains("bearer") || reason.to_lowercase().contains("password") {
                redaction_failed = true;
                break;
            }
        }
        
        logs.push_front(event);
        if logs.len() > 1000 {
            logs.pop_back();
        }
    }

    if redaction_failed {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Unredacted secrets detected in telemetry payload"})),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"status": "ingested"})),
    )
}
