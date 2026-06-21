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
            fail_on_warnings: Some(false),
        },
    };

    Ok(draft)
}
