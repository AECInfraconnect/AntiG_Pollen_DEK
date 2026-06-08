use crate::state::AppState;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use dek_domain_schema::*;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminDashboardTemplate {
    pub tenants: HashMap<String, Tenant>,
    pub principals: HashMap<String, Principal>,
    pub devices: HashMap<String, DekDevice>,
    pub agents: HashMap<String, AiAgent>,
    pub mcp_servers: HashMap<String, McpServer>,
    pub tools: HashMap<String, Tool>,
    pub resources: HashMap<String, Resource>,
    pub relationships: Vec<Relationship>,
    pub policies: HashMap<String, Policy>,
    pub pep_deployments: HashMap<String, PepDeployment>,
}

pub async fn admin_dashboard(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();

    let template = AdminDashboardTemplate {
        tenants: reg.tenants.clone(),
        principals: reg.principals.clone(),
        devices: reg.devices.clone(),
        agents: reg.agents.clone(),
        mcp_servers: reg.mcp_servers.clone(),
        tools: reg.tools.clone(),
        resources: reg.resources.clone(),
        relationships: reg.relationships.clone(),
        policies: reg.policies.clone(),
        pep_deployments: reg.pep_deployments.clone(),
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Template rendering failed",
        )
            .into_response(),
    }
}

pub async fn admin_bundle_poison(
    axum::extract::Path(bundle_id): axum::extract::Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    // In a real mock, we might mark this specific bundle ID as poisoned
    // For now we'll just log an audit event that it was poisoned.
    state.audit_logs.lock().unwrap().push(crate::state::AuditLog {
        timestamp: chrono::Utc::now().to_rfc3339(),
        actor: "test-harness".to_string(),
        action: "POISON_BUNDLE".to_string(),
        details: format!("Poisoned bundle {}", bundle_id),
    });
    
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({"status": "poisoned", "bundle_id": bundle_id})),
    )
}

