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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Mock Pollen Cloud Server with MTLS...");

    let app = Router::new()
        .route("/telemetry", post(ingest_telemetry))
        .route("/bundles/latest", get(get_latest_bundle))
        .route("/config/:device_id", get(get_config));

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
use rand_core::OsRng;

async fn get_latest_bundle() -> Json<Value> {
    info!("CLOUD: Device requested latest bundle.");

    // Generate real ED25519 keypair for mock
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let public_key = signing_key.verifying_key();

    // Dynamically resolve WASM path based on debug vs release
    let wasm_path =
        if std::path::Path::new("target/wasm32-wasip1/release/dummy_policy.wasm").exists() {
            "target/wasm32-wasip1/release/dummy_policy.wasm"
        } else {
            "target/wasm32-wasip1/debug/dummy_policy.wasm"
        };

    let payload = json!({
        "openfga": {
            "endpoint": "http://127.0.0.1:8080",
            "store_id": "store_123"
        },
        "cedar": {
            "policy_src": "permit(\n  principal == User::\"user_bob\",\n  action == Action::\"tools/call\",\n  resource == Resource::\"mcp_tool\"\n);"
        },
        "opa_wasm": {
            "policy_path": wasm_path
        }
    });

    let payload_string = serde_json::to_string(&payload).unwrap();
    let signature = signing_key.sign(payload_string.as_bytes());

    use base64::Engine;
    let b64_pubkey = base64::prelude::BASE64_STANDARD.encode(public_key.as_bytes());
    let b64_sig = base64::prelude::BASE64_STANDARD.encode(signature.to_bytes());

    Json(json!({
        "bundle_id": "bnd-mcp-authz-002",
        "version": "1.0.4",
        "signature": b64_sig,
        "public_key": b64_pubkey,
        "payload": payload
    }))
}

async fn get_config(Path(device_id): Path<String>) -> Json<Value> {
    info!("CLOUD: Device {} requested config.", device_id);

    // Dynamically resolve WASM path based on debug vs release
    let wasm_path =
        if std::path::Path::new("target/wasm32-wasip1/release/dummy_policy.wasm").exists() {
            "target/wasm32-wasip1/release/dummy_policy.wasm"
        } else {
            "target/wasm32-wasip1/debug/dummy_policy.wasm"
        };

    Json(json!({
        "device_id": device_id,
        "tenant_id": "tenant-production-1",
        "mtls": {
            "client_cert_path": "certs/client.crt",
            "client_key_path": "certs/client.key",
            "root_ca_path": "certs/root_ca.crt"
        },
        "policy_config": {
            "openfga": {
                "endpoint": "http://127.0.0.1:8080",
                "store_id": "store_123"
            },
            "cedar": {
                "policy_src": "permit(\n  principal == User::\"user_bob\",\n  action == Action::\"tools/call\",\n  resource == Resource::\"mcp_tool\"\n);"
            },
            "opa_wasm": {
                "policy_path": wasm_path
            }
        }
    }))
}
