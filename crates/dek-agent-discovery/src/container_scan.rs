use crate::model::*;
use anyhow::Result;
use std::process::Command;

pub fn scan_containers() -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();

    // Check if docker is available
    if let Ok(output) = Command::new("docker").args(["ps", "--format", "{{json .}}"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(container) = serde_json::from_str::<serde_json::Value>(line) {
                    let image = container.get("Image").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                    let names = container.get("Names").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                    let id = container.get("ID").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    
                    // Look for AI agent / model server containers
                    if image.contains("ollama") || image.contains("vllm") || image.contains("llama") || image.contains("mcp") || names.contains("mcp") {
                        evidence.push(DiscoveryEvidenceV2 {
                            evidence_id: uuid::Uuid::new_v4().to_string(),
                            source: EvidenceSource::Container,
                            confidence: 0.9,
                            observed_at: chrono::Utc::now().to_rfc3339(),
                            privacy_class: PrivacyClass::PublicMetadata,
                            redacted: false,
                            data: serde_json::json!({
                                "container_id": id,
                                "image": image,
                                "names": names,
                                "engine": "docker"
                            }),
                            merge_key: Some(id.clone()),
                            source_path_hash: Some(crate::redaction::sha256_string(&format!("docker_{}", id))),
                            source_path_redacted: Some(format!("docker_{}", id)),
                        });
                    }
                }
            }
        }
    }

    Ok(evidence)
}
