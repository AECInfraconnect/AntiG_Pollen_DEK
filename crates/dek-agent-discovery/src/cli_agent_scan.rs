use crate::model::*;
use anyhow::Result;

pub fn scan_cli_agents() -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();

    // Stub logic: If we had a catalog, we'd check ~/.codex/config.toml, ~/.config/claude-code, etc.
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    if home.is_empty() {
        return Ok(evidence);
    }

    // Claude Code Stub
    let mut claude_code_config = std::path::PathBuf::from(&home);
    claude_code_config.push(".claude.json");
    if claude_code_config.exists() {
        evidence.push(DiscoveryEvidenceV2 {
            evidence_id: uuid::Uuid::new_v4().to_string(),
            source: EvidenceSource::CliAgent,
            confidence: 0.85,
            observed_at: chrono::Utc::now().to_rfc3339(),
            privacy_class: PrivacyClass::InternalMetadata,
            redacted: true,
            data: serde_json::json!({
                "cli_agent": "claude-code"
            }),
            merge_key: Some("cli:claude-code".into()),
            source_path_hash: Some(crate::redaction::sha256_string(
                &claude_code_config.to_string_lossy(),
            )),
            source_path_redacted: Some(".claude.json".into()),
        });
    }

    // Aider
    let mut aider_config = std::path::PathBuf::from(&home);
    aider_config.push(".aider.conf.yml");
    if aider_config.exists() {
        evidence.push(DiscoveryEvidenceV2 {
            evidence_id: uuid::Uuid::new_v4().to_string(),
            source: EvidenceSource::CliAgent,
            confidence: 0.85,
            observed_at: chrono::Utc::now().to_rfc3339(),
            privacy_class: PrivacyClass::InternalMetadata,
            redacted: true,
            data: serde_json::json!({
                "cli_agent": "aider"
            }),
            merge_key: Some("cli:aider".into()),
            source_path_hash: Some(crate::redaction::sha256_string(
                &aider_config.to_string_lossy(),
            )),
            source_path_redacted: Some(".aider.conf.yml".into()),
        });
    }

    // Goose
    let mut goose_config = std::path::PathBuf::from(&home);
    goose_config.push(".config/goose/config.toml");
    if goose_config.exists() {
        evidence.push(DiscoveryEvidenceV2 {
            evidence_id: uuid::Uuid::new_v4().to_string(),
            source: EvidenceSource::CliAgent,
            confidence: 0.85,
            observed_at: chrono::Utc::now().to_rfc3339(),
            privacy_class: PrivacyClass::InternalMetadata,
            redacted: true,
            data: serde_json::json!({
                "cli_agent": "goose"
            }),
            merge_key: Some("cli:goose".into()),
            source_path_hash: Some(crate::redaction::sha256_string(
                &goose_config.to_string_lossy(),
            )),
            source_path_redacted: Some(".config/goose/config.toml".into()),
        });
    }

    // Open Interpreter
    let mut oi_config = std::path::PathBuf::from(&home);
    oi_config.push(".config/open-interpreter/config.yaml");
    if oi_config.exists() {
        evidence.push(DiscoveryEvidenceV2 {
            evidence_id: uuid::Uuid::new_v4().to_string(),
            source: EvidenceSource::CliAgent,
            confidence: 0.85,
            observed_at: chrono::Utc::now().to_rfc3339(),
            privacy_class: PrivacyClass::InternalMetadata,
            redacted: true,
            data: serde_json::json!({
                "cli_agent": "open-interpreter"
            }),
            merge_key: Some("cli:open-interpreter".into()),
            source_path_hash: Some(crate::redaction::sha256_string(
                &oi_config.to_string_lossy(),
            )),
            source_path_redacted: Some(".config/open-interpreter/config.yaml".into()),
        });
    }

    // Cline / Roo Code (VSCode extensions typically store global state, but CLI variants exist)
    let mut cline_config = std::path::PathBuf::from(&home);
    cline_config.push(".cline");
    if cline_config.exists() {
        evidence.push(DiscoveryEvidenceV2 {
            evidence_id: uuid::Uuid::new_v4().to_string(),
            source: EvidenceSource::CliAgent,
            confidence: 0.85,
            observed_at: chrono::Utc::now().to_rfc3339(),
            privacy_class: PrivacyClass::InternalMetadata,
            redacted: true,
            data: serde_json::json!({
                "cli_agent": "cline"
            }),
            merge_key: Some("cli:cline".into()),
            source_path_hash: Some(crate::redaction::sha256_string(
                &cline_config.to_string_lossy(),
            )),
            source_path_redacted: Some(".cline".into()),
        });
    }

    Ok(evidence)
}
