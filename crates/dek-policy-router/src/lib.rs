#![warn(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
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

    pub async fn clear_caches(&self) {
        for evaluator in self.evaluators.values() {
            evaluator.clear_cache().await;
        }
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

        tracing::info!("== Adaptive Routing: Matched Route '{}' ==", route.id);

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
                match evaluator.evaluate(payload.clone()).await {
                    Ok(res) => {
                        tracing::info!("Evaluator {} returned: {}", ev_id, res.decision);

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
                    }
                    Err(dek_policy_runtime::PolicyError::Unavailable(msg)) => {
                        tracing::warn!("required PDP unavailable: {msg}; failing closed");
                        combined_decision.allow = false;
                        combined_decision.decision = "deny".into();
                        combined_decision.reason = format!("required PDP unavailable: {}", msg);
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("PDP error: {e}; failing closed");
                        combined_decision.allow = false;
                        combined_decision.decision = "deny".into();
                        combined_decision.reason = format!("PDP error: {}", e);
                        break;
                    }
                }
            } else {
                tracing::warn!(
                    "Error: Required evaluator {} not found. Failing closed.",
                    ev_id
                );
                combined_decision.allow = false;
                combined_decision.decision = "deny".into();
                combined_decision.reason = format!(
                    "Required evaluator {} not configured or failed to load",
                    ev_id
                );
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use dek_policy_runtime::PolicyDecision;

    struct DummyRuntime;
    #[async_trait]
    impl PolicyRuntime for DummyRuntime {
        async fn evaluate(&self, _input: serde_json::Value) -> std::result::Result<PolicyDecision, dek_policy_runtime::PolicyError> {
            Ok(PolicyDecision {
                evaluator_id: "dummy".into(),
                evaluator_type: "dummy".into(),
                required: true,
                status: "success".into(),
                decision: "allow".into(),
                allow: true,
                reason: "mocked".into(),
                effects: serde_json::json!({}),
                obligations: vec![],
                metadata: serde_json::json!({}),
            })
        }
        fn version(&self) -> String {
            "1.0".into()
        }
    }

    #[tokio::test]
    async fn test_empty_router_denies_all() {
        let router = PolicyRouter::new();
        let payload = serde_json::json!({ "request_type": "tools/call" });
        let res = router.authorize(payload).await.unwrap();
        assert_eq!(res.decision, "deny");
        assert_eq!(res.reason, "no matching route");
    }

    #[tokio::test]
    async fn test_route_matches_and_allows() {
        let mut router = PolicyRouter::new();
        router.register_evaluator("dummy", Box::new(DummyRuntime));
        router.set_routes(vec![Route {
            id: "route1".into(),
            priority: 10,
            match_rule: MatchRule {
                method: Some("test".into()),
                tool_category: None,
            },
            pdp_required: vec!["dummy".into()],
            pdp_conditional: vec![],
        }]);

        let payload = serde_json::json!({ "request_type": "test" });
        let res = router.authorize(payload).await.unwrap();
        assert_eq!(res.decision, "allow");
    }
}
