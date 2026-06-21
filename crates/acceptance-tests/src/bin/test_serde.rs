// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DekConfig {
    pub policy_config: Option<PolicyConfig>,
}

fn main() -> anyhow::Result<()> {
    let json = serde_json::json!({
        "policy_config": {
            "routes": [1, 2, 3]
        }
    });
    
    let config: DekConfig = serde_json::from_value(json)?;
    println!("Parsed: {:?}", config);
    
    let val = serde_json::to_value(&config.policy_config)?;
    println!("Serialized policy_config: {}", val);
    
    Ok(())
}
