use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;
use crate::state::{AppState, AuditLog, PolicyBundle};
use crate::BUNDLE_SEED;
use crate::spire::is_device_revoked;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/policies/publish", post(admin_publish_policy))
        .route("/admin/policies/rollout", post(admin_set_rollout))
        .route("/v1/tenants/:tenant_id/devices/:device_id/bundles/latest", get(get_latest_bundle))
}

#[derive(serde::Deserialize)]
struct PublishPolicyReq {
    cedar_src: String,
    openfga_store: String,
}

async fn admin_publish_policy(State(state): State<AppState>, Json(req): Json<PublishPolicyReq>) -> impl IntoResponse {
    let mut rollout = state.rollout.lock().unwrap();
    let current_v: usize = rollout.latest_bundle.version.parse().unwrap_or(1);
    let new_v = current_v + 1;
    let new_version_str = new_v.to_string();

    rollout.latest_bundle = PolicyBundle {
        version: new_version_str.clone(),
        cedar_src: req.cedar_src,
        openfga_store: req.openfga_store,
    };
    
    state.audit_logs.lock().unwrap().push(AuditLog {
        timestamp: Utc::now().to_rfc3339(),
        actor: "admin".to_string(),
        action: "PUBLISH_POLICY".to_string(),
        details: format!("Published new policy version {}", new_version_str),
    });

    Json(json!({"status": "published", "new_version": new_version_str}))
}

#[derive(serde::Deserialize)]
struct RolloutReq {
    canary_percentage: u8,
    canary_bundle_version: String,
    canary_cedar_src: String,
    canary_openfga_store: String,
}

async fn admin_set_rollout(State(state): State<AppState>, Json(req): Json<RolloutReq>) -> impl IntoResponse {
    let mut rollout = state.rollout.lock().unwrap();
    rollout.canary_percentage = req.canary_percentage;
    rollout.canary_bundle = Some(PolicyBundle {
        version: req.canary_bundle_version.clone(),
        cedar_src: req.canary_cedar_src,
        openfga_store: req.canary_openfga_store,
    });
    
    state.audit_logs.lock().unwrap().push(AuditLog {
        timestamp: Utc::now().to_rfc3339(),
        actor: "admin".to_string(),
        action: "UPDATE_ROLLOUT".to_string(),
        details: format!("Set canary rollout for {} at {}%", req.canary_bundle_version, req.canary_percentage),
    });

    Json(json!({"status": "rollout_updated"}))
}

async fn get_latest_bundle(Path((_tenant_id, device_id)): Path<(String, String)>, State(state): State<AppState>) -> impl IntoResponse {
    if is_device_revoked(&state, &device_id) {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "device revoked"})));
    }

    let bundle = {
        let rollout = state.rollout.lock().unwrap();
        
        let mut hash_val: usize = 0;
        for b in device_id.bytes() {
            hash_val = hash_val.wrapping_add(b as usize);
        }
        let dev_pct = (hash_val % 100) as u8;

        if let Some(ref canary) = rollout.canary_bundle {
            if dev_pct < rollout.canary_percentage {
                canary.clone()
            } else {
                rollout.latest_bundle.clone()
            }
        } else {
            rollout.latest_bundle.clone()
        }
    };

    let signing_key = SigningKey::from_bytes(&BUNDLE_SEED);
    use base64::Engine;
    let public_key = signing_key.verifying_key();

    let wasm_path = if std::path::Path::new("plugins/dummy_policy.wasm").exists() {
        "plugins/dummy_policy.wasm"
    } else if std::path::Path::new("target/wasm32-wasip1/release/dummy_policy.wasm").exists() {
        "target/wasm32-wasip1/release/dummy_policy.wasm"
    } else {
        "target/wasm32-wasip1/debug/dummy_policy.wasm"
    };

    let payload = json!({
        "jwt_config": {
            "public_key_pem": state.rsa_public_key_pem.clone(),
            "issuer_url": "https://127.0.0.1:43891",
            "audience": ["pollen-dek"]
        },
        "openfga": { "endpoint": "http://127.0.0.1:8080", "store_id": bundle.openfga_store },
        "cedar": { "policy_src": bundle.cedar_src },
        "opa_wasm": { "policy_path": wasm_path },
        "routes": [
            { "id": "route_tools_call", "priority": 100,
              "match_rule": { "method": "tools/call", "tool_category": null },
              "pdp_required": ["openfga", "opa_wasm"],
              "pdp_conditional": [ { "evaluator": "cedar", "required_payload_key": "*" } ] },
            { "id": "route_default", "priority": 10,
              "match_rule": { "method": "*", "tool_category": null },
              "pdp_required": ["openfga"], "pdp_conditional": [] }
        ]
    });

    let payload_string = serde_json::to_string(&payload).unwrap();
    let signature = signing_key.sign(payload_string.as_bytes());
    (StatusCode::OK, Json(json!({
        "bundle_id": format!("bnd-mcp-authz-{}", bundle.version),
        "version": bundle.version,
        "signature": base64::prelude::BASE64_STANDARD.encode(signature.to_bytes()),
        "public_key": base64::prelude::BASE64_STANDARD.encode(public_key.as_bytes()),
        "payload": payload
    })))
}
