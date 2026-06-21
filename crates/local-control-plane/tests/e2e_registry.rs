#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::duplicated_attributes)]
use reqwest::Client;
use serde_json::json;

mod common;

#[tokio::test]
async fn e2e_register_agent_tool_resource_entity() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let base = harness.base_url.clone();
    let client = Client::new();

    let meta = json!({
        "schema_version": "v1",
        "tenant_id": "local",
        "workspace_id": "default",
        "environment_id": "local",
        "created_at": "2026-06-10T00:00:00Z",
        "updated_at": "2026-06-10T00:00:00Z",
        "created_by": "local-admin",
        "updated_by": "local-admin",
        "source": "manual",
        "status": "active",
        "tags": []
    });

    let agent = json!({
        "meta": meta,
        "agent_id": "agent-e2e",
        "name": "E2E Agent",
        "agent_type": "custom_mcp_client",
        "vendor": "test",
        "runtime": { "runtime_name": "test", "version": "1" },
        "entrypoints": [],
        "declared_tools": [],
        "declared_resources": [],
        "identity": {},
        "trust_level": "medium",
        "capabilities": [],
        "labels": {}
    });

    let res = client
        .post(format!("{base}/v1/tenants/local/registry/agents"))
        .json(&agent)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        201,
        "Expected 201 Created for agent registration"
    );

    let list = client
        .get(format!("{base}/v1/tenants/local/registry/agents"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|a| a["agent_id"] == "agent-e2e"),
        "Agent not found in list"
    );
}
