// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use chrono::{DateTime, Utc};
use dek_domain_schema::{
    capability_inventory::AgentCapabilityInventory,
    deployment_session::LocalizedText,
    feasibility::{ControlMethod, InternalPep, PolicyFeasibilityStatus, RequiredUserAction},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCapabilitySnapshot {
    pub snapshot_id: String,
    pub device_id: String,
    pub os: crate::OsInfo,
    pub agents: Vec<AgentCapabilityInventory>,
    pub methods: Vec<ControlMethodCapability>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMethodCapability {
    pub method: ControlMethod,
    pub internal_pep: InternalPep,
    pub status: CapabilityStatus,
    pub can_observe: bool,
    pub can_enforce: bool,
    pub requires_admin: bool,
    pub requires_user_approval: bool,
    pub confidence: f32,
    pub evidence: Vec<CapabilityEvidence>,
    pub user_message: LocalizedText,
    pub next_action: Option<RequiredUserAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub source: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Ready,
    ReadyAfterApproval,
    InstalledInactive,
    MissingPermission,
    MissingComponent,
    UnsupportedOnThisOs,
    UnsupportedForThisAgent,
    Unhealthy,
    Unknown,
}

pub fn capability_to_user_status(cap: &ControlMethodCapability) -> PolicyFeasibilityStatus {
    match cap.status {
        CapabilityStatus::Ready if cap.can_enforce => PolicyFeasibilityStatus::CanEnforceNow,
        CapabilityStatus::ReadyAfterApproval if cap.can_enforce => {
            PolicyFeasibilityStatus::CanEnforceAfterApproval
        }
        CapabilityStatus::Ready if cap.can_observe => PolicyFeasibilityStatus::CanObserveOnly,
        CapabilityStatus::MissingPermission | CapabilityStatus::MissingComponent => {
            PolicyFeasibilityStatus::NeedsSetup
        }
        CapabilityStatus::UnsupportedOnThisOs | CapabilityStatus::UnsupportedForThisAgent => {
            PolicyFeasibilityStatus::Unsupported
        }
        _ => PolicyFeasibilityStatus::Unknown,
    }
}
