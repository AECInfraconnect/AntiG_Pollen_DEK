use crate::context::PolicyContext;
use crate::Route;
use dek_policy_runtime::PolicyDecision;

#[allow(clippy::result_large_err)]
pub fn match_route<'a>(
    routes: &'a [Route],
    ctx: &PolicyContext<'_>,
) -> Result<&'a Route, PolicyDecision> {
    for route in routes {
        let mut matches = true;

        if let Some(ref m) = route.match_rule.method {
            if m != "*" && m != ctx.method {
                matches = false;
            }
        }
        if let Some(ref cat) = route.match_rule.tool_category {
            if Some(cat.as_str()) != ctx.tool_category && cat != "*" {
                matches = false;
            }
        }
        if let Some(ref res) = route.match_rule.resource_type {
            if Some(res.as_str()) != ctx.resource_type && res != "*" {
                matches = false;
            }
        }
        if let Some(ref sev) = route.match_rule.severity_level {
            if Some(sev.as_str()) != ctx.severity_level && sev != "*" {
                matches = false;
            }
        }

        if matches {
            return Ok(route);
        }
    }

    Err(PolicyDecision {
        evaluator_id: "router_default".into(),
        evaluator_type: "router".into(),
        required: true,
        status: "success".into(),
        decision: "deny".into(),
        allow: false,
        reason: "no matching route".into(),
        effects: serde_json::json!({}),
        obligations: vec![],
        metadata: serde_json::json!({ "reason": "no route matched" }),
        explanation: None,
        user_action_required: false,
        user_action_th: None,
    })
}
