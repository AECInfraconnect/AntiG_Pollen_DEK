use arc_swap::ArcSwap;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dek_activation::snapshot::RuntimeSnapshot;
use dek_decision::{DecisionRequest, DecisionResponse};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

#[derive(Clone)]
struct ApiState {
    snapshot: Arc<ArcSwap<RuntimeSnapshot>>,
}

pub async fn start_sidecar_api(
    snapshot: Arc<ArcSwap<RuntimeSnapshot>>,
    port: u16,
) -> anyhow::Result<()> {
    let state = ApiState { snapshot };

    let app = Router::new()
        .route("/v1/healthz", get(healthz))
        .route("/v1/readyz", get(readyz))
        .route("/v1/capabilities", get(capabilities))
        .route("/v1/decision/check", post(check))
        .route("/v1/decision/batch-check", post(batch_check))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    info!("Starting Sidecar API on {}", addr);
    let listener = TcpListener::bind(&addr).await?;

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("Sidecar API server failed: {}", e);
        }
    });

    Ok(())
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, "READY")
}

async fn capabilities() -> impl IntoResponse {
    Json(serde_json::json!({
        "modes": ["sidecar", "mcp-http", "mcp-stdio"],
        "evaluators": ["openfga", "cedar", "wasm"],
        "forward_proxy": false
    }))
}

async fn check(
    State(state): State<ApiState>,
    Json(req): Json<DecisionRequest>,
) -> impl IntoResponse {
    let snap = state.snapshot.load();
    let val = serde_json::to_value(&req).unwrap_or(serde_json::json!({}));

    let start = std::time::Instant::now();
    let res =
        snap.router
            .authorize(val)
            .await
            .unwrap_or_else(|_| dek_policy_runtime::PolicyDecision {
                evaluator_id: "core_api".into(),
                evaluator_type: "router".into(),
                required: true,
                status: "error".into(),
                decision: "deny".into(),
                allow: false,
                reason: "Policy evaluation failed".into(),
                effects: serde_json::json!({}),
                obligations: vec![],
                metadata: serde_json::json!({}),
            });
    let latency = start.elapsed().as_millis() as u64;

    let response = DecisionResponse {
        decision_id: uuid::Uuid::new_v4().to_string(),
        allow: res.allow,
        reason_code: if res.allow {
            "OK".into()
        } else {
            "DENIED_BY_POLICY".into()
        },
        reason: res.reason,
        obligations: vec![],
        effects: res.effects,
        policy_bundle_id: "active".into(),
        policy_bundle_version: "v1".into(),
        evaluator_results: vec![],
        latency_ms: latency,
    };

    (StatusCode::OK, Json(response))
}

async fn batch_check(
    State(state): State<ApiState>,
    Json(reqs): Json<Vec<DecisionRequest>>,
) -> impl IntoResponse {
    let mut responses = Vec::with_capacity(reqs.len());
    // Simple serial evaluation for now. Could be parallelized.
    for req in reqs {
        let snap = state.snapshot.load();
        let val = serde_json::to_value(&req).unwrap_or(serde_json::json!({}));

        let start = std::time::Instant::now();
        let res = snap.router.authorize(val).await.unwrap_or_else(|_| {
            dek_policy_runtime::PolicyDecision {
                evaluator_id: "core_api".into(),
                evaluator_type: "router".into(),
                required: true,
                status: "error".into(),
                decision: "deny".into(),
                allow: false,
                reason: "Policy evaluation failed".into(),
                effects: serde_json::json!({}),
                obligations: vec![],
                metadata: serde_json::json!({}),
            }
        });
        let latency = start.elapsed().as_millis() as u64;

        responses.push(DecisionResponse {
            decision_id: uuid::Uuid::new_v4().to_string(),
            allow: res.allow,
            reason_code: if res.allow {
                "OK".into()
            } else {
                "DENIED_BY_POLICY".into()
            },
            reason: res.reason,
            obligations: vec![],
            effects: res.effects,
            policy_bundle_id: "active".into(),
            policy_bundle_version: "v1".into(),
            evaluator_results: vec![],
            latency_ms: latency,
        });
    }

    (StatusCode::OK, Json(responses))
}
