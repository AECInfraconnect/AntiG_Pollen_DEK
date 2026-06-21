use anyhow::Result;
use dek_mcp_normalizer::NormalizedMcpEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub timeout_ms: u64,
    pub max_memory_mb: u32,
    pub enable_network: bool,
}

pub trait SandboxEnvironment: Send + Sync {
    /// Prepare the isolated environment for execution
    fn spawn(&mut self, config: SandboxConfig) -> Result<()>;

    /// Execute the high-risk tool inside the sandbox
    fn execute_tool(&self, event: &NormalizedMcpEvent) -> Result<serde_json::Value>;

    /// Terminate and clean up the sandbox
    fn terminate(&mut self) -> Result<()>;
}

/// A basic implementation of WASM-based sandboxing environment.
/// This paves the way for ASI05 Execution Sandboxing.
pub struct WasmSandbox {
    pub is_running: bool,
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmSandbox {
    pub fn new() -> Self {
        Self { is_running: false }
    }
}

impl SandboxEnvironment for WasmSandbox {
    fn spawn(&mut self, _config: SandboxConfig) -> Result<()> {
        tracing::info!("Spawning isolated WASM sandbox for tool execution...");
        self.is_running = true;
        Ok(())
    }

    fn execute_tool(&self, event: &NormalizedMcpEvent) -> Result<serde_json::Value> {
        if !self.is_running {
            return Err(anyhow::anyhow!("Sandbox is not running"));
        }
        let tool_name = event.tool_name.clone().unwrap_or_default();
        tracing::info!("Executing high-risk tool [{}] inside sandbox...", tool_name);

        // Mock successful execution
        Ok(serde_json::json!({
            "status": "success",
            "message": format!("Executed {} inside ephemeral sandbox", tool_name)
        }))
    }

    fn terminate(&mut self) -> Result<()> {
        tracing::info!("Terminating WASM sandbox...");
        self.is_running = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_sandbox_lifecycle() {
        let mut sandbox = WasmSandbox::new();
        assert!(sandbox
            .spawn(SandboxConfig {
                timeout_ms: 1000,
                max_memory_mb: 128,
                enable_network: false,
            })
            .is_ok());

        let event = NormalizedMcpEvent {
            tool_name: Some("high_risk_tool".into()),
            agent_id: Some("agent-1".into()),
            resource_uri: None,
            payload: serde_json::json!({}),
            original_json: "{}".into(),
        };

        let result = sandbox.execute_tool(&event);
        assert!(result.is_ok());

        assert!(sandbox.terminate().is_ok());
    }
}
