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
use dek_activation::snapshot::{RuntimeSnapshot, DekMetadata};
use dek_wasm_host::WasmtimePluginHost;
use dek_mcp_normalizer::http::HttpTransportAdapter;
use std::sync::Arc;
use dek_telemetry::CloudTelemetrySink;

pub struct AppState {
    pub http_adapter: HttpTransportAdapter,
    /// Lock-free, atomically swappable policy snapshot.
    pub snapshot: ArcSwap<RuntimeSnapshot>,
    /// Shadow snapshot for parallel evaluation.
    pub shadow_snapshot: ArcSwapOption<RuntimeSnapshot>,
    /// Telemetry Sink
    pub telemetry: Option<Arc<CloudTelemetrySink>>,
}

impl AppState {
    pub fn new(
        http_adapter: HttpTransportAdapter,
        initial: RuntimeSnapshot,
        telemetry: Option<Arc<CloudTelemetrySink>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            http_adapter,
            snapshot: ArcSwap::from_pointee(initial),
            shadow_snapshot: ArcSwapOption::empty(),
            telemetry,
        })
    }

    /// Atomic hot-reload.
    pub fn reload(&self, snapshot: RuntimeSnapshot) {
        self.snapshot.store(Arc::new(snapshot));
    }

    /// Atomic hot-reload of the shadow bundle.
    pub fn reload_shadow(&self, snapshot: RuntimeSnapshot) {
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
