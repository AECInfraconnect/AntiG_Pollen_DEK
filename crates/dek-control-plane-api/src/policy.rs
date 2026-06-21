use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::registry::ObjectMeta;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyDraft {
    pub meta: ObjectMeta,
    pub policy_id: String,
    pub name: String,
    pub description: Option<String>,
    pub policy_type: PolicyType,
    pub targets: PolicyTargets,
    pub source: PolicySource,
    pub compile_options: PolicyCompileOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyType {
    Rego,
    Cedar,
    OpenFga,
    PiiRedaction,
    Route,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyTargets {
    pub agent_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub resource_ids: Vec<String>,
    pub entity_ids: Vec<String>,
    pub route_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PolicySource {
    RawText {
        language: String,
        text: String,
    },
    Template {
        template_id: String,
        params: serde_json::Value,
    },
    Structured {
        ir: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyCompileOptions {
    pub optimization_level: Option<String>,
    pub fail_on_warnings: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyLifecycleStatus {
    Draft,
    Validated,
    SimulationPassed,
    Compiled,
    PendingApproval,
    Approved,
    Published,
    Active,
    RolledBack,
    Archived,
}
