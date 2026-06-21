use anyhow::Result;
use dek_policy_runtime::{PolicyDecision, PolicyRuntime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub id: String,
    pub priority: i32,
    pub match_rule: MatchRule,
    pub pdp_required: Vec<String>,
    pub pdp_conditional: Vec<ConditionalPdp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
    pub method: Option<String>,
    pub tool_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalPdp {
    pub evaluator: String,
    pub required_payload_key: String, // Mock condition evaluation
}

pub struct PolicyRouter {
    routes: Vec<Route>,
    evaluators: HashMap<String, Box<dyn PolicyRuntime>>,
}

impl PolicyRouter {
    pub fn new() -> Self {
        Self {
            routes: vec![],
            evaluators: HashMap::new(),
        }
    }

    pub fn register_evaluator(&mut self, id: &str, evaluator: Box<dyn PolicyRuntime>) {
        self.evaluators.insert(id.to_string(), evaluator);
    }

    pub fn set_routes(&mut self, mut routes: Vec<Route>) {
        routes.sort_by_key(|b| std::cmp::Reverse(b.priority)); // Highest priority first
        self.routes = routes;
    }

    pub async fn authorize(&self, payload: serde_json::Value) -> Result<PolicyDecision> {
        // Support both old nested schema and new NormalizedMcpEvent schema
        let method = payload
            .get("request_type")
            .and_then(|v| v.as_str())
            .or_else(|| {
                payload
                    .get("mcp")
                    .and_then(|mcp| mcp.get("method"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");

        let mut matched_route = None;
        for route in &self.routes {
            if let Some(ref m) = route.match_rule.method {
                if m == method || m == "*" {
                    matched_route = Some(route);
                    break;
                }
            }
        }

        let route = match matched_route {
            Some(r) => r,
            None => {
                return Ok(PolicyDecision {
                    evaluator_id: "router_default".into(),
                    evaluator_type: "router".into(),
                    required: true,
                    status: "success".into(),
                    decision: "deny".into(),
                    allow: false,
                    reason: "no matching route".into(),
                    effects: serde_json::json!({}),
                    obligations: vec![],
                    metadata: serde_json::json!({}),
                })
            }
        };

        println!("== Adaptive Routing: Matched Route '{}' ==", route.id);

        let mut combined_decision = PolicyDecision {
            evaluator_id: "router_combiner".into(),
            evaluator_type: "router".into(),
            required: true,
            status: "success".into(),
            decision: "allow".into(),
            allow: true,
            reason: "All evaluators passed".into(),
            effects: serde_json::json!({}),
            obligations: vec![],
            metadata: serde_json::json!({}),
        };

        let mut to_evaluate = route.pdp_required.clone();
        for cond in &route.pdp_conditional {
            if payload.get(&cond.required_payload_key).is_some() || cond.required_payload_key == "*"
            {
                to_evaluate.push(cond.evaluator.clone());
            }
        }

        for ev_id in to_evaluate {
            if let Some(evaluator) = self.evaluators.get(&ev_id) {
                let res = evaluator.evaluate(payload.clone()).await?;
                println!("Evaluator {} returned: {}", ev_id, res.decision);

                // Combine obligations
                combined_decision
                    .obligations
                    .extend(res.obligations.clone());

                // Merge effects (simple mock merge)
                if let serde_json::Value::Object(mut combined_map) =
                    combined_decision.effects.clone()
                {
                    if let serde_json::Value::Object(res_map) = res.effects.clone() {
                        for (k, v) in res_map {
                            combined_map.insert(k, v);
                        }
                    }
                    combined_decision.effects = serde_json::Value::Object(combined_map);
                }

                if !res.allow {
                    // Deny overrides
                    combined_decision.allow = false;
                    combined_decision.decision = "deny".into();
                    combined_decision.reason = format!("Blocked by {}", ev_id);
                    // Short-circuit on deny
                    break;
                }
            } else {
                println!("Error: Required evaluator {} not found. Failing closed.", ev_id);
                combined_decision.allow = false;
                combined_decision.decision = "deny".into();
                combined_decision.reason = format!("Required evaluator {} not configured or failed to load", ev_id);
                break;
            }
        }

        Ok(combined_decision)
    }
}

impl Default for PolicyRouter {
    fn default() -> Self {
        Self::new()
    }
}
