use anyhow::Result;
use dek_control_plane_api::policy::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub success: bool,
    pub bytecode: Option<Vec<u8>>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub allowed: bool,
    pub evaluation_time_ms: u64,
    pub log_output: Vec<String>,
}

#[async_trait::async_trait]
pub trait PolicyCompiler: Send + Sync {
    async fn validate(&self, draft: &PolicyDraft) -> Result<ValidationResult>;
    async fn compile(&self, draft: &PolicyDraft) -> Result<CompilationResult>;
    async fn simulate(
        &self,
        draft: &PolicyDraft,
        input: serde_json::Value,
    ) -> Result<SimulationResult>;
}

// -----------------------------------------------------------------------------
// Skeletons
// -----------------------------------------------------------------------------

pub struct RegoCompiler;

#[async_trait::async_trait]
impl PolicyCompiler for RegoCompiler {
    async fn validate(&self, _draft: &PolicyDraft) -> Result<ValidationResult> {
        Ok(ValidationResult { is_valid: true, errors: vec![] })
    }
    async fn compile(&self, _draft: &PolicyDraft) -> Result<CompilationResult> {
        Ok(CompilationResult { success: true, bytecode: Some(b"mock_rego_bytecode".to_vec()), errors: vec![] })
    }
    async fn simulate(&self, _draft: &PolicyDraft, _input: serde_json::Value) -> Result<SimulationResult> {
        Ok(SimulationResult { allowed: true, evaluation_time_ms: 1, log_output: vec![] })
    }
}

pub struct CedarCompiler;

#[async_trait::async_trait]
impl PolicyCompiler for CedarCompiler {
    async fn validate(&self, _draft: &PolicyDraft) -> Result<ValidationResult> {
        Ok(ValidationResult { is_valid: true, errors: vec![] })
    }
    async fn compile(&self, _draft: &PolicyDraft) -> Result<CompilationResult> {
        Ok(CompilationResult { success: true, bytecode: Some(b"mock_cedar_bytecode".to_vec()), errors: vec![] })
    }
    async fn simulate(&self, _draft: &PolicyDraft, _input: serde_json::Value) -> Result<SimulationResult> {
        Ok(SimulationResult { allowed: true, evaluation_time_ms: 1, log_output: vec![] })
    }
}

pub struct OpenFgaCompiler;

#[async_trait::async_trait]
impl PolicyCompiler for OpenFgaCompiler {
    async fn validate(&self, _draft: &PolicyDraft) -> Result<ValidationResult> {
        Ok(ValidationResult { is_valid: true, errors: vec![] })
    }
    async fn compile(&self, _draft: &PolicyDraft) -> Result<CompilationResult> {
        Ok(CompilationResult { success: true, bytecode: Some(b"mock_openfga_bytecode".to_vec()), errors: vec![] })
    }
    async fn simulate(&self, _draft: &PolicyDraft, _input: serde_json::Value) -> Result<SimulationResult> {
        Ok(SimulationResult { allowed: true, evaluation_time_ms: 1, log_output: vec![] })
    }
}

pub struct CompositePolicyCompiler;

#[async_trait::async_trait]
impl PolicyCompiler for CompositePolicyCompiler {
    async fn validate(&self, _draft: &PolicyDraft) -> Result<ValidationResult> {
        Ok(ValidationResult { is_valid: true, errors: vec![] })
    }
    async fn compile(&self, _draft: &PolicyDraft) -> Result<CompilationResult> {
        Ok(CompilationResult { success: true, bytecode: Some(b"mock_composite_bytecode".to_vec()), errors: vec![] })
    }
    async fn simulate(&self, _draft: &PolicyDraft, _input: serde_json::Value) -> Result<SimulationResult> {
        Ok(SimulationResult { allowed: true, evaluation_time_ms: 1, log_output: vec![] })
    }
}
