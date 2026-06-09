use axum::{
    extract::{State, Json},
    response::IntoResponse,
    routing::post,
    Router,
};
use dek_plugin_sdk::{EvalRequest, PolicyDecision, DecisionEffect, DecisionStatus};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/sidecar/eval", post(eval_policy))
}

async fn eval_policy(
    State(_state): State<AppState>,
    Json(_req): Json<EvalRequest>,
) -> impl IntoResponse {
    // Basic mock sidecar evaluation endpoint
    // In a real implementation, this would route to the local policy engine
    let decision = PolicyDecision {
        evaluator_id: "sidecar_mock".into(),
        evaluator_type: "sidecar".into(),
        required: true,
        status: DecisionStatus::Success,
        decision: DecisionEffect::Allow,
        reason: "Allowed by mock sidecar".into(),
        obligations: vec![],
        effects: serde_json::json!({}),
        metadata: serde_json::json!({}),
    };
    
    axum::Json(decision)
}
