// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AcceptanceScenario {
    pub id: String,
    pub name: String,
    pub platforms: Vec<String>,
    pub requires_admin: bool,
    pub given: GivenState,
    pub when: Vec<ActionStep>,
    pub then: Vec<AssertionStep>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GivenState {
    pub mock_cloud_profile: Option<String>,
    pub enrolled_device: bool,
    pub active_bundle: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStep {
    RunDek(bool),
    CallMcpTool {
        agent_id: String,
        server_id: String,
        tool_name: String,
        resource_uri: Option<String>,
    },
    PublishPolicyBundle(String),
    WaitForHotReload(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssertionStep {
    FirstDecision(String),
    SecondDecision(String),
    MockCloudHasDecisionLogs(usize),
    ActiveBundleVersion(u64),
    NoProcessCrash(bool),
}

pub fn parse_scenario(content: &str) -> anyhow::Result<AcceptanceScenario> {
    let scenario: AcceptanceScenario = serde_yaml::from_str(content)?;
    Ok(scenario)
}
