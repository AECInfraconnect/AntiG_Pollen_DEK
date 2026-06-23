use dek_plugin_sdk::{pollen_plugin, EvalRequest, PolicyDecision};

pollen_plugin!(HelloWorldPlugin);

struct HelloWorldPlugin;

impl HelloWorldPlugin {
    fn new() -> Self {
        Self
    }

    fn decide(&self, req: EvalRequest) -> Result<PolicyDecision, String> {
        // A simple hello world plugin that allows if action is "hello"
        if req.action.as_deref() == Some("hello") {
            Ok(PolicyDecision {
                evaluator_id: "hello-world-v1".to_string(),
                evaluator_type: "plugin".to_string(),
                required: true,
                allow: true,
                reason: Some("Action 'hello' is allowed by HelloWorldPlugin".to_string()),
                effects: vec![],
                obligations: vec![],
                metadata: Default::default(),
            })
        } else {
            Ok(PolicyDecision {
                evaluator_id: "hello-world-v1".to_string(),
                evaluator_type: "plugin".to_string(),
                required: true,
                allow: false,
                reason: Some("Only action 'hello' is allowed".to_string()),
                effects: vec![],
                obligations: vec![],
                metadata: Default::default(),
            })
        }
    }
}
