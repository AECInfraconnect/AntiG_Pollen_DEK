// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::capability_snapshot_v2::{ControlDomainV2, ControlLevelV2, RuntimeMode, SetupAction};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OwaspRiskId {
    Llm01PromptInjection,
    Llm02SensitiveInformationDisclosure,
    Llm03SupplyChain,
    Llm04DataAndModelPoisoning,
    Llm05ImproperOutputHandling,
    Llm06ExcessiveAgency,
    Llm07SystemPromptLeakage,
    Llm08VectorEmbeddingWeakness,
    Llm09Misinformation,
    Llm10UnboundedConsumption,
    Ast01MaliciousSkills,
    Ast02SupplyChainCompromise,
    Ast03OverPrivilegedSkills,
    Ast04InsecureMetadata,
    Ast05UntrustedExternalInstructions,
    Ast06WeakIsolation,
    Ast07UpdateDrift,
    Ast08PoorScanning,
    Ast09NoGovernance,
    Ast10CrossPlatformReuse,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLevel {
    NotApplicable,
    NotVisible,
    ObserveOnly,
    WarnOnly,
    AskBeforeAction,
    EnforcePartial,
    EnforceFull,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityReasonCode {
    FullyProtected,
    ObserveOnlyNoLocalControlMethod,
    ObserveOnlyPermissionRequired,
    ObserveOnlyUnsupportedOs,
    PartialAgentToolControlOnly,
    PartialNetworkControlOnly,
    NeedsMcpConfigChange,
    NeedsBrowserExtension,
    NeedsOsNetworkExtension,
    NeedsAdminPrivilege,
    NeedsServerSideProxy,
    NeedsCloudEnrollment,
    SimulatorOnly,
    PolicyNotApplicableToAgent,
    WarmCheckFailed,
    BundleRejected,
    ContractVersionMismatch,
}

impl CapabilityReasonCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullyProtected => "fully_protected",
            Self::ObserveOnlyNoLocalControlMethod => "observe_only_no_local_control_method",
            Self::ObserveOnlyPermissionRequired => "observe_only_permission_required",
            Self::ObserveOnlyUnsupportedOs => "observe_only_unsupported_os",
            Self::PartialAgentToolControlOnly => "partial_agent_tool_control_only",
            Self::PartialNetworkControlOnly => "partial_network_control_only",
            Self::NeedsMcpConfigChange => "needs_mcp_config_change",
            Self::NeedsBrowserExtension => "needs_browser_extension",
            Self::NeedsOsNetworkExtension => "needs_os_network_extension",
            Self::NeedsAdminPrivilege => "needs_admin_privilege",
            Self::NeedsServerSideProxy => "needs_server_side_proxy",
            Self::NeedsCloudEnrollment => "needs_cloud_enrollment",
            Self::SimulatorOnly => "simulator_only",
            Self::PolicyNotApplicableToAgent => "policy_not_applicable_to_agent",
            Self::WarmCheckFailed => "warm_check_failed",
            Self::BundleRejected => "bundle_rejected",
            Self::ContractVersionMismatch => "contract_version_mismatch",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SecurityCoverageItem {
    pub risk_id: OwaspRiskId,
    pub policy_id: String,
    pub agent_id: Option<String>,
    pub entity_id: Option<String>,
    pub domain: ControlDomainV2,
    pub requested_level: ControlLevelV2,
    pub achievable_level: CoverageLevel,
    pub observe_capable: bool,
    pub enforce_capable: bool,
    pub enforced_for_real: bool,
    pub chosen_control_method: Option<String>,
    pub decision_engine: Option<String>,
    pub reason_code: CapabilityReasonCode,
    pub friendly_en: String,
    pub friendly_th: String,
    pub setup_actions: Vec<SetupAction>,
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoverageVerdict {
    Full,
    Partial,
    ObserveOnly,
    NeedsSetup,
    NotApplicable,
    Unsupported,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PolicyCoverageSummary {
    pub requested_level: ControlLevelV2,
    pub overall_achievable_level: CoverageLevel,
    pub verdict: CoverageVerdict,
    pub title_en: String,
    pub title_th: String,
    pub friendly_en: String,
    pub friendly_th: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PolicyCoverageReport {
    pub schema_version: String,
    pub coverage_id: String,
    pub tenant_id: String,
    pub device_id: String,
    pub local_cloud_profile: String,
    pub mode: RuntimeMode,
    pub generated_at: DateTime<Utc>,
    pub summary: PolicyCoverageSummary,
    pub items: Vec<SecurityCoverageItem>,
    pub setup_actions: Vec<SetupAction>,
}
