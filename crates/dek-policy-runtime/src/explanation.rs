// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionExplanation {
    pub decision: String,
    pub allow: bool,
    pub pdp_engine: Option<String>,
    pub pdp_reason_th: String,
    pub pep_plane: String,
    pub pep_capability: String,
    pub pep_reason_th: String,
    pub enforced_for_real: bool,
    pub success: bool,
    pub status_badge: StatusBadge,
    pub user_action_th: Option<String>,
    pub correlation_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusBadge {
    Ok,
    Degraded,
    Failed,
    ActionNeeded,
}
