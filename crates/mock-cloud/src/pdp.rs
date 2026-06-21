use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    pub tenant_id: String,
    pub principal_id: String,
    pub resource_id: String,
    pub action: String,
    pub agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub decision: String,
    pub matching_policies: Vec<String>,
}

pub async fn simulate_pdp(
    State(state): State<AppState>,
    Json(req): Json<AccessRequest>,
) -> axum::response::Result<Json<AccessDecision>, axum::http::StatusCode> {
    let registry = state.registry.lock().unwrap();

    let mut decision = "Deny".to_string();
    let mut matching_policies = vec![];

    // Mock evaluation logic
    // 1. Check if principal and resource exist in tenant
    if !registry.principals.contains_key(&req.principal_id) || !registry.resources.contains_key(&req.resource_id) {
        return Ok(Json(AccessDecision {
            decision: "Deny".to_string(),
            matching_policies: vec![],
        }));
    }

    // 2. Scan policies for matches (Mock implementation)
    for (id, policy) in &registry.policies {
        if policy.tenant_id == req.tenant_id {
            // Loose mock check for the simulator
            if policy.effect.to_lowercase() == "allow" {
                decision = "Allow".to_string();
                matching_policies.push(id.clone());
            } else if policy.effect.to_lowercase() == "deny" {
                // Deny overrides allow
                decision = "Deny".to_string();
                matching_policies.push(id.clone());
                break;
            }
        }
    }

    Ok(Json(AccessDecision {
        decision,
        matching_policies,
    }))
}

// Sandbox API - Simulate mutation before evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxMutation {
    pub action: String, // e.g. "AddRole", "AddRelationship"
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRequest {
    pub access_request: AccessRequest,
    pub temporary_state_mutations: Vec<SandboxMutation>,
}

pub async fn sandbox_simulate(
    State(_state): State<AppState>,
    Json(_req): Json<SandboxRequest>,
) -> axum::response::Result<Json<AccessDecision>, axum::http::StatusCode> {
    // In a real implementation we would clone RegistryState, apply mutations, and run PDP
    // For Mock-Cloud, we just return a simulated positive decision indicating the sandbox processed the mutations
    let decision = AccessDecision {
        decision: "Allow".to_string(),
        matching_policies: vec!["mock_sandbox_policy".to_string()],
    };
    Ok(Json(decision))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/pdp/simulate", post(simulate_pdp))
        .route("/v1/pdp/sandbox", post(sandbox_simulate))
}
