// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LocalCapabilitySnapshotV2 {
    pub schema_version: String,
    pub tenant_id: String,
    pub device_id: String,
    pub os: OsInfoV2,
    pub mode: RuntimeMode,
    pub generated_at: DateTime<Utc>,
    pub control_methods: Vec<ControlMethodCapabilityV2>,
    pub observation_sources: Vec<ObservationSourceCapability>,
    pub setup_actions: Vec<SetupAction>,
    pub contract: ContractCompatibilityStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct OsInfoV2 {
    pub family: String,
    pub version: String,
    pub arch: String,
    pub is_server: bool,
    pub elevated: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    #[default]
    DesktopSimple,
    DesktopAdvanced,
    EnterpriseServer,
    Sovereign,
    AirGap,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ControlMethodCapabilityV2 {
    pub method_id: String,
    pub display_name_en: String,
    pub display_name_th: String,
    pub domains: Vec<ControlDomainV2>,
    pub max_level: ControlLevelV2,
    pub status: MethodReadiness,
    pub maturity: MethodMaturity,
    pub install_state: InstallState,
    pub warm_check: Option<WarmCheckStatus>,
    pub setup_action_ids: Vec<String>,
    pub limitations_en: Vec<String>,
    pub limitations_th: Vec<String>,
}

impl ControlMethodCapabilityV2 {
    pub fn can_control(&self, domain: &ControlDomainV2) -> bool {
        self.domains.iter().any(|d| d == domain)
    }

    pub fn can_enforce_for_real(&self, requested: &ControlLevelV2) -> bool {
        self.status == MethodReadiness::Available
            && self.maturity != MethodMaturity::Simulator
            && self.max_level >= *requested
            && *requested >= ControlLevelV2::Enforce
    }

    pub fn can_observe(&self) -> bool {
        matches!(
            self.status,
            MethodReadiness::Available
                | MethodReadiness::Degraded
                | MethodReadiness::NeedsPermission
                | MethodReadiness::NeedsConfiguration
                | MethodReadiness::SimulatorOnly
        ) && self.max_level >= ControlLevelV2::Observe
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlDomainV2 {
    McpToolCall,
    NetworkEgress,
    Dns,
    FileAccess,
    ProcessLaunch,
    BrowserAiSession,
    IdentityUse,
    SkillInstall,
    SkillRuntime,
    TokenCost,
    PromptContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevelV2 {
    Observe,
    Warn,
    Ask,
    Enforce,
    StrictDeny,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MethodReadiness {
    Available,
    Degraded,
    NeedsInstall,
    NeedsPermission,
    NeedsConfiguration,
    SimulatorOnly,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstallState {
    BuiltIn,
    Installed,
    InstalledButDisabled,
    NotInstalled,
    ExternalRequired,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MethodMaturity {
    Production,
    Beta,
    Preview,
    Simulator,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WarmCheckStatus {
    NotRun,
    Passed,
    Failed,
    NotApplicable,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ObservationSourceCapability {
    pub source_id: String,
    pub display_name_en: String,
    pub display_name_th: String,
    pub status: MethodReadiness,
    pub domains: Vec<ControlDomainV2>,
    pub privacy_note_en: String,
    pub privacy_note_th: String,
    pub setup_action_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetupAction {
    pub action_id: String,
    pub title_en: String,
    pub title_th: String,
    pub detail_en: String,
    pub detail_th: String,
    pub requires_admin: bool,
    pub requires_restart: bool,
    pub estimated_minutes: u8,
    pub docs_path: Option<String>,
    pub safe_to_skip: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContractCompatibilityStatus {
    pub local_contract_version: String,
    pub compatible_cloud_contracts: Vec<String>,
    pub status: String,
    pub reason_code: Option<String>,
}
