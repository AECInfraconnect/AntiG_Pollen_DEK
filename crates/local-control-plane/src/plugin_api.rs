use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use dek_agent_observer::model::{AgentObservationEvent, DecisionInfo, EventKind, ResourceAccess};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

const INSTALLED_PLUGIN_OBJECT: &str = "plugin_installed";
const PLUGIN_AUDIT_AGENT_ID: &str = "pollek-plugin-marketplace";

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant/marketplace/items",
            get(list_marketplace_items),
        )
        .route(
            "/v1/tenants/:tenant/marketplace/items/:id",
            get(marketplace_item_detail),
        )
        .route("/v1/tenants/:tenant/plugins", get(list_plugins))
        .route("/v1/tenants/:tenant/plugins/install", post(install_plugin))
        .route("/v1/tenants/:tenant/plugins/:id", delete(uninstall_plugin))
        .route(
            "/v1/tenants/:tenant/plugins/:id/toggle",
            post(toggle_plugin),
        )
        .route(
            "/v1/tenants/:tenant/plugins/:id/enable",
            post(enable_plugin),
        )
        .route(
            "/v1/tenants/:tenant/plugins/:id/disable",
            post(disable_plugin),
        )
        .route("/v1/tenants/:tenant/plugins/:id/test", post(test_plugin))
}

fn marketplace_items() -> Vec<Value> {
    vec![
        json!({
            "id": "com.pollek.pii-redactor",
            "name": "PII Redactor",
            "version": "1.0.0",
            "kind": "telemetry.transform",
            "publisher": "AEC Infraconnect",
            "verified": true,
            "rating": 4.8,
            "installs": 1280,
            "capabilities": ["telemetry:read", "telemetry:write"],
            "human_capabilities": [
                "Redacts private data from local telemetry before export or display",
                "Does not send data off this device"
            ],
            "os": ["windows", "linux", "macos"],
            "min_engine_version": "1.0.0",
            "signature_ok": true,
            "signature_state": "valid",
            "description_en": "Masks common PII fields in activity metadata.",
            "description_th": "Masks common private-data fields in local activity metadata.",
            "privacy_note": "Local transform only. No network access requested.",
            "source": "local_catalog"
        }),
        json!({
            "id": "com.pollek.definition-feed",
            "name": "AI Agent Definition Feed",
            "version": "0.3.0",
            "kind": "definition.feed",
            "publisher": "AEC Infraconnect",
            "verified": true,
            "rating": 4.6,
            "installs": 920,
            "capabilities": ["definitions:write", "candidates:write"],
            "human_capabilities": [
                "Adds well-known AI app signatures and friendly explanations",
                "Improves discovery and observe labels"
            ],
            "os": ["windows", "linux", "macos"],
            "min_engine_version": "1.0.0",
            "signature_ok": true,
            "signature_state": "valid",
            "description_en": "Updates local AI app definitions used by discovery and reference intel.",
            "description_th": "Updates local AI app definitions and friendly explanations.",
            "privacy_note": "Writes local definitions. No native OS capability requested.",
            "source": "local_catalog"
        }),
        json!({
            "id": "com.example.splunk-exporter",
            "name": "Splunk Telemetry Exporter",
            "version": "0.1.0",
            "kind": "telemetry.exporter",
            "publisher": "Example Labs",
            "verified": false,
            "rating": 0.0,
            "installs": 0,
            "capabilities": ["telemetry:read", "http_out:splunk.example.com:443"],
            "human_capabilities": [
                "Reads activity metadata",
                "Sends selected telemetry to splunk.example.com"
            ],
            "os": ["windows", "linux", "macos"],
            "min_engine_version": "1.0.0",
            "signature_ok": false,
            "signature_state": "test_only",
            "description_en": "Developer preview exporter for a Splunk HEC endpoint.",
            "description_th": "Developer preview exporter for sending selected telemetry to Splunk.",
            "privacy_note": "This plugin can send activity metadata off this device. Install only for testing.",
            "source": "local_catalog"
        }),
    ]
}

async fn list_marketplace_items(
    Path(_tenant): Path<String>,
    State(_state): State<AppState>,
) -> Json<Value> {
    Json(json!({
        "schema_version": "pollek.marketplace.v1",
        "items": marketplace_items()
    }))
}

async fn marketplace_item_detail(
    Path((_tenant, id)): Path<(String, String)>,
    State(_state): State<AppState>,
) -> Json<Value> {
    let item = marketplace_items()
        .into_iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(id.as_str()));
    Json(json!({
        "schema_version": "pollek.marketplace.item.v1",
        "item": item
    }))
}

async fn list_plugins(
    Path(tenant): Path<String>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match state
        .registry_store
        .list_raw(&tenant, INSTALLED_PLUGIN_OBJECT)
        .await
    {
        Ok(mut items) => {
            items.sort_by(|a, b| {
                let left = a.get("name").and_then(Value::as_str).unwrap_or_default();
                let right = b.get("name").and_then(Value::as_str).unwrap_or_default();
                left.cmp(right)
            });
            (
                StatusCode::OK,
                Json(json!({
                    "schema_version": "pollek.installed_plugins.v1",
                    "items": items
                })),
            )
        }
        Err(err) => error_response(err),
    }
}

#[derive(Debug, Deserialize)]
struct InstallPayload {
    id: String,
    #[serde(default)]
    granted_caps: Vec<String>,
}

async fn install_plugin(
    Path(tenant): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<InstallPayload>,
) -> (StatusCode, Json<Value>) {
    let item = marketplace_items()
        .into_iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(payload.id.as_str()))
        .unwrap_or_else(|| sideload_item(&payload.id));
    let plugin = installed_plugin_from_item(&item, payload.granted_caps, true, "healthy");

    match state
        .registry_store
        .upsert_raw(
            &tenant,
            INSTALLED_PLUGIN_OBJECT,
            plugin
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(&payload.id),
            &plugin,
        )
        .await
    {
        Ok(()) => {
            let _ = record_plugin_activity(&state, &tenant, "plugin_installed", &plugin).await;
            (StatusCode::OK, Json(plugin))
        }
        Err(err) => error_response(err),
    }
}

async fn uninstall_plugin(
    Path((tenant, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    let existing = state
        .registry_store
        .get_raw(&tenant, INSTALLED_PLUGIN_OBJECT, &id)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| sideload_item(&id));

    match state
        .registry_store
        .delete_raw(&tenant, INSTALLED_PLUGIN_OBJECT, &id)
        .await
    {
        Ok(deleted) => {
            let _ = record_plugin_activity(&state, &tenant, "plugin_uninstalled", &existing).await;
            (
                StatusCode::OK,
                Json(json!({
                    "status": if deleted { "uninstalled" } else { "not_installed" },
                    "id": id,
                    "revoked_caps": true,
                    "cleared_plugin_namespace": deleted
                })),
            )
        }
        Err(err) => error_response(err),
    }
}

#[derive(Debug, Deserialize)]
struct TogglePayload {
    enabled: bool,
}

async fn toggle_plugin(
    Path((tenant, id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(payload): Json<TogglePayload>,
) -> (StatusCode, Json<Value>) {
    match load_or_catalog_plugin(&state, &tenant, &id).await {
        Ok(mut plugin) => {
            plugin["enabled"] = json!(payload.enabled);
            plugin["health"] = json!(if payload.enabled {
                "healthy"
            } else {
                "disabled"
            });
            plugin["last_seen"] = json!(chrono::Utc::now().to_rfc3339());
            let store_result = state
                .registry_store
                .upsert_raw(&tenant, INSTALLED_PLUGIN_OBJECT, &id, &plugin)
                .await;
            match store_result {
                Ok(()) => {
                    let action = if payload.enabled {
                        "plugin_enabled"
                    } else {
                        "plugin_disabled"
                    };
                    let _ = record_plugin_activity(&state, &tenant, action, &plugin).await;
                    (StatusCode::OK, Json(plugin))
                }
                Err(err) => error_response(err),
            }
        }
        Err(err) => error_response(err),
    }
}

async fn enable_plugin(
    Path((tenant, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    toggle_without_body(tenant, state, id, true).await
}

async fn disable_plugin(
    Path((tenant, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    toggle_without_body(tenant, state, id, false).await
}

async fn toggle_without_body(
    tenant: String,
    state: AppState,
    id: String,
    enabled: bool,
) -> (StatusCode, Json<Value>) {
    match load_or_catalog_plugin(&state, &tenant, &id).await {
        Ok(mut plugin) => {
            plugin["enabled"] = json!(enabled);
            plugin["health"] = json!(if enabled { "healthy" } else { "disabled" });
            plugin["last_seen"] = json!(chrono::Utc::now().to_rfc3339());
            match state
                .registry_store
                .upsert_raw(&tenant, INSTALLED_PLUGIN_OBJECT, &id, &plugin)
                .await
            {
                Ok(()) => {
                    let action = if enabled {
                        "plugin_enabled"
                    } else {
                        "plugin_disabled"
                    };
                    let _ = record_plugin_activity(&state, &tenant, action, &plugin).await;
                    (StatusCode::OK, Json(plugin))
                }
                Err(err) => error_response(err),
            }
        }
        Err(err) => error_response(err),
    }
}

async fn test_plugin(
    Path((tenant, id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(_payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    match load_or_catalog_plugin(&state, &tenant, &id).await {
        Ok(mut plugin) => {
            plugin["health"] = json!("healthy");
            plugin["last_seen"] = json!(chrono::Utc::now().to_rfc3339());
            let _ = state
                .registry_store
                .upsert_raw(&tenant, INSTALLED_PLUGIN_OBJECT, &id, &plugin)
                .await;
            let _ = record_plugin_activity(&state, &tenant, "plugin_health_checked", &plugin).await;
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "message": format!("Plugin {} health check recorded", id),
                    "output": {}
                })),
            )
        }
        Err(err) => error_response(err),
    }
}

fn installed_plugin_from_item(
    item: &Value,
    granted_caps: Vec<String>,
    enabled: bool,
    health: &str,
) -> Value {
    let now = chrono::Utc::now().to_rfc3339();
    let caps = if granted_caps.is_empty() {
        string_array(item, "capabilities")
    } else {
        granted_caps
    };
    json!({
        "schema_version": "pollek.installed_plugin.v1",
        "id": item.get("id").cloned().unwrap_or_else(|| json!("unknown-plugin")),
        "name": item.get("name").cloned().unwrap_or_else(|| json!("Unknown plugin")),
        "version": item.get("version").cloned().unwrap_or_else(|| json!("unknown")),
        "kind": item.get("kind").cloned().unwrap_or_else(|| json!("unknown")),
        "enabled": enabled,
        "granted_caps": caps,
        "human_grants": item.get("human_capabilities").cloned().unwrap_or_else(|| json!([])),
        "health": health,
        "source": item.get("source").cloned().unwrap_or_else(|| json!("sideload")),
        "signature_state": item.get("signature_state").cloned().unwrap_or_else(|| json!("unknown")),
        "privacy_note": item.get("privacy_note").cloned().unwrap_or(Value::Null),
        "last_seen": now,
        "installed_at": now
    })
}

async fn load_or_catalog_plugin(state: &AppState, tenant: &str, id: &str) -> anyhow::Result<Value> {
    if let Some(plugin) = state
        .registry_store
        .get_raw(tenant, INSTALLED_PLUGIN_OBJECT, id)
        .await?
    {
        return Ok(plugin);
    }
    let item = marketplace_items()
        .into_iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
        .unwrap_or_else(|| sideload_item(id));
    Ok(installed_plugin_from_item(
        &item,
        string_array(&item, "capabilities"),
        false,
        "disabled",
    ))
}

fn sideload_item(id: &str) -> Value {
    json!({
        "id": id,
        "name": id,
        "version": "unknown",
        "kind": "unknown",
        "publisher": "Local sideload",
        "verified": false,
        "rating": 0.0,
        "installs": 0,
        "capabilities": [],
        "human_capabilities": [],
        "os": ["windows", "linux", "macos"],
        "min_engine_version": "unknown",
        "signature_ok": false,
        "signature_state": "unknown",
        "description_en": "Local plugin not found in the marketplace catalog.",
        "privacy_note": "Review the local manifest before enabling this plugin.",
        "source": "sideload"
    })
}

fn string_array(item: &Value, key: &str) -> Vec<String> {
    item.get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

async fn record_plugin_activity(
    state: &AppState,
    tenant: &str,
    action: &str,
    plugin: &Value,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now();
    let plugin_id = plugin
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown-plugin");
    let plugin_name = plugin
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(plugin_id);
    let granted_caps = string_array(plugin, "granted_caps");
    let sensitive = granted_caps.iter().any(|capability| {
        capability.starts_with("http_out:")
            || capability.starts_with("native:")
            || capability.contains(":write")
    });
    let payload = json!({
        "schema_version": "pollek.plugin_activity.v1",
        "plugin_id": plugin_id,
        "plugin_name": plugin_name,
        "action": action,
        "enabled": plugin.get("enabled").and_then(Value::as_bool),
        "health": plugin.get("health").and_then(Value::as_str),
        "granted_caps": granted_caps,
        "signature_state": plugin.get("signature_state").and_then(Value::as_str),
        "privacy_note": plugin.get("privacy_note").and_then(Value::as_str),
        "source": "plugin_registry"
    });
    let event_id = format!("plugin-{action}-{plugin_id}-{}", now.timestamp_millis());
    let event = AgentObservationEvent {
        event_id: event_id.clone(),
        tenant_id: tenant.to_string(),
        trace_id: event_id.clone(),
        agent_id: Some(PLUGIN_AUDIT_AGENT_ID.to_string()),
        shadow_candidate_id: None,
        tool_id: None,
        resource_id: Some(plugin_id.to_string()),
        surface: "plugin_marketplace".to_string(),
        action: action.to_string(),
        pep_type: Some("local_plugin_registry".to_string()),
        risk_level: Some(if sensitive { "medium" } else { "low" }.to_string()),
        timestamp: now.to_rfc3339(),
        payload_json: serde_json::to_string(&payload)?,
        token_usage: None,
        browser_scope: None,
        event_kind: EventKind::ResourceAccess,
        decision: Some(DecisionInfo {
            allow: true,
            reason_code: action.to_string(),
            obligations: vec!["record_plugin_audit_event".to_string()],
            matched_policy_ids: Vec::new(),
            compliance_tags: vec!["plugin_audit".to_string()],
            pep_plane: Some("local_plugin_registry".to_string()),
            enforced_for_real: Some(false),
            status_badge: Some("audit".to_string()),
            message_th: None,
        }),
        tool_call: None,
        resource_access: Some(ResourceAccess {
            resource_type: "plugin".to_string(),
            target_redacted: plugin_name.to_string(),
            bytes: None,
            verb: action.to_string(),
        }),
        latency_ms: None,
        provider: None,
    };
    state
        .observability_store
        .insert_observation_event(&event)
        .await?;
    state
        .telemetry_store
        .put_telemetry(tenant, "plugin_audit", &event_id, &payload)
        .await?;
    Ok(())
}

fn error_response(err: anyhow::Error) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": err.to_string() })),
    )
}
