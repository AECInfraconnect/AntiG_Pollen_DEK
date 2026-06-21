// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::ActivationError;
use dek_config::DekConfig;
use dek_policy_router::PolicyRouter;
use std::sync::Arc;
use tracing::info;

pub async fn hydrate_runtime(
    config: &DekConfig,
    payload: &serde_json::Value,
) -> Result<Arc<PolicyRouter>, ActivationError> {
    info!("Hydrating runtime from payload...");

    let mut router = PolicyRouter::new();

    // Attempt to load route configuration
    // (In dek-router-builder it panics or unwrap so we catch and map to ActivationError)
    dek_router_builder::load_router_config(&mut router, payload);

    // Load OpenFGA Adapter
    if let Some(openfga_cfg) = config
        .policy_config
        .as_ref()
        .and_then(|c| c.openfga.as_ref())
    {
        match dek_openfga::OpenFgaAdapter::new(&openfga_cfg.endpoint, &openfga_cfg.store_id, None) {
            Ok(adapter) => {
                info!(
                    "Loaded OpenFGA adapter at endpoint: {}",
                    openfga_cfg.endpoint
                );
                router.register_evaluator("openfga", Box::new(adapter));
            }
            Err(e) => {
                return Err(ActivationError::RuntimeHydrationFailed(format!(
                    "Failed to load OpenFGA adapter: {}",
                    e
                )))
            }
        }
    }

    // Load Cedar Adapter
    if let Some(cedar_cfg) = config.policy_config.as_ref().and_then(|c| c.cedar.as_ref()) {
        match dek_cedar::CedarAdapter::new(&cedar_cfg.policy_src) {
            Ok(adapter) => {
                info!("Loaded Cedar adapter from src: {:?}", cedar_cfg.policy_src);
                router.register_evaluator("cedar", Box::new(adapter));
            }
            Err(e) => {
                return Err(ActivationError::RuntimeHydrationFailed(format!(
                    "Failed to load Cedar adapter: {}",
                    e
                )))
            }
        }
    }

    info!("Runtime hydration complete.");
    Ok(Arc::new(router))
}

