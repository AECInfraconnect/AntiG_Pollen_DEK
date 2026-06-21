use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router as AxumRouter,
};

use dek_mcp_normalizer::{http::HttpTransportAdapter, TransportAdapter};
use dek_openfga::OpenFgaAdapter;
use dek_policy_router::PolicyRouter;
use dek_wasm_host::{PluginHost, WasmtimePluginHost};
use serde_json::{json, Value};
use arc_swap::ArcSwap;
use dek_auth::{extract_bearer, AuthError, Verifier, VerifierConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Clone)]
struct DekMetadata {
    tenant_id: String,
    device_id: String,
    spiffe_id: Option<String>,
}

struct AppState {
    plugin_host: WasmtimePluginHost,
    router: ArcSwap<PolicyRouter>,
    http_adapter: HttpTransportAdapter,
    metadata: ArcSwap<DekMetadata>,
    verifier: ArcSwap<Verifier>,
}

use dek_config::{BootstrapConfig, DekConfig};

#[tokio::main]
async fn main() -> Result<()> {
    dek_config::logging::init_logging("dek-mcp-proxy").unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
    });
    info!("Starting Pollen DEK MCP Proxy...");

    let bootstrap = BootstrapConfig::load_or_default("bootstrap.json")?;
    let device_id = bootstrap.device_id.clone();

    // Attempt to fetch DekConfig to get tenant_id. Default if unreachable.
    let tenant_id = match DekConfig::fetch_from_cloud(&bootstrap, "https://127.0.0.1:43891").await {
        Ok(cfg) => cfg.tenant_id,
        Err(e) => {
            warn!(
                "Failed to fetch DekConfig from cloud, defaulting tenant. Error: {}",
                e
            );
            "default-tenant".to_string()
        }
    };

    // 1. Load initial config (if available) or fallback
    let mut router = PolicyRouter::new();

    // Initialize metadata
    let mut initial_metadata = DekMetadata {
        tenant_id: tenant_id.clone(),
        device_id: device_id.clone(),
        spiffe_id: None,
    };
    let mut initial_verifier_cfg = VerifierConfig::default();

    // Attempt to load from staged bundle first
    let bundle_path_buf = dek_config::paths::get_active_bundle_path();
    let staged_path = std::path::Path::new(&bundle_path_buf);
    if staged_path.exists() {
        if let Ok(content) = std::fs::read_to_string(staged_path) {
            if let Ok(payload) = serde_json::from_str::<Value>(&content) {
                info!("Loaded initial configuration from staged active_bundle.json");
                dek_router_builder::load_router_config(&mut router, &payload);
                if let Some(t) = payload.get("tenant_id").and_then(|v| v.as_str()) {
                    initial_metadata.tenant_id = t.to_string();
                }
                if let Some(s) = payload.get("spiffe_id").and_then(|v| v.as_str()) {
                    initial_metadata.spiffe_id = Some(s.to_string());
                }
                if let Some(jwt_cfg) = payload.get("jwt_config") {
                    if let Some(pem) = jwt_cfg.get("public_key_pem").and_then(|v| v.as_str()) {
                        initial_verifier_cfg.public_key_pem = Some(pem.to_string());
                    }
                    if let Some(jwks_val) = jwt_cfg.get("jwks") {
                        if let Ok(jwks) = serde_json::from_value(jwks_val.clone()) {
                            initial_verifier_cfg.jwks = Some(jwks);
                        }
                    }
                    if let Some(issuer) = jwt_cfg.get("issuer_url").and_then(|v| v.as_str()) {
                        initial_verifier_cfg.issuer = Some(issuer.to_string());
                    }
                    if let Some(aud_val) = jwt_cfg.get("audience") {
                        if let Ok(aud) = serde_json::from_value(aud_val.clone()) {
                            initial_verifier_cfg.audience = Some(aud);
                        }
                    }
                }
            }
        }
    } else {
        // Fallback defaults if no policy config
        if let Ok(adapter) = OpenFgaAdapter::new("http://localhost:8080", "store_123", None) {
            router.register_evaluator("openfga", Box::new(adapter));
        }
        // Removed fallback to Cedar requiring user_bob
    }

    // Determine plugin paths
    let mut plugin_paths = std::collections::HashMap::new();
    
    // Resolve plugins path via standard installation directory or env var
    let base_dir = dek_config::paths::get_plugin_dir().to_string_lossy().into_owned();

    let paths_to_try = vec![
        format!("{}/pii_redactor.wasm", base_dir),
        "target/wasm32-wasip1/release/pii_redactor.wasm".to_string(),
        "target/wasm32-wasip1/debug/pii_redactor.wasm".to_string(),
    ];

    for p in paths_to_try {
        if std::path::Path::new(&p).exists() {
            plugin_paths.insert("pii-redactor".to_string(), p.to_string());
            break;
        }
    }

    let state = Arc::new(AppState {
        plugin_host: WasmtimePluginHost::new(plugin_paths)?,
        router: ArcSwap::from_pointee(router),
        http_adapter: HttpTransportAdapter,
        metadata: ArcSwap::from_pointee(initial_metadata),
        verifier: ArcSwap::from_pointee(Verifier::new(initial_verifier_cfg)),
    });

    // Start background file watcher for hot-reloading
    let state_clone = state.clone();
    tokio::spawn(async move {
        use notify::event::ModifyKind;
        use notify::{EventKind, RecursiveMode, Watcher};
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut watcher = match notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to initialize file watcher: {}", e);
                return;
            }
        };

        let bundle_path_clone = bundle_path_buf.clone();
        let staged_path_local = std::path::Path::new(&bundle_path_clone);
        let parent_dir = staged_path_local.parent().unwrap_or(std::path::Path::new("."));
        if let Err(e) = watcher.watch(parent_dir, RecursiveMode::NonRecursive) {
            error!("Failed to watch target directory: {}", e);
            return;
        }

        info!("Started background file watcher for hot-reloading on {}", staged_path_local.display());

        while let Some(event) = rx.recv().await {
            match event.kind {
                EventKind::Modify(ModifyKind::Data(_))
                | EventKind::Modify(ModifyKind::Any)
                | EventKind::Create(_) => {
                    let path = event.paths.first();
                    if let Some(p) = path {
                        if p.ends_with("active_bundle.json") {
                            info!(
                                "Detected change in active_bundle.json. Attempting hot-reload..."
                            );

                            if let Ok(content) = std::fs::read_to_string(p) {
                                if let Ok(payload) = serde_json::from_str::<Value>(&content) {
                                    let mut new_router = PolicyRouter::new();
                                    // Apply dynamic routing configuration securely
                                    dek_router_builder::load_router_config(
                                        &mut new_router,
                                        &payload,
                                    );

                                    // Safely swap the router
                                    state_clone.router.store(Arc::new(new_router));

                                    let mut metadata_clone = (**state_clone.metadata.load()).clone();
                                    let mut verifier_cfg = VerifierConfig::default();
                                    if let Some(t) =
                                        payload.get("tenant_id").and_then(|v| v.as_str())
                                    {
                                        metadata_clone.tenant_id = t.to_string();
                                    }
                                    if let Some(s) =
                                        payload.get("spiffe_id").and_then(|v| v.as_str())
                                    {
                                        metadata_clone.spiffe_id = Some(s.to_string());
                                    }
                                    if let Some(jwt_cfg) = payload.get("jwt_config") {
                                        if let Some(pem) =
                                            jwt_cfg.get("public_key_pem").and_then(|v| v.as_str())
                                        {
                                            verifier_cfg.public_key_pem =
                                                Some(pem.to_string());
                                        }
                                        if let Some(jwks_val) = jwt_cfg.get("jwks") {
                                            if let Ok(jwks) = serde_json::from_value(jwks_val.clone()) {
                                                verifier_cfg.jwks = Some(jwks);
                                            }
                                        }
                                        if let Some(issuer) = jwt_cfg.get("issuer_url").and_then(|v| v.as_str()) {
                                            verifier_cfg.issuer = Some(issuer.to_string());
                                        }
                                        if let Some(aud_val) = jwt_cfg.get("audience") {
                                            if let Ok(aud) = serde_json::from_value(aud_val.clone()) {
                                                verifier_cfg.audience = Some(aud);
                                            }
                                        }
                                    }
                                    state_clone.metadata.store(Arc::new(metadata_clone));
                                    state_clone.verifier.store(Arc::new(Verifier::new(verifier_cfg)));

                                    info!("Hot-reloaded policies and metadata from disk successfully!");
                                } else {
                                    error!("Failed to parse active_bundle.json payload");
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    let app = AxumRouter::new()
        .route("/mcp", post(handle_mcp_request))
        // Layer 2 Opt-in Proxy Redirect Handlers
        .fallback(handle_forward_proxy)
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:43890").await?;
    info!("MCP Proxy + Forward Proxy listening on http://127.0.0.1:43890");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("MCP Proxy shut down gracefully.");
    Ok(())
}

async fn handle_forward_proxy() -> impl IntoResponse {
    // Basic forward proxy placeholder
    (StatusCode::BAD_GATEWAY, "Forward proxy not yet fully implemented")
}

async fn shutdown_signal() {
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
    info!("Shutdown signal received, starting graceful shutdown");
}

async fn handle_mcp_request(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    info!("Intercepted MCP Request: {}", payload);

    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    
    let metadata = (**state.metadata.load()).clone();
    let verifier = state.verifier.load();

    let token = match extract_bearer(auth_header) {
        Ok(t) => t,
        Err(_) => {
            warn!("missing bearer token");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing bearer token" })),
            )
                .into_response();
        }
    };

    let identity = match verifier.verify(token) {
        Ok(id) => id,
        Err(AuthError::NoKeyConfigured) => {
            warn!("auth not configured");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Auth not configured" })),
            )
                .into_response();
        }
        Err(e) => {
            warn!("jwt verification failed: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid token" })),
            )
                .into_response();
        }
    };

    let principal = identity.principal;
    let jwt_tenant_id = identity.tenant_id;

    let final_tenant_id = jwt_tenant_id.unwrap_or(metadata.tenant_id);

    // Normalize Event
    let normalized = match state.http_adapter.normalize_request(
        payload.clone(),
        &final_tenant_id,
        &metadata.device_id,
        metadata.spiffe_id.as_deref(),
        Some(&principal),
    ) {
        Ok(n) => n,
        Err(_) => {
            error!("Failed to normalize request");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Failed to normalize request" })),
            )
                .into_response();
        }
    };

    let mut policy_input = serde_json::to_value(&normalized).unwrap_or(json!({}));
    // Provide backwards compatibility for existing mock PDPs
    policy_input["action"] = json!(normalized.tool_name.unwrap_or(normalized.request_type));
    policy_input["principal"] = json!(principal);
    policy_input["resource"] = json!("mcp_tool");

    // Evaluate against the Adaptive Policy Pipeline
    let decision_result = state.router.load().authorize(policy_input).await;

    match decision_result {
        Ok(decision) => {
            info!("Final Pipeline Decision: {:?}", decision);
            if decision.allow {
                let response = json!({
                    "status": "allowed",
                    "message": "The request passed the PEP evaluation.",
                    "decision": decision
                });

                // Apply PII redaction plugin if required
                if let Ok(redacted) = state.plugin_host.invoke("pii-redactor", response.clone()) {
                    info!("Applied PII redaction plugin successfully.");
                    (StatusCode::OK, Json(redacted)).into_response()
                } else {
                    (StatusCode::OK, Json(response)).into_response()
                }
            } else {
                (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "status": "denied",
                        "reason": decision.reason,
                        "decision": decision
                    })),
                )
                    .into_response()
            }
        }
        Err(e) => {
            error!("Policy Evaluation Error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "denied",
                    "reason": "policy_evaluation_error"
                })),
            )
                .into_response()
        }
    }
}
