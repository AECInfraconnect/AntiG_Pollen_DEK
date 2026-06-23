// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde_json::Value;
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input);

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

        // Write modified JSON payload to stdout
        if let Ok(output_str) = serde_json::to_string(&payload) {
            #[allow(clippy::print_stdout)]
            print!("{}", output_str);
        }
    }
}
