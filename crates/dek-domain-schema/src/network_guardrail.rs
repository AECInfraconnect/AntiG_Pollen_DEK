// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkGuardrailEffect {
    #[serde(rename = "ALLOW")]
    Allow,
    #[serde(rename = "DENY")]
    Deny,
    #[serde(rename = "ALLOW_OR_DENY")]
    AllowOrDeny,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkTargets {
    #[serde(default)]
    pub agents: Vec<String>,
    #[serde(default)]
    pub processes: Vec<String>,
    #[serde(default)]
    pub users: Vec<String>,
    #[serde(default)]
    pub devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDestination {
    pub r#type: String, // "domain", "cidr", "port"
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkConditions {
    #[serde(default)]
    pub destinations: Vec<NetworkDestination>,
    #[serde(default)]
    pub protocols: Vec<String>,
    pub time_window: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkFallback {
    pub cloud_unavailable: String,
    pub policy_stale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledNetworkRules {
    pub policy_id: String,
    pub policy_type: String, // e.g. "NETWORK_EGRESS_GUARDRAIL"
    pub version: u64,
    pub risk_tier: String,
    pub targets: NetworkTargets,
    pub conditions: NetworkConditions,
    pub effect: NetworkGuardrailEffect,
    #[serde(default)]
    pub obligations: Vec<String>,
    pub fallback: NetworkFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GuardrailStatus {
    Active(GenerationId),
    Degraded(String),
    Failed(String),
}

