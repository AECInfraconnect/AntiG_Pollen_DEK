// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::model::{ControlMode, DeployPresetRequest, PolicyPresetV2, RenderedArtifact};
use dek_control_plane_api::policy::{
    PolicyCompileOptions, PolicyDraft, PolicySource, PolicyTargets, PolicyType,
};
use dek_control_plane_api::registry::ObjectMeta;
use dek_guard_pipeline::config::{GuardConfig, GuardMode};

pub fn render(
    preset: &PolicyPresetV2,
    req: &DeployPresetRequest,
) -> anyhow::Result<Vec<RenderedArtifact>> {
    // Basic validation of unknown params
    for key in req.params.keys() {
        if !preset.parameters.iter().any(|p| p.key == *key) {
            return Err(anyhow::anyhow!("Unknown parameter: {}", key));
        }
    }

    // Basic validation of required
    for param in &preset.parameters {
        if !req.params.contains_key(&param.key) && param.required {
            return Err(anyhow::anyhow!("Missing required parameter: {}", param.key));
        }
    }

    let mut artifacts = Vec::new();

    match preset.id.as_str() {
        "pii.redact_before_external_llm" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.pii.redact_before_external_llm",
                "default allow := true\n# GuardPipeline enforces PII redaction in the response filter data plane".into(),
            ));
            artifacts.push(render_guard_pipeline_config(preset, req)?);
        }
        "fs.folder_allowlist" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.fs.folder_scope",
                "default allow := false\n# TODO: Folder allowlist logic".into(),
            ));
            artifacts.push(RenderedArtifact::pep_config(
                "{\n  \"action\": \"block_fs\"\n}".into(),
            ));
        }
        "budget.daily_token_cap" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.budget.daily_token_cap",
                "default allow := true\n# TODO: Budget logic".into(),
            ));
        }
        "budget.monthly_cost_cap" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.budget.monthly_cost_cap",
                "default allow := true\n# TODO: Monthly budget logic".into(),
            ));
        }
        "content.prompt_injection_guard" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.content.prompt_injection_guard",
                "default allow := true\n# GuardPipeline evaluates prompt injection before PDP routing".into(),
            ));
            artifacts.push(render_guard_pipeline_config(preset, req)?);
        }
        "content.system_prompt_leak_guard" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.content.system_prompt_leak_guard",
                "default allow := true\n# GuardPipeline evaluates model output through /v1/filter/response".into(),
            ));
            artifacts.push(render_guard_pipeline_config(preset, req)?);
        }
        "secrets.block_api_key_exposure" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.secrets.block_api_key_exposure",
                "default allow := true\n# GuardPipeline redacts or blocks secret echo before returning output".into(),
            ));
            artifacts.push(render_guard_pipeline_config(preset, req)?);
        }
        "fs.secrets_file_guard" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.fs.secrets_file_guard",
                "default allow := true\n# TODO: FS Secrets guard logic".into(),
            ));
        }
        "personal.email_send_approval" => {
            artifacts.push(RenderedArtifact::cedar(
                "personal.email_send_approval",
                "forbid (principal, action, resource); // TODO: Email approval logic".into(),
            ));
        }
        "personal.drive_external_share_guard" => {
            artifacts.push(RenderedArtifact::cedar(
                "personal.drive_external_share_guard",
                "forbid (principal, action, resource); // TODO: Drive approval logic".into(),
            ));
        }
        "network.shadow_ai_external_llm_block" => {
            artifacts.push(RenderedArtifact::rego(
                "pollek.presets.network.shadow_ai",
                "default allow := true\n# TODO: Network logic".into(),
            ));
        }
        "mcp.high_risk_tool_approval" => {
            artifacts.push(RenderedArtifact::cedar(
                "mcp.high_risk_tool_approval",
                "forbid (principal, action, resource); // TODO: High risk tool logic".into(),
            ));
        }
        "mcp.tool_allowlist" => {
            artifacts.push(RenderedArtifact::openfga(
                "mcp.tool_allowlist",
                "model\n  schema 1.1\n// TODO: OpenFGA logic".into(),
            ));
        }
        "personal.drive_folder_scope" => {
            artifacts.push(RenderedArtifact::openfga(
                "personal.drive_folder_scope",
                "model\n  schema 1.1\ntype user\ntype folder\n  relations\n    define viewer: [user]\ntype document\n  relations\n    define parent: [folder]\n    define viewer: viewer from parent\n".into(),
            ));
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported preset: {}", preset.id));
        }
    }

    Ok(artifacts)
}

fn guard_mode_for_control_mode(control_mode: &ControlMode) -> GuardMode {
    match control_mode {
        ControlMode::Observe => GuardMode::Observe,
        ControlMode::Warn | ControlMode::Approval => GuardMode::Warn,
        ControlMode::Enforce => GuardMode::Enforce,
        ControlMode::StrictDeny => GuardMode::StrictDeny,
    }
}

fn guard_config_for_preset(preset_id: &str, control_mode: &ControlMode) -> Option<GuardConfig> {
    let mut cfg = GuardConfig {
        mode: guard_mode_for_control_mode(control_mode),
        ..GuardConfig::default()
    };

    match preset_id {
        "content.prompt_injection_guard" => {
            cfg.request_guard_enabled = true;
            cfg.response_guard_enabled = false;
            Some(cfg)
        }
        "content.system_prompt_leak_guard" => {
            cfg.request_guard_enabled = false;
            cfg.response_guard_enabled = true;
            Some(cfg)
        }
        "pii.redact_before_external_llm" | "secrets.block_api_key_exposure" => {
            cfg.request_guard_enabled = true;
            cfg.response_guard_enabled = true;
            Some(cfg)
        }
        _ => None,
    }
}

fn render_guard_pipeline_config(
    preset: &PolicyPresetV2,
    req: &DeployPresetRequest,
) -> anyhow::Result<RenderedArtifact> {
    let Some(config) = guard_config_for_preset(&preset.id, &req.control_mode) else {
        return Err(anyhow::anyhow!(
            "Preset does not support GuardPipeline config: {}",
            preset.id
        ));
    };
    let value = serde_json::json!({
        "schema_version": "guard-pipeline-config.v1",
        "preset_id": &preset.id,
        "preset_version": &preset.version,
        "control_mode": &req.control_mode,
        "data_plane": "/v1/filter/response",
        "guard_pipeline": config
    });
    Ok(RenderedArtifact::pep_config(serde_json::to_string_pretty(
        &value,
    )?))
}

pub fn to_policy_draft(
    tenant_id: &str,
    preset: &PolicyPresetV2,
    req: &DeployPresetRequest,
) -> anyhow::Result<Option<PolicyDraft>> {
    let rendered = render(preset, req)?;

    // Find the first rego, cedar, or openfga artifact to turn into a draft
    let policy_artifact = rendered
        .into_iter()
        .find(|a| a.language == "rego" || a.language == "cedar" || a.language == "openfga");

    if let Some(artifact) = policy_artifact {
        let policy_type = match artifact.language.as_str() {
            "rego" => PolicyType::Rego,
            "cedar" => PolicyType::Cedar,
            "openfga" => PolicyType::OpenFga,
            _ => return Err(anyhow::anyhow!("Unknown language type")),
        };

        let targets = PolicyTargets {
            agent_ids: req.targets.agent_ids.clone(),
            tool_ids: req.targets.tool_ids.clone(),
            resource_ids: req.targets.resource_ids.clone(),
            entity_ids: vec![],
            route_ids: vec![],
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
                preset.id.replace('.', "_"),
                uuid::Uuid::new_v4().simple()
            ),
            name: preset.title.clone(),
            description: Some(preset.short_description.clone()),
            policy_type,
            targets,
            source: PolicySource::RawText {
                language: artifact.language,
                text: artifact.content,
            },
            compile_options: PolicyCompileOptions {
                optimization_level: None,
                fail_on_warnings: Some(true),
            },
        };
        Ok(Some(draft))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::get_builtin_preset;
    use crate::model::{ControlMode, PresetTargets};
    use serde::Deserialize;

    const GUARD_PRESET_CORPUS: &str = include_str!("../tests/corpus/guard_preset_config.jsonl");

    #[derive(Debug, Deserialize)]
    struct GuardPresetCorpusCase {
        id: String,
        preset_id: String,
        control_mode: String,
        expected_guard_mode: String,
        status: String,
    }

    fn control_mode_from_name(value: &str) -> anyhow::Result<ControlMode> {
        Ok(serde_json::from_value(serde_json::json!(value))?)
    }

    fn guard_config_json(artifacts: &[RenderedArtifact]) -> anyhow::Result<serde_json::Value> {
        for artifact in artifacts {
            if artifact.language == "json" && artifact.content.contains("guard_pipeline") {
                return Ok(serde_json::from_str(&artifact.content)?);
            }
        }
        Err(anyhow::anyhow!("missing guard pipeline config artifact"))
    }

    fn request(preset: &PolicyPresetV2, control_mode: ControlMode) -> DeployPresetRequest {
        let mut params: std::collections::BTreeMap<String, serde_json::Value> = Default::default();
        for param in &preset.parameters {
            if param.required {
                params.insert(param.key.clone(), param.default_value.clone());
            }
        }
        DeployPresetRequest {
            preset_id: preset.id.clone(),
            preset_version: None,
            control_mode,
            selected_pep_types: Vec::new(),
            targets: PresetTargets::default(),
            params,
            dry_run_first: true,
            pdp_route: None,
        }
    }

    #[test]
    fn guard_control_mode_mapping_matches_golden_corpus() -> anyhow::Result<()> {
        for line in GUARD_PRESET_CORPUS
            .lines()
            .filter(|line| !line.trim().is_empty())
        {
            let case: GuardPresetCorpusCase = serde_json::from_str(line)?;
            if case.status != "active" {
                continue;
            }
            let Some(preset) = get_builtin_preset(&case.preset_id) else {
                return Err(anyhow::anyhow!("unknown preset {}", case.preset_id));
            };
            let req = request(&preset, control_mode_from_name(&case.control_mode)?);
            let rendered = render(&preset, &req)?;
            let config = guard_config_json(&rendered)?;
            let mode = config
                .get("guard_pipeline")
                .and_then(|value| value.get("mode"))
                .and_then(|value| value.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing guard mode"))?;

            assert!(case.id.starts_with("rt-pr9-"));
            assert_eq!(mode, case.expected_guard_mode);
            assert_eq!(config["data_plane"], "/v1/filter/response");
        }
        Ok(())
    }

    #[test]
    fn approval_control_mode_maps_to_warn_guard_mode() {
        assert_eq!(
            guard_mode_for_control_mode(&ControlMode::Approval),
            GuardMode::Warn
        );
    }
}
