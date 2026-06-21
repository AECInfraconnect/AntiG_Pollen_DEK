// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevel {
    Off,
    ObserveOnly,
    Warn,
    RequireApproval,
    Enforce,
    StrictDeny,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PresetCategory {
    ShadowAi,
    CostBudget,
    ToolPermission,
    ResourceClassification,
    OwnershipDelegation,
    ProviderGovernance,
    ToolIntegrity,
    DataLossPrevention,
    ApprovalWorkflow,
    BreakGlass,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PresetLanguage {
    Rego,
    Cedar,
    OpenFga,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PresetParameter {
    pub key: String,
    pub label: String,
    pub description: String,
    pub param_type: String,
    pub required: bool,
    pub default_value: serde_json::Value,
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PresetTemplate {
    pub source: String,
    pub entrypoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PresetTestCase {
    pub name: String,
    pub input: serde_json::Value,
    pub expected_decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyPreset {
    pub preset_id: String,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub category: PresetCategory,
    pub language: PresetLanguage,
    pub recommended_pep_types: Vec<String>,
    pub supported_control_levels: Vec<ControlLevel>,
    pub default_control_level: ControlLevel,
    pub risk_tags: Vec<String>,
    pub owasp_tags: Vec<String>,
    pub parameters: Vec<PresetParameter>,
    pub template: PresetTemplate,
    pub test_cases: Vec<PresetTestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct PresetApplyTargets {
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub tool_ids: Vec<String>,
    #[serde(default)]
    pub resource_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PresetApplyRequest {
    #[serde(default)]
    pub targets: PresetApplyTargets,
    pub control_level: ControlLevel,
    #[serde(default)]
    pub params: std::collections::HashMap<String, serde_json::Value>,
}
