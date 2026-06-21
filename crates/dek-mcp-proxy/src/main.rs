use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router as AxumRouter,
};
use base64::{engine::general_purpose, Engine as _};
use dek_cedar::CedarAdapter;
use dek_mcp_normalizer::{http::HttpTransportAdapter, TransportAdapter};
use dek_openfga::OpenFgaAdapter;
use dek_policy_router::{ConditionalPdp, MatchRule, PolicyRouter, Route};
use dek_wasm_host::{PluginHost, WasmtimePluginHost};
use serde_json::{json, Value};
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
    router: RwLock<PolicyRouter>,
    http_adapter: HttpTransportAdapter,
    metadata: RwLock<DekMetadata>,
}

use dek_config::{BootstrapConfig, DekConfig};

use dek_config::MtlsConfig;

fn load_router_config(router: &mut PolicyRouter, payload: &Value) {
    let mtls: Option<MtlsConfig> = payload.get("mtls").and_then(|v| serde_json::from_value(v.clone()).ok());

    if let Some(openfga) = payload.get("openfga") {
        let endpoint = openfga
            .get("endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:8080");
        let store_id = openfga
            .get("store_id")
            .and_then(|v| v.as_str())
            .unwrap_or("store_123");
        
        match OpenFgaAdapter::new(endpoint, store_id, mtls.as_ref()) {
            Ok(adapter) => router.register_evaluator("openfga", Box::new(adapter)),
            Err(e) => error!("Failed to initialize OpenFGA Adapter with mTLS: {}", e),
        }
    }
    if let Some(cedar) = payload.get("cedar") {
        let policy_src = cedar
            .get("policy_src")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        router.register_evaluator("cedar", Box::new(CedarAdapter::new(policy_src)));
    }
    if let Some(wasm) = payload.get("opa_wasm") {
        let policy_path = wasm
            .get("policy_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if std::path::Path::new(policy_path).exists() {
            if let Ok(runtime) = dek_policy_runtime::WasmtimePolicyRuntime::new(policy_path) {
                router.register_evaluator("opa_wasm", Box::new(runtime));
            } else {
                error!("Failed to initialize WASM Policy Runtime for path: {}", policy_path);
            }
        } else {
            error!("WASM policy file not found at: {}", policy_path);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
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

    // Attempt to load from staged bundle first
    let staged_path = std::path::Path::new("target/active_bundle.json");
    if staged_path.exists() {
        if let Ok(content) = std::fs::read_to_string(staged_path) {
            if let Ok(payload) = serde_json::from_str::<Value>(&content) {
                info!("Loaded initial configuration from staged active_bundle.json");
                load_router_config(&mut router, &payload);
                if let Some(t) = payload.get("tenant_id").and_then(|v| v.as_str()) {
                    initial_metadata.tenant_id = t.to_string();
                }
                if let Some(s) = payload.get("spiffe_id").and_then(|v| v.as_str()) {
                    initial_metadata.spiffe_id = Some(s.to_string());
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

    // Define initial routes
    let routes = vec![
        Route {
            id: "route_tools_call".to_string(),
            priority: 100,
            match_rule: MatchRule {
                method: Some("tools/call".to_string()),
                tool_category: None,
            },
            pdp_required: vec!["openfga".to_string(), "opa_wasm".to_string()],
            pdp_conditional: vec![ConditionalPdp {
                evaluator: "cedar".to_string(),
                required_payload_key: "*".to_string(), // Always run cedar for now
            }],
        },
        Route {
            id: "route_default".to_string(),
            priority: 10,
            match_rule: MatchRule {
                method: Some("*".to_string()),
                tool_category: None,
            },
            pdp_required: vec!["openfga".to_string()],
            pdp_conditional: vec![],
        },
    ];

    router.set_routes(routes);

    // Determine plugin paths
    let mut plugin_paths = std::collections::HashMap::new();
    let paths_to_try = vec![
        "target/wasm32-wasip1/release/pii_redactor.wasm",
        "target/wasm32-wasip1/debug/pii_redactor.wasm",
        "../../target/wasm32-wasip1/release/pii_redactor.wasm",
        "../../target/wasm32-wasip1/debug/pii_redactor.wasm",
    ];
    for p in paths_to_try {
        if std::path::Path::new(p).exists() {
            plugin_paths.insert("pii-redactor".to_string(), p.to_string());
            break;
        }
    }

    let state = Arc::new(AppState {
        plugin_host: WasmtimePluginHost::new(plugin_paths)?,
        router: RwLock::new(router),
        http_adapter: HttpTransportAdapter,
        metadata: RwLock::new(initial_metadata),
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

        if let Err(e) = watcher.watch(std::path::Path::new("target"), RecursiveMode::NonRecursive) {
            error!("Failed to watch target directory: {}", e);
            return;
        }

        info!("Started background file watcher for hot-reloading on target/active_bundle.json");

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
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await; // wait for write to finish

                            if let Ok(content) = std::fs::read_to_string(p) {
                                if let Ok(payload) = serde_json::from_str::<Value>(&content) {
                                    let mut new_router = PolicyRouter::new();
                                    load_router_config(&mut new_router, &payload);

                                    // Preserve routes for now
                                    let routes = vec![
                                        Route {
                                            id: "route_tools_call".to_string(),
                                            priority: 100,
                                            match_rule: MatchRule {
                                                method: Some("tools/call".to_string()),
                                                tool_category: None,
                                            },
                                            pdp_required: vec![
                                                "openfga".to_string(),
                                                "opa_wasm".to_string(),
                                            ],
                                            pdp_conditional: vec![ConditionalPdp {
                                                evaluator: "cedar".to_string(),
                                                required_payload_key: "*".to_string(),
                                            }],
                                        },
                                        Route {
                                            id: "route_default".to_string(),
                                            priority: 10,
                                            match_rule: MatchRule {
                                                method: Some("*".to_string()),
                                                tool_category: None,
                                            },
                                            pdp_required: vec!["openfga".to_string()],
                                            pdp_conditional: vec![],
                                        },
                                    ];
                                    new_router.set_routes(routes);

                                    // Hot swap
                                    let mut current_router = state_clone.router.write().await;
                                    *current_router = new_router;

                                    let mut metadata_lock = state_clone.metadata.write().await;
                                    if let Some(t) = payload.get("tenant_id").and_then(|v| v.as_str()) {
                                        metadata_lock.tenant_id = t.to_string();
                                    }
                                    if let Some(s) = payload.get("spiffe_id").and_then(|v| v.as_str()) {
                                        metadata_lock.spiffe_id = Some(s.to_string());
                                    }

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
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:43890").await?;
    info!("MCP Proxy listening on http://127.0.0.1:43890/mcp");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("MCP Proxy shut down gracefully.");
    Ok(())
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

    // JWT Extraction
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let mut jwt_tenant_id = None;
    let principal = if let Some(token) = auth_header {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() == 3 {
            // JWT payload is URL_SAFE_NO_PAD
            let decoded_opt = general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
                .or_else(|_| general_purpose::URL_SAFE.decode(parts[1]));

            if let Ok(decoded) = decoded_opt {
                if let Ok(claims) = serde_json::from_slice::<Value>(&decoded) {
                    jwt_tenant_id = claims.get("tenant_id").or(claims.get("tenant")).and_then(|s| s.as_str()).map(|s| s.to_string());
                    claims.get("sub").and_then(|s| s.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let principal = match principal {
        Some(p) => p,
        None => {
            warn!("Missing or invalid JWT token in Authorization header");
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid JWT token in Authorization header" })),
            )
                .into_response();
        }
    };

    let metadata = state.metadata.read().await.clone();
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
    let decision_result = state.router.read().await.authorize(policy_input).await;

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
