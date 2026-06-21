//! state.rs — P2 refactor of dek-mcp-proxy shared state.
//!
//! Replaces `RwLock<DekMetadata>` + `RwLock<PolicyRouter>` (cloned on every
//! request — line ~292 cloned the whole struct incl. the JwkSet) with
//! `ArcSwap`, giving lock-free reads (cheap Arc clone) and atomic hot-reload.
//! The JWT verifier is rebuilt once per bundle reload, not per request.
//!
//! Cargo.toml (dek-mcp-proxy):
//!     arc-swap = "1"
//!     dek-auth = { path = "../dek-auth" }
//!     # jsonwebtoken can be dropped here once all JWT logic lives in dek-auth

use arc_swap::{ArcSwap, ArcSwapOption};
use dek_auth::{Verifier, VerifierConfig};
use dek_policy_router::PolicyRouter;
use dek_wasm_host::WasmtimePluginHost;
use dek_mcp_normalizer::http::HttpTransportAdapter;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct DekMetadata {
    pub tenant_id: String,
    pub device_id: String,
    pub spiffe_id: Option<String>,
    pub jwt_public_key_pem: Option<String>,
    pub jwks: Option<jsonwebtoken::jwk::JwkSet>,
    pub issuer_url: Option<String>,
    /// NEW: enforced audience(s). Wire this from bundle `jwt_config.audience`.
    pub audience: Option<Vec<String>>,
}

/// One immutable snapshot of everything that hot-reload swaps together.
/// Building these as a unit keeps router/metadata/verifier consistent.
pub struct PolicySnapshot {
    pub router: PolicyRouter,
    pub metadata: DekMetadata,
    pub verifier: Verifier,
}

impl PolicySnapshot {
    pub fn build(router: PolicyRouter, metadata: DekMetadata) -> Self {
        let verifier = Verifier::new(VerifierConfig {
            jwks: metadata.jwks.clone(),
            public_key_pem: metadata.jwt_public_key_pem.clone(),
            issuer: metadata.issuer_url.clone(),
            audience: metadata.audience.clone(),
            leeway_secs: 60,
        });
        Self { router, metadata, verifier }
    }
}
use dek_telemetry::CloudTelemetrySink;

pub struct AppState {
    pub plugin_host: WasmtimePluginHost,
    pub http_adapter: HttpTransportAdapter,
    /// Lock-free, atomically swappable policy snapshot.
    pub snapshot: ArcSwap<PolicySnapshot>,
    /// Shadow snapshot for parallel evaluation.
    pub shadow_snapshot: ArcSwapOption<PolicySnapshot>,
    /// Telemetry Sink
    pub telemetry: Option<Arc<CloudTelemetrySink>>,
}

impl AppState {
    pub fn new(
        plugin_host: WasmtimePluginHost,
        http_adapter: HttpTransportAdapter,
        initial: PolicySnapshot,
        telemetry: Option<Arc<CloudTelemetrySink>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            plugin_host,
            http_adapter,
            snapshot: ArcSwap::from_pointee(initial),
            shadow_snapshot: ArcSwapOption::empty(),
            telemetry,
        })
    }

    /// Atomic hot-reload — called by the bundle file-watcher.
    pub fn reload(&self, snapshot: PolicySnapshot) {
        self.snapshot.store(Arc::new(snapshot));
    }

    /// Atomic hot-reload of the shadow bundle.
    pub fn reload_shadow(&self, snapshot: PolicySnapshot) {
        self.shadow_snapshot.store(Some(Arc::new(snapshot)));
    }
}

// ---------------------------------------------------------------------------
// Handler usage (sketch) — note: NO per-request clone of metadata/JWKS.
// ---------------------------------------------------------------------------
//
// use axum::http::StatusCode;
// use dek_auth::{extract_bearer, AuthError};
//
// async fn handle_mcp_request(
//     State(state): State<Arc<AppState>>,
//     headers: HeaderMap,
//     Json(payload): Json<Value>,
// ) -> Response {
//     let snap = state.snapshot.load();             // Arc clone, lock-free, O(1)
//
//     // ---- AuthN via dek-auth (exp/aud enforced inside) ----
//     let token = match extract_bearer(headers.get("authorization").and_then(|h| h.to_str().ok())) {
//         Ok(t) => t,
//         Err(_) => return (StatusCode::UNAUTHORIZED, "missing bearer token").into_response(),
//     };
//     let identity = match snap.verifier.verify(token) {
//         Ok(id) => id,
//         Err(AuthError::NoKeyConfigured) =>
//             return (StatusCode::INTERNAL_SERVER_ERROR, "auth not configured").into_response(),
//         Err(e) => {
//             warn!("jwt verification failed: {e}");
//             return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
//         }
//     };
//
//     let tenant_id = identity.tenant_id.unwrap_or_else(|| snap.metadata.tenant_id.clone());
//     let principal = identity.principal;
//
//     // normalize -> route using snap.router (no lock held; snap is an Arc)
//     let decision = snap.router.route_and_evaluate(/* normalized event */).await;
//     // ... unchanged from here ...
//     todo!()
// }
//
// ---------------------------------------------------------------------------
// Bundle watcher reload (sketch) — build a fresh snapshot, then atomic swap.
// ---------------------------------------------------------------------------
//
//   let mut router = PolicyRouter::new();
//   dek_router_builder::load_router_config(&mut router, &payload);   // existing
//   let metadata = parse_metadata_from_bundle(&payload);            // incl. audience
//   state.reload(PolicySnapshot::build(router, metadata));          // lock-free swap
//
// stdio-wrapper: use the same dek_auth::Verifier + PolicySnapshot pattern so the
// two PEPs enforce identically (kills the duplicated JWT/bundle-load logic, B3).
