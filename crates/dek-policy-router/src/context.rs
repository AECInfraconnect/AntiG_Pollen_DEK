use serde_json::Value;
use std::sync::Arc;

pub struct PolicyContext<'a> {
    pub payload: Arc<Value>,
    pub method: &'a str,
    pub tool_category: Option<&'a str>,
    pub resource_type: Option<&'a str>,
    pub severity_level: Option<&'a str>,
}

impl<'a> PolicyContext<'a> {
    pub fn extract(payload: &'a Arc<Value>) -> Self {
        let method = payload
            .get("request_type")
            .and_then(|v| v.as_str())
            .or_else(|| {
                payload
                    .get("mcp")
                    .and_then(|mcp| mcp.get("method"))
                    .and_then(|v| v.as_str())
            })
            .or_else(|| payload.get("action").and_then(|v| v.as_str()))
            .unwrap_or("");

        let tool_category = payload
            .get("mcp")
            .and_then(|mcp| mcp.get("category"))
            .and_then(|v| v.as_str());

        let resource_type = payload.get("resource").and_then(|v| {
            if v.is_object() {
                v.get("kind")
                    .or_else(|| v.get("resource_type"))
                    .or_else(|| v.get("type"))
                    .and_then(|k| k.as_str())
            } else {
                v.as_str()
            }
        });

        let severity_level = payload.get("severity").and_then(|v| v.as_str());

        Self {
            payload: payload.clone(),
            method,
            tool_category,
            resource_type,
            severity_level,
        }
    }
}
