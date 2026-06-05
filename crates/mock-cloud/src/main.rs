use anyhow::{Context, Result};
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use rustls::{server::WebPkiClientVerifier, RootCertStore, ServerConfig};
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde_json::{json, Value};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use axum::extract::State;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
struct AppState {
    revision: Arc<AtomicUsize>,
    rsa_public_key_pem: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Mock Pollen Cloud Server with MTLS...");

    let mut rng = rand_core::OsRng;
    let rsa_bits = 2048;
    let priv_key = rsa::RsaPrivateKey::new(&mut rng, rsa_bits).expect("failed to generate a key");
    let pub_key = rsa::RsaPublicKey::from(&priv_key);
    let rsa_public_key_pem =
        rsa::pkcs8::EncodePublicKey::to_public_key_pem(&pub_key, rsa::pkcs8::LineEnding::LF)
            .expect("failed to encode pub key");

    let state = AppState {
        revision: Arc::new(AtomicUsize::new(1)),
        rsa_public_key_pem,
    };

    let app = Router::new()
        .route("/telemetry", post(ingest_telemetry))
        .route("/bundles/latest", get(get_latest_bundle))
        .route("/config/:device_id", get(get_config))
        .route("/spire/node/attest", post(attest_node))
        .with_state(state);

    // Load server certificate and key
    let certs = load_certs("../../certs/server.crt")?;
    let key = load_private_key("../../certs/server.key")?;

    // Load root CA for client verification
    let mut root_cert_store = RootCertStore::empty();
    let ca_certs = load_certs("../../certs/root_ca.crt")?;
    root_cert_store.add_parsable_certificates(ca_certs);

    // Create client verifier requiring client certificate
    let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_cert_store))
        .build()
        .context("Failed to build client verifier")?;

    // Create ServerConfig
    let mut server_config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs, key)
        .context("Failed to create ServerConfig")?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let rustls_config = RustlsConfig::from_config(Arc::new(server_config));
    let addr = SocketAddr::from(([127, 0, 0, 1], 43891));

    info!("Mock Cloud listening on https://127.0.0.1:43891");

    let handle = Handle::new();
    let shutdown_handle = handle.clone();

    tokio::spawn(async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        info!("Mock Cloud received shutdown signal, shutting down...");
        shutdown_handle.graceful_shutdown(None);
    });

    axum_server::bind_rustls(addr, rustls_config)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;

    info!("Mock Cloud shut down gracefully.");
    Ok(())
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = certs(&mut reader).collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let key = private_key(&mut reader)?.context("No private key found")?;
    Ok(key)
}

async fn ingest_telemetry(Json(payload): Json<Value>) -> Json<Value> {
    info!("CLOUD RECEIVED TELEMETRY: {}", payload);
    Json(json!({"status": "ingested"}))
}

use ed25519_dalek::{Signer, SigningKey};

async fn get_latest_bundle(State(state): State<AppState>) -> Json<Value> {
    let rev = state.revision.fetch_add(1, Ordering::SeqCst);
    info!(
        "CLOUD: Device requested latest bundle. Returning revision {}",
        rev
    );

    // Use a static 32-byte seed for deterministic testing identity
    let seed: [u8; 32] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32,
        0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
        0x32, 0x10,
    ];
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key();

    let b64_pubkey = base64::prelude::BASE64_STANDARD.encode(public_key.as_bytes());
    info!("STATIC ED25519 PUBLIC KEY FOR BUNDLE: {}", b64_pubkey);

    // Dynamically resolve WASM path based on debug vs release
    let wasm_path = if std::path::Path::new("plugins/dummy_policy.wasm").exists() {
        "plugins/dummy_policy.wasm"
    } else if std::path::Path::new("target/wasm32-wasip1/release/dummy_policy.wasm").exists() {
        "target/wasm32-wasip1/release/dummy_policy.wasm"
    } else {
        "target/wasm32-wasip1/debug/dummy_policy.wasm"
    };

    let store_id = format!("store_rev_{}", rev);

    let payload = json!({
        "jwt_config": {
            "public_key_pem": state.rsa_public_key_pem.clone()
        },
        "openfga": {
            "endpoint": "http://127.0.0.1:8080",
            "store_id": store_id
        },
        "cedar": {
            "policy_src": format!("permit(\n  principal == User::\"user_bob\",\n  action == Action::\"tools/call\",\n  resource == Resource::\"mcp_tool\"\n); // rev {}", rev)
        },
        "opa_wasm": {
            "policy_path": wasm_path
        },
        "routes": [
            {
                "id": "route_tools_call",
                "priority": 100,
                "match_rule": {
                    "method": "tools/call",
                    "tool_category": null
                },
                "pdp_required": ["openfga", "opa_wasm"],
                "pdp_conditional": [
                    {
                        "evaluator": "cedar",
                        "required_payload_key": "*"
                    }
                ]
            },
            {
                "id": "route_default",
                "priority": 10,
                "match_rule": {
                    "method": "*",
                    "tool_category": null
                },
                "pdp_required": ["openfga"],
                "pdp_conditional": []
            }
        ]
    });

    let payload_string = serde_json::to_string(&payload).unwrap();
    let signature = signing_key.sign(payload_string.as_bytes());

    use base64::Engine;
    let b64_pubkey = base64::prelude::BASE64_STANDARD.encode(public_key.as_bytes());
    let b64_sig = base64::prelude::BASE64_STANDARD.encode(signature.to_bytes());

    Json(json!({
        "bundle_id": format!("bnd-mcp-authz-{:03}", rev),
        "version": format!("1.0.{}", rev),
        "signature": b64_sig,
        "public_key": b64_pubkey,
        "payload": payload
    }))
}

async fn get_config(Path(device_id): Path<String>, State(state): State<AppState>) -> Json<Value> {
    let rev = state.revision.fetch_add(1, Ordering::SeqCst);
    info!(
        "CLOUD: Device {} requested config. Returning revision {}",
        device_id, rev
    );

    // Dynamically resolve WASM path based on debug vs release
    let wasm_path = if std::path::Path::new("plugins/dummy_policy.wasm").exists() {
        "plugins/dummy_policy.wasm"
    } else if std::path::Path::new("target/wasm32-wasip1/release/dummy_policy.wasm").exists() {
        "target/wasm32-wasip1/release/dummy_policy.wasm"
    } else {
        "target/wasm32-wasip1/debug/dummy_policy.wasm"
    };

    let store_id = format!("store_rev_{}", rev);

    Json(json!({
        "device_id": device_id,
        "tenant_id": "tenant-production-1",
        "mtls": {
            "client_cert_path": "certs/client.crt",
            "client_key_path": "certs/client.key",
            "root_ca_path": "certs/root_ca.crt"
        },
        "spire_server": {
            "endpoint": "https://127.0.0.1:43891/spire"
        },
        "jwt_config": {
            "public_key_pem": state.rsa_public_key_pem.clone()
        },
        "policy_config": {
            "openfga": {
                "endpoint": "http://127.0.0.1:8080",
                "store_id": store_id
            },
            "cedar": {
                "policy_src": format!("permit(\n  principal == User::\"user_bob\",\n  action == Action::\"tools/call\",\n  resource == Resource::\"mcp_tool\"\n); // rev {}", rev)
            },
            "opa_wasm": {
                "policy_path": wasm_path
            },
            "routes": [
                {
                    "id": "route_tools_call",
                    "priority": 100,
                    "match_rule": {
                        "method": "tools/call",
                        "tool_category": null
                    },
                    "pdp_required": ["openfga", "opa_wasm"],
                    "pdp_conditional": [
                        {
                            "evaluator": "cedar",
                            "required_payload_key": "*"
                        }
                    ]
                },
                {
                    "id": "route_default",
                    "priority": 10,
                    "match_rule": {
                        "method": "*",
                        "tool_category": null
                    },
                    "pdp_required": ["openfga"],
                    "pdp_conditional": []
                }
            ]
        }
    }))
}

async fn attest_node(Json(payload): Json<Value>) -> Json<Value> {
    let device_id = payload
        .get("device_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown-device");
    let spiffe_id = format!(
        "spiffe://pollen.cloud/tenant-production-1/device/{}",
        device_id
    );
    info!(
        "CLOUD: Attesting node {}, issuing SPIFFE ID: {}",
        device_id, spiffe_id
    );
    Json(json!({
        "spiffe_id": spiffe_id
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_pubkey() {
        use base64::Engine;
        let seed: [u8; 32] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98,
            0x76, 0x54, 0x32, 0x10,
        ];
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key();
        let b64_pubkey = base64::prelude::BASE64_STANDARD.encode(public_key.as_bytes());
        println!("HARDCODED_PUBKEY={}", b64_pubkey);
    }
}
