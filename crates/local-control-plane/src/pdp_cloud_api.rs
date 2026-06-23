// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::state::AppState;
use axum::{
    extract::Path,
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudPdpProfile {
    pub tenant_id: Option<String>,
    pub device_id: Option<String>,
    pub pdp_endpoint: Option<String>,
    pub contract_version: Option<String>,
    pub auth_method: Option<String>,
    pub status: String,
    pub manual_override_enabled: bool,
    pub health: Option<serde_json::Value>,
}

impl Default for CloudPdpProfile {
    fn default() -> Self {
        Self {
            tenant_id: None,
            device_id: None,
            pdp_endpoint: None,
            contract_version: None,
            auth_method: None,
            status: "disconnected".to_string(),
            manual_override_enabled: false,
            health: None,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant/pdp/cloud",
            get(get_cloud_profile)
                .patch(update_cloud_profile)
                .delete(disconnect_cloud_profile),
        )
        .route("/v1/tenants/:tenant/pdp/cloud/login", post(cloud_login))
        .route(
            "/v1/tenants/:tenant/pdp/cloud/discover",
            post(cloud_discover),
        )
        .route("/v1/tenants/:tenant/pdp/cloud/probe", post(cloud_probe))
}

async fn get_cloud_profile(
    Path(_tenant): Path<String>,
    State(_st): State<AppState>,
) -> Json<CloudPdpProfile> {
    // Stub: In reality this would read from SQLite via st.pdp_store
    Json(CloudPdpProfile::default())
}

async fn update_cloud_profile(
    Path(_tenant): Path<String>,
    State(_st): State<AppState>,
    Json(payload): Json<CloudPdpProfile>,
) -> Json<CloudPdpProfile> {
    // Stub: Save payload
    Json(payload)
}

async fn disconnect_cloud_profile(
    Path(_tenant): Path<String>,
    State(_st): State<AppState>,
) -> Json<serde_json::Value> {
    // Stub: Clear profile
    Json(serde_json::json!({ "status": "disconnected" }))
}

async fn cloud_login(
    Path(_tenant): Path<String>,
    State(st): State<AppState>,
) -> Json<CloudPdpProfile> {
    let mock_endpoint = "http://127.0.0.1:43891";
    let client = reqwest::Client::new();

    // Attempt contract discovery
    let contract_ver = match client
        .get(&format!("{}/.well-known/pollen-contract", mock_endpoint))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                json.get("preferred")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0")
                    .to_string()
            } else {
                "1.0.0".to_string()
            }
        }
        _ => "1.0.0".to_string(),
    };

    let profile = CloudPdpProfile {
        tenant_id: Some("mock-tenant".into()),
        device_id: Some("mock-device-123".into()),
        pdp_endpoint: Some(mock_endpoint.into()),
        contract_version: Some(contract_ver),
        auth_method: Some("mTLS".into()),
        status: "ready".into(),
        manual_override_enabled: false,
        health: Some(serde_json::json!({ "status": "healthy" })),
    };

    // Store the cloud profile locally (stubbed persistence for now since pdp_store schema doesn't have an exact cloud profile column, we can store it as a runtime).
    let _ = st
        .pdp_store
        .upsert_runtime(
            "local",
            "pollen_cloud",
            &serde_json::to_value(&profile).unwrap(),
        )
        .await;

    Json(profile)
}

async fn cloud_discover(
    Path(tenant): Path<String>,
    State(st): State<AppState>,
) -> Json<CloudPdpProfile> {
    cloud_login(Path(tenant), State(st)).await
}

async fn cloud_probe(
    Path(_tenant): Path<String>,
    State(_st): State<AppState>,
) -> Json<serde_json::Value> {
    // Stub: probe cloud PDP
    Json(serde_json::json!({
        "ok": true,
        "latency_ms": 45,
        "detail": "reachable"
    }))
}
