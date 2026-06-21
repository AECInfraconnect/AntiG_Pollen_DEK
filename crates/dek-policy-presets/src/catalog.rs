// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::model::*;

pub fn builtin_presets() -> Vec<PolicyPreset> {
    vec![
        shadow_ai_preset(),
        require_approval_preset(),
        cost_budget_preset(),
        mcp_allowlist_preset(),
    ]
}

pub fn get_builtin_preset(id: &str) -> Option<PolicyPreset> {
    builtin_presets().into_iter().find(|p| p.preset_id == id)
}

fn shadow_ai_preset() -> PolicyPreset {
    PolicyPreset {
        preset_id: "rego.shadow_ai_block_external_llm".into(),
        version: "1.0.0".into(),
        display_name: "Block Shadow AI External LLM".into(),
        description: "Block unregistered/shadow AI agents from reaching public LLM providers."
            .into(),
        category: PresetCategory::ShadowAi,
        language: PresetLanguage::Rego,
        recommended_pep_types: vec!["linux_ebpf".into(), "http_gateway".into()],
        supported_control_levels: vec![
            ControlLevel::ObserveOnly,
            ControlLevel::Warn,
            ControlLevel::Enforce,
            ControlLevel::StrictDeny,
        ],
        default_control_level: ControlLevel::ObserveOnly,
        risk_tags: vec!["Shadow AI".into(), "Data Exfiltration".into()],
        owasp_tags: vec!["LLM06".into()],
        parameters: vec![],
        template: PresetTemplate {
            source: r#"
package pollen.presets.shadow_ai_block_external_llm
import future.keywords.if
import future.keywords.in

default allow := true
default reason := "allowed"

llm_domains := {
  "api.openai.com",
  "api.anthropic.com",
  "generativelanguage.googleapis.com",
  "api.mistral.ai",
  "api.groq.com"
}

allow := false if {
  input.agent.registered == false
  input.network.fqdn in llm_domains
  input.control_level in {"enforce", "strict_deny"}
}

reason := "shadow_ai_external_llm_blocked" if {
  not allow
}

warn := true if {
  input.agent.registered == false
  input.network.fqdn in llm_domains
  input.control_level in {"observe_only", "warn"}
}
"#
            .into(),
            entrypoint: Some("pollen/presets/shadow_ai_block_external_llm/allow".into()),
        },
        test_cases: vec![],
    }
}

fn require_approval_preset() -> PolicyPreset {
    PolicyPreset {
        preset_id: "cedar.require_approval_for_high_risk_tool".into(),
        version: "1.0.0".into(),
        display_name: "Require Approval for High-Risk Tools".into(),
        description: "Pause action until user/admin approves the tool execution.".into(),
        category: PresetCategory::ApprovalWorkflow,
        language: PresetLanguage::Cedar,
        recommended_pep_types: vec!["mcp_proxy".into(), "stdio_wrapper".into()],
        supported_control_levels: vec![
            ControlLevel::RequireApproval,
            ControlLevel::Enforce,
            ControlLevel::StrictDeny,
        ],
        default_control_level: ControlLevel::RequireApproval,
        risk_tags: vec!["Excessive Agency".into()],
        owasp_tags: vec!["LLM08".into()],
        parameters: vec![],
        template: PresetTemplate {
            source: r#"
@preset("cedar.require_approval_for_high_risk_tool")
forbid (
  principal,
  action == Pollen::Action::"tool.invoke",
  resource
)
when {
  resource.risk_level in ["high", "critical"] &&
  context.control_level in ["require_approval", "enforce", "strict_deny"] &&
  context.approval_ticket == ""
};
"#
            .into(),
            entrypoint: None,
        },
        test_cases: vec![],
    }
}

fn cost_budget_preset() -> PolicyPreset {
    PolicyPreset {
        preset_id: "cedar.cost_budget_enforcement".into(),
        version: "1.0.0".into(),
        display_name: "Cost Budget Enforcement".into(),
        description: "Deny execution if tool usage exceeds daily token budget.".into(),
        category: PresetCategory::CostBudget,
        language: PresetLanguage::Cedar,
        recommended_pep_types: vec!["mcp_proxy".into()],
        supported_control_levels: vec![
            ControlLevel::ObserveOnly,
            ControlLevel::Warn,
            ControlLevel::Enforce,
            ControlLevel::StrictDeny,
        ],
        default_control_level: ControlLevel::ObserveOnly,
        risk_tags: vec!["Financial Risk".into()],
        owasp_tags: vec!["LLM09".into()],
        parameters: vec![PresetParameter {
            key: "daily_limit_usd".into(),
            label: "Daily Limit (USD)".into(),
            description: "Maximum daily USD spend per user".into(),
            param_type: "number".into(),
            required: true,
            default_value: serde_json::json!(10),
            allowed_values: None,
        }],
        template: PresetTemplate {
            source: r#"
@preset("cedar.cost_budget_enforcement")
forbid (
  principal,
  action == Pollen::Action::"tool.invoke",
  resource
)
when {
  context.control_level in ["enforce", "strict_deny"] &&
  context.current_spend_usd > {{daily_limit_usd}}
};
"#.into(),
            entrypoint: None,
        },
        test_cases: vec![],
    }
}

fn mcp_allowlist_preset() -> PolicyPreset {
    PolicyPreset {
        preset_id: "openfga.mcp_tool_allowlist".into(),
        version: "1.0.0".into(),
        display_name: "MCP Tool Allowlist".into(),
        description: "Allow execution only if user is explicitly granted access to the tool in OpenFGA.".into(),
        category: PresetCategory::ToolPermission,
        language: PresetLanguage::OpenFga,
        recommended_pep_types: vec!["mcp_proxy".into()],
        supported_control_levels: vec![
            ControlLevel::ObserveOnly,
            ControlLevel::Warn,
            ControlLevel::Enforce,
            ControlLevel::StrictDeny,
        ],
        default_control_level: ControlLevel::ObserveOnly,
        risk_tags: vec!["Unauthorized Access".into()],
        owasp_tags: vec!["LLM01".into(), "LLM02".into()],
        parameters: vec![],
        template: PresetTemplate {
            source: r#"
# OpenFGA implicit model. 
# The remote PEP will perform a check: 
# tuple_key: { user: input.principal, relation: "can_invoke", object: input.resource }
"#.into(),
            entrypoint: None,
        },
        test_cases: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_cedar_presets_compile() {
        let presets = builtin_presets();
        for preset in presets {
            if matches!(preset.language, PresetLanguage::Cedar) {
                // If it has parameters, we might need to replace them with dummy values
                let mut source = preset.template.source.clone();
                for param in &preset.parameters {
                    let token = format!("{{{{{}}}}}", param.key);
                    source = source.replace(&token, "0"); // Dummy numeric/string
                }
                let res = cedar_policy::PolicySet::from_str(&source);
                assert!(res.is_ok(), "Failed to compile Cedar preset {}: {:?}", preset.preset_id, res.err());
            }
        }
    }
}
