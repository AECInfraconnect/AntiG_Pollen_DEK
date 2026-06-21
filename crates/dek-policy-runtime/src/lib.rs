use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub evaluator_id: String,
    pub evaluator_type: String,
    pub required: bool,
    pub status: String,
    pub decision: String,
    pub allow: bool,
    pub reason: String,
    pub effects: serde_json::Value,
    pub obligations: Vec<String>,
    pub metadata: serde_json::Value,
}

use async_trait::async_trait;

#[async_trait]
pub trait PolicyRuntime: Send + Sync {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision>;
    fn version(&self) -> String;
}

// Replaced with uniform PolicyDecision schema

/// A Mock Runtime for Phase 2 that simulates an OPA policy evaluation
/// This is highly testable without needing a full WebAssembly ABI integration initially.
pub struct MockPolicyRuntime;

#[async_trait]
impl PolicyRuntime for MockPolicyRuntime {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision> {
        // A simple mock matching the SRS Appendix A policy:
        // allow if mcp.method == "tools/call" and mcp.tool_name == "safe.echo"
        let allow = if let Some(mcp) = input.get("mcp") {
            if let Some(tool) = mcp.get("tool_name") {
                tool.as_str() == Some("safe.echo")
            } else {
                false
            }
        } else {
            false
        };

        Ok(PolicyDecision {
            evaluator_id: "opa_wasm_mock".to_string(),
            evaluator_type: "local_pdp".to_string(),
            required: true,
            status: "success".to_string(),
            decision: if allow {
                "allow".to_string()
            } else {
                "deny".to_string()
            },
            allow,
            reason: if allow {
                "allowed by guardrail policy".to_string()
            } else {
                "tool is not allowed".to_string()
            },
            effects: serde_json::json!({ "audit": true }),
            obligations: vec!["write_decision_log".to_string()],
            metadata: serde_json::json!({ "version": "mock-v0.1.0" }),
        })
    }

    fn version(&self) -> String {
        "mock-v0.1.0".to_string()
    }
}

use wasmtime::*;
use wasmtime_wasi::preview1;
use wasmtime_wasi::WasiCtxBuilder;
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};

/// The actual WASM runtime host
pub struct WasmtimePolicyRuntime {
    engine: Engine,
    module: Module,
    wasm_path: String,
}

impl WasmtimePolicyRuntime {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)
            .map_err(|e| anyhow::anyhow!("Failed to load WASM module: {}", e))?;

        Ok(Self {
            engine,
            module,
            wasm_path: wasm_path.to_string(),
        })
    }
}

#[async_trait]
impl PolicyRuntime for WasmtimePolicyRuntime {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision> {
        let input_str = serde_json::to_string(&input)?;
        let stdin = MemoryInputPipe::new(bytes::Bytes::from(input_str.into_bytes()));
        let stdout = MemoryOutputPipe::new(10 * 1024 * 1024); // 10MB capacity

        let mut builder = WasiCtxBuilder::new();
        builder.stdin(stdin.clone());
        builder.stdout(stdout.clone());
        builder.inherit_stderr(); // For debugging
        let wasi = builder.build_p1();

        let mut store = Store::new(&self.engine, wasi);
        let mut linker = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |s| s)?;

        // Run plugin from pre-compiled module (thread-safe, concurrent)
        let instance = linker.instantiate(&mut store, &self.module)?;
        let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;

        let mut reason = "Executed WASM policy".to_string();
        let mut allow = false;

        match func.call(&mut store, ()) {
            Ok(_) => {
                // Read result from stdout memory pipe
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
            evaluator_id: "opa_wasm_native".to_string(),
            evaluator_type: "wasm_pdp".to_string(),
            required: true,
            status: "success".to_string(),
            decision: if allow {
                "allow".to_string()
            } else {
                "deny".to_string()
            },
            allow,
            reason,
            effects: serde_json::json!({}),
            obligations: vec![],
            metadata: serde_json::json!({ "wasm_path": self.wasm_path }),
        })
    }

    fn version(&self) -> String {
        "wasm-native-v1.1.0".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_mock_policy_allow() {
        let runtime = MockPolicyRuntime;
        let input = json!({
            "mcp": {
                "method": "tools/call",
                "tool_name": "safe.echo"
            }
        });

        let decision = runtime.evaluate(input).await.unwrap();
        assert!(decision.allow);
        assert_eq!(decision.reason, "allowed by guardrail policy");
    }

    #[tokio::test]
    async fn test_mock_policy_deny() {
        let runtime = MockPolicyRuntime;
        let input = json!({
            "mcp": {
                "method": "tools/call",
                "tool_name": "shell.run"
            }
        });

        let decision = runtime.evaluate(input).await.unwrap();
        assert!(!decision.allow);
        assert_eq!(decision.reason, "tool is not allowed");
    }
}
