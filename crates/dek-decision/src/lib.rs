// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub id: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub request_id: String,
    pub trace_id: Option<String>,
    pub tenant_id: String,
    pub device_id: String,
    pub principal: Principal,
    pub agent: Option<AgentIdentity>,
    pub action: String,
    pub resource: ResourceRef,
    pub context: serde_json::Value,
    pub input_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub kind: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorResult {
    pub evaluator_id: String,
    pub allow: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub decision_id: String,
    pub allow: bool,
    pub reason_code: String,
    pub reason: String,
    pub obligations: Vec<Obligation>,
    pub effects: serde_json::Value,
    pub policy_bundle_id: String,
    pub policy_bundle_version: String,
    pub evaluator_results: Vec<EvaluatorResult>,
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
