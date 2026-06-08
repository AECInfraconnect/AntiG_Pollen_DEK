use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use crate::state::{AppState, DeviceStatus};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants", get(list_tenants).post(create_tenant))
        .route("/v1/tenants/:tenant_id", get(get_tenant).patch(patch_tenant))
        .route("/v1/tenants/:tenant_id/devices", get(list_devices))
        .route("/v1/tenants/:tenant_id/devices/:device_id", get(get_device_info).patch(patch_device))
        .route("/v1/tenants/:tenant_id/agents", get(list_agents).post(create_agent))
        .route("/v1/tenants/:tenant_id/agents/:agent_id", get(get_agent).patch(patch_agent))
        .route("/v1/tenants/:tenant_id/entities", get(list_entities).post(create_entity))
        .route("/v1/tenants/:tenant_id/entities/:entity_id", get(get_entity))
        .route("/v1/tenants/:tenant_id/resources", get(list_resources).post(create_resource))
        .route("/v1/tenants/:tenant_id/resources/:resource_id", get(get_resource))
        .route("/v1/tenants/:tenant_id/relationships", get(list_relationships).post(create_relationship))
}

// Tenenat
async fn list_tenants(State(state): State<AppState>) -> impl IntoResponse {
    let tenants = state.tenants.lock().unwrap();
    let res: Vec<Value> = tenants.values().cloned().collect();
    (StatusCode::OK, Json(res))
}

async fn create_tenant(State(state): State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
    let mut tenants = state.tenants.lock().unwrap();
    let id = payload.get("tenant_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    tenants.insert(id.to_string(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_tenant(Path(tenant_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let tenants = state.tenants.lock().unwrap();
    if let Some(t) = tenants.get(&tenant_id) {
        (StatusCode::OK, Json(t.clone()))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

async fn patch_tenant(Path(tenant_id): Path<String>, State(state): State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
    let mut tenants = state.tenants.lock().unwrap();
    if let Some(t) = tenants.get_mut(&tenant_id) {
        if let (Value::Object(mut base), Value::Object(new_data)) = (t.clone(), payload) {
            base.extend(new_data);
            *t = Value::Object(base);
        }
        (StatusCode::OK, Json(t.clone()))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
    }
}

// Devices
async fn list_devices(Path(tenant_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let devices = state.devices.lock().unwrap();
    let tenant_devices: Vec<DeviceStatus> = devices.values().filter(|d| d.tenant_id == tenant_id).cloned().collect();
    (StatusCode::OK, Json(tenant_devices))
}

async fn get_device_info(Path((tenant_id, device_id)): Path<(String, String)>, State(state): State<AppState>) -> impl IntoResponse {
    let devices = state.devices.lock().unwrap();
    if let Some(dev) = devices.get(&device_id) {
        if dev.tenant_id == tenant_id {
            return (StatusCode::OK, Json(json!(dev.clone())));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" })))
}

#[derive(serde::Deserialize)]
struct PatchDeviceReq {
    profile: Option<String>,
}

async fn patch_device(Path((tenant_id, device_id)): Path<(String, String)>, State(state): State<AppState>, Json(req): Json<PatchDeviceReq>) -> impl IntoResponse {
    let mut devices = state.devices.lock().unwrap();
    if let Some(dev) = devices.get_mut(&device_id) {
        if dev.tenant_id == tenant_id {
            if let Some(p) = req.profile {
                dev.profile = p;
            }
            return (StatusCode::OK, Json(json!(dev.clone())));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" })))
}

// Agents
async fn list_agents(Path(tenant_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let agents = state.agents.lock().unwrap();
    let res: Vec<Value> = agents.values().filter(|a| a.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id)).cloned().collect();
    (StatusCode::OK, Json(res))
}

async fn create_agent(Path(tenant_id): Path<String>, State(state): State<AppState>, Json(mut payload): Json<Value>) -> impl IntoResponse {
    let mut agents = state.agents.lock().unwrap();
    payload["tenant_id"] = json!(tenant_id);
    let id = payload.get("agent_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    agents.insert(id.to_string(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_agent(Path((tenant_id, agent_id)): Path<(String, String)>, State(state): State<AppState>) -> impl IntoResponse {
    let agents = state.agents.lock().unwrap();
    if let Some(a) = agents.get(&agent_id) {
        if a.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id) {
            return (StatusCode::OK, Json(a.clone()));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

async fn patch_agent(Path((tenant_id, agent_id)): Path<(String, String)>, State(state): State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
    let mut agents = state.agents.lock().unwrap();
    if let Some(a) = agents.get_mut(&agent_id) {
        if a.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id) {
            if let (Value::Object(mut base), Value::Object(new_data)) = (a.clone(), payload) {
                base.extend(new_data);
                *a = Value::Object(base);
            }
            return (StatusCode::OK, Json(a.clone()));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

// Entities
async fn list_entities(Path(tenant_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let entities = state.entities.lock().unwrap();
    let res: Vec<Value> = entities.values().filter(|e| e.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id)).cloned().collect();
    (StatusCode::OK, Json(res))
}

async fn create_entity(Path(tenant_id): Path<String>, State(state): State<AppState>, Json(mut payload): Json<Value>) -> impl IntoResponse {
    let mut entities = state.entities.lock().unwrap();
    payload["tenant_id"] = json!(tenant_id);
    let id = payload.get("entity_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    entities.insert(id.to_string(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_entity(Path((tenant_id, entity_id)): Path<(String, String)>, State(state): State<AppState>) -> impl IntoResponse {
    let entities = state.entities.lock().unwrap();
    if let Some(e) = entities.get(&entity_id) {
        if e.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id) {
            return (StatusCode::OK, Json(e.clone()));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

// Resources
async fn list_resources(Path(tenant_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let resources = state.resources.lock().unwrap();
    let res: Vec<Value> = resources.values().filter(|r| r.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id)).cloned().collect();
    (StatusCode::OK, Json(res))
}

async fn create_resource(Path(tenant_id): Path<String>, State(state): State<AppState>, Json(mut payload): Json<Value>) -> impl IntoResponse {
    let mut resources = state.resources.lock().unwrap();
    payload["tenant_id"] = json!(tenant_id);
    let id = payload.get("resource_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    resources.insert(id.to_string(), payload.clone());
    (StatusCode::CREATED, Json(payload))
}

async fn get_resource(Path((tenant_id, resource_id)): Path<(String, String)>, State(state): State<AppState>) -> impl IntoResponse {
    let resources = state.resources.lock().unwrap();
    if let Some(r) = resources.get(&resource_id) {
        if r.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id) {
            return (StatusCode::OK, Json(r.clone()));
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

// Relationships
async fn list_relationships(Path(tenant_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let relationships = state.relationships.lock().unwrap();
    let res: Vec<Value> = relationships.iter().filter(|r| r.get("tenant_id").and_then(|v| v.as_str()) == Some(&tenant_id)).cloned().collect();
    (StatusCode::OK, Json(res))
}

async fn create_relationship(Path(tenant_id): Path<String>, State(state): State<AppState>, Json(mut payload): Json<Value>) -> impl IntoResponse {
    let mut relationships = state.relationships.lock().unwrap();
    payload["tenant_id"] = json!(tenant_id);
    relationships.push(payload.clone());
    (StatusCode::CREATED, Json(payload))
}
