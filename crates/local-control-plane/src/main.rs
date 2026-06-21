use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use dek_control_plane_api::identity::ControlPlaneIdentity;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

mod bundle;
mod policy;
mod registry;
mod signing;
mod store;
mod telemetry;

use signing::LocalSigner;


#[derive(Clone)]
pub struct AppState {
    pub identity: ControlPlaneIdentity,
    pub registry_store: Arc<dyn store::RegistryStore>,
    pub policy_store: Arc<dyn store::RegistryStore>,
    pub telemetry_store: Arc<dyn store::TelemetryStore>,
    pub signer: Arc<LocalSigner>,
    pub build_number: Arc<AtomicU64>,
}

pub async fn local_tenant_guard(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if state.identity.tenant_id == "local" && tenant_id != "local" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "Local Admin Dashboard only supports tenant_id=local"}),
            ),
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // SQLite local store
    let db_url = std::env::var("DEK_LCP_DB")
        .unwrap_or_else(|_| "sqlite://./pollen-local.db?mode=rwc".to_string());
    let store = Arc::new(store::SqliteStore::new(&db_url).await?);

    let data_dir = std::path::PathBuf::from(
        std::env::var("DEK_LCP_DATA").unwrap_or_else(|_| "./pollen-local-data".into()));
    let signer = Arc::new(LocalSigner::load_or_create(&data_dir)?);
    info!("local control-plane signing key: {} (pub {})", signer.key_id, signer.public_key_b64());

    let state = AppState {
        identity: ControlPlaneIdentity::local_default(),
        registry_store: store.clone(),
        policy_store: store.clone(),
        telemetry_store: store,
        signer,
        build_number: Arc::new(AtomicU64::new(1)),
    };

    let static_dir = std::env::var("DEK_DASHBOARD_DIR")
        .unwrap_or_else(|_| "../../apps/local-admin-dashboard/dist".to_string());

    let app = Router::new()
        .merge(registry::router())
        .merge(policy::router())
        .merge(telemetry::router())
        .merge(bundle_routes())
        .fallback_service(
            ServeDir::new(&static_dir)
                .not_found_service(ServeFile::new(format!("{}/index.html", static_dir))),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    info!("Local Control Plane listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}

pub fn bundle_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/devices/:device/bundles/manifest", axum::routing::get(get_manifest))
        .route("/v1/tenants/:tenant/devices/:device/bundles/artifacts/:sha", axum::routing::get(get_artifact))
        .route("/v1/tenants/:tenant/devices/:device/trusted-keys", axum::routing::get(get_trusted_keys))
        .route("/v1/tenants/:tenant/devices/:device/config", axum::routing::get(get_mock_config))
}

async fn get_mock_config(
    axum::extract::Path((tenant, _device)): axum::extract::Path<(String, String)>,
    axum::extract::State(st): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    let mut combined_cedar = String::new();

    if let Ok(Some(manifest_val)) = st.policy_store.get_policy_raw(&tenant, "bundle:latest").await {
        if let Ok(manifest) = serde_json::from_value::<dek_control_plane_api::bundle::PollenPolicyBundleManifest>(manifest_val) {
            for artifact in manifest.artifacts {
                if artifact.adapter_id == "cedar" {
                    if let Ok(Some(bytes)) = st.policy_store.get_blob(&tenant, &artifact.path).await {
                        if let Ok(text) = String::from_utf8(bytes) {
                            combined_cedar.push_str(&text);
                            combined_cedar.push('\n');
                        }
                    }
                }
            }
        }
    }

    axum::Json(serde_json::json!({
        "device_id": "device-001",
        "tenant_id": tenant,
        "mtls": {
            "root_ca_path": "certs/root_ca.crt",
            "client_cert_path": "certs/device.crt",
            "client_key_path": "certs/device.key"
        },
        "policy_config": {
            "mode": "strict_enforce",
            "fail_closed": true,
            "cedar": {
                "policy_src": combined_cedar
            },
            "routes": [
                {
                    "id": "route_default",
                    "priority": 10,
                    "match_rule": { "method": "*", "tool_category": null },
                    "pdp_required": ["cedar"],
                    "pdp_conditional": []
                }
            ]
        }
    }))
}

async fn get_trusted_keys(State(st): State<AppState>) -> impl axum::response::IntoResponse {
    Json(serde_json::json!({ "keys": [{
        "key_id": st.signer.key_id, "public_b64": st.signer.public_key_b64(),
        "status": "active", "not_before_unix": 0, "not_after_unix": 0
    }]}))
}

async fn get_manifest(Path((tenant, _device)): Path<(String, String)>, State(st): State<AppState>) -> impl axum::response::IntoResponse {
    match st.policy_store.get_policy_raw(&tenant, "bundle:latest").await {
        Ok(Some(val)) => (StatusCode::OK, Json(val)),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"no bundle"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_artifact(Path((tenant, _device, sha)): Path<(String, String, String)>, State(st): State<AppState>) -> impl axum::response::IntoResponse {
    if sha == "network_guardrails.json" {
        let signed_bytes = serde_json::to_vec(&serde_json::json!([])).unwrap();
        let sig_b64 = st.signer.sign_b64(&signed_bytes);
        let signed_payload = serde_json::json!({
            "signed": [],
            "signatures": [{
                "signature_id": st.signer.key_id,
                "signature_type": "ed25519",
                "payload": sig_b64,
                "public_key_fingerprint": st.signer.public_key_b64(),
            }]
        });
        return (axum::http::StatusCode::OK, serde_json::to_vec(&signed_payload).unwrap());
    }
    
    let path = format!("artifacts/{sha}");
    match st.policy_store.get_blob(&tenant, &path).await {
        Ok(Some(bytes)) => (axum::http::StatusCode::OK, bytes),
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, vec![]),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, vec![]),
    }
}
