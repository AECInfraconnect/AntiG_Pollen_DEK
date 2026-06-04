use anyhow::Result;
use serde_json::Value;

pub trait ResponseRedactor {
    fn redact(&self, response: Value, obligations: &[String]) -> Result<Value>;
}

pub struct PiiRedactor;

impl PiiRedactor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PiiRedactor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseRedactor for PiiRedactor {
    fn redact(&self, mut response: Value, obligations: &[String]) -> Result<Value> {
        let should_redact = obligations
            .iter()
            .any(|o| o == "redact_response" || o == "redact_pii");

        if !should_redact {
            return Ok(response);
        }

        // Extremely basic redaction logic for demonstration purposes
        if let Some(obj) = response.as_object_mut() {
            if let Some(result) = obj.get_mut("result") {
                if let Some(res_obj) = result.as_object_mut() {
                    for (k, v) in res_obj.iter_mut() {
                        let key_lower = k.to_lowercase();
                        if key_lower.contains("secret")
                            || key_lower.contains("password")
                            || key_lower.contains("token")
                            || key_lower.contains("key")
                        {
                            *v = Value::String("[REDACTED]".to_string());
                        }
                    }
                }
            }
        }

        Ok(response)
    }
}
