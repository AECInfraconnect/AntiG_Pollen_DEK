use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "template_type", rename_all = "snake_case")]
pub enum PolicyTemplate {
    Rbac {
        roles: Vec<String>,
    },
    Abac {
        required_attributes: HashMap<String, String>,
    },
    Rebac {
        required_relation: String,
        target_entity_type: String,
    },
    Risk {
        max_risk_score: u8,
        require_mfa_if_above: Option<u8>,
    },
    Hybrid {
        templates: Vec<PolicyTemplate>,
        operator: LogicalOperator,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogicalOperator {
    And,
    Or,
}
