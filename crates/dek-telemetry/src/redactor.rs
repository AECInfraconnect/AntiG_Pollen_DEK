// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![allow(clippy::expect_used, clippy::len_without_is_empty)]
use regex::Regex;
use serde_json::Value;

pub struct Redactor {
    patterns: Vec<Regex>,
}

impl Redactor {
    pub fn new() -> Self {
        let patterns = vec![
            // Basic regex to catch typical secrets.
            // Bearer tokens
            Regex::new(r"Bearer\s+[A-Za-z0-9\-\._~\+/]+")
                .expect("PII regex is a valid compile-time constant"), //
            // Basic Auth
            Regex::new(r"Basic\s+[A-Za-z0-9\+/=]+")
                .expect("PII regex is a valid compile-time constant"), //
            // Common API keys
            Regex::new(r"(?i)(api_key|apikey|sk_live|sk_test|sk-[a-zA-Z0-9]{32,})")
                .expect("PII regex is a valid compile-time constant"), //
        ];
        Self { patterns }
    }

    pub fn redact_string(&self, mut s: String) -> String {
        for pattern in &self.patterns {
            s = pattern.replace_all(&s, "[REDACTED]").to_string();
        }
        s
    }

    pub fn redact_value(&self, value: &mut Value) {
        match value {
            Value::String(s) => {
                *s = self.redact_string(s.clone());
            }
            Value::Array(arr) => {
                for v in arr.iter_mut() {
                    self.redact_value(v);
                }
            }
            Value::Object(obj) => {
                for (k, v) in obj.iter_mut() {
                    // Check if the key indicates a secret and mask entire value
                    if k.to_lowercase().contains("token")
                        || k.to_lowercase().contains("password")
                        || k.to_lowercase().contains("secret")
                    {
                        if let Value::String(_) = v {
                            *v = Value::String("[REDACTED_SECRET]".to_string());
                            continue;
                        }
                    }
                    self.redact_value(v);
                }
            }
            _ => {}
        }
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}
