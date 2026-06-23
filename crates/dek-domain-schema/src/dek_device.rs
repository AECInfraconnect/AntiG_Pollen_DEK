// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DekDevice {
    pub schema_version: String,
    pub device_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub hostname: String,
    pub os: String,
    pub dek_version: String,
    pub spiffe_id: String,
    pub pep_capabilities: Vec<String>,
    pub enforcement_ceiling: String,
    pub status: String,
    pub last_seen_at: String,

    // P2 Fleet Management
    #[serde(default)]
    pub rollout_ring: Option<RolloutRing>,
    #[serde(default)]
    pub device_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RolloutRing {
    Alpha,
    Beta,
    #[default]
    Stable,
}
