use dek_policy_runtime::PolicyDecision;

pub fn initial_decision() -> PolicyDecision {
    PolicyDecision {
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
    }
}

pub fn merge_decision(combined: &mut PolicyDecision, res: PolicyDecision, strategy: &str) {
    combined.obligations.extend(res.obligations.clone());

    if let serde_json::Value::Object(mut combined_map) = combined.effects.clone() {
        if let serde_json::Value::Object(res_map) = res.effects.clone() {
            for (k, v) in res_map {
                combined_map.insert(k, v);
            }
        }
        combined.effects = serde_json::Value::Object(combined_map);
    }

    match strategy {
        "deny_overrides" => {
            if !res.allow {
                combined.allow = false;
                combined.decision = "deny".into();
                combined.reason = res.reason;
            }
        }
        "permit_overrides" => {
            if res.allow {
                combined.allow = true;
                combined.decision = "allow".into();
                combined.reason = res.reason;
            }
        }
        "first_applicable" => {
            // First applicable takes precedence, so we don't modify combined if it already has a non-default reason
            // For simplicity, we just use the first result that is definitive.
            if combined.reason == "All evaluators passed" {
                combined.allow = res.allow;
                combined.decision = res.decision.clone();
                combined.reason = res.reason.clone();
            }
        }
        _ => {
            // Default: deny overrides
            if !res.allow {
                combined.allow = false;
                combined.decision = "deny".into();
                combined.reason = res.reason;
            }
        }
    }
}
