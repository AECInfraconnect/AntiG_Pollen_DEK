use async_trait::async_trait;
use dek_plugin_sdk::{
    DecisionEffect, DecisionStatus, EvalRequest, PluginError, PluginIdentity, PluginResult,
    PluginType, PolicyDecision, PolicyEvaluator, DEK_PLUGIN_API_VERSION,
};
use wasmtime::*;

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

        // Map OPA builtins
        linker.func_wrap(
            "env",
            "opa_abort",
            |mut caller: Caller<'_, RuntimeState>, addr: i32| {
                let Some(mem_export) = caller.get_export("memory") else {
                    return;
                };
                let Some(mem) = mem_export.into_memory() else {
                    return;
                };
                let data = mem.data(&caller);
                let mut end = addr as usize;
                while end < data.len() && data[end] != 0 {
                    end += 1;
                }
                let msg = std::str::from_utf8(&data[addr as usize..end]).unwrap_or("unknown error");
                tracing::error!("OPA Abort: {}", msg);
            },
        )?;

        linker.func_wrap(
            "env",
            "opa_println",
            |mut caller: Caller<'_, RuntimeState>, addr: i32| {
                let Some(mem_export) = caller.get_export("memory") else {
                    return;
                };
                let Some(mem) = mem_export.into_memory() else {
                    return;
                };
                let data = mem.data(&caller);
                let mut end = addr as usize;
                while end < data.len() && data[end] != 0 {
                    end += 1;
                }
                let msg = std::str::from_utf8(&data[addr as usize..end]).unwrap_or("");
                tracing::info!("OPA: {}", msg);
            },
        )?;

        linker.func_wrap(
            "env",
            "opa_builtin0",
            |_caller: Caller<'_, RuntimeState>, _id: i32, _ctx: i32| -> i32 { 0 },
        )?;
        linker.func_wrap(
            "env",
            "opa_builtin1",
            |_caller: Caller<'_, RuntimeState>, _id: i32, _ctx: i32, _1: i32| -> i32 { 0 },
        )?;
        linker.func_wrap(
            "env",
            "opa_builtin2",
            |_caller: Caller<'_, RuntimeState>, _id: i32, _ctx: i32, _1: i32, _2: i32| -> i32 { 0 },
        )?;
        linker.func_wrap(
            "env",
            "opa_builtin3",
            |_caller: Caller<'_, RuntimeState>,
             _id: i32,
             _ctx: i32,
             _1: i32,
             _2: i32,
             _3: i32|
             -> i32 { 0 },
        )?;
        linker.func_wrap(
            "env",
            "opa_builtin4",
            |_caller: Caller<'_, RuntimeState>,
             _id: i32,
             _ctx: i32,
             _1: i32,
             _2: i32,
             _3: i32,
             _4: i32|
             -> i32 { 0 },
        )?;

        let instance_pre = linker.instantiate_pre(&module)?;

        Ok(Self {
            engine,
            instance_pre,
            wasm_path: wasm_path.to_string(),
            profile,
        })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn probe(&self) -> anyhow::Result<PolicyDecision> {
        let req = EvalRequest {
            request_id: format!("probe_{}", uuid::Uuid::new_v4()),
            tenant_id: Some("local".into()),
            subject: None,
            action: None,
            resource: None,
            payload: serde_json::json!({
                "action": "probe",
                "resource": "system"
            }),
            context: std::collections::BTreeMap::new(),
        };
        self.evaluate(req)
            .await
            .map_err(|e| anyhow::anyhow!("Probe failed: {:?}", e))
    }

    fn write_json_to_memory(
        &self,
        store: &mut Store<RuntimeState>,
        instance: &Instance,
        json_str: &str,
    ) -> anyhow::Result<i32> {
        let malloc = instance.get_typed_func::<i32, i32>(store.as_context_mut(), "opa_malloc")?;
        let json_parse =
            instance.get_typed_func::<(i32, i32), i32>(store.as_context_mut(), "opa_json_parse")?;

        let bytes = json_str.as_bytes();
        let len = bytes.len() as i32;
        let ptr = malloc.call(store.as_context_mut(), len)?;

        let memory = instance
            .get_memory(store.as_context_mut(), "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;
        memory.write(store.as_context_mut(), ptr as usize, bytes)?;

        let parsed_addr = json_parse.call(store.as_context_mut(), (ptr, len))?;
        if parsed_addr == 0 {
            return Err(anyhow::anyhow!("opa_json_parse failed"));
        }

        Ok(parsed_addr)
    }

    fn read_json_from_memory(
        &self,
        store: &mut Store<RuntimeState>,
        instance: &Instance,
        addr: i32,
    ) -> anyhow::Result<String> {
        let json_dump =
            instance.get_typed_func::<i32, i32>(store.as_context_mut(), "opa_json_dump")?;
        let str_ptr = json_dump.call(store.as_context_mut(), addr)?;

        let memory = instance
            .get_memory(store.as_context_mut(), "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory export not found"))?;

        let data = memory.data(store.as_context_mut());
        let mut end = str_ptr as usize;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }

        let s = std::str::from_utf8(&data[str_ptr as usize..end])?.to_string();
        Ok(s)
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

        let limits = StoreLimitsBuilder::new()
            .memory_size(self.profile.max_memory_bytes)
            .build();

        let state = RuntimeState { limits };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|state| &mut state.limits);
        store
            .set_fuel(self.profile.max_fuel)
            .map_err(|e| PluginError::Execution(format!("failed to set fuel: {e}")))?;

        let instance = self
            .instance_pre
            .instantiate(&mut store)
            .map_err(|e| PluginError::Execution(format!("instantiate error: {e}")))?;

        let mut reason = "Executed WASM policy".to_string();
        let mut allow = false;
        let mut effects = serde_json::json!({});

        // 1. Initialize evaluation context
        let eval_ctx_new = instance
            .get_typed_func::<(), i32>(&mut store, "opa_eval_ctx_new")
            .map_err(|e| PluginError::Execution(format!("missing opa_eval_ctx_new: {e}")))?;
        let ctx = eval_ctx_new
            .call(&mut store, ())
            .map_err(|e| PluginError::Execution(format!("opa_eval_ctx_new failed: {e}")))?;

        // 2. Write input
        let input_addr = self
            .write_json_to_memory(&mut store, &instance, &input_str)
            .map_err(|e| PluginError::Execution(format!("write input error: {e}")))?;

        let eval_ctx_set_input = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "opa_eval_ctx_set_input")
            .map_err(|e| PluginError::Execution(format!("missing opa_eval_ctx_set_input: {e}")))?;
        eval_ctx_set_input
            .call(&mut store, (ctx, input_addr))
            .map_err(|e| PluginError::Execution(format!("opa_eval_ctx_set_input failed: {e}")))?;

        // 3. Evaluate
        let eval = instance
            .get_typed_func::<i32, ()>(&mut store, "opa_eval")
            .map_err(|e| PluginError::Execution(format!("missing opa_eval: {e}")))?;
        eval.call(&mut store, ctx)
            .map_err(|e| PluginError::Execution(format!("opa_eval failed: {e}")))?;

        // 4. Get result
        let eval_ctx_get_result = instance
            .get_typed_func::<i32, i32>(&mut store, "opa_eval_ctx_get_result")
            .map_err(|e| PluginError::Execution(format!("missing opa_eval_ctx_get_result: {e}")))?;
        let result_addr = eval_ctx_get_result
            .call(&mut store, ctx)
            .map_err(|e| PluginError::Execution(format!("opa_eval_ctx_get_result failed: {e}")))?;

        if result_addr != 0 {
            let res_json = self
                .read_json_from_memory(&mut store, &instance, result_addr)
                .map_err(|e| PluginError::Execution(format!("read result error: {e}")))?;

            if let Ok(output_val) = serde_json::from_str::<serde_json::Value>(&res_json) {
                // OPA result is typically an array of result bindings, e.g. `[{ "result": true }]`
                if let Some(arr) = output_val.as_array() {
                    if let Some(first) = arr.first() {
                        if let Some(res) = first.get("result") {
                            // Extract allow boolean
                            if let Some(b) = res.as_bool() {
                                allow = b;
                            } else if let Some(obj) = res.as_object() {
                                allow = obj.get("allow").and_then(|v| v.as_bool()).unwrap_or(false);
                                if let Some(r) = obj.get("reason").and_then(|v| v.as_str()) {
                                    reason = r.to_string();
                                }
                            }
                            effects = res.clone();
                        }
                    }
                } else {
                    // Fallback if not an array
                    allow = output_val
                        .get("allow")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                }
            } else {
                reason = format!("Failed to parse WASM output JSON: {}", res_json);
            }
        } else {
            reason = "No result from evaluation".into();
        }

        Ok(PolicyDecision {
            evaluator_id: "opa_wasm".into(),
            evaluator_type: "wasm_pdp".into(),
            required: true,
            status: DecisionStatus::Success,
            decision: if allow {
                DecisionEffect::Allow
            } else {
                DecisionEffect::Deny
            },
            reason,
            effects,
            obligations: vec![],
            metadata: serde_json::json!({ "wasm_path": self.wasm_path }),
        })
    }
}
