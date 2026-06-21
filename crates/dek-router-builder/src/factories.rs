// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use dek_pdp_sdk::{AdapterFactory, AdapterInfo, BuildError, PolicyRuntime};
use serde_json::Value;
use std::sync::Arc;

#[cfg(feature = "adapter-cedar")]
pub struct CedarFactory;

#[cfg(feature = "adapter-cedar")]
impl AdapterFactory for CedarFactory {
    fn info(&self) -> AdapterInfo {
        AdapterInfo::new(
            "cedar",
            "AWS Cedar Policy Engine Adapter",
            env!("CARGO_PKG_VERSION"),
        )
    }

    fn build(&self, config: &Value) -> Result<Box<dyn PolicyRuntime>, BuildError> {
        let policy_src = config
            .get("policy_src")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        dek_cedar::CedarAdapter::new(policy_src)
            .map(|adapter| {
                Box::new(dek_plugin_host::EvaluatorAdapter::new(Arc::new(adapter)))
                    as Box<dyn PolicyRuntime>
            })
            .map_err(|e| BuildError::InitFailed(e.to_string()))
    }
}

#[cfg(feature = "adapter-opa")]
pub struct OpaFactory;

#[cfg(feature = "adapter-opa")]
impl AdapterFactory for OpaFactory {
    fn info(&self) -> AdapterInfo {
        AdapterInfo::new(
            "opa_wasm",
            "Open Policy Agent WASM Adapter",
            env!("CARGO_PKG_VERSION"),
        )
    }

    fn build(&self, config: &Value) -> Result<Box<dyn PolicyRuntime>, BuildError> {
        let policy_path = config
            .get("policy_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BuildError::MissingConfig("policy_path".to_string()))?;

        if !std::path::Path::new(policy_path).exists() {
            return Err(BuildError::InvalidConfig(format!(
                "WASM policy file not found at: {}",
                policy_path
            )));
        }

        dek_opa_wasm::OpaWasmAdapter::new(policy_path, None)
            .map(|adapter| {
                Box::new(dek_plugin_host::EvaluatorAdapter::new(Arc::new(adapter)))
                    as Box<dyn PolicyRuntime>
            })
            .map_err(|e| BuildError::InitFailed(e.to_string()))
    }
}

#[cfg(feature = "adapter-openfga")]
pub struct OpenFgaFactory;

#[cfg(feature = "adapter-openfga")]
impl AdapterFactory for OpenFgaFactory {
    fn info(&self) -> AdapterInfo {
        AdapterInfo::new(
            "openfga",
            "OpenFGA Remote Evaluator Adapter",
            env!("CARGO_PKG_VERSION"),
        )
    }

    fn build(&self, config: &Value) -> Result<Box<dyn PolicyRuntime>, BuildError> {
        let endpoint = config
            .get("endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:8080");
        let store_id = config
            .get("store_id")
            .and_then(|v| v.as_str())
            .unwrap_or("store_123");

        let mtls: Option<dek_config::MtlsConfig> = config
            .get("mtls")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        dek_openfga::OpenFgaAdapter::new(endpoint, store_id, mtls.as_ref())
            .map(|adapter| {
                Box::new(dek_plugin_host::EvaluatorAdapter::new(Arc::new(adapter)))
                    as Box<dyn PolicyRuntime>
            })
            .map_err(|e| BuildError::InitFailed(e.to_string()))
    }
}
