#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::duplicated_attributes
)]
use reqwest::Client;
use serde_json::json;

mod common;

#[tokio::test]
async fn entity_graph_joins_registry_policy_observation_and_activity() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let base = harness.base_url.clone();
    let client = Client::new();

    let meta = json!({
        "schema_version": "v1",
        "tenant_id": "local",
        "workspace_id": "default",
        "environment_id": "local",
        "created_at": "2026-06-26T00:00:00Z",
        "updated_at": "2026-06-26T00:00:00Z",
        "created_by": "local-admin",
        "updated_by": "local-admin",
        "source": "manual",
        "status": "active",
        "tags": []
    });

    let agent = json!({
        "meta": meta.clone(),
        "agent_id": "agent-graph-e2e",
        "name": "Graph E2E Agent",
        "agent_type": "custom_mcp_client",
        "vendor": "test",
        "runtime": { "runtime_name": "codex", "version": "1" },
        "entrypoints": [],
        "declared_tools": ["tool-read"],
        "declared_resources": ["res-customer"],
        "identity": { "spiffe_id": "spiffe://local/agent/graph-e2e" },
        "trust_level": "medium",
        "capabilities": ["mcp_tool_call"],
        "labels": {},
        "enforcement_mode": "Enforce"
    });

    let res = client
        .post(format!("{base}/v1/tenants/local/registry/agents"))
        .json(&agent)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let policy = json!({
        "meta": meta,
        "policy_id": "pol-graph-e2e",
        "name": "Protect Graph Agent",
        "description": "Blocks risky tool calls in the graph e2e test.",
        "policy_type": "cedar",
        "targets": {
            "agent_ids": ["agent-graph-e2e"],
            "tool_ids": ["tool-read"],
            "resource_ids": ["res-customer"],
            "entity_ids": [],
            "route_ids": []
        },
        "source": {
            "kind": "raw_text",
            "language": "cedar",
            "text": "permit(principal, action, resource);"
        },
        "compile_options": { "fail_on_warnings": true }
    });

    let res = client
        .post(format!("{base}/v1/tenants/local/policies"))
        .json(&policy)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let observation = json!({
        "event_id": "obs-graph-e2e",
        "tenant_id": "local",
        "trace_id": "trace-graph-e2e",
        "agent_id": "agent-graph-e2e",
        "shadow_candidate_id": null,
        "tool_id": "tool-read",
        "resource_id": "res-customer",
        "surface": "mcp",
        "action": "read_file",
        "pep_type": "mcp_proxy",
        "risk_level": "high",
        "timestamp": "2026-06-26T00:01:00Z",
        "payload_json": "{}",
        "token_usage": {
            "input_tokens": 12,
            "output_tokens": 3,
            "total_tokens": 15,
            "model": "test-model"
        },
        "event_kind": "tool_call",
        "decision": {
            "allow": false,
            "reason_code": "blocked_by_test_policy",
            "obligations": [],
            "matched_policy_ids": ["pol-graph-e2e"],
            "compliance_tags": ["test"],
            "pep_plane": "mcp_proxy",
            "enforced_for_real": true,
            "status_badge": "denied",
            "message_th": null
        },
        "tool_call": {
            "tool_name": "tool-read",
            "server": "mcp-test",
            "args_summary": "redacted",
            "result_status": "blocked"
        },
        "resource_access": null,
        "latency_ms": 7,
        "provider": "local"
    });

    let res = client
        .post(format!("{base}/v1/tenants/local/observations"))
        .json(&observation)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let guard_incident = json!({
        "schema_version": "telemetry-envelope.v1",
        "event_id": "guard-graph-e2e",
        "event_type": "guard_incident",
        "timestamp": "2026-06-26T00:00:30Z",
        "tenant_id": "local",
        "agent_id": "agent-graph-e2e",
        "redaction_applied": true,
        "payload": {
            "guard_event": {
                "event_id": "guard-graph-e2e",
                "ts": "2026-06-26T00:00:30Z",
                "tenant_id": "local",
                "agent_id": "agent-graph-e2e",
                "direction": "request",
                "action": "redact",
                "categories": ["llm01_prompt_injection"],
                "injection_score": 0.93,
                "findings_summary": [{ "kind": "api_key", "count": 1 }],
                "severity": "warn",
                "remediation": {
                    "user_message": "Redacted prompt safety event.",
                    "recommended_actions": [],
                    "doc_url": null,
                    "can_override": false
                },
                "redaction_applied": true
            }
        }
    });

    let res = client
        .post(format!("{base}/v1/tenants/local/telemetry/events"))
        .json(&json!({ "events": [guard_incident] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let graph = client
        .get(format!("{base}/v1/tenants/local/entity-graph"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(graph["schema_version"], "entity-graph.v1");
    let nodes = graph["nodes"].as_array().unwrap();
    assert!(nodes
        .iter()
        .any(|node| node["id"] == "agent:agent-graph-e2e"));
    assert!(nodes
        .iter()
        .any(|node| node["id"] == "policy:pol-graph-e2e"));
    let edges = graph["edges"].as_array().unwrap();
    assert!(edges.iter().any(|edge| {
        edge["source"] == "policy:pol-graph-e2e"
            && edge["target"] == "agent:agent-graph-e2e"
            && edge["relation"] == "governs"
    }));
    assert!(edges.iter().any(|edge| {
        edge["source"] == "agent:agent-graph-e2e"
            && edge["target"] == "tool:tool-read"
            && edge["relation"] == "uses"
    }));

    let entity_360 = client
        .get(format!(
            "{base}/v1/tenants/local/entity-graph/nodes/agent/agent-graph-e2e"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(entity_360["schema_version"], "entity-360.v1");
    assert_eq!(entity_360["entity"]["entity_id"], "agent-graph-e2e");
    assert!(!entity_360["activity"].as_array().unwrap().is_empty());

    let resource_360 = client
        .get(format!(
            "{base}/v1/tenants/local/entity-graph/node?entity_type=resource&entity_id=res-customer"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(resource_360["schema_version"], "entity-360.v1");
    assert_eq!(resource_360["entity"]["entity_id"], "res-customer");

    let timeline = client
        .get(format!(
            "{base}/v1/tenants/local/activity-timeline?agent_id=agent-graph-e2e"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(timeline["schema_version"], "activity-timeline.v1");
    assert_eq!(timeline["items"][0]["decision"], "deny");
    assert_eq!(timeline["items"][0]["enforcement_mode"], "enforce");
    assert!(timeline["items"].as_array().unwrap().iter().any(|item| {
        item["event_id"] == "guard-graph-e2e"
            && item["decision"] == "redact"
            && item["resource"]["label"] == "Prompt injection attempt"
    }));

    let user_activity = client
        .get(format!(
            "{base}/v1/tenants/local/user-friendly-activity?agent_id=agent-graph-e2e"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(
        user_activity["schema_version"],
        "user-friendly-activity-list.v1"
    );
    assert_eq!(
        user_activity["items"][0]["schema_version"],
        "user-friendly-activity.v1"
    );
    assert_eq!(user_activity["items"][0]["agent_name"], "Graph E2E Agent");
    assert_eq!(user_activity["items"][0]["result"], "blocked");
    assert!(user_activity["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["event_id"] == "guard-graph-e2e"
                && item["category"] == "safety"
                && item["result"] == "redacted"
                && item["target_label"] == "Prompt injection attempt"
        }));
    assert_eq!(
        user_activity["items"][0]["privacy_note"],
        "Pollek shows activity metadata here, not file contents, email bodies, raw prompts, or raw responses."
    );

    let plugin_install = client
        .post(format!("{base}/v1/tenants/local/plugins/install"))
        .json(&json!({
            "id": "com.pollek.pii-redactor",
            "granted_caps": ["telemetry:read", "telemetry:write"]
        }))
        .send()
        .await
        .unwrap();
    assert!(plugin_install.status().is_success());

    let plugin_activity = client
        .get(format!(
            "{base}/v1/tenants/local/user-friendly-activity?limit=50"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert!(
        plugin_activity["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| {
                item["agent_name"] == "Pollek Plugin Marketplace"
                    && item["category"] == "plugins"
                    && item["action"] == "install"
                    && item["target_label"] == "PII Redactor"
            }),
        "{plugin_activity}"
    );
}

#[tokio::test]
async fn unknown_v1_routes_return_json_not_dashboard_html() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let client = Client::new();

    let response = client
        .get(format!(
            "{}/v1/tenants/local/not-a-dashboard-route",
            harness.base_url
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = response.text().await.unwrap();

    assert!(
        content_type.contains("application/json"),
        "expected JSON content-type, got {content_type}; body={body}"
    );
    assert!(
        !body.to_ascii_lowercase().contains("<!doctype html"),
        "API fallback leaked dashboard HTML: {body}"
    );
}
