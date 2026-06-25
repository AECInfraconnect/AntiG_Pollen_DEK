// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dek_enforcement_api::planner::{
    assess_feasibility, negotiate, ControlDomain, ControlLevel, ControlMethodCap,
    DomainFeasibility, LocalCapabilitySnapshot, MethodStatus, Policy, PolicyFeasibilityResult,
};
use serde::{Deserialize, Serialize};

use crate::error::ApiResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/host/capabilities", get(get_host_capabilities))
        .route("/v1/discovery/scan", post(scan_agents))
        .route("/v1/discovery/scan/:job", get(get_scan_result))
        .route("/v1/policy/suggestions", post(get_policy_suggestions))
        .route("/v1/policy/feasibility", post(evaluate_feasibility))
        .route("/v1/deploy/session", post(create_deploy_session))
        .route(
            "/v1/deploy/session/:id/confirm",
            post(confirm_deploy_session),
        )
        .route("/v1/deploy/session/:id/apply", post(apply_deploy_session))
}

fn get_current_snapshot() -> LocalCapabilitySnapshot {
    let method_id = match std::env::consts::OS {
        "windows" => "windows_wfp_um",
        "macos" => "macos_netext",
        _ => "linux_ebpf",
    };
    LocalCapabilitySnapshot {
        control_methods: vec![ControlMethodCap {
            id: method_id.into(),
            domains: vec![ControlDomain::Network],
            max_level: ControlLevel::Enforce,
            status: MethodStatus::Available,
        }],
    }
}

async fn get_host_capabilities() -> ApiResult<(StatusCode, Json<LocalCapabilitySnapshot>)> {
    Ok((StatusCode::OK, Json(get_current_snapshot())))
}

#[derive(Serialize)]
struct ScanResponse {
    job_id: String,
}

async fn scan_agents() -> ApiResult<(StatusCode, Json<ScanResponse>)> {
    Ok((
        StatusCode::OK,
        Json(ScanResponse {
            job_id: "job-123".into(),
        }),
    ))
}

async fn get_scan_result(
    Path(job_id): Path<String>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"job_id": job_id, "status": "completed"})),
    ))
}

#[derive(Deserialize)]
struct SuggestionsRequest {
    agents: Vec<String>,
}

#[derive(Serialize)]
struct PolicySuggestion {
    id: String,
    title_th: String,
    title_en: String,
    domains: Vec<String>,
    recommended_level: String,
}

async fn get_policy_suggestions(
    Json(_req): Json<SuggestionsRequest>,
) -> ApiResult<(StatusCode, Json<Vec<PolicySuggestion>>)> {
    Ok((
        StatusCode::OK,
        Json(vec![PolicySuggestion {
            id: "sugg-1".into(),
            title_th: "จำกัดการเข้าถึงเครือข่าย".into(),
            title_en: "Restrict Network Access".into(),
            domains: vec!["network".into()],
            recommended_level: "enforce".into(),
        }]),
    ))
}

#[derive(Deserialize)]
struct FeasibilityRequest {
    policy: serde_json::Value,
    requested_level: ControlLevel,
}

async fn evaluate_feasibility(
    Json(req): Json<FeasibilityRequest>,
) -> ApiResult<(StatusCode, Json<PolicyFeasibilityResult>)> {
    let pol = Policy {
        id: "mock_pol".into(),
        requested_level: req.requested_level,
    };
    let snap = get_current_snapshot();
    let res = assess_feasibility(&pol, &snap);
    Ok((StatusCode::OK, Json(res)))
}

#[derive(Deserialize)]
struct CreateDeployRequest {
    policy: serde_json::Value,
    agents: Vec<String>,
    requested_level: ControlLevel,
}

#[derive(Serialize)]
struct DeploySession {
    id: String,
    feasibility: PolicyFeasibilityResult,
    status: String,
}

async fn create_deploy_session(
    Json(req): Json<CreateDeployRequest>,
) -> ApiResult<(StatusCode, Json<DeploySession>)> {
    let pol = Policy {
        id: "mock_pol".into(),
        requested_level: req.requested_level,
    };
    let snap = get_current_snapshot();
    let res = assess_feasibility(&pol, &snap);
    Ok((
        StatusCode::OK,
        Json(DeploySession {
            id: "sess-123".into(),
            feasibility: res,
            status: "pending".into(),
        }),
    ))
}

async fn confirm_deploy_session(
    Path(_id): Path<String>,
) -> ApiResult<(
    StatusCode,
    Json<dek_enforcement_api::planner::ControlMethodPlan>,
)> {
    let pol = Policy {
        id: "mock_pol".into(),
        requested_level: ControlLevel::Enforce,
    };
    let snap = get_current_snapshot();
    let res = assess_feasibility(&pol, &snap);
    let plan = negotiate(&res);
    Ok((StatusCode::OK, Json(plan)))
}

#[derive(Serialize)]
struct DeployReport {
    status: String,
}

async fn apply_deploy_session(
    Path(_id): Path<String>,
) -> ApiResult<(StatusCode, Json<DeployReport>)> {
    Ok((
        StatusCode::OK,
        Json(DeployReport {
            status: "success".into(),
        }),
    ))
}
