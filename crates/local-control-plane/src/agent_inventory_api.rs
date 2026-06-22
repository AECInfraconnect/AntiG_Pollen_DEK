// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use dek_domain_schema::AgentCapabilityInventory;

use crate::{
    error::{ApiError, ApiResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/agent-inventory", get(list_inventory))
        .route(
            "/v1/tenants/:tenant/agent-inventory/:agent_id",
            get(get_inventory),
        )
        .route(
            "/v1/tenants/:tenant/agent-inventory/rebuild",
            post(rebuild_inventory),
        )
}

async fn list_inventory(
    Path(tenant): Path<String>,
    State(st): State<AppState>,
) -> ApiResult<Json<Vec<AgentCapabilityInventory>>> {
    let items = st
        .registry_store
        .list_agent_inventories(&tenant)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(items))
}

async fn get_inventory(
    Path((tenant, agent_id)): Path<(String, String)>,
    State(st): State<AppState>,
) -> ApiResult<Json<AgentCapabilityInventory>> {
    let item = st
        .registry_store
        .get_agent_inventory(&tenant, &agent_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound(agent_id))?;
    Ok(Json(item))
}

async fn rebuild_inventory(
    Path(tenant): Path<String>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let candidates_raw = st
        .registry_store
        .list_raw(&tenant, "discovery_candidate")
        .await
        .map_err(ApiError::Internal)?;

    let mut rebuilt_count = 0;

    for raw in candidates_raw {
        if let Ok(candidate) =
            serde_json::from_value::<dek_agent_discovery::model::DiscoveredAgentCandidateV2>(raw)
        {
            let agent_kind_str = serde_json::to_string(&candidate.inferred_agent_type)
                .unwrap_or_else(|_| "\"UnknownAiProcess\"".to_string());
            let agent_kind: dek_domain_schema::AgentKind = serde_json::from_str(&agent_kind_str)
                .unwrap_or(dek_domain_schema::AgentKind::UnknownAiProcess);

            let mut mcp_surfaces = Vec::new();
            for mcp in &candidate.discovered_mcp_servers {
                mcp_surfaces.push(dek_domain_schema::McpSurface {
                    server_name: mcp.server_name.clone(),
                    client_hint: "discovered".to_string(),
                    transport: dek_domain_schema::McpTransportKind::Stdio,
                    command_template: mcp.command.clone().map(|c| vec![c]),
                    endpoint_domain: None,
                    has_auth_header: false,
                    env_key_names: vec![],
                    tools_known: vec![],
                    resources_known: vec![],
                });
            }

            let inv = dek_domain_schema::AgentCapabilityInventory {
                schema_version: "agent-capability-inventory.v1".to_string(),
                tenant_id: tenant.clone(),
                device_id: candidate.device_id.clone(),
                agent_id: candidate.candidate_id.clone(), // using candidate_id as agent_id for discovered but unregistered
                candidate_id: Some(candidate.candidate_id.clone()),
                display_name: candidate.display_name.clone(),
                agent_type: agent_kind,
                trust_level: candidate.suggested_registration.trust_level.clone(),
                confidence: candidate.confidence,
                risk_score: candidate.risk_score,
                process: None,
                config_surfaces: vec![],
                mcp_surfaces,
                model_endpoints: vec![],
                browser_surfaces: vec![],
                file_surfaces: vec![],
                network_surfaces: vec![],
                supported_pep_bindings: vec![],
                supported_pdp_routes: vec![],
                telemetry_capabilities: dek_domain_schema::TelemetryCapabilities {
                    emits_tool_logs: false,
                    emits_resource_logs: false,
                    emits_decision_logs: false,
                    emits_network_logs: false,
                    format: "pollen".to_string(),
                },
                last_scan_id: "rebuild".to_string(),
                last_seen_at: chrono::Utc::now().to_rfc3339(),
            };

            let _ = st.registry_store.upsert_agent_inventory(inv).await;
            rebuilt_count += 1;
        }
    }

    Ok(Json(
        serde_json::json!({"status": "rebuilt", "count": rebuilt_count}),
    ))
}
