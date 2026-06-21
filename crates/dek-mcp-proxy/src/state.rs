use std::sync::Arc;
use arc_swap::ArcSwap;
use dek_auth::Verifier;
use dek_policy_router::PolicyRouter;
use dek_mcp_normalizer::http::HttpTransportAdapter;
use dek_wasm_host::WasmtimePluginHost;

#[derive(Clone)]
pub struct DekMetadata {
    pub tenant_id: String,
    pub device_id: String,
    pub spiffe_id: Option<String>,
}

pub struct PolicySnapshot {
    pub metadata: DekMetadata,
    pub verifier: Verifier,
    pub router: PolicyRouter,
}

pub struct AppState {
    pub plugin_host: WasmtimePluginHost,
    pub http_adapter: HttpTransportAdapter,
    pub snapshot: ArcSwap<PolicySnapshot>,
}

impl AppState {
    pub fn new(
        plugin_host: WasmtimePluginHost,
        http_adapter: HttpTransportAdapter,
        snapshot: PolicySnapshot,
    ) -> Self {
        Self {
            plugin_host,
            http_adapter,
            snapshot: ArcSwap::from_pointee(snapshot),
        }
    }
}
