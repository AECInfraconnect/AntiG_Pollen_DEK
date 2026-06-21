use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use ed25519_dalek::Signer;
use serde_json::json;
use crate::state::AppState;
use crate::{BUNDLE_SEED, bundle_pubkey_b64};
use crate::spire::is_device_revoked;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant_id/devices/:device_id/bundles/metadata/:role", get(get_tuf_metadata))
        .route("/v1/tenants/:tenant_id/devices/:device_id/bundles/artifacts/:hash", get(get_tuf_artifact))
}

async fn get_tuf_metadata(Path((_tenant_id, device_id, role)): Path<(String, String, String)>, State(state): State<AppState>) -> impl IntoResponse {
    if is_device_revoked(&state, &device_id) {
        return (StatusCode::FORBIDDEN, Json(json!({"error": "device revoked"})));
    }

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&BUNDLE_SEED);
    
    let now = Utc::now();
    let expires = now + chrono::Duration::days(7);

    let (payload, _role_name) = match role.as_str() {
        "root.json" => {
            (json!({
                "signed": {
                    "_type": "root",
                    "spec_version": "1.0",
                    "version": 1,
                    "expires": expires.to_rfc3339(),
                    "keys": {
                        "key-prod-1": {
                            "keytype": "ed25519",
                            "scheme": "ed25519",
                            "keyval": {
                                "public": bundle_pubkey_b64()
                            }
                        }
                    },
                    "roles": {
                        "root": { "keyids": ["key-prod-1"], "threshold": 1 },
                        "snapshot": { "keyids": ["key-prod-1"], "threshold": 1 },
                        "targets": { "keyids": ["key-prod-1"], "threshold": 1 },
                        "timestamp": { "keyids": ["key-prod-1"], "threshold": 1 }
                    }
                },
                "signatures": []
            }), "root")
        },
        "targets.json" => {
            (json!({
                "signed": {
                    "_type": "targets",
                    "spec_version": "1.0",
                    "version": 1,
                    "expires": expires.to_rfc3339(),
                    "targets": {
                        "routes.json": {
                            "hashes": {
                                "sha256": "mock_hash_routes"
                            },
                            "length": 1234
                        },
                        "bundle_manifest.json": {
                            "hashes": {
                                "sha256": "mock_hash_manifest"
                            },
                            "length": 5678
                        }
                    }
                },
                "signatures": []
            }), "targets")
        },
        "snapshot.json" => {
            (json!({
                "signed": {
                    "_type": "snapshot",
                    "spec_version": "1.0",
                    "version": 1,
                    "expires": expires.to_rfc3339(),
                    "meta": {
                        "targets.json": {
                            "version": 1
                        }
                    }
                },
                "signatures": []
            }), "snapshot")
        },
        "timestamp.json" => {
            (json!({
                "signed": {
                    "_type": "timestamp",
                    "spec_version": "1.0",
                    "version": 1,
                    "expires": expires.to_rfc3339(),
                    "meta": {
                        "snapshot.json": {
                            "version": 1
                        }
                    }
                },
                "signatures": []
            }), "timestamp")
        },
        _ => return (StatusCode::NOT_FOUND, Json(json!({"error": "role not found"}))),
    };

    let signed_bytes = serde_json::to_vec(&payload["signed"]).unwrap();
    let signature = signing_key.sign(&signed_bytes);

    let mut response = payload;
    use base64::Engine;
    response["signatures"] = json!([{
        "keyid": "key-prod-1",
        "sig": base64::prelude::BASE64_STANDARD.encode(signature.to_bytes())
    }]);

    (StatusCode::OK, Json(response))
}

async fn get_tuf_artifact(Path((_tenant_id, _device_id, hash)): Path<(String, String, String)>, State(_state): State<AppState>) -> impl IntoResponse {
    match hash.as_str() {
        "mock_hash_routes" => {
            (StatusCode::OK, Json(json!([
                { "id": "route_tools_call", "priority": 100, "match_rule": { "method": "tools/call", "tool_category": null }, "pdp_required": ["openfga"] }
            ])))
        },
        "mock_hash_manifest" => {
            (StatusCode::OK, Json(json!({
                "manifest_version": "1.0",
                "bundle_id": "bnd-123",
                "bundle_version": "1.0.0",
                "bundle_generation": 1,
                "tenant_id": "tenant-production-1",
                "created_at": Utc::now().to_rfc3339(),
                "expires_at": (Utc::now() + chrono::Duration::days(7)).to_rfc3339(),
                "activation_mode": "full",
                "artifacts": []
            })))
        },
        _ => (StatusCode::NOT_FOUND, Json(json!({"error": "artifact not found"})))
    }
}
