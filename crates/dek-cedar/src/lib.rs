use anyhow::Result;
use async_trait::async_trait;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityId, EntityTypeName, EntityUid, PolicySet,
    Request,
};
use dek_policy_runtime::{PolicyDecision, PolicyRuntime};
use std::str::FromStr;

pub struct CedarAdapter {
    policy_src: String,
}

impl CedarAdapter {
    pub fn new(policy_src: &str) -> Self {
        Self {
            policy_src: policy_src.to_string(),
        }
    }
}

#[async_trait]
impl PolicyRuntime for CedarAdapter {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision> {
        let principal = input
            .get("principal")
            .and_then(|v| v.as_str())
            .unwrap_or("User::\"unknown\"");
        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("Action::\"unknown\"");
        let resource = input
            .get("resource")
            .and_then(|v| v.as_str())
            .unwrap_or("Resource::\"unknown\"");

        println!("Evaluating Cedar Policy:\n{}", self.policy_src);

        // Parse the policy set
        let policy_set = match PolicySet::from_str(&self.policy_src) {
            Ok(ps) => ps,
            Err(e) => {
                println!("Cedar Parse Error: {}", e);
                return Ok(PolicyDecision {
                    evaluator_id: "cedar_native".to_string(),
                    evaluator_type: "local_pdp".to_string(),
                    required: true,
                    status: "error".to_string(),
                    decision: "deny".to_string(),
                    allow: false,
                    reason: format!("Cedar parse error: {}", e),
                    effects: serde_json::json!({}),
                    obligations: vec![],
                    metadata: serde_json::json!({}),
                });
            }
        };

        // For simplicity, we assume strings are fully qualified like `User::"alice"`
        // Or if they are just raw strings, we wrap them.
        let make_uid = |type_name: &str, id: &str| -> EntityUid {
            if id.contains("::") {
                EntityUid::from_str(id).unwrap_or_else(|_| {
                    EntityUid::from_type_name_and_id(
                        EntityTypeName::from_str(type_name).unwrap(),
                        EntityId::from_str("unknown").unwrap(),
                    )
                })
            } else {
                EntityUid::from_type_name_and_id(
                    EntityTypeName::from_str(type_name).unwrap(),
                    EntityId::from_str(id).unwrap(),
                )
            }
        };

        let principal_uid = make_uid("User", principal);
        let action_uid = make_uid("Action", action);
        let resource_uid = make_uid("Resource", resource);

        let request = Request::new(
            principal_uid,
            action_uid,
            resource_uid,
            Context::empty(),
            None,
        )
        .map_err(|e| anyhow::anyhow!("Cedar Request Error: {}", e))?;

        let entities = Entities::empty();
        let authorizer = Authorizer::new();
        let answer = authorizer.is_authorized(&request, &policy_set, &entities);

        let allowed = answer.decision() == Decision::Allow;

        Ok(PolicyDecision {
            evaluator_id: "cedar_native".to_string(),
            evaluator_type: "local_pdp".to_string(),
            required: true,
            status: "success".to_string(),
            decision: if allowed {
                "allow".to_string()
            } else {
                "deny".to_string()
            },
            allow: allowed,
            reason: if allowed {
                "Allowed by Cedar policy".to_string()
            } else {
                "Denied by Cedar policy".to_string()
            },
            effects: serde_json::json!({}),
            obligations: vec![],
            metadata: serde_json::json!({ "policy_version": "1.0", "diagnostics": format!("{:?}", answer.diagnostics()) }),
        })
    }

    fn version(&self) -> String {
        "cedar-v1.0.0".to_string()
    }
}
