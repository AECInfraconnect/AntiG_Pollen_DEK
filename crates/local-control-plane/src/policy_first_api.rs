// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use dek_deployment_planner::SuggestedPolicy;
use dek_domain_schema::{
    control_level::ControlLevel,
    deployment_session::{
        DeploymentScope, DeploymentSession, DeploymentSessionStatus, LocalizedText,
    },
    feasibility::{PolicyFeasibilityRequest, PolicyFeasibilityResult, PolicyFeasibilityStatus},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::deployment_orchestrator::{DeploymentOrchestrator, StoreEventSink};
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
        .route("/v1/deployment-sessions/:id/retry", post(retry_deployment))
        .route(
            "/v1/deployment-sessions/:id/rollback",
            post(rollback_deployment),
        )
}

async fn scan(State(_st): State<AppState>) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // In a real app, this triggers the background scanner
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "scanned"})),
    ))
}

fn get_mock_snapshot() -> dek_capability_registry::LocalCapabilitySnapshot {
    dek_capability_registry::LocalCapabilitySnapshot {
        snapshot_id: "snap-123".into(),
        device_id: "local".into(),
        os: dek_capability_registry::OsInfo {
            r#type: "windows".into(),
            version: "11".into(),
            arch: "x86_64".into(),
        },
        agents: vec![],
        methods: vec![],
        generated_at: chrono::Utc::now(),
    }
}

async fn get_latest_snapshot(
    State(_st): State<AppState>,
) -> ApiResult<(
    StatusCode,
    Json<dek_capability_registry::LocalCapabilitySnapshot>,
)> {
    Ok((StatusCode::OK, Json(get_mock_snapshot())))
}

async fn get_policy_suggestions(
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<Vec<SuggestedPolicy>>)> {
    let suggestions = vec![
        SuggestedPolicy {
            suggestion_id: "sugg-1".into(),
            policy_template_id: "approve_risky_tool_calls".into(),
            display_name: LocalizedText {
                en: "Approve risky tool calls".into(),
                th: "อนุมัติการเรียกใช้ Tool ที่มีความเสี่ยง".into(),
            },
            description: LocalizedText {
                en: "Agent exposes MCP tools, so POLLEK can review tool calls before execution."
                    .into(),
                th: "Agent นี้มี MCP tools ระบบจึงสามารถให้ตรวจและอนุมัติ tool call ก่อนทำงานได้".into(),
            },
            target_agent_ids: vec!["claude_desktop".into()],
            recommended_control_level: ControlLevel::Approval,
            feasibility: PolicyFeasibilityStatus::CanEnforceAfterApproval,
            confidence: 0.95,
            reason_codes: vec!["mcp_stdio_detected".into()],
            setup_required: vec![],
        },
        SuggestedPolicy {
            suggestion_id: "sugg-2".into(),
            policy_template_id: "limit_token_or_cost_usage".into(),
            display_name: LocalizedText {
                en: "Limit token or cost usage".into(),
                th: "จำกัดปริมาณ Token หรือค่าใช้จ่าย".into(),
            },
            description: LocalizedText {
                en: "Local API traffic can be measured and rate-limited.".into(),
                th: "ระบบสามารถวัดและจำกัดการใช้งานผ่าน local API ได้".into(),
            },
            target_agent_ids: vec!["local_ollama".into()],
            recommended_control_level: ControlLevel::Warn,
            feasibility: PolicyFeasibilityStatus::CanEnforceNow,
            confidence: 0.9,
            reason_codes: vec!["local_api_detected".into()],
            setup_required: vec![],
        },
    ];
    Ok((StatusCode::OK, Json(suggestions)))
}

async fn evaluate_feasibility(
    State(_st): State<AppState>,
    Json(req): Json<PolicyFeasibilityRequest>,
) -> ApiResult<(StatusCode, Json<Vec<PolicyFeasibilityResult>>)> {
    let snapshot = get_mock_snapshot();
    let result = dek_deployment_planner::evaluate_policy_feasibility(req, &snapshot);
    Ok((StatusCode::OK, Json(result)))
}

async fn create_deployment_session(
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<DeploymentSession>)> {
    let mut session = DeploymentSession {
        deployment_id: Uuid::new_v4().to_string(),
        policy_id: "policy-tmp".into(),
        policy_version: "1.0".into(),
        requested_control_level: ControlLevel::Enforce,
        target_scope: DeploymentScope::Device {
            device_id: "local".into(),
        },
        status: DeploymentSessionStatus::ScanStarted,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: "local_admin".into(),
    };

    let sink = Arc::new(StoreEventSink::new());
    let orchestrator = DeploymentOrchestrator::new(sink);

    // Run the state machine progression
    let _ = orchestrator
        .transition(&mut session, DeploymentSessionStatus::ScanCompleted)
        .await;
    let _ = orchestrator
        .transition(
            &mut session,
            DeploymentSessionStatus::CapabilitySnapshotCreated,
        )
        .await;
    let _ = orchestrator
        .transition(
            &mut session,
            DeploymentSessionStatus::PolicyFeasibilityEvaluated,
        )
        .await;
    let _ = orchestrator
        .transition(&mut session, DeploymentSessionStatus::DeploymentPlanCreated)
        .await;

    Ok((StatusCode::OK, Json(session)))
}

async fn approve_action(
    Path((_session_id, _action_id)): Path<(String, String)>,
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "approved"})),
    ))
}

async fn retry_deployment(
    Path(session_id): Path<String>,
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "retrying",
            "deployment_id": session_id
        })),
    ))
}

async fn rollback_deployment(
    Path(session_id): Path<String>,
    State(_st): State<AppState>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "rolled_back",
            "deployment_id": session_id
        })),
    ))
}
