// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::model::{PolicyPreset, PresetApplyRequest};
use dek_control_plane_api::policy::{
    PolicyCompileOptions, PolicyDraft, PolicySource, PolicyTargets, PolicyType,
};
use dek_control_plane_api::registry::ObjectMeta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPreset {
    pub language: String,
    pub content: String,
    pub warnings: Vec<String>,
}

pub fn render(preset: &PolicyPreset, req: &PresetApplyRequest) -> anyhow::Result<RenderedPreset> {
    // Validate unknown params
    for key in req.params.keys() {
        if !preset.parameters.iter().any(|p| p.key == *key) {
            return Err(anyhow::anyhow!("Unknown parameter: {}", key));
        }
    }

    // Validate required and allowed values
    for param in &preset.parameters {
        if let Some(val) = req.params.get(&param.key) {
            // Check allowed values
            if let Some(allowed) = &param.allowed_values {
                if !allowed.contains(val) {
                    return Err(anyhow::anyhow!("Value {} for parameter {} is not in allowed values", val, param.key));
                }
            }
        } else if param.required {
            return Err(anyhow::anyhow!("Missing required parameter: {}", param.key));
        }
    }

    let mut source = preset.template.source.clone();

    for (key, value) in &req.params {
        let token = format!("{{{{{}}}}}", key);
        let replacement = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };
        source = source.replace(&token, &replacement);
    }

    let control_level_str = serde_json::to_string(&req.control_level)
        .unwrap_or_else(|_| "\"unknown\"".into())
        .replace("\"", "");

    source = source.replace("{{control_level}}", &control_level_str);

    let language_str = match preset.language {
        crate::model::PresetLanguage::Rego => "rego",
        crate::model::PresetLanguage::Cedar => "cedar",
        crate::model::PresetLanguage::OpenFga => "openfga",
    };

    Ok(RenderedPreset {
        language: language_str.to_string(),
        content: source,
        warnings: vec![],
    })
}

pub fn to_policy_draft(
    tenant_id: &str,
    preset: &PolicyPreset,
    req: &PresetApplyRequest,
) -> anyhow::Result<PolicyDraft> {
    let rendered = render(preset, req)?;

    let targets = PolicyTargets {
        agent_ids: req.targets.agent_ids.clone(),
        tool_ids: req.targets.tool_ids.clone(),
        resource_ids: req.targets.resource_ids.clone(),
        entity_ids: vec![],
        route_ids: vec![],
    };

    let policy_type = match preset.language {
        crate::model::PresetLanguage::Rego => PolicyType::Rego,
        crate::model::PresetLanguage::Cedar => PolicyType::Cedar,
        crate::model::PresetLanguage::OpenFga => PolicyType::OpenFga,
    };

    let draft = PolicyDraft {
        meta: ObjectMeta {
            schema_version: "v1".to_string(),
            tenant_id: tenant_id.to_string(),
            workspace_id: "default".to_string(),
            environment_id: "default".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
            created_by: "".to_string(),
            updated_by: "".to_string(),
            source: dek_control_plane_api::registry::RegistrationSource::Manual,
            status: dek_control_plane_api::registry::RegistryStatus::Draft,
            tags: vec![],
        },
        policy_id: format!(
            "pol_{}_{}",
            preset.preset_id.replace('.', "_"),
            uuid::Uuid::new_v4().simple()
        ),
        name: preset.display_name.clone(),
        description: Some(preset.description.clone()),
        policy_type,
        targets,
        source: PolicySource::RawText {
            language: rendered.language,
            text: rendered.content,
        },
        compile_options: PolicyCompileOptions {
            optimization_level: None,
            fail_on_warnings: Some(true),
        },
    };

    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ControlLevel, PresetCategory, PresetLanguage, PresetParameter, PresetTemplate};
    use std::collections::HashMap;

    fn mock_preset() -> PolicyPreset {
        PolicyPreset {
            preset_id: "test".into(),
            version: "1".into(),
            display_name: "Test".into(),
            description: "Test".into(),
            category: PresetCategory::ShadowAi,
            language: PresetLanguage::Rego,
            recommended_pep_types: vec![],
            supported_control_levels: vec![],
            default_control_level: ControlLevel::ObserveOnly,
            risk_tags: vec![],
            owasp_tags: vec![],
            parameters: vec![
                PresetParameter {
                    key: "req_param".into(),
                    label: "Req".into(),
                    description: "Req".into(),
                    param_type: "string".into(),
                    required: true,
                    default_value: serde_json::Value::Null,
                    allowed_values: None,
                },
                PresetParameter {
                    key: "enum_param".into(),
                    label: "Enum".into(),
                    description: "Enum".into(),
                    param_type: "string".into(),
                    required: false,
                    default_value: serde_json::Value::Null,
                    allowed_values: Some(vec![serde_json::json!("A"), serde_json::json!("B")]),
                },
            ],
            template: PresetTemplate {
                source: "hello {{req_param}}".into(),
                entrypoint: None,
            },
            test_cases: vec![],
        }
    }

    #[test]
    fn test_missing_required_param() {
        let preset = mock_preset();
        let req = PresetApplyRequest {
            targets: Default::default(),
            control_level: ControlLevel::ObserveOnly,
            params: HashMap::new(),
        };
        let res = render(&preset, &req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Missing required parameter: req_param");
    }

    #[test]
    fn test_unknown_param() {
        let preset = mock_preset();
        let mut params = HashMap::new();
        params.insert("req_param".into(), serde_json::json!("val"));
        params.insert("unknown".into(), serde_json::json!("val"));
        let req = PresetApplyRequest {
            targets: Default::default(),
            control_level: ControlLevel::ObserveOnly,
            params,
        };
        let res = render(&preset, &req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Unknown parameter: unknown");
    }

    #[test]
    fn test_invalid_enum_val() {
        let preset = mock_preset();
        let mut params = HashMap::new();
        params.insert("req_param".into(), serde_json::json!("val"));
        params.insert("enum_param".into(), serde_json::json!("C"));
        let req = PresetApplyRequest {
            targets: Default::default(),
            control_level: ControlLevel::ObserveOnly,
            params,
        };
        let res = render(&preset, &req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Value \"C\" for parameter enum_param is not in allowed values");
    }

    #[test]
    fn test_valid_render() {
        let preset = mock_preset();
        let mut params = HashMap::new();
        params.insert("req_param".into(), serde_json::json!("world"));
        params.insert("enum_param".into(), serde_json::json!("A"));
        let req = PresetApplyRequest {
            targets: Default::default(),
            control_level: ControlLevel::ObserveOnly,
            params,
        };
        let res = render(&preset, &req).unwrap();
        assert_eq!(res.content, "hello world");
    }
}
