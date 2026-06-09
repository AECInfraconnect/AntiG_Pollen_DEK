// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

pub mod factories;

use dek_policy_router::PolicyRouter;
use dek_pdp_sdk::AdapterRegistry;
use serde_json::Value;
use tracing::{error, warn};

/// Returns a default registry with all compiled-in features enabled
pub fn default_registry() -> AdapterRegistry {
    let mut registry = AdapterRegistry::new();
    
    #[cfg(feature = "adapter-cedar")]
    registry.register(Box::new(factories::CedarFactory));
    
    #[cfg(feature = "adapter-opa")]
    registry.register(Box::new(factories::OpaFactory));
    
    #[cfg(feature = "adapter-openfga")]
    registry.register(Box::new(factories::OpenFgaFactory));
    
    registry
}

pub fn load_router_config(router: &mut PolicyRouter, payload: &Value) {
    let registry = default_registry();

    if let Some(scale_val) = payload.get("scale") {
        if let Ok(scale) = serde_json::from_value::<dek_config::ScaleConfig>(scale_val.clone()) {
            router.set_scale_config(
                scale.pdp_timeout_ms,
                scale.breaker_failure_threshold,
                scale.breaker_cooldown_secs,
            );
        }
    }

    if let Some(openfga) = payload.get("openfga") {
        match registry.build_adapter("openfga", openfga) {
            Ok(adapter) => router.register_evaluator("openfga", adapter),
            Err(e) => {
                if registry.get("openfga").is_none() {
                    warn!("OpenFGA adapter requested but not compiled in this build.");
                } else {
                    error!("Failed to initialize OpenFGA Adapter: {}", e);
                }
            }
        }
    }

    if let Some(cedar) = payload.get("cedar") {
        match registry.build_adapter("cedar", cedar) {
            Ok(adapter) => router.register_evaluator("cedar", adapter),
            Err(e) => {
                if registry.get("cedar").is_none() {
                    warn!("Cedar adapter requested but not compiled in this build.");
                } else {
                    error!("Failed to initialize Cedar Adapter: {}", e);
                }
            }
        }
    }

    if let Some(wasm) = payload.get("opa_wasm") {
        match registry.build_adapter("opa_wasm", wasm) {
            Ok(adapter) => router.register_evaluator("opa_wasm", adapter),
            Err(e) => {
                if registry.get("opa_wasm").is_none() {
                    warn!("OPA adapter requested but not compiled in this build.");
                } else {
                    error!("Failed to initialize WASM Policy Runtime: {}", e);
                }
            }
        }
    }

    if let Some(routes_val) = payload.get("routes") {
        match serde_json::from_value::<Vec<dek_policy_router::Route>>(routes_val.clone()) {
            Ok(routes) => {
                router.set_routes(routes);
            }
            Err(e) => {
                error!("Failed to parse routes from bundle: {} (routes_val: {})", e, routes_val);
            }
        }
    }
}
