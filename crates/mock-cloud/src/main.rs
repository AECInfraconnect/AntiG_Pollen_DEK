use anyhow::{Context, Result};
use axum::{
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use rustls::{server::WebPkiClientVerifier, RootCertStore, ServerConfig};
use rustls_pemfile::{certs, private_key};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

// Static ed25519 seed used to sign policy bundles. The pinned_bundle_public_key
// returned at /enroll is derived from this so the enrolled DEK pins the right key.
const BUNDLE_SEED: [u8; 32] = [
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
];

fn bundle_pubkey_b64() -> String {
    let sk = SigningKey::from_bytes(&BUNDLE_SEED);
    base64::prelude::BASE64_STANDARD.encode(sk.verifying_key().as_bytes())
}

#[derive(Clone)]
struct AppState {
    revision: Arc<AtomicUsize>,
    rsa_public_key_pem: String,
    /// device_code -> poll count (drives the authorization_pending -> granted flow)
    pending: Arc<Mutex<HashMap<String, u32>>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    tracing_subscriber::fmt::init();
    info!("Starting Mock Pollen Cloud (mTLS API :43891 + HTTPS Enrollment :43892)...");

    let mut rng = rand_core::OsRng;
    let priv_key = rsa::RsaPrivateKey::new(&mut rng, 2048).expect("rsa keygen");
    let pub_key = rsa::RsaPublicKey::from(&priv_key);
    let rsa_public_key_pem =
        rsa::pkcs8::EncodePublicKey::to_public_key_pem(&pub_key, rsa::pkcs8::LineEnding::LF)
            .expect("encode pub key");

    let state = AppState {
        revision: Arc::new(AtomicUsize::new(1)),
        rsa_public_key_pem,
        pending: Arc::new(Mutex::new(HashMap::new())),
    };

    // ---- mTLS API (post-enrollment): config / bundles / telemetry ----
    let api = Router::new()
        .route("/v1/tenants/:tenant_id/devices/:device_id/telemetry", post(ingest_telemetry))
        .route("/v1/tenants/:tenant_id/devices/:device_id/bundles/latest", get(get_latest_bundle))
        .route("/v1/tenants/:tenant_id/devices/:device_id/config", get(get_config))
        .route("/v1/tenants/:tenant_id/devices/:device_id/spire/svid/renew", post(renew_csr))
        .route("/v1/tenants/:tenant_id/devices/:device_id/decision-logs", post(ingest_decision_logs))
        .route("/v1/tenants/:tenant_id/devices/:device_id/health", post(report_health))
        .with_state(state.clone());

    // ---- Enrollment listener (PRE-identity, NO client cert) ----
    let enroll = Router::new()
        .route("/oauth/device_authorization", post(device_authorization))
        .route("/oauth/token", post(token))
        .route("/enroll", post(enroll_device))
        .route("/spire/node/attest", post(attest_csr)) // join-token + CSR -> X.509-SVID
        .route("/device", get(device_page))
        .with_state(state.clone());

    // Load server certificate and key
    let certs_der = load_certs("../../certs/server.crt")?;
    let key_der = load_private_key("../../certs/server.key")?;

    // ---- :43891 mTLS Config ----
    let mut root_cert_store = RootCertStore::empty();
    let ca_certs = load_certs("../../certs/root_ca.crt")?;
    root_cert_store.add_parsable_certificates(ca_certs);
    let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_cert_store))
        .build()
        .context("build client verifier")?;

    let mut server_config_mtls = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs_der.clone(), key_der.clone_key())
        .context("server config mtls")?;
    server_config_mtls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let rustls_config_mtls = RustlsConfig::from_config(Arc::new(server_config_mtls));
    let addr_mtls = SocketAddr::from(([127, 0, 0, 1], 43891));

    // ---- :43892 HTTPS Self-Signed Config ----
    let mut server_config_https = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs_der, key_der)
        .context("server config https")?;
    server_config_https.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let rustls_config_https = RustlsConfig::from_config(Arc::new(server_config_https));
    let addr_https = SocketAddr::from(([127, 0, 0, 1], 43892));

    info!("Mock Cloud mTLS API on https://127.0.0.1:43891");
    info!("Mock Cloud HTTPS Enrollment API on https://127.0.0.1:43892");

    let handle = Handle::new();
    let shutdown_handle = handle.clone();

    tokio::spawn(async move {
        let ctrl_c = async { tokio::signal::ctrl_c().await.expect("ctrl-c") };
        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("signal").recv().await;
        };
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();
        tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
        info!("shutting down...");
        shutdown_handle.graceful_shutdown(None);
    });

    let mtls_server = axum_server::bind_rustls(addr_mtls, rustls_config_mtls)
        .handle(handle.clone())
        .serve(api.into_make_service());

    let https_server = axum_server::bind_rustls(addr_https, rustls_config_https)
        .handle(handle)
        .serve(enroll.into_make_service());

    let _ = tokio::try_join!(mtls_server, https_server)?;
    info!("Mock Cloud shut down gracefully.");
    Ok(())
}

// =========================== Enrollment handlers ===========================

#[derive(Deserialize)]
struct DeviceAuthForm {
    #[allow(dead_code)]
    client_id: Option<String>,
    #[allow(dead_code)]
    scope: Option<String>,
}

async fn device_authorization(
    State(state): State<AppState>,
    Form(_form): Form<DeviceAuthForm>,
) -> Json<Value> {
    let device_code = rand_hex(16);
    let user_code = rand_user_code();
    state.pending.lock().unwrap().insert(device_code.clone(), 0);
    info!("CLOUD: device_authorization -> user_code {}", user_code);
    Json(json!({
        "device_code": device_code,
        "user_code": user_code,
        "verification_uri": "https://127.0.0.1:43892/device",
        "verification_uri_complete": format!("https://127.0.0.1:43892/device?code={}", user_code),
        "expires_in": 300,
        "interval": 1
    }))
}

#[derive(Deserialize)]
struct TokenForm {
    #[allow(dead_code)]
    grant_type: Option<String>,
    device_code: Option<String>,
    #[allow(dead_code)]
    client_id: Option<String>,
}

async fn token(
    State(state): State<AppState>,
    Form(form): Form<TokenForm>,
) -> (StatusCode, Json<Value>) {
    let dc = form.device_code.unwrap_or_default();
    let mut m = state.pending.lock().unwrap();
    match m.get_mut(&dc) {
        None => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "expired_token" })),
        ),
        Some(count) => {
            *count += 1;
            // Simulate the user taking a moment: pending on the first poll,
            // granted from the second onward.
            if *count < 2 {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "authorization_pending" })),
                )
            } else {
                m.remove(&dc);
                info!("CLOUD: token granted for device_code {}", dc);
                (
                    StatusCode::OK,
                    Json(json!({
                        "access_token": format!("mock-access-{}", dc),
                        "token_type": "Bearer",
                        "expires_in": 3600
                    })),
                )
            }
        }
    }
}

async fn enroll_device(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    // Mock accepts any Bearer token; real cloud validates it.
    let has_bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|h| h.starts_with("Bearer "))
        .unwrap_or(false);
    if !has_bearer {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "missing bearer token" })),
        );
    }

    let trust_bundle = std::fs::read_to_string("../../certs/root_ca.crt").unwrap_or_default();
    let device_id = "device-001";
    let join_token = rand_hex(16);
    info!("CLOUD: enroll -> issuing join_token for {}", device_id);

    (
        StatusCode::OK,
        Json(json!({
            "join_token": join_token,
            // join-token attestation runs on the same HTTPS listener (pre-identity)
            "spire_endpoint": "https://127.0.0.1:43892/spire",
            "trust_bundle_pem": trust_bundle,
            "pinned_bundle_public_key": bundle_pubkey_b64(),
            "tenant_id": "tenant-production-1",
            "device_id": device_id,
            "spiffe_id": format!("spiffe://pollen.cloud/tenant-production-1/device/{}", device_id),
            // base URL for the post-enrollment mTLS API
            "cloud_url": "https://127.0.0.1:43891"
        })),
    )
}

#[derive(Deserialize)]
struct JoinAttest {
    #[allow(dead_code)]
    join_token: String,
    device_id: String,
    csr_pem: String,
}

async fn attest_csr(Json(req): Json<JoinAttest>) -> (StatusCode, Json<Value>) {
    let spiffe_id = format!(
        "spiffe://pollen.cloud/tenant-production-1/device/{}",
        req.device_id
    );
    match sign_csr(&req.csr_pem, &spiffe_id) {
        Ok((cert_pem, trust_bundle)) => {
            info!("CLOUD: signed X.509-SVID for {}", spiffe_id);
            (
                StatusCode::OK,
                Json(json!({
                    "svid_cert_pem": cert_pem,
                    "spiffe_id": spiffe_id,
                    "trust_bundle_pem": trust_bundle
                })),
            )
        }
        Err(e) => {
            warn!("CLOUD: CSR signing failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("csr signing failed: {e}") })),
            )
        }
    }
}

async fn renew_csr(Path((_tenant_id, _device_id)): Path<(String, String)>, Json(req): Json<JoinAttest>) -> (StatusCode, Json<Value>) {
    let spiffe_id = format!(
        "spiffe://pollen.cloud/tenant-production-1/device/{}",
        req.device_id
    );
    match sign_csr(&req.csr_pem, &spiffe_id) {
        Ok((cert_pem, trust_bundle)) => {
            info!("CLOUD: renewed X.509-SVID for {}", spiffe_id);
            (
                StatusCode::OK,
                Json(json!({
                    "svid_cert_pem": cert_pem,
                    "spiffe_id": spiffe_id,
                    "trust_bundle_pem": trust_bundle
                })),
            )
        }
        Err(e) => {
            warn!("CLOUD: CSR renewal failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("csr renewal failed: {e}") })),
            )
        }
    }
}

/// Sign the client's CSR with the root CA.
/// Uses rcgen 0.11 API to match cert-gen.
fn sign_csr(csr_pem: &str, spiffe_id: &str) -> Result<(String, String)> {
    use rcgen::{Certificate, CertificateParams, CertificateSigningRequest, KeyPair, SanType};

    let ca_key_pem =
        std::fs::read_to_string("../../certs/root_ca.key").context("read root_ca.key")?;
    let ca_cert_pem =
        std::fs::read_to_string("../../certs/root_ca.crt").context("read root_ca.crt")?;

    let ca_key = KeyPair::from_pem(&ca_key_pem).context("parse CA key")?;
    let ca_params =
        CertificateParams::from_ca_cert_pem(&ca_cert_pem, ca_key).context("CA params")?;
    let ca = Certificate::from_params(ca_params).context("CA cert")?;

    let mut csr = CertificateSigningRequest::from_pem(csr_pem).context("parse CSR")?;
    // Server controls identity: stamp the SPIFFE ID as a URI SAN.
    csr.params.subject_alt_names.push(SanType::URI(spiffe_id.to_string()));

    let cert_pem = csr.serialize_pem_with_signer(&ca).context("sign CSR")?;

    Ok((cert_pem, ca_cert_pem))
}

async fn device_page() -> axum::response::Html<&'static str> {
    axum::response::Html(
        "<h2>Pollen Cloud — Device Approval (mock)</h2>\
         <p>In the real product you'd log in and approve here. \
         The mock auto-approves on the second token poll.</p>",
    )
}

// =============================== helpers ===============================

fn rand_hex(n_bytes: usize) -> String {
    use rand_core::RngCore;
    let mut b = vec![0u8; n_bytes];
    rand_core::OsRng.fill_bytes(&mut b);
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

fn rand_user_code() -> String {
    use rand_core::RngCore;
    const ALPHA: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ"; // no vowels/ambiguous
    let mut b = [0u8; 8];
    rand_core::OsRng.fill_bytes(&mut b);
    let c: String = b.iter().map(|x| ALPHA[(*x as usize) % ALPHA.len()] as char).collect();
    format!("{}-{}", &c[0..4], &c[4..8])
}

// =================== existing handlers (unchanged) ===================

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    Ok(certs(&mut reader).collect::<Result<Vec<_>, _>>()?)
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    Ok(private_key(&mut reader)?.context("No private key found")?)
}

async fn ingest_telemetry(Path((_tenant_id, _device_id)): Path<(String, String)>, Json(payload): Json<Value>) -> Json<Value> {
    info!("CLOUD RECEIVED TELEMETRY: {}", payload);
    Json(json!({ "status": "ingested" }))
}

async fn ingest_decision_logs(Path((_tenant_id, _device_id)): Path<(String, String)>, Json(payload): Json<Value>) -> Json<Value> {
    info!("CLOUD RECEIVED DECISION LOGS: {}", payload);
    Json(json!({ "status": "ingested" }))
}

async fn report_health(Path((_tenant_id, _device_id)): Path<(String, String)>, Json(payload): Json<Value>) -> Json<Value> {
    info!("CLOUD RECEIVED HEALTH REPORT: {}", payload);
    Json(json!({ "status": "ok" }))
}

async fn get_latest_bundle(Path((_tenant_id, _device_id)): Path<(String, String)>, State(state): State<AppState>) -> Json<Value> {
    let rev = state.revision.fetch_add(1, Ordering::SeqCst);
    let signing_key = SigningKey::from_bytes(&BUNDLE_SEED);
    let public_key = signing_key.verifying_key();

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
            "public_key_pem": state.rsa_public_key_pem.clone(),
            "issuer_url": "https://127.0.0.1:43891",
            "audience": ["pollen-dek"]
        },
        "openfga": { "endpoint": "http://127.0.0.1:8080", "store_id": store_id },
        "cedar": { "policy_src": format!("permit(\n  principal == User::\"user_bob\",\n  action == Action::\"tools/call\",\n  resource == Resource::\"mcp_tool\"\n); // rev {}", rev) },
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

    let payload_string = serde_jcs::to_string(&payload).unwrap();
    let signature = signing_key.sign(payload_string.as_bytes());
    Json(json!({
        "bundle_id": format!("bnd-mcp-authz-{:03}", rev),
        "version": format!("1.0.{}", rev),
        "signature": base64::prelude::BASE64_STANDARD.encode(signature.to_bytes()),
        "public_key": base64::prelude::BASE64_STANDARD.encode(public_key.as_bytes()),
        "payload": payload
    }))
}

async fn get_config(Path((_tenant_id, device_id)): Path<(String, String)>, State(state): State<AppState>) -> Json<Value> {
    let rev = state.revision.fetch_add(1, Ordering::SeqCst);
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
        "mtls": { "client_cert_path": "certs/client.crt", "client_key_path": "certs/client.key", "root_ca_path": "certs/root_ca.crt" },
        "spire_server": { "endpoint": "https://127.0.0.1:43891/spire" },
        "jwt_config": { "public_key_pem": state.rsa_public_key_pem.clone(), "issuer_url": "https://127.0.0.1:43891", "audience": ["pollen-dek"] },
        "policy_config": {
            "openfga": { "endpoint": "http://127.0.0.1:8080", "store_id": store_id },
            "cedar": { "policy_src": format!("permit(\n  principal == User::\"user_bob\",\n  action == Action::\"tools/call\",\n  resource == Resource::\"mcp_tool\"\n); // rev {}", rev) },
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
        }
    }))
}
