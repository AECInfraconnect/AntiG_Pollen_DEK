// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use dek_plugin_sdk::{EvalRequest, PluginError, PolicyEvaluator, TransformPlugin};
use dek_policy_router::{PolicyRouter, Route};
use dek_policy_runtime::{PolicyDecision as OldPolicyDecision, PolicyError, PolicyRuntime};
use std::collections::HashMap;
use std::sync::Arc;

pub struct PluginHost {
    router: PolicyRouter,
    policy_evaluators: HashMap<String, Arc<dyn PolicyEvaluator>>,
    transform_plugins: HashMap<String, Arc<dyn TransformPlugin>>,
}

struct EvaluatorAdapter {
    evaluator: Arc<dyn PolicyEvaluator>,
}

#[async_trait::async_trait]
impl PolicyRuntime for EvaluatorAdapter {
    async fn evaluate(&self, input: serde_json::Value) -> Result<OldPolicyDecision, PolicyError> {
        let req = EvalRequest {
            request_id: "auto-req".into(),
            tenant_id: None,
            subject: None,
            action: None,
            resource: None,
            payload: input,
            context: Default::default(),
        };

        match self.evaluator.evaluate(req).await {
            Ok(decision) => {
                let is_allow = decision.is_allow();
                Ok(OldPolicyDecision {
                    evaluator_id: decision.evaluator_id,
                    evaluator_type: decision.evaluator_type,
                    required: decision.required,
                    status: "success".into(),
                    decision: if is_allow {
                        "allow".into()
                    } else {
                        "deny".into()
                    },
                    allow: is_allow,
                reason: decision.reason,
                effects: decision.effects,
                obligations: decision.obligations,
                metadata: decision.metadata,
            }),
            Err(PluginError::Unavailable(msg)) => Err(PolicyError::Unavailable(msg)),
            Err(e) => Err(PolicyError::Eval(e.to_string())),
        }
    }

    fn version(&self) -> String {
        self.evaluator.identity().version.clone()
    }
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            router: PolicyRouter::new(),
            policy_evaluators: HashMap::new(),
            transform_plugins: HashMap::new(),
        }
    }

    pub fn set_routes(&mut self, routes: Vec<Route>) {
        self.router.set_routes(routes);
    }

    pub fn register_evaluator(&mut self, evaluator: Arc<dyn PolicyEvaluator>) {
        let id = evaluator.identity().id.clone();
        self.policy_evaluators.insert(id.clone(), evaluator.clone());

        let adapter = EvaluatorAdapter { evaluator };
        self.router.register_evaluator(&id, Box::new(adapter));
    }

    pub fn register_transform(&mut self, transform: Arc<dyn TransformPlugin>) {
        let id = transform.identity().id.clone();
        self.transform_plugins.insert(id, transform);
    }

    pub async fn authorize(&self, payload: serde_json::Value) -> anyhow::Result<OldPolicyDecision> {
        self.router.authorize(payload).await
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}
