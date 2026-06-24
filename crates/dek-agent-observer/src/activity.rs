// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCounts {
    pub total_decisions: u32,
    pub denied_actions: u32,
    pub mcp_invocations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub timestamp: String,
    pub event_type: String, // "network_egress", "mcp_tool_call", "file_read", etc.
    pub decision: Option<String>,
    pub resource: String,
    pub reason: String,

    #[serde(default)]
    pub pep_plane: Option<String>,
    #[serde(default)]
    pub enforced_for_real: Option<bool>,
    #[serde(default)]
    pub status_badge: Option<String>,
    #[serde(default)]
    pub message_th: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySet {
    pub start_time: String,
    pub end_time: String,
    pub duration_seconds: u32,
    pub counts: ActivityCounts,
    pub items: Vec<ActivityItem>,
}

impl ActivitySet {
    pub fn new(start_time: String) -> Self {
        Self {
            start_time,
            end_time: "".into(),
            duration_seconds: 0,
            counts: ActivityCounts {
                total_decisions: 0,
                denied_actions: 0,
                mcp_invocations: 0,
            },
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: ActivityItem) {
        if item.decision.as_deref() == Some("deny") {
            self.counts.denied_actions += 1;
        }
        if item.event_type == "mcp_tool_call" {
            self.counts.mcp_invocations += 1;
        }
        if item.decision.is_some() {
            self.counts.total_decisions += 1;
        }
        self.items.push(item);
    }
}

pub fn group_into_sets(raw_events: Vec<ActivityItem>, _max_idle_seconds: u32) -> Vec<ActivitySet> {
    // A simple mock grouping logic for now.
    // In reality, this would iterate over timestamped events and split into sets when idle duration exceeds max_idle_seconds.
    let mut sets = Vec::new();
    if raw_events.is_empty() {
        return sets;
    }

    let mut current_set = ActivitySet::new(raw_events[0].timestamp.clone());
    for item in raw_events {
        current_set.add_item(item);
    }
    // Set end_time to last event
    if let Some(last) = current_set.items.last() {
        current_set.end_time = last.timestamp.clone();
        // In reality we'd parse timestamps to get duration
        current_set.duration_seconds = 10;
    }
    sets.push(current_set);
    sets
}
