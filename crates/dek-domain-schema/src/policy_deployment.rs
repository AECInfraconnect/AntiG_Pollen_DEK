// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ControlMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    Simulated,
    Active,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyDeployment {
    pub schema_version: String,
    pub tenant_id: String,
    pub deployment_id: String,
    pub status: DeploymentStatus,
    pub preset_id: Option<String>,
    pub preset_version: Option<String>,
    pub control_mode: ControlMode,
    pub targets: serde_json::Value,
    pub params: serde_json::Value,
    pub control_bindings: Vec<ControlBinding>,
    pub rollback_snapshot_json: Option<String>,

    // P2 Fleet Management target rules
    #[serde(default)]
    pub target_rollout_ring: Option<crate::dek_device::RolloutRing>,
    #[serde(default)]
    pub target_device_groups: Vec<String>,

    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BindingStatus {
    Pending,
    Applied,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ControlBinding {
    pub binding_id: String,
    pub agent_id: String,
    pub pep_type: String,
    pub action: String,
    pub status: BindingStatus,
    pub config_backup_id: Option<String>,
    pub binding_json: serde_json::Value,
}
