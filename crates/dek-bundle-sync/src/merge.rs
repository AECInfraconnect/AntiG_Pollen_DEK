// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use anyhow::Result;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrecedenceLevel {
    Baseline = 1,
    Tenant = 2,
    DeviceGroup = 3,
    Device = 4,
    EmergencyDeny = 5,
}

/// Merge configuration safely enforcing precedence
pub fn merge_safe(configs: Vec<(PrecedenceLevel, Value)>) -> Result<Value> {
    let mut merged = serde_json::json!({});

    // Sort by precedence level, ascending
    let mut sorted_configs = configs;
    sorted_configs.sort_by_key(|(level, _)| *level as u8);

    for (level, config) in sorted_configs {
        if let Some(obj) = config.as_object() {
            for (k, v) in obj {
                // For EmergencyDeny, we might enforce specific overrides or denies
                // For now, higher precedence fully overwrites
                if level as u8 == PrecedenceLevel::EmergencyDeny as u8 {
                    // Inject emergency context
                    merged[k] = v.clone();
                } else {
                    merged[k] = v.clone();
                }
            }
        }
    }

    Ok(merged)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn test_merge_precedence() {
        let configs = vec![
            (
                PrecedenceLevel::Baseline,
                serde_json::json!({"a": 1, "b": 1}),
            ),
            (PrecedenceLevel::Tenant, serde_json::json!({"b": 2, "c": 2})),
            (
                PrecedenceLevel::EmergencyDeny,
                serde_json::json!({"a": 5, "d": 5}),
            ),
            (
                PrecedenceLevel::DeviceGroup,
                serde_json::json!({"c": 3, "d": 3}),
            ),
        ];

        let merged = merge_safe(configs).unwrap();
        assert_eq!(merged["a"], 5); // Emergency overrides baseline
        assert_eq!(merged["b"], 2); // Tenant overrides baseline
        assert_eq!(merged["c"], 3); // DeviceGroup overrides Tenant
        assert_eq!(merged["d"], 5); // Emergency overrides DeviceGroup
    }
}
