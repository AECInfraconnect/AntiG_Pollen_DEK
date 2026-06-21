// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::state::{AppState, ApprovalRequest, AuditLog};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant_id/approvals", post(submit_approval))
        .route("/mock/admin/approvals", get(admin_approvals_view))
        .route(
            "/mock/admin/approvals/:ref_id/:action",
            post(admin_approve_deny),
        )
}

#[derive(serde::Deserialize)]
pub struct SubmitApprovalReq {
    pub device_id: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
}

pub async fn submit_approval(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(req): Json<SubmitApprovalReq>,
) -> impl IntoResponse {
    let ref_id = crate::state::rand_hex(8);
    let approval = ApprovalRequest {
        ref_id: ref_id.clone(),
        tenant_id,
        device_id: req.device_id,
        principal: req.principal,
        action: req.action,
        resource: req.resource,
        status: "PENDING".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    let mut approvals = state.approvals.lock().unwrap();
    approvals.insert(ref_id.clone(), approval);

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "pending",
            "ref_id": ref_id,
            "message": "Approval request submitted. Awaiting human intervention."
        })),
    )
}

#[derive(Template)]
#[template(path = "approvals.html")]
struct ApprovalsTemplate {
    approvals: Vec<ApprovalRequest>,
}

pub async fn admin_approvals_view(State(state): State<AppState>) -> impl IntoResponse {
    let approvals_guard = state.approvals.lock().unwrap();
    let mut approvals: Vec<ApprovalRequest> = approvals_guard.values().cloned().collect();
    approvals.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // newest first

    let tpl = ApprovalsTemplate { approvals };
    Html(
        tpl.render()
            .unwrap_or_else(|e| format!("Template render error: {}", e)),
    )
}

pub async fn admin_approve_deny(
    State(state): State<AppState>,
    Path((ref_id, action)): Path<(String, String)>,
) -> impl IntoResponse {
    if action != "approve" && action != "deny" {
        return Redirect::to("/mock/admin/approvals");
    }

    let mut approvals = state.approvals.lock().unwrap();
    if let Some(approval) = approvals.get_mut(&ref_id) {
        approval.status = if action == "approve" {
            "APPROVED".to_string()
        } else {
            "DENIED".to_string()
        };

        state.audit_logs.lock().unwrap().push(AuditLog {
            timestamp: Utc::now().to_rfc3339(),
            actor: "admin".to_string(),
            action: format!("APPROVAL_{}", action.to_uppercase()),
            details: format!("{} request {} for {}", action, ref_id, approval.principal),
        });

        // Bump revision so DEK syncs again
        if action == "approve" {
            use std::sync::atomic::Ordering;
            state.revision.fetch_add(1, Ordering::SeqCst);
        }
    }

    Redirect::to("/mock/admin/approvals")
}
