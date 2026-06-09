use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use dek_control_plane_api::registry::*;
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        // Agents
        .route(
            "/v1/tenants/:tenant_id/registry/agents",
            get(list_agents).post(create_agent),
        )
        .route(
            "/v1/tenants/:tenant_id/registry/agents/:agent_id",
            get(get_agent).patch(patch_agent).delete(delete_agent),
        )
        // MCP Servers
        .route(
            "/v1/tenants/:tenant_id/registry/mcp-servers",
            get(list_mcp_servers).post(create_mcp_server),
        )
        .route(
            "/v1/tenants/:tenant_id/registry/mcp-servers/:server_id",
            get(get_mcp_server).patch(patch_mcp_server),
        )
        // Tools
        .route(
            "/v1/tenants/:tenant_id/registry/tools",
            get(list_tools).post(create_tool),
        )
        .route(
            "/v1/tenants/:tenant_id/registry/tools/:tool_id",
            get(get_tool).patch(patch_tool),
        )
        // Resources
        .route(
            "/v1/tenants/:tenant_id/registry/resources",
            get(list_resources).post(create_resource),
        )
        .route(
            "/v1/tenants/:tenant_id/registry/resources/:resource_id",
            get(get_resource).patch(patch_resource),
        )
        // Entities
        .route(
            "/v1/tenants/:tenant_id/registry/entities",
            get(list_entities).post(create_entity),
        )
        .route(
            "/v1/tenants/:tenant_id/registry/entities/:entity_id",
            get(get_entity).patch(patch_entity),
        )
        // Relationships
        .route(
            "/v1/tenants/:tenant_id/registry/relationships",
            get(list_relationships).post(create_relationship),
        )
        .route(
            "/v1/tenants/:tenant_id/registry/relationships/:relationship_id",
            delete(delete_relationship),
        )
}

// -----------------------------------------------------------------------------
// Agents
// -----------------------------------------------------------------------------
async fn list_agents(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.list_agents(&tenant_id).await {
        Ok(agents) => (StatusCode::OK, Json(json!(agents))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_agent(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<AiAgent>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.registry_store.upsert_agent(payload.clone()).await {
        Ok(agent) => (StatusCode::CREATED, Json(json!(agent))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_agent(
    Path((tenant_id, agent_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.get_agent(&tenant_id, &agent_id).await {
        Ok(Some(agent)) => (StatusCode::OK, Json(json!(agent))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn patch_agent(
    Path((tenant_id, _agent_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(mut payload): Json<AiAgent>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.registry_store.upsert_agent(payload.clone()).await {
        Ok(agent) => (StatusCode::OK, Json(json!(agent))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn delete_agent() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

// -----------------------------------------------------------------------------
// MCP Servers
// -----------------------------------------------------------------------------
async fn list_mcp_servers(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.list_mcp_servers(&tenant_id).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_mcp_server(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<McpServer>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state
        .registry_store
        .upsert_mcp_server(payload.clone())
        .await
    {
        Ok(item) => (StatusCode::CREATED, Json(json!(item))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_mcp_server() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn patch_mcp_server() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

// -----------------------------------------------------------------------------
// Tools
// -----------------------------------------------------------------------------
async fn list_tools(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.list_tools(&tenant_id).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_tool(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<Tool>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.registry_store.upsert_tool(payload.clone()).await {
        Ok(item) => (StatusCode::CREATED, Json(json!(item))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_tool() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn patch_tool() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

// -----------------------------------------------------------------------------
// Resources
// -----------------------------------------------------------------------------
async fn list_resources(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.list_resources(&tenant_id).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_resource(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<Resource>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.registry_store.upsert_resource(payload.clone()).await {
        Ok(item) => (StatusCode::CREATED, Json(json!(item))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_resource() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn patch_resource() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

// -----------------------------------------------------------------------------
// Entities
// -----------------------------------------------------------------------------
async fn list_entities(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.list_entities(&tenant_id).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_entity(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<Entity>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.registry_store.upsert_entity(payload.clone()).await {
        Ok(item) => (StatusCode::CREATED, Json(json!(item))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_entity() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn patch_entity() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

// -----------------------------------------------------------------------------
// Relationships
// -----------------------------------------------------------------------------
async fn list_relationships(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.registry_store.list_relationships(&tenant_id).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_relationship(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<Relationship>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state
        .registry_store
        .upsert_relationship(payload.clone())
        .await
    {
        Ok(item) => (StatusCode::CREATED, Json(json!(item))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn delete_relationship() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}
