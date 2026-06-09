#![warn(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
use anyhow::Result;
use async_trait::async_trait;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityId, EntityTypeName, EntityUid, PolicySet,
    Request,
};
use dek_policy_runtime::{PolicyDecision, PolicyRuntime};
use moka::sync::Cache;
use std::str::FromStr;
use std::time::Duration;

pub struct CedarAdapter {
    policy_src: String,
    policy_set: PolicySet,
    cache: Cache<String, PolicyDecision>,
}

impl CedarAdapter {
    pub fn new(policy_src: &str) -> Result<Self> {
        let policy_set = PolicySet::from_str(policy_src)
            .map_err(|e| anyhow::anyhow!("Cedar Parse Error: {}", e))?;
        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(60))
            .build();
        Ok(Self {
            policy_src: policy_src.to_string(),
            policy_set,
            cache,
        })
    }
}

#[async_trait]
impl PolicyRuntime for CedarAdapter {
    async fn evaluate(&self, input: serde_json::Value) -> dek_policy_runtime::PolicyResult {
        let cache_key = serde_json::to_string(&input).unwrap_or_default();
        if !cache_key.is_empty() {
            if let Some(decision) = self.cache.get(&cache_key) {
                return Ok(decision);
            }
        }

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

        tracing::info!("Evaluating Cedar Policy:\n{}", self.policy_src);

        let context = match input.get("context") {
            Some(ctx_val) => Context::from_json_value(ctx_val.clone(), None).map_err(|e| {
                dek_policy_runtime::PolicyError::Invalid(format!("Context parse error: {}", e))
            })?,
            None => Context::empty(),
        };

        let entities = match input.get("entities") {
            Some(ent_val) => Entities::from_json_value(ent_val.clone(), None).map_err(|e| {
                dek_policy_runtime::PolicyError::Invalid(format!("Entities parse error: {}", e))
            })?,
            None => Entities::empty(),
        };

        let make_uid = |type_name: &str,
                        id: &str|
         -> std::result::Result<EntityUid, dek_policy_runtime::PolicyError> {
            if id.contains("::") {
                EntityUid::from_str(id).map_err(|e| {
                    dek_policy_runtime::PolicyError::Invalid(format!(
                        "EntityUid parse error: {}",
                        e
                    ))
                })
            } else {
                Ok(EntityUid::from_type_name_and_id(
                    EntityTypeName::from_str(type_name).map_err(|e| {
                        dek_policy_runtime::PolicyError::Invalid(format!(
                            "EntityTypeName parse error: {}",
                            e
                        ))
                    })?,
                    EntityId::from_str(id).map_err(|e| {
                        dek_policy_runtime::PolicyError::Invalid(format!(
                            "EntityId parse error: {}",
                            e
                        ))
                    })?,
                ))
            }
        };

        let principal_uid = make_uid("User", principal)?;
        let action_uid = make_uid("Action", action)?;
        let resource_uid = make_uid("Resource", resource)?;

        let request = Request::new(principal_uid, action_uid, resource_uid, context, None)
            .map_err(|e| {
                dek_policy_runtime::PolicyError::Eval(format!("Cedar Request Error: {}", e))
            })?;

        let authorizer = Authorizer::new();
        let answer = authorizer.is_authorized(&request, &self.policy_set, &entities);

        let allowed = answer.decision() == Decision::Allow;

        let decision_res = PolicyDecision {
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
        };

        if !cache_key.is_empty() {
            self.cache.insert(cache_key, decision_res.clone());
        }

        Ok(decision_res)
    }

    async fn clear_cache(&self) {
        self.cache.invalidate_all();
    }

    fn version(&self) -> String {
        "cedar-v1.0.0".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cedar_json_parse() -> anyhow::Result<()> {
        let ctx_val = json!({ "ip": "127.0.0.1" });
        let _ctx = Context::from_json_value(ctx_val, None)?;

        let ent_val = serde_json::json!([]);
        let _ents = Entities::from_json_value(ent_val, None)?;

        Ok(())
    }
}
