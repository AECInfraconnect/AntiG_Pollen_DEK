use dek_plugin_sdk::{EvalRequest, PolicyDecision};

struct HelloWorldPlugin;

impl HelloWorldPlugin {
    fn new() -> Self {
        Self
    }

    fn decide(&self, req: EvalRequest) -> Result<PolicyDecision, String> {
        // A simple hello world plugin that allows if action is "hello"
        if req.action.as_deref() == Some("hello") {
            Ok(PolicyDecision::allow(
                "hello-world-v1",
                "Action 'hello' is allowed by HelloWorldPlugin",
            ))
        } else {
            Ok(PolicyDecision::deny(
                "hello-world-v1",
                "Only action 'hello' is allowed",
            ))
        }
    }
}
