use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyIntent {
    pub api_version: String,
    pub kind: String,
    pub metadata: PolicyMetadata,
    pub spec: PolicySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyMetadata {
    pub id: String,
    pub tenant: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PolicySpec {
    pub decision_mode: DecisionMode,
    pub priority: Option<i32>,
    pub subjects: SubjectSelector,
    pub actions: Vec<String>,
    pub resources: ResourceSelector,
    #[serde(default)]
    pub constraints: HashMap<String, serde_json::Value>,
    pub enforcement: EnforcementConfig,
    #[serde(default)]
    pub obligations: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMode {
    Enforce,
    Observe,
    Shadow,
    DryRun,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubjectSelector {
    #[serde(default)]
    pub include: Vec<EntitySelector>,
    #[serde(default)]
    pub exclude: Vec<EntitySelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceSelector {
    #[serde(default)]
    pub include: Vec<EntitySelector>,
    #[serde(default)]
    pub exclude: Vec<EntitySelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitySelector {
    pub r#type: String,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default)]
    pub selector: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnforcementConfig {
    #[serde(default)]
    pub preferred_pep_types: Vec<String>,
    #[serde(default)]
    pub os_scope: Vec<String>,
    pub fallback: FallbackAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FallbackAction {
    Deny,
    AllowAudit,
    LastGood,
    Quarantine,
}
