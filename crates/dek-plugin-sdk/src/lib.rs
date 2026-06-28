// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub const DEK_PLUGIN_API_VERSION: &str = "v1alpha1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub plugin_type: PluginType,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    PolicyEvaluator,
    Transform,
    TelemetrySink,
    ModelProvider,
    EnforcementProvider,
    ControlPlane,
    Detector,
    ResourceClassifier,
    TelemetryProcessor,
    PepAdapter,
    PdpConnector,
}

impl Default for PluginType {
    fn default() -> Self {
        Self::PolicyEvaluator
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginAbi {
    Component,
    Core,
}

impl Default for PluginAbi {
    fn default() -> Self {
        Self::Core
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginKind {
    #[serde(rename = "discovery.scanner")]
    DiscoveryScanner,
    #[serde(rename = "discovery.signature")]
    DiscoverySignature,
    #[serde(rename = "observe.collector")]
    ObserveCollector,
    #[serde(rename = "enforce.method")]
    EnforceMethod,
    #[serde(rename = "policy.evaluator")]
    PolicyEvaluator,
    #[serde(rename = "policy.preset")]
    PolicyPreset,
    #[serde(rename = "telemetry.exporter")]
    TelemetryExporter,
    #[serde(rename = "telemetry.transform")]
    TelemetryTransform,
    #[serde(rename = "resource.classifier")]
    ResourceClassifier,
    #[serde(rename = "risk.scorer")]
    RiskScorer,
    #[serde(rename = "definition.feed")]
    DefinitionFeed,
    #[serde(rename = "notify.channel")]
    NotifyChannel,
    #[serde(rename = "compliance.profile")]
    ComplianceProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginCapabilities {
    #[serde(default)]
    pub host: Vec<String>,
    #[serde(default)]
    pub http_out: Vec<String>,
    #[serde(default)]
    pub kv: Vec<String>,
    #[serde(default)]
    pub native: Vec<String>,
    #[serde(default)]
    pub data_scope: Vec<String>,
}

impl PluginCapabilities {
    pub fn requested(&self) -> Vec<RequestedCapability> {
        let mut requested = Vec::new();
        requested.extend(
            self.host
                .iter()
                .cloned()
                .map(|value| RequestedCapability::Host(value)),
        );
        requested.extend(
            self.http_out
                .iter()
                .cloned()
                .map(|value| RequestedCapability::HttpOut(value)),
        );
        requested.extend(
            self.kv
                .iter()
                .cloned()
                .map(|value| RequestedCapability::Kv(value)),
        );
        requested.extend(
            self.native
                .iter()
                .cloned()
                .map(|value| RequestedCapability::Native(value)),
        );
        requested.extend(
            self.data_scope
                .iter()
                .cloned()
                .map(|value| RequestedCapability::DataScope(value)),
        );
        requested
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case", tag = "scope", content = "value")]
pub enum RequestedCapability {
    Host(String),
    HttpOut(String),
    Kv(String),
    Native(String),
    DataScope(String),
}

impl RequestedCapability {
    pub fn is_basic(&self) -> bool {
        matches!(self, Self::Host(value) if value == "logging" || value == "clock")
    }

    pub fn is_sensitive(&self) -> bool {
        !self.is_basic()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginSignature {
    #[serde(rename = "type")]
    pub signature_type: Option<String>,
    pub bundle: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginAuthor {
    pub name: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRequest {
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub subject: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub payload: serde_json::Value,
    pub context: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub evaluator_id: String,
    pub evaluator_type: String,
    pub required: bool,
    pub status: DecisionStatus,
    pub decision: DecisionEffect,
    pub reason: String,
    pub obligations: Vec<String>,
    pub effects: serde_json::Value,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Success,
    Error,
    Unavailable,
    Timeout,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEffect {
    Allow,
    Deny,
    Abstain,
}

impl PolicyDecision {
    pub fn allow(evaluator_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            evaluator_id: evaluator_id.into(),
            evaluator_type: "policy_evaluator".into(),
            required: true,
            status: DecisionStatus::Success,
            decision: DecisionEffect::Allow,
            reason: reason.into(),
            obligations: vec![],
            effects: serde_json::json!({}),
            metadata: serde_json::json!({}),
        }
    }

    pub fn deny(evaluator_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            evaluator_id: evaluator_id.into(),
            evaluator_type: "policy_evaluator".into(),
            required: true,
            status: DecisionStatus::Success,
            decision: DecisionEffect::Deny,
            reason: reason.into(),
            obligations: vec![],
            effects: serde_json::json!({}),
            metadata: serde_json::json!({}),
        }
    }

    pub fn is_allow(&self) -> bool {
        self.decision == DecisionEffect::Allow && self.status == DecisionStatus::Success
    }
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin unavailable: {0}")]
    Unavailable(String),
    #[error("invalid plugin input: {0}")]
    Invalid(String),
    #[error("plugin timeout: {0}")]
    Timeout(String),
    #[error("plugin execution failed: {0}")]
    Execution(String),
}

pub type PluginResult<T> = Result<T, PluginError>;

#[async_trait]
pub trait PolicyEvaluator: Send + Sync {
    fn identity(&self) -> PluginIdentity;
    async fn evaluate(&self, request: EvalRequest) -> PluginResult<PolicyDecision>;
    async fn clear_cache(&self) -> PluginResult<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRequest {
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub direction: TransformDirection,
    pub payload: serde_json::Value,
    pub context: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformDirection {
    Request,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResponse {
    pub payload: serde_json::Value,
    pub redactions: Vec<RedactionFinding>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionFinding {
    pub kind: String,
    pub confidence: f32,
    pub path: String,
    pub replacement: String,
}

#[async_trait]
pub trait TransformPlugin: Send + Sync {
    fn identity(&self) -> PluginIdentity;
    async fn transform(&self, request: TransformRequest) -> PluginResult<TransformResponse>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEnvelope {
    pub decision_id: String,
    pub allowed: bool,
    pub mode: String,
    pub reason: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub pep_type: String,
    pub pdp_runtime_id: String,
    pub route_id: String,
    pub policy_bundle_id: String,
    pub policy_version: String,
    pub latency_ms: u64,
    pub fallback_used: bool,
    pub obligations: Vec<DecisionObligation>,
    pub redactions: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecisionObligation {
    RedactField { path: String },
    RequireApproval { approver_group: String },
    LogOnly,
    MaskOutput,
    LimitTokens { max_tokens: u64 },
    BlockNetwork { host: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLimits {
    pub memory_bytes: usize,
    pub fuel: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

impl Default for WasmLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 64 * 1024 * 1024,
            fuel: 10_000_000,
            timeout_ms: 150,
            max_output_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub kind: Option<PluginKind>,
    #[serde(default)]
    pub wit_world: Option<String>,
    #[serde(default)]
    pub abi: PluginAbi,
    #[serde(default)]
    pub min_engine_version: Option<String>,
    #[serde(default)]
    pub max_engine_version: Option<String>,
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub capabilities: PluginCapabilities,
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub author: Option<PluginAuthor>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub signature: Option<PluginSignature>,
    #[serde(default)]
    pub sbom: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(rename = "type", default)]
    pub plugin_type: PluginType,
    #[serde(default)]
    pub runtime: String,
    #[serde(default)]
    pub entrypoint: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub limits: WasmLimits,
    #[serde(default)]
    pub signing: std::collections::BTreeMap<String, String>,
}

impl PluginManifest {
    pub fn entry_path(&self) -> &str {
        self.entry
            .as_deref()
            .or_else(|| {
                if self.entrypoint.is_empty() {
                    None
                } else {
                    Some(self.entrypoint.as_str())
                }
            })
            .unwrap_or("plugin.wasm")
    }

    pub fn requested_capabilities(&self) -> Vec<RequestedCapability> {
        let mut requested = self.capabilities.requested();
        requested.extend(
            self.permissions
                .iter()
                .cloned()
                .map(RequestedCapability::DataScope),
        );
        requested.sort();
        requested.dedup();
        requested
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPolicy {
    pub allowed_plugins: Vec<String>,
}

impl PluginPolicy {
    pub fn ensure_allowed(
        &self,
        plugin_id: &str,
        _operation: &str,
        _permissions: &[String],
    ) -> anyhow::Result<()> {
        if !self.allowed_plugins.contains(&plugin_id.to_string())
            && !self.allowed_plugins.contains(&"*".to_string())
        {
            anyhow::bail!("Plugin {} is not allowed by policy", plugin_id);
        }
        Ok(())
    }

    pub fn timeout_ms(&self, _plugin_id: &str) -> u64 {
        150 // default timeout 150ms
    }

    pub fn validate_output(
        &self,
        _plugin_id: &str,
        _operation: &str,
        _output: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(()) // naive validation
    }
}
