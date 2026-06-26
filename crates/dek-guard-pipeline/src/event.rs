// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::GuardAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardFindingSummary {
    pub kind: String,
    pub confidence: f32,
    pub path: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardRemediation {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardEvent {
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub direction: String,
    pub action: GuardAction,
    pub injection_score: f32,
    pub categories: Vec<String>,
    pub findings: Vec<GuardFindingSummary>,
    pub normalization_steps: Vec<String>,
    pub remediation: Vec<GuardRemediation>,
}
