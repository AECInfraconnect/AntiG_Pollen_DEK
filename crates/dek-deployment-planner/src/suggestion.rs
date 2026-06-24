// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use dek_capability_registry::LocalCapabilitySnapshot;
use dek_domain_schema::{
    control_level::ControlLevel,
    deployment_session::LocalizedText,
    feasibility::{PolicyFeasibilityStatus, RequiredUserAction},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedPolicy {
    pub suggestion_id: String,
    pub policy_template_id: String,
    pub display_name: LocalizedText,
    pub description: LocalizedText,
    pub target_agent_ids: Vec<String>,
    pub recommended_control_level: ControlLevel,
    pub feasibility: PolicyFeasibilityStatus,
    pub confidence: f32,
    pub reason_codes: Vec<String>,
    pub setup_required: Vec<RequiredUserAction>,
}

pub trait PolicySuggestionEngine {
    fn suggest(&self, snapshot: &LocalCapabilitySnapshot) -> Vec<SuggestedPolicy>;
}
