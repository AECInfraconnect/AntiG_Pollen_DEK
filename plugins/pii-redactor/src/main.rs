use serde_json::Value;
use std::fs;

fn main() {
    let input = fs::read_to_string("input.json").unwrap_or_default();
    if input.trim().is_empty() {
        return;
    }

    if let Ok(mut payload) = serde_json::from_str::<Value>(&input) {
        // Redact PII logic
        if let Some(msg) = payload.get_mut("message") {
            if let Some(msg_str) = msg.as_str() {
                let redacted = msg_str.replace("secret", "***").replace("password", "***");
                *msg = Value::String(redacted);
            }
        }

        // Write modified JSON payload
        if let Ok(output_str) = serde_json::to_string(&payload) {
            let _ = fs::write("output.json", output_str);
        }
    }
}
