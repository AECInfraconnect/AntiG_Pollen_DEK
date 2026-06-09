use crate::state::AppState;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use dek_domain_schema::TelemetryEvent;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mock/admin/decision-logs", get(get_decision_logs_json))
        .route("/mock/admin/decision-logs/view", get(view_decision_logs))
}

pub async fn get_decision_logs_json(State(state): State<AppState>) -> impl IntoResponse {
    let events = state.telemetry_events.lock().unwrap();
    let decisions: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            TelemetryEvent::Decision { timestamp, device_id, principal_id, action, resource_id, decision, reason, .. } => {
                Some(serde_json::json!({
                    "timestamp": timestamp,
                    "device_id": device_id,
                    "principal": principal_id,
                    "action": action,
                    "resource": resource_id,
                    "decision": decision,
                    "reason": reason,
                }))
            }
            _ => None,
        })
        .collect();

    Json(decisions)
}

#[derive(Template)]
#[template(path = "decision_logs.html")]
struct DecisionLogsTemplate {
    logs: Vec<DecisionLogEntry>,
}

struct DecisionLogEntry {
    timestamp: String,
    device_id: String,
    principal: String,
    action: String,
    resource: String,
    decision: String,
    reason: String,
}

pub async fn view_decision_logs(State(state): State<AppState>) -> impl IntoResponse {
    let events = state.telemetry_events.lock().unwrap();
    let mut logs: Vec<DecisionLogEntry> = events
        .iter()
        .filter_map(|e| match e {
            TelemetryEvent::Decision { timestamp, device_id, principal_id, action, resource_id, decision, reason, .. } => {
                Some(DecisionLogEntry {
                    timestamp: timestamp.clone(),
                    device_id: device_id.clone(),
                    principal: principal_id.clone(),
                    action: action.clone(),
                    resource: resource_id.clone(),
                    decision: decision.clone(),
                    reason: reason.clone(),
                })
            }
            _ => None,
        })
        .collect();
    
    logs.reverse(); // Newest first

    let tpl = DecisionLogsTemplate { logs };
    Html(tpl.render().unwrap_or_else(|e| format!("Template render error: {}", e)))
}
