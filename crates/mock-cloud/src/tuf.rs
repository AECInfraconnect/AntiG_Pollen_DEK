// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::spire::is_device_revoked;
use crate::state::AppState;
use crate::{bundle_pubkey_b64, BUNDLE_SEED};
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

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/bundles/metadata/:role",
            get(get_tuf_metadata),
        )
        .route(
            "/v1/tenants/:tenant_id/devices/:device_id/bundles/artifacts/:hash",
            get(get_tuf_artifact),
        )
        .route("/v1/updater/metadata/:role", get(get_updater_tuf_metadata))
        .route("/v1/updater/artifacts/:hash", get(get_updater_tuf_artifact))
}

async fn get_tuf_metadata(
    Path((_tenant_id, device_id, role)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if is_device_revoked(&state, &device_id) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "device revoked"})),
        );
    }

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&BUNDLE_SEED);

    let now = Utc::now();
    let expires = now + chrono::Duration::days(7);

    let (payload, _role_name) = match role.as_str() {
        "root.json" => (
            json!({
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
            }),
            "root",
        ),
        "targets.json" => {
            let routes_json = json!([
                { "id": "route_tools_call", "priority": 100, "match_rule": { "method": "tools/call", "tool_category": null }, "pdp_required": ["cedar"] }
            ]);
            let manifest_json = json!({
                "manifest_version": "1.0",
                "bundle_id": "bnd-123",
                "bundle_version": "1.0.0",
                "bundle_generation": 1,
                "tenant_id": "tenant-production-1",
                "created_at": "2024-01-01T00:00:00Z",
                "expires_at": "2030-01-01T00:00:00Z",
                "activation_mode": "full",
                "artifacts": []
            });

            use sha2::{Digest, Sha256};
            let mut h1 = Sha256::new();
            h1.update(serde_json::to_vec(&routes_json).unwrap());
            let routes_hash = hex::encode(h1.finalize());

            let mut h2 = Sha256::new();
            h2.update(serde_json::to_vec(&manifest_json).unwrap());
            let manifest_hash = hex::encode(h2.finalize());

            (
                json!({
                    "signed": {
                        "_type": "targets",
                        "spec_version": "1.0",
                        "version": 1,
                        "expires": expires.to_rfc3339(),
                        "targets": {
                            "routes.json": {
                                "hashes": {
                                    "sha256": routes_hash
                                },
                                "length": 1234
                            },
                            "bundle_manifest.json": {
                                "hashes": {
                                    "sha256": manifest_hash
                                },
                                "length": 5678
                            }
                        }
                    },
                    "signatures": []
                }),
                "targets",
            )
        }
        "snapshot.json" => (
            json!({
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
            }),
            "snapshot",
        ),
        "timestamp.json" => (
            json!({
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
            }),
            "timestamp",
        ),
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "role not found"})),
            )
        }
    };

    let signed_bytes = serde_jcs::to_vec(&payload["signed"]).unwrap();
    let signature = signing_key.sign(&signed_bytes);

    let mut response = payload;
    use base64::Engine;
    response["signatures"] = json!([{
        "keyid": "bootstrap",
        "sig": base64::prelude::BASE64_STANDARD.encode(signature.to_bytes())
    }]);

    (StatusCode::OK, Json(response))
}

async fn get_tuf_artifact(
    Path((_tenant_id, _device_id, hash)): Path<(String, String, String)>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let routes_json = json!([
        { "id": "route_tools_call", "priority": 100, "match_rule": { "method": "tools/call", "tool_category": null }, "pdp_required": ["cedar"] }
    ]);
    let manifest_json = json!({
        "manifest_version": "1.0",
        "bundle_id": "bnd-123",
        "bundle_version": "1.0.0",
        "bundle_generation": 1,
        "tenant_id": "tenant-production-1",
        "created_at": "2024-01-01T00:00:00Z",
        "expires_at": "2030-01-01T00:00:00Z",
        "activation_mode": "full",
        "artifacts": []
    });

    use sha2::{Digest, Sha256};
    let mut h1 = Sha256::new();
    h1.update(serde_json::to_vec(&routes_json).unwrap());
    let routes_hash = hex::encode(h1.finalize());

    let mut h2 = Sha256::new();
    h2.update(serde_json::to_vec(&manifest_json).unwrap());
    let manifest_hash = hex::encode(h2.finalize());

    if hash == routes_hash {
        (StatusCode::OK, Json(routes_json))
    } else if hash == manifest_hash {
        (StatusCode::OK, Json(manifest_json))
    } else if hash == "dummy_hash_for_policies" {
        let cedar_src = _state
            .rollout
            .lock()
            .unwrap()
            .latest_bundle
            .cedar_src
            .clone();
        let payload = json!({
            "policies": [
                {
                    "id": "pol_1",
                    "type": "cedar",
                    "content": cedar_src
                }
            ],
            "routes": [
                {
                    "id": "route_tools_call",
                    "priority": 100,
                    "match_rule": {
                        "method": "tools/call",
                        "tool_category": null,
                        "resource_type": "mcp_tool"
                    },
                    "pdp_required": ["cedar"]
                }
            ]
        });
        (StatusCode::OK, Json(payload))
    } else if hash == "dummy_hash_for_registry" {
        (StatusCode::OK, Json(json!({ "registry": {} })))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "artifact not found"})),
        )
    }
}

async fn get_updater_tuf_metadata(Path(role): Path<String>) -> impl IntoResponse {
    let now = Utc::now();
    let expires = now + chrono::Duration::days(7);

    // Mock an executable that is exactly the string "MOCK_EXE_CONTENT"
    let mock_exe_content = b"MOCK_EXE_CONTENT";
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(mock_exe_content);
    let exe_hash = hex::encode(h.finalize());
    let exe_length = mock_exe_content.len();

    let payload = match role.as_str() {
        "targets.json" => {
            json!({
                "signed": {
                    "_type": "targets",
                    "spec_version": "1.0",
                    "version": 1,
                    "expires": expires.to_rfc3339(),
                    "targets": {
                        "dek-core.exe": {
                            "hashes": {
                                "sha256": exe_hash
                            },
                            "length": exe_length,
                            "custom": {
                                "platform": "windows-amd64"
                            }
                        }
                    }
                },
                "signatures": []
            })
        }
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "role not found"})),
            )
        }
    };

    (StatusCode::OK, Json(payload))
}

async fn get_updater_tuf_artifact(Path(hash): Path<String>) -> impl IntoResponse {
    let mock_exe_content = b"MOCK_EXE_CONTENT";
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(mock_exe_content);
    let exe_hash = hex::encode(h.finalize());

    if hash == exe_hash {
        (
            StatusCode::OK,
            axum::body::Bytes::from_static(mock_exe_content).into_response(),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "artifact not found"})).into_response(),
        )
    }
}
