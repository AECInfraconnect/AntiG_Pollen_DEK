use crate::context::PolicyContext;
use crate::EngineSelector;
use crate::Route;

pub struct EnginePlan {
    pub evaluate_queue: Vec<String>,
    pub fallback_queue: Vec<String>,
    pub shadow_pdps: Vec<String>,
}

impl EnginePlan {
    pub fn build(
        route: &Route,
        ctx: &PolicyContext<'_>,
        available_evaluators: &[String],
        selected_pool_pdp: Option<String>,
    ) -> Option<Self> {
        let mut to_evaluate = route.pdp_required.clone();
        for cond in &route.pdp_conditional {
            if ctx.payload.get(&cond.required_payload_key).is_some()
                || cond.required_payload_key == "*"
            {
                to_evaluate.push(cond.evaluator.clone());
            }
        }
        if let Some(pdp) = selected_pool_pdp {
            to_evaluate.push(pdp);
        }

        if to_evaluate.is_empty() && route.primary_pdp_id.is_empty() {
            match EngineSelector::resolve(ctx.method, &ctx.payload, available_evaluators) {
                Some(engine) => {
                    to_evaluate.push(engine);
                }
                None => {
                    return None; // No suitable engine
                }
            }
        }

        if !route.primary_pdp_id.is_empty() {
            to_evaluate.insert(0, route.primary_pdp_id.clone());
        }

        Some(Self {
            evaluate_queue: to_evaluate,
            fallback_queue: route.fallback_pdp_ids.clone(),
            shadow_pdps: route.shadow_pdp_ids.clone(),
        })
    }
}
