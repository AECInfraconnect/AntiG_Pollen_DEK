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
