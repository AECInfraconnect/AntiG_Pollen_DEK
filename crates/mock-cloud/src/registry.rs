// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dek_domain_schema::*;
use serde_json::json;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
};

pub fn router() -> Router<AppState> {
    let internal_routes = Router::new()
        // Tenants
        .route("/tenants", get(list_tenants).post(create_tenant))
        .route("/tenants/:id", get(get_tenant).patch(patch_tenant))
        // Principals
        .route("/principals", get(list_principals).post(create_principal))
        .route("/principals/:id", get(get_principal).patch(patch_principal))
        // Devices
        .route("/devices", get(list_devices).post(create_device))
        .route("/devices/:id", get(get_device).patch(patch_device))
        .route("/devices/:id/capabilities", post(register_capabilities))
        // Agents
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).patch(patch_agent))
        // MCP Servers
        .route("/mcp_servers", get(list_mcp_servers).post(create_mcp_server))
        .route("/mcp_servers/:id", get(get_mcp_server).patch(patch_mcp_server))
        // Tools
        .route("/tools", get(list_tools).post(create_tool))
        .route("/tools/:id", get(get_tool).patch(patch_tool))
        // Resources
        .route("/resources", get(list_resources).post(create_resource))
        .route("/resources/:id", get(get_resource).patch(patch_resource))
        // Relationships
        .route("/relationships", get(list_relationships).post(create_relationship))
        // Policies
        .route("/policies", get(list_policies).post(create_policy))
        .route("/policies/:id", get(get_policy).patch(patch_policy))
        // PEP Deployments
        .route("/pep_deployments", get(list_pep_deployments).post(create_pep_deployment))
        .route("/pep_deployments/:id", get(get_pep_deployment).patch(patch_pep_deployment));

    Router::new()
        .nest("/v1/registry", internal_routes.clone())
        .nest("/v1/tenants/:tenant_id/registry", internal_routes)
        .route_layer(middleware::from_fn(mock_rbac_middleware))
}

async fn mock_rbac_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Only enforce on state-changing methods
    let method = req.method().clone();
    if method == axum::http::Method::POST || method == axum::http::Method::PATCH || method == axum::http::Method::DELETE || method == axum::http::Method::PUT {
        if let Some(role) = req.headers().get("x-mock-role") {
            if let Ok(role_str) = role.to_str() {
                if role_str != "admin" && role_str != "tenant-admin" {
                    return Err(StatusCode::FORBIDDEN);
                }
            } else {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        // If x-mock-role is missing, we allow it to maintain backward compatibility,
        // or we could enforce it. The spec says "mock via x-mock-role", so we check if present.
    }
    Ok(next.run(req).await)
}

// -----------------------------------------------------------------------------
// Tenants
// -----------------------------------------------------------------------------
async fn list_tenants(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<Tenant> = reg.tenants.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_tenant(
    State(state): State<AppState>,
    Json(payload): Json<Tenant>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.tenants
        .insert(payload.tenant_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_tenant(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.tenants.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_tenant(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<Tenant>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.tenants.contains_key(&id) {
        reg.tenants.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// Principals
// -----------------------------------------------------------------------------
async fn list_principals(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<Principal> = reg.principals.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_principal(
    State(state): State<AppState>,
    Json(payload): Json<Principal>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.principals
        .insert(payload.principal_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_principal(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.principals.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_principal(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<Principal>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.principals.contains_key(&id) {
        reg.principals.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// Devices
// -----------------------------------------------------------------------------
async fn list_devices(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<DekDevice> = reg.devices.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_device(
    State(state): State<AppState>,
    Json(payload): Json<DekDevice>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.devices
        .insert(payload.device_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_device(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.devices.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_device(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<DekDevice>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.devices.contains_key(&id) {
        reg.devices.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn register_capabilities(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<dek_domain_schema::DeviceRegistrationRequest>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut devices = state.devices.lock().unwrap();
    if let Some(dev) = devices.get_mut(&id) {
        dev.capabilities = payload.capabilities.clone();
        (
            StatusCode::OK,
            Json(json!({"status": "updated", "capabilities": dev.capabilities})),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "device not found"})),
        )
    }
}

// -----------------------------------------------------------------------------
// Agents
// -----------------------------------------------------------------------------
async fn list_agents(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<AiAgent> = reg.agents.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_agent(
    State(state): State<AppState>,
    Json(payload): Json<AiAgent>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.agents.insert(payload.agent_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_agent(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.agents.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_agent(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<AiAgent>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.agents.contains_key(&id) {
        reg.agents.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// MCP Servers
// -----------------------------------------------------------------------------
async fn list_mcp_servers(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<McpServer> = reg.mcp_servers.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_mcp_server(
    State(state): State<AppState>,
    Json(payload): Json<McpServer>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.mcp_servers
        .insert(payload.server_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_mcp_server(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.mcp_servers.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_mcp_server(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<McpServer>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.mcp_servers.contains_key(&id) {
        reg.mcp_servers.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// Tools
// -----------------------------------------------------------------------------
async fn list_tools(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<Tool> = reg.tools.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_tool(
    State(state): State<AppState>,
    Json(payload): Json<Tool>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.tools.insert(payload.tool_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_tool(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.tools.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_tool(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<Tool>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.tools.contains_key(&id) {
        reg.tools.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// Resources
// -----------------------------------------------------------------------------
async fn list_resources(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<Resource> = reg.resources.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_resource(
    State(state): State<AppState>,
    Json(payload): Json<Resource>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.resources
        .insert(payload.resource_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_resource(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.resources.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_resource(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<Resource>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.resources.contains_key(&id) {
        reg.resources.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// Relationships
// -----------------------------------------------------------------------------
async fn list_relationships(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<Relationship> = reg.relationships.clone();
    (StatusCode::OK, Json(items))
}

async fn create_relationship(
    State(state): State<AppState>,
    Json(payload): Json<Relationship>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.relationships.push(payload.clone());
    (StatusCode::CREATED, Json(payload))
}

// -----------------------------------------------------------------------------
// Policies
// -----------------------------------------------------------------------------
async fn list_policies(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<Policy> = reg.policies.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_policy(
    State(state): State<AppState>,
    Json(payload): Json<Policy>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.policies
        .insert(payload.policy_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_policy(Path(params): Path<std::collections::HashMap<String, String>>, State(state): State<AppState>) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.policies.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_policy(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<Policy>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.policies.contains_key(&id) {
        reg.policies.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// -----------------------------------------------------------------------------
// PEP Deployments
// -----------------------------------------------------------------------------
async fn list_pep_deployments(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.lock().unwrap();
    let items: Vec<PepDeployment> = reg.pep_deployments.values().cloned().collect();
    (StatusCode::OK, Json(items))
}

async fn create_pep_deployment(
    State(state): State<AppState>,
    Json(payload): Json<PepDeployment>,
) -> impl IntoResponse {
    let mut reg = state.registry.lock().unwrap();
    reg.pep_deployments
        .insert(payload.pep_deployment_id.clone(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_pep_deployment(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let reg = state.registry.lock().unwrap();
    if let Some(item) = reg.pep_deployments.get(&id) {
        (StatusCode::OK, Json(json!(item)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_pep_deployment(
    Path(params): Path<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
    Json(payload): Json<PepDeployment>,
) -> impl IntoResponse {
    let id = params.get("id").cloned().unwrap_or_default();
    let mut reg = state.registry.lock().unwrap();
    if reg.pep_deployments.contains_key(&id) {
        reg.pep_deployments.insert(id.clone(), payload.clone());
        (StatusCode::OK, Json(json!(payload)))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}
