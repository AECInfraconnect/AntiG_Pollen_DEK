// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardMode {
    Observe,
    Warn,
    #[default]
    Enforce,
    StrictDeny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardThresholds {
    pub injection_warn_score: f32,
    pub injection_deny_score: f32,
    pub pii_confidence: f32,
}

impl Default for GuardThresholds {
    fn default() -> Self {
        Self {
            injection_warn_score: 0.45,
            injection_deny_score: 0.75,
            pii_confidence: 0.80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardConfig {
    pub mode: GuardMode,
    pub request_guard_enabled: bool,
    pub response_guard_enabled: bool,
    pub telemetry_enabled: bool,
    pub enable_classifier: bool,
    pub enable_ner: bool,
    pub ner_provider: Option<ThirdPartyNerConfig>,
    pub enable_spotlight: bool,
    pub output_canary: Option<String>,
    pub thresholds: GuardThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NerProviderKind {
    Gliner,
    CustomHttp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThirdPartyNerConfig {
    pub provider_id: String,
    pub provider_kind: NerProviderKind,
    pub endpoint: String,
    pub labels: Vec<String>,
    pub min_confidence: f32,
    pub timeout_ms: u64,
}

impl ThirdPartyNerConfig {
    pub fn gliner(endpoint: impl Into<String>) -> Self {
        Self {
            provider_id: "gliner".to_string(),
            provider_kind: NerProviderKind::Gliner,
            endpoint: endpoint.into(),
            labels: vec![
                "person".to_string(),
                "address".to_string(),
                "organization".to_string(),
            ],
            min_confidence: 0.80,
            timeout_ms: 80,
        }
    }
}

impl Default for GuardConfig {
    fn default() -> Self {
        Self {
            mode: GuardMode::Enforce,
            request_guard_enabled: true,
            response_guard_enabled: true,
            telemetry_enabled: true,
            enable_classifier: false,
            enable_ner: false,
            ner_provider: None,
            enable_spotlight: true,
            output_canary: None,
            thresholds: GuardThresholds::default(),
        }
    }
}
