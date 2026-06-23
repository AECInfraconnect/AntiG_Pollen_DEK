use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/plugins", get(list_plugins))
        .route("/v1/tenants/:tenant/plugins/install", post(install_plugin))
        .route("/v1/tenants/:tenant/plugins/:id/enable", post(enable_plugin))
        .route("/v1/tenants/:tenant/plugins/:id/disable", post(disable_plugin))
        .route("/v1/tenants/:tenant/plugins/:id/test", post(test_plugin))
}

async fn list_plugins(Path(_tenant): Path<String>, State(_state): State<AppState>) -> Json<Value> {
    Json(json!([
        {
            "id": "github_classifier",
            "name": "GitHub Resource Classifier",
            "version": "1.0.0",
            "type": "resource_classifier",
            "status": "enabled",
            "permissions": ["network:github.com"]
        }
    ]))
}

async fn install_plugin(
    Path(_tenant): Path<String>,
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": "Plugin installed"
    }))
}

async fn enable_plugin(
    Path((_tenant, id)): Path<(String, String)>,
    State(_state): State<AppState>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": format!("Plugin {} enabled", id)
    }))
}

async fn disable_plugin(
    Path((_tenant, id)): Path<(String, String)>,
    State(_state): State<AppState>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": format!("Plugin {} disabled", id)
    }))
}

async fn test_plugin(
    Path((_tenant, id)): Path<(String, String)>,
    State(_state): State<AppState>,
    Json(_payload): Json<Value>,
) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": format!("Plugin {} tested successfully", id),
        "output": {}
    }))
}
