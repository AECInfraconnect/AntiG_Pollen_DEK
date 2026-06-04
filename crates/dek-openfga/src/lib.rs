use anyhow::Result;
use async_trait::async_trait;
use dek_policy_runtime::{PolicyDecision, PolicyRuntime};
use reqwest::Client;
use serde_json::json;

pub struct OpenFgaAdapter {
    endpoint: String,
    store_id: String,
    client: Client,
}

impl OpenFgaAdapter {
    pub fn new(endpoint: &str, store_id: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            store_id: store_id.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl PolicyRuntime for OpenFgaAdapter {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision> {
        let principal = input
            .get("principal")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let action = input.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let resource = input
            .get("resource")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        println!(
            "Checking OpenFGA at {}/stores/{}/check",
            self.endpoint, self.store_id
        );
        println!(
            "Tuple: user={}, relation={}, object={}",
            principal, action, resource
        );

        let url = format!("{}/stores/{}/check", self.endpoint, self.store_id);
        let payload = json!({
            "tuple_key": {
                "user": principal,
                "relation": action,
                "object": resource
            }
        });

        // Make the real HTTP request to OpenFGA
        let mut allowed = false;
        let reason;

        match self.client.post(&url).json(&payload).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    if let Ok(resp_json) = res.json::<serde_json::Value>().await {
                        allowed = resp_json
                            .get("allowed")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        reason = if allowed {
                            "OpenFGA remote check allowed".to_string()
                        } else {
                            "OpenFGA remote check denied".to_string()
                        };
                    } else {
                        reason = "Failed to parse OpenFGA JSON response".to_string();
                    }
                } else {
                    reason = format!("OpenFGA returned status: {}", res.status());
                }
            }
            Err(e) => {
                reason = format!("Failed to connect to OpenFGA: {}", e);
            }
        }

        Ok(PolicyDecision {
            evaluator_id: "openfga_remote".to_string(),
            evaluator_type: "remote_pdp".to_string(),
            required: true,
            status: "success".to_string(),
            decision: if allowed {
                "allow".to_string()
            } else {
                "deny".to_string()
            },
            allow: allowed,
            reason,
            effects: serde_json::json!({}),
            obligations: vec![],
            metadata: serde_json::json!({ "store_id": self.store_id }),
        })
    }

    fn version(&self) -> String {
        "openfga-v1.0.0".to_string()
    }
}
