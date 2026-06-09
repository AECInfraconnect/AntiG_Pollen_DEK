// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use async_trait::async_trait;
use dek_plugin_sdk::{
    DecisionEffect, DecisionStatus, EvalRequest, PluginError, PluginIdentity, PluginResult,
    PluginType, PolicyDecision, PolicyEvaluator, DEK_PLUGIN_API_VERSION,
};
use wasmtime::*;
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::preview1;
use wasmtime_wasi::WasiCtxBuilder;

#[derive(Debug, Clone)]
pub struct WasmProfile {
    pub max_memory_bytes: usize,
    pub max_fuel: u64,
}

impl Default for WasmProfile {
    fn default() -> Self {
        Self {
            max_memory_bytes: 10 * 1024 * 1024,
            max_fuel: 1_000_000,
        }
    }
}

struct RuntimeState {
    wasi: wasmtime_wasi::preview1::WasiP1Ctx,
    limits: StoreLimits,
}

pub struct OpaWasmAdapter {
    engine: Engine,
    instance_pre: InstancePre<RuntimeState>,
    wasm_path: String,
    profile: WasmProfile,
}

impl OpaWasmAdapter {
    pub fn new(wasm_path: &str, profile: Option<WasmProfile>) -> anyhow::Result<Self> {
        let profile = profile.unwrap_or_default();

        let mut config = Config::new();
        config.consume_fuel(true);
        config.max_wasm_stack(1024 * 1024);

        let engine = Engine::new(&config).map_err(|e| anyhow::anyhow!("Engine error: {}", e))?;
        let module = Module::from_file(&engine, wasm_path)
            .map_err(|e| anyhow::anyhow!("WASM load error: {}", e))?;

        let mut linker = Linker::new(&engine);
        preview1::add_to_linker_sync(&mut linker, |s: &mut RuntimeState| &mut s.wasi)?;
        let instance_pre = linker.instantiate_pre(&module)?;

        Ok(Self {
            engine,
            instance_pre,
            wasm_path: wasm_path.to_string(),
            profile,
        })
    }
}

#[async_trait]
impl PolicyEvaluator for OpaWasmAdapter {
    fn identity(&self) -> PluginIdentity {
        PluginIdentity {
            id: "opa_wasm".into(),
            name: "OPA WebAssembly Evaluator".into(),
            version: "1.0.0".into(),
            vendor: "AEC Infraconnect".into(),
            plugin_type: PluginType::PolicyEvaluator,
            api_version: DEK_PLUGIN_API_VERSION.into(),
        }
    }

    async fn evaluate(&self, input: EvalRequest) -> PluginResult<PolicyDecision> {
        let input_str = serde_json::to_string(&input.payload)
            .map_err(|e| PluginError::Invalid(e.to_string()))?;
        let stdin = MemoryInputPipe::new(bytes::Bytes::from(input_str.into_bytes()));
        let stdout = MemoryOutputPipe::new(self.profile.max_memory_bytes);

        let mut builder = WasiCtxBuilder::new();
        builder.stdin(stdin.clone());
        builder.stdout(stdout.clone());
        builder.inherit_stderr();
        let wasi = builder.build_p1();

        let limits = StoreLimitsBuilder::new()
            .memory_size(self.profile.max_memory_bytes)
            .build();

        let state = RuntimeState { wasi, limits };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(self.profile.max_fuel)
            .map_err(|e| PluginError::Execution(format!("failed to set fuel: {e}")))?;

        let instance = self
            .instance_pre
            .instantiate(&mut store)
            .map_err(|e| PluginError::Execution(format!("instantiate error: {e}")))?;
        let func = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .map_err(|e| PluginError::Execution(format!("missing _start: {e}")))?;

        let mut reason = "Executed WASM policy".to_string();
        let mut allow = false;

        match func.call(&mut store, ()) {
            Ok(_) => {
                let out_bytes = stdout.contents();
                let output_str = String::from_utf8_lossy(&out_bytes);

                if let Ok(output_val) = serde_json::from_str::<serde_json::Value>(&output_str) {
                    allow = output_val
                        .get("allow")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if let Some(r) = output_val.get("reason").and_then(|v| v.as_str()) {
                        reason = r.to_string();
                    }
                } else {
                    reason = format!("Failed to parse WASM output JSON: {}", output_str);
                }
            }
            Err(e) => {
                reason = format!("WASM execution failed: {}", e);
            }
        }

        Ok(PolicyDecision {
            evaluator_id: "opa_wasm".into(),
            evaluator_type: "wasm_pdp".into(),
            required: true,
            status: DecisionStatus::Success,
            decision: if allow { DecisionEffect::Allow } else { DecisionEffect::Deny },
            reason,
            effects: serde_json::json!({}),
            obligations: vec![],
            metadata: serde_json::json!({ "wasm_path": self.wasm_path }),
        })
    }
}
