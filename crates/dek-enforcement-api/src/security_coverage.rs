// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use dek_domain_schema::{
    capability_snapshot_v2::{
        ControlDomainV2, ControlLevelV2, ControlMethodCapabilityV2, LocalCapabilitySnapshotV2,
        MethodMaturity, MethodReadiness, RuntimeMode, SetupAction,
    },
    security_coverage::{
        CapabilityReasonCode, CoverageLevel, CoverageVerdict, OwaspRiskId, PolicyCoverageReport,
        PolicyCoverageSummary, SecurityCoverageItem,
    },
};

#[derive(Clone, Debug)]
pub struct CoverageRequest {
    pub tenant_id: String,
    pub device_id: String,
    pub agent_id: Option<String>,
    pub entity_id: Option<String>,
    pub policy_id: String,
    pub requested_level: ControlLevelV2,
    pub mode: RuntimeMode,
    pub local_cloud_profile: String,
    pub evidence_ids: Vec<String>,
}

pub fn assess_policy_coverage(
    req: CoverageRequest,
    snapshot: &LocalCapabilitySnapshotV2,
) -> PolicyCoverageReport {
    let profile = policy_profile(&req.policy_id);
    let mut items = Vec::new();

    for risk in &profile.risks {
        for domain in &profile.domains {
            items.push(coverage_item(&req, snapshot, risk.clone(), domain.clone()));
        }
    }

    let setup_actions = collect_setup_actions(snapshot, &items);
    let summary = summarize(&req, &profile.title_en, &profile.title_th, &items);

    PolicyCoverageReport {
        schema_version: "security-coverage.v1".into(),
        coverage_id: format!("cov_{}", uuid::Uuid::new_v4()),
        tenant_id: req.tenant_id,
        device_id: req.device_id,
        local_cloud_profile: req.local_cloud_profile,
        mode: req.mode,
        generated_at: chrono::Utc::now(),
        summary,
        items,
        setup_actions,
    }
}

#[derive(Clone)]
struct PolicyProfile {
    title_en: String,
    title_th: String,
    risks: Vec<OwaspRiskId>,
    domains: Vec<ControlDomainV2>,
}

fn policy_profile(policy_id: &str) -> PolicyProfile {
    let id = policy_id.to_ascii_lowercase();
    if id.contains("pii") || id.contains("redact") || id.contains("external_llm") {
        return PolicyProfile {
            title_en: "Block PII to External AI".into(),
            title_th: "บล็อกข้อมูลส่วนบุคคลไปยัง AI ภายนอก".into(),
            risks: vec![
                OwaspRiskId::Llm02SensitiveInformationDisclosure,
                OwaspRiskId::Llm06ExcessiveAgency,
            ],
            domains: vec![
                ControlDomainV2::McpToolCall,
                ControlDomainV2::PromptContent,
                ControlDomainV2::NetworkEgress,
            ],
        };
    }
    if id.contains("prompt") || id.contains("injection") {
        return PolicyProfile {
            title_en: "Detect Prompt Injection".into(),
            title_th: "ตรวจจับ Prompt Injection".into(),
            risks: vec![
                OwaspRiskId::Llm01PromptInjection,
                OwaspRiskId::Ast05UntrustedExternalInstructions,
            ],
            domains: vec![ControlDomainV2::PromptContent, ControlDomainV2::McpToolCall],
        };
    }
    if id.contains("shadow") || id.contains("network") || id.contains("egress") {
        return PolicyProfile {
            title_en: "Block Shadow AI Cloud APIs".into(),
            title_th: "บล็อก Shadow AI Cloud APIs".into(),
            risks: vec![
                OwaspRiskId::Llm02SensitiveInformationDisclosure,
                OwaspRiskId::Llm10UnboundedConsumption,
                OwaspRiskId::Ast09NoGovernance,
            ],
            domains: vec![ControlDomainV2::NetworkEgress, ControlDomainV2::Dns],
        };
    }
    if id.contains("skill") || id.contains("plugin") || id.contains("supply") {
        return PolicyProfile {
            title_en: "Skill and Plugin Supply Chain Guard".into(),
            title_th: "ป้องกันความเสี่ยง Supply Chain ของ Skill และ Plugin".into(),
            risks: vec![
                OwaspRiskId::Llm03SupplyChain,
                OwaspRiskId::Ast01MaliciousSkills,
                OwaspRiskId::Ast02SupplyChainCompromise,
            ],
            domains: vec![
                ControlDomainV2::SkillInstall,
                ControlDomainV2::SkillRuntime,
                ControlDomainV2::FileAccess,
            ],
        };
    }
    if id.contains("identity") {
        return PolicyProfile {
            title_en: "Identity Use Guard".into(),
            title_th: "ควบคุมการใช้ Identity".into(),
            risks: vec![
                OwaspRiskId::Llm06ExcessiveAgency,
                OwaspRiskId::Ast03OverPrivilegedSkills,
            ],
            domains: vec![ControlDomainV2::IdentityUse, ControlDomainV2::McpToolCall],
        };
    }
    if id.contains("budget") || id.contains("token") || id.contains("cost") {
        return PolicyProfile {
            title_en: "Cost and Token Limit".into(),
            title_th: "จำกัดค่าใช้จ่ายและ Token".into(),
            risks: vec![OwaspRiskId::Llm10UnboundedConsumption],
            domains: vec![ControlDomainV2::TokenCost, ControlDomainV2::NetworkEgress],
        };
    }
    PolicyProfile {
        title_en: "Require Approval for Risky Tools".into(),
        title_th: "ขออนุมัติก่อนใช้เครื่องมือเสี่ยง".into(),
        risks: vec![
            OwaspRiskId::Llm06ExcessiveAgency,
            OwaspRiskId::Ast03OverPrivilegedSkills,
        ],
        domains: vec![ControlDomainV2::McpToolCall, ControlDomainV2::SkillRuntime],
    }
}

fn coverage_item(
    req: &CoverageRequest,
    snapshot: &LocalCapabilitySnapshotV2,
    risk_id: OwaspRiskId,
    domain: ControlDomainV2,
) -> SecurityCoverageItem {
    let method = choose_method(snapshot, &domain, &req.requested_level);
    match method {
        Some(method) => method_item(req, snapshot, risk_id, domain, method),
        None => no_method_item(req, snapshot, risk_id, domain),
    }
}

fn choose_method<'a>(
    snapshot: &'a LocalCapabilitySnapshotV2,
    domain: &ControlDomainV2,
    requested: &ControlLevelV2,
) -> Option<&'a ControlMethodCapabilityV2> {
    let mut candidates = snapshot
        .control_methods
        .iter()
        .filter(|m| m.can_control(domain))
        .collect::<Vec<_>>();

    candidates.sort_by_key(|m| method_rank(m, requested));
    candidates.into_iter().next()
}

fn method_rank(method: &ControlMethodCapabilityV2, requested: &ControlLevelV2) -> u8 {
    if method.can_enforce_for_real(requested) {
        return 0;
    }
    match (&method.status, &method.maturity) {
        (MethodReadiness::Available, MethodMaturity::Production | MethodMaturity::Beta) => 1,
        (MethodReadiness::Available, MethodMaturity::Preview) => 2,
        (MethodReadiness::Degraded, _) => 3,
        (MethodReadiness::NeedsConfiguration, _) => 4,
        (MethodReadiness::NeedsPermission, _) => 5,
        (MethodReadiness::NeedsInstall, _) => 6,
        (MethodReadiness::SimulatorOnly, _) | (_, MethodMaturity::Simulator) => 7,
        (MethodReadiness::Unsupported, _) => 8,
        (MethodReadiness::Failed, _) => 9,
    }
}

fn method_item(
    req: &CoverageRequest,
    snapshot: &LocalCapabilitySnapshotV2,
    risk_id: OwaspRiskId,
    domain: ControlDomainV2,
    method: &ControlMethodCapabilityV2,
) -> SecurityCoverageItem {
    let reason = reason_for_method(method, &domain);
    let enforce_real = method.can_enforce_for_real(&req.requested_level);
    let achievable_level = if enforce_real {
        CoverageLevel::EnforceFull
    } else if method.status == MethodReadiness::Available
        && method.max_level >= ControlLevelV2::Ask
        && method.maturity != MethodMaturity::Simulator
    {
        CoverageLevel::AskBeforeAction
    } else if method.can_observe() {
        CoverageLevel::ObserveOnly
    } else {
        CoverageLevel::NotVisible
    };

    let actions = method
        .setup_action_ids
        .iter()
        .filter_map(|id| find_setup_action(snapshot, id))
        .collect::<Vec<_>>();
    let friendly = friendly_for(&reason, &domain, method);

    SecurityCoverageItem {
        risk_id,
        policy_id: req.policy_id.clone(),
        agent_id: req.agent_id.clone(),
        entity_id: req.entity_id.clone(),
        domain,
        requested_level: req.requested_level.clone(),
        achievable_level,
        observe_capable: method.can_observe(),
        enforce_capable: enforce_real,
        enforced_for_real: enforce_real,
        chosen_control_method: Some(method.method_id.clone()),
        decision_engine: enforce_real.then(|| "opa_wasm".to_string()),
        reason_code: reason,
        friendly_en: friendly.0,
        friendly_th: friendly.1,
        setup_actions: actions,
        evidence_ids: req.evidence_ids.clone(),
    }
}

fn no_method_item(
    req: &CoverageRequest,
    snapshot: &LocalCapabilitySnapshotV2,
    risk_id: OwaspRiskId,
    domain: ControlDomainV2,
) -> SecurityCoverageItem {
    let actions = setup_for_domain(snapshot, &domain);
    SecurityCoverageItem {
        risk_id,
        policy_id: req.policy_id.clone(),
        agent_id: req.agent_id.clone(),
        entity_id: req.entity_id.clone(),
        domain: domain.clone(),
        requested_level: req.requested_level.clone(),
        achievable_level: CoverageLevel::NotVisible,
        observe_capable: false,
        enforce_capable: false,
        enforced_for_real: false,
        chosen_control_method: None,
        decision_engine: None,
        reason_code: CapabilityReasonCode::ObserveOnlyNoLocalControlMethod,
        friendly_en: format!(
            "Pollek cannot currently see or control {domain:?} for this agent on this device."
        ),
        friendly_th: format!("Pollek ยังไม่สามารถมองเห็นหรือควบคุม {domain:?} ของ Agent นี้บนเครื่องนี้ได้"),
        setup_actions: actions,
        evidence_ids: req.evidence_ids.clone(),
    }
}

fn reason_for_method(
    method: &ControlMethodCapabilityV2,
    domain: &ControlDomainV2,
) -> CapabilityReasonCode {
    if method.maturity == MethodMaturity::Simulator
        || method.status == MethodReadiness::SimulatorOnly
    {
        return CapabilityReasonCode::SimulatorOnly;
    }
    match method.status {
        MethodReadiness::Available => CapabilityReasonCode::FullyProtected,
        MethodReadiness::NeedsPermission => {
            if matches!(
                domain,
                ControlDomainV2::NetworkEgress | ControlDomainV2::Dns
            ) {
                CapabilityReasonCode::NeedsOsNetworkExtension
            } else {
                CapabilityReasonCode::ObserveOnlyPermissionRequired
            }
        }
        MethodReadiness::NeedsInstall => {
            if matches!(domain, ControlDomainV2::BrowserAiSession) {
                CapabilityReasonCode::NeedsBrowserExtension
            } else if matches!(
                domain,
                ControlDomainV2::McpToolCall | ControlDomainV2::PromptContent
            ) {
                CapabilityReasonCode::NeedsMcpConfigChange
            } else {
                CapabilityReasonCode::NeedsOsNetworkExtension
            }
        }
        MethodReadiness::NeedsConfiguration => CapabilityReasonCode::NeedsMcpConfigChange,
        MethodReadiness::Degraded => CapabilityReasonCode::ObserveOnlyPermissionRequired,
        MethodReadiness::SimulatorOnly => CapabilityReasonCode::SimulatorOnly,
        MethodReadiness::Unsupported => CapabilityReasonCode::ObserveOnlyUnsupportedOs,
        MethodReadiness::Failed => CapabilityReasonCode::WarmCheckFailed,
    }
}

fn friendly_for(
    reason: &CapabilityReasonCode,
    domain: &ControlDomainV2,
    method: &ControlMethodCapabilityV2,
) -> (String, String) {
    match reason {
        CapabilityReasonCode::FullyProtected => (
            format!(
                "Pollek can enforce {domain:?} with {}.",
                method.display_name_en
            ),
            format!(
                "Pollek สามารถบังคับใช้ {domain:?} ด้วย {}",
                method.display_name_th
            ),
        ),
        CapabilityReasonCode::SimulatorOnly => (
            "This signal is simulated for testing. Real blocking is not enabled.".into(),
            "สัญญาณนี้เป็นการจำลองเพื่อทดสอบ ยังไม่ได้เปิดใช้การบล็อกจริง".into(),
        ),
        CapabilityReasonCode::NeedsOsNetworkExtension => (
            "Pollek can preview or observe this path, but real blocking needs device-level network control setup.".into(),
            "Pollek แสดงตัวอย่างหรือสังเกตเส้นทางนี้ได้ แต่การบล็อกจริงต้องตั้งค่าการควบคุมเครือข่ายระดับเครื่อง".into(),
        ),
        CapabilityReasonCode::NeedsMcpConfigChange => (
            "Pollek needs permission to bind this agent to an MCP wrapper or proxy before enforcing tool calls.".into(),
            "Pollek ต้องได้รับอนุญาตให้ผูก Agent นี้กับ MCP wrapper หรือ proxy ก่อนบังคับใช้ tool calls".into(),
        ),
        CapabilityReasonCode::NeedsBrowserExtension => (
            "Browser AI activity requires the Pollek browser extension before it can be controlled.".into(),
            "กิจกรรม AI บน Browser ต้องติดตั้ง Pollek browser extension ก่อนจึงจะควบคุมได้".into(),
        ),
        _ => (
            format!("Pollek cannot fully enforce {domain:?} with the current setup."),
            format!("Pollek ยังไม่สามารถบังคับใช้ {domain:?} ได้ครบด้วยการตั้งค่าปัจจุบัน"),
        ),
    }
}

fn find_setup_action(snapshot: &LocalCapabilitySnapshotV2, id: &str) -> Option<SetupAction> {
    snapshot
        .setup_actions
        .iter()
        .find(|action| action.action_id == id)
        .cloned()
}

fn setup_for_domain(
    snapshot: &LocalCapabilitySnapshotV2,
    domain: &ControlDomainV2,
) -> Vec<SetupAction> {
    let wanted = match domain {
        ControlDomainV2::NetworkEgress | ControlDomainV2::Dns => "network",
        ControlDomainV2::McpToolCall | ControlDomainV2::PromptContent => "mcp",
        ControlDomainV2::BrowserAiSession => "browser",
        ControlDomainV2::FileAccess => "file",
        _ => "",
    };
    snapshot
        .setup_actions
        .iter()
        .filter(|action| action.action_id.contains(wanted))
        .cloned()
        .collect()
}

fn collect_setup_actions(
    snapshot: &LocalCapabilitySnapshotV2,
    items: &[SecurityCoverageItem],
) -> Vec<SetupAction> {
    let mut out = Vec::new();
    for item in items {
        for action in &item.setup_actions {
            if !out
                .iter()
                .any(|a: &SetupAction| a.action_id == action.action_id)
            {
                out.push(action.clone());
            }
        }
    }
    if out.is_empty() {
        out.extend(
            snapshot
                .setup_actions
                .iter()
                .filter(|a| !a.safe_to_skip)
                .cloned(),
        );
    }
    out
}

fn summarize(
    req: &CoverageRequest,
    title_en: &str,
    title_th: &str,
    items: &[SecurityCoverageItem],
) -> PolicyCoverageSummary {
    let any_full = items
        .iter()
        .any(|i| i.achievable_level == CoverageLevel::EnforceFull);
    let any_observe = items
        .iter()
        .any(|i| i.achievable_level == CoverageLevel::ObserveOnly);
    let any_setup = items.iter().any(|i| !i.setup_actions.is_empty());
    let all_full = !items.is_empty()
        && items
            .iter()
            .all(|i| i.achievable_level == CoverageLevel::EnforceFull);

    let (verdict, level, friendly_en, friendly_th) = if all_full {
        (
            CoverageVerdict::Full,
            CoverageLevel::EnforceFull,
            format!("{title_en} can be enforced on this device."),
            format!("{title_th} สามารถบังคับใช้บนเครื่องนี้ได้"),
        )
    } else if any_full && any_observe {
        (
            CoverageVerdict::Partial,
            CoverageLevel::EnforcePartial,
            format!("{title_en} is partially enforceable; some activity will remain observe-only until setup is complete."),
            format!("{title_th} บังคับใช้ได้บางส่วน และบางกิจกรรมจะเป็นการสังเกตเท่านั้นจนกว่าจะตั้งค่าเพิ่ม"),
        )
    } else if any_setup {
        (
            CoverageVerdict::NeedsSetup,
            CoverageLevel::ObserveOnly,
            format!("{title_en} needs setup before real enforcement is available."),
            format!("{title_th} ต้องตั้งค่าเพิ่มเติมก่อนจะบังคับใช้จริงได้"),
        )
    } else {
        (
            CoverageVerdict::ObserveOnly,
            CoverageLevel::ObserveOnly,
            format!("{title_en} can be monitored, but not enforced with the current local capability snapshot."),
            format!("{title_th} ติดตามได้ แต่ยังบังคับใช้ไม่ได้ด้วย capability ปัจจุบันของเครื่องนี้"),
        )
    };

    PolicyCoverageSummary {
        requested_level: req.requested_level.clone(),
        overall_achievable_level: level,
        verdict,
        title_en: title_en.to_string(),
        title_th: title_th.to_string(),
        friendly_en,
        friendly_th,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dek_domain_schema::capability_snapshot_v2::{
        ContractCompatibilityStatus, InstallState, MethodMaturity, ObservationSourceCapability,
        OsInfoV2, WarmCheckStatus,
    };

    fn setup_action(id: &str) -> SetupAction {
        SetupAction {
            action_id: id.into(),
            title_en: "Install network control".into(),
            title_th: "ติดตั้งตัวควบคุมเครือข่าย".into(),
            detail_en: "Required for real blocking.".into(),
            detail_th: "จำเป็นสำหรับการบล็อกจริง".into(),
            requires_admin: true,
            requires_restart: false,
            estimated_minutes: 3,
            docs_path: None,
            safe_to_skip: true,
        }
    }

    fn snapshot(method: ControlMethodCapabilityV2) -> LocalCapabilitySnapshotV2 {
        LocalCapabilitySnapshotV2 {
            schema_version: "local-capability-snapshot.v2".into(),
            tenant_id: "local".into(),
            device_id: "dev_test".into(),
            os: OsInfoV2 {
                family: "windows".into(),
                version: "test".into(),
                arch: "x86_64".into(),
                is_server: false,
                elevated: false,
            },
            mode: RuntimeMode::DesktopSimple,
            generated_at: chrono::Utc::now(),
            control_methods: vec![method],
            observation_sources: vec![ObservationSourceCapability {
                source_id: "process_metadata".into(),
                display_name_en: "Process metadata".into(),
                display_name_th: "ข้อมูล process".into(),
                status: MethodReadiness::Available,
                domains: vec![ControlDomainV2::ProcessLaunch],
                privacy_note_en: "Metadata only.".into(),
                privacy_note_th: "เฉพาะ metadata".into(),
                setup_action_ids: vec![],
            }],
            setup_actions: vec![setup_action("install_device_network_control")],
            contract: ContractCompatibilityStatus {
                local_contract_version: "2026.06.26".into(),
                compatible_cloud_contracts: vec![">=2026.06.01 <2026.09.00".into()],
                status: "compatible".into(),
                reason_code: None,
            },
        }
    }

    #[test]
    fn simulator_is_not_real_enforcement() {
        let snap = snapshot(ControlMethodCapabilityV2 {
            method_id: "egress_simulator".into(),
            display_name_en: "Egress simulator".into(),
            display_name_th: "ตัวจำลอง egress".into(),
            domains: vec![ControlDomainV2::NetworkEgress],
            max_level: ControlLevelV2::Observe,
            status: MethodReadiness::SimulatorOnly,
            maturity: MethodMaturity::Simulator,
            install_state: InstallState::BuiltIn,
            warm_check: Some(WarmCheckStatus::NotApplicable),
            setup_action_ids: vec![],
            limitations_en: vec![],
            limitations_th: vec![],
        });
        let report = assess_policy_coverage(
            CoverageRequest {
                tenant_id: "local".into(),
                device_id: "dev_test".into(),
                agent_id: Some("agent".into()),
                entity_id: None,
                policy_id: "network.shadow_ai_external_llm_block".into(),
                requested_level: ControlLevelV2::Enforce,
                mode: RuntimeMode::DesktopSimple,
                local_cloud_profile: "local_only".into(),
                evidence_ids: vec![],
            },
            &snap,
        );
        assert!(report.items.iter().any(|item| {
            item.reason_code == CapabilityReasonCode::SimulatorOnly && !item.enforced_for_real
        }));
    }
}
