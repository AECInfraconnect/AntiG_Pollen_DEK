// SPDX-License-Identifier: Apache-2.0
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReputationStatus {
    Allowed,
    Denied(String),
    Unknown,
}

#[derive(Debug)]
pub struct ReputationRegistry {
    entries: RwLock<HashMap<String, ReputationStatus>>,
}

impl Default for ReputationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationRegistry {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_status(&self, mcp_id: &str, status: ReputationStatus) {
        if let Ok(mut w) = self.entries.write() {
            w.insert(mcp_id.to_string(), status);
        }
    }

    pub fn get_status(&self, mcp_id: &str) -> ReputationStatus {
        if let Ok(r) = self.entries.read() {
            r.get(mcp_id).cloned().unwrap_or(ReputationStatus::Unknown)
        } else {
            ReputationStatus::Unknown
        }
    }

    pub fn is_denied(&self, mcp_id: &str) -> bool {
        matches!(self.get_status(mcp_id), ReputationStatus::Denied(_))
    }
}
