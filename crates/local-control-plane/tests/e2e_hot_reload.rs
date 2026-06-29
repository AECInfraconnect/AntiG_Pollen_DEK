#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::duplicated_attributes
)]
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};

mod common;

#[tokio::test]
async fn e2e_hot_reload_sse() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let base = harness.base_url.clone();
    let client = Client::new();

    // Spawn a task to listen to SSE
    let mut resp = client
        .get(format!("{base}/v1/tenants/local/devices/dev-1/events"))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    // Wait briefly for the SSE connection to establish
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Trigger a policy publish
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
        "status": "draft",
        "tags": []
    });

    let policy = json!({
        "meta": meta,
        "policy_id": "policy-e2e-hotreload",
        "name": "Hot Reload Test",
        "description": "SSE trigger",
        "policy_type": "cedar",
        "targets": {
            "agent_ids": [], "tool_ids": [], "resource_ids": [], "entity_ids": [], "route_ids": []
        },
        "source": {
            "kind": "raw_text",
            "language": "cedar",
            "text": "permit(principal, action, resource);"
        },
        "compile_options": { "fail_on_warnings": true }
    });

    client
        .post(format!("{base}/v1/tenants/local/policies"))
        .json(&policy)
        .send()
        .await
        .unwrap();

    let published = client
        .post(format!(
            "{base}/v1/tenants/local/policies/policy-e2e-hotreload/publish"
        ))
        .json(&policy)
        .send()
        .await
        .unwrap();

    assert!(published.status().is_success());

    // Now read the SSE stream to see if we got the bundle_id
    // Wait for the first message
    let mut found = false;
    for _ in 0..5 {
        let chunk = resp.chunk().await.unwrap().unwrap();
        let text = String::from_utf8(chunk.to_vec()).unwrap();
        if text.contains("data: bundle-") {
            found = true;
            break;
        }
    }

    // It should contain 'data: bundle-...'
    assert!(found, "SSE did not receive bundle push within 5 chunks");
}

#[tokio::test]
async fn e2e_cloud_dispatch_config_and_hot_reload_paths() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let base = harness.base_url.clone();
    let client = Client::new();

    let config_payload = json!({
        "schema_version": "pollek.cloud.connection-update.v1",
        "tenant_id": "local",
        "lcp_id": "lcp_local",
        "device_id": "device_local_windows",
        "pdp_endpoint": "http://127.0.0.1:8790",
        "cloud_url": "http://127.0.0.1:8790",
        "contract_version": "2026.06.29",
        "auth_method": "spiffe-oauth-mtls-required",
        "status": "configured",
        "manual_override_enabled": false,
        "health": {
            "status": "configured",
            "detail": "Configured by signed Pollek Cloud control dispatch for config.update."
        },
        "action": "config.update",
        "runtime_configuration": {
            "poll_seconds": 5,
            "event_stream": "http://127.0.0.1:8790/api/events",
            "entity_sync": "http://127.0.0.1:8790/api/entities/ingest",
            "telemetry_batches": "http://127.0.0.1:8790/v1/telemetry/batches"
        },
        "requested_by": "e2e",
        "requested_at": "2026-06-29T00:00:00Z"
    });
    let config_envelope = signed_envelope(
        "ctrl_e2e_config",
        "config.update",
        "/v1/tenants/local/pdp/cloud",
        &config_payload,
    );
    let mut config_body = config_payload.clone();
    config_body["control_envelope"] = config_envelope;

    let config_response = client
        .patch(format!("{base}/v1/tenants/local/pdp/cloud"))
        .json(&config_body)
        .send()
        .await
        .unwrap();
    assert!(
        config_response.status().is_success(),
        "config dispatch failed: {}",
        config_response.text().await.unwrap()
    );

    let hot_reload_payload = json!({
        "schema_version": "pollek.cloud.connection-update.v1",
        "tenant_id": "local",
        "lcp_id": "lcp_local",
        "device_id": "device_local_windows",
        "pdp_endpoint": "http://127.0.0.1:8790",
        "cloud_url": "http://127.0.0.1:8790",
        "contract_version": "2026.06.29",
        "auth_method": "spiffe-oauth-mtls-required",
        "status": "configured",
        "manual_override_enabled": false,
        "action": "policy.hot_reload",
        "policy_bundle": {
            "bundle_id": "bnd_local_dev_baseline",
            "name": "Local Dev Baseline",
            "revision": "2026.06.29.001",
            "hot_reload": true,
            "signed": true
        },
        "runtime_configuration": {
            "poll_seconds": 5,
            "event_stream": "http://127.0.0.1:8790/api/events",
            "entity_sync": "http://127.0.0.1:8790/api/entities/ingest",
            "telemetry_batches": "http://127.0.0.1:8790/v1/telemetry/batches"
        },
        "requested_by": "e2e",
        "requested_at": "2026-06-29T00:00:00Z"
    });

    for (control_id, path) in [
        (
            "ctrl_e2e_hot_reload_tenant",
            "/v1/tenants/local/bundles/hot-reload",
        ),
        (
            "ctrl_e2e_hot_reload_policy_bundle",
            "/v1/tenants/local/policy-bundles/hot-reload",
        ),
        (
            "ctrl_e2e_hot_reload_named",
            "/v1/policy-bundles/bnd_local_dev_baseline/hot-reload",
        ),
    ] {
        let envelope = signed_envelope(control_id, "policy.hot_reload", path, &hot_reload_payload);
        let body = json!({
            "schema_version": "pollek.cloud.secure-control-message.v1",
            "envelope": envelope,
            "payload": hot_reload_payload,
            "security_posture": {
                "dev_mode_warnings": ["Local HTTP loopback is allowed only for development protocol testing."]
            }
        });
        let response = client
            .post(format!("{base}{path}"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "hot reload dispatch failed for {path}: {}",
            response.text().await.unwrap()
        );
    }

    let latest = client
        .get(format!(
            "{base}/v1/tenants/local/devices/device_local_windows/bundles/manifest"
        ))
        .send()
        .await
        .unwrap();
    assert!(latest.status().is_success());
    let latest_json: serde_json::Value = latest.json().await.unwrap();
    assert_eq!(
        latest_json["bundle_id"],
        json!("bnd_local_dev_baseline"),
        "cloud hot reload did not update bundle:latest"
    );
}

fn signed_envelope(
    control_id: &str,
    action: &str,
    allowed_path: &str,
    payload: &serde_json::Value,
) -> serde_json::Value {
    let mut envelope = json!({
        "schema_version": "pollek.cloud.signed-control-envelope.v1",
        "control_id": control_id,
        "tenant_id": "local",
        "issuer": "pollek-cloud",
        "audience": "lcp_local",
        "lcp_id": "lcp_local",
        "action": action,
        "scope": ["contract.read", "configuration.write", "policy.rollout", "hot_reload.dispatch"],
        "allowed_paths": [allowed_path],
        "issued_at": "2026-06-29T00:00:00Z",
        "expires_at": "2099-01-01T00:00:00Z",
        "nonce": format!("nonce-{control_id}"),
        "payload_hash": sha256_hex(stable_json(payload).as_bytes()),
        "signer": {
            "alg": "HS256-dev",
            "kid": "local-dev-ephemeral"
        }
    });
    let signature = hmac_sha256_base64url(
        b"local-dev-ephemeral-control-key",
        stable_json(&envelope).as_bytes(),
    );
    envelope["signature"] = json!(signature);
    envelope
}

fn stable_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => serde_json::to_string(value).unwrap(),
        serde_json::Value::Array(items) => {
            let rendered = items.iter().map(stable_json).collect::<Vec<_>>().join(",");
            format!("[{rendered}]")
        }
        serde_json::Value::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            let rendered = keys
                .into_iter()
                .map(|key| {
                    let encoded_key = serde_json::to_string(key).unwrap();
                    let encoded_value = stable_json(&object[key]);
                    format!("{encoded_key}:{encoded_value}")
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{rendered}}}")
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn hmac_sha256_base64url(key: &[u8], data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    const BLOCK_SIZE: usize = 64;
    let mut key_block = [0_u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let digest = Sha256::digest(key);
        key_block[..digest.len()].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut outer = [0x5c_u8; BLOCK_SIZE];
    let mut inner = [0x36_u8; BLOCK_SIZE];
    for index in 0..BLOCK_SIZE {
        outer[index] ^= key_block[index];
        inner[index] ^= key_block[index];
    }

    let mut inner_hash = Sha256::new();
    inner_hash.update(inner);
    inner_hash.update(data);
    let inner_result = inner_hash.finalize();

    let mut outer_hash = Sha256::new();
    outer_hash.update(outer);
    outer_hash.update(inner_result);
    URL_SAFE_NO_PAD.encode(outer_hash.finalize())
}
