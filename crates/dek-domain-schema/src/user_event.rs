// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedText {
    pub th: String,
    pub en: String,
}

impl LocalizedText {
    pub fn new(en: impl Into<String>, th: impl Into<String>) -> Self {
        Self {
            th: th.into(),
            en: en.into(),
        }
    }

    pub fn with_detail(mut self, detail: impl std::fmt::Display) -> Self {
        let detail = detail.to_string();
        self.en = format!("{} ({})", self.en, detail);
        self.th = format!("{} ({})", self.th, detail);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Discovery,
    Security,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredUserAction {
    pub kind: String,
    pub label: LocalizedText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserVisibleEvent {
    pub id: String,
    pub timestamp: String,
    pub category: EventCategory,
    pub status: EventStatus,
    pub message: LocalizedText,
    pub action_required: Option<RequiredUserAction>,
}
