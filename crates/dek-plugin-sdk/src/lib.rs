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
