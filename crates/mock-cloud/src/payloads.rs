// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde_json::json;

pub fn mock_policies_payload(cedar_src: &str) -> serde_json::Value {
    json!({
        "policies": [
            {
                "id": "pol_1",
                "type": "cedar",
                "content": cedar_src
            }
        ],
        "routes": [
            {
                "id": "route_tools_call",
                "priority": 100,
                "match_rule": {
                    "method": "tools/call",
                    "tool_category": serde_json::Value::Null,
                    "resource_type": "mcp_tool"
                },
                "pdp_required": ["cedar"]
            },
            {
                "id": "route_default",
                "priority": 10,
                "match_rule": {
                    "method": "*",
                    "tool_category": serde_json::Value::Null
                },
                "pdp_required": ["cedar"]
            }
        ]
    })
}

pub fn mock_registry_payload() -> serde_json::Value {
    json!({ "registry": {} })
}

pub fn mock_network_guardrails_payload() -> serde_json::Value {
    json!([])
}

pub fn hash_payload(val: &serde_json::Value) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(serde_json::to_vec(val).unwrap());
    hex::encode(h.finalize())
}
