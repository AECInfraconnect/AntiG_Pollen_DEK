use crate::model::*;
use anyhow::Result;
use std::path::PathBuf;

pub fn scan_browsers() -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();

    let paths = get_browser_extension_paths();
    for base_path in paths {
        if !base_path.exists() {
            continue;
        }

        // Read extension directories
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let ext_id = entry.file_name().to_string_lossy().to_string();
                
                // Usually extensions have version folders inside
                if let Ok(versions) = std::fs::read_dir(entry.path()) {
                    for version in versions.flatten() {
                        let manifest_path = version.path().join("manifest.json");
                        if manifest_path.exists() {
                            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_lowercase();
                                    let desc = json.get("description").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                                    
                                    // Check for AI keywords
                                    if name.contains("ai") || name.contains("gpt") || name.contains("llm") || name.contains("copilot") || desc.contains("llm") {
                                        evidence.push(DiscoveryEvidenceV2 {
                                            evidence_id: uuid::Uuid::new_v4().to_string(),
                                            source: EvidenceSource::BrowserExtension,
                                            confidence: 0.8,
                                            observed_at: chrono::Utc::now().to_rfc3339(),
                                            privacy_class: PrivacyClass::InternalMetadata,
                                            redacted: false,
                                            data: serde_json::json!({
                                                "browser_profile": base_path.to_string_lossy(),
                                                "extension_id": ext_id,
                                                "name": name,
                                            }),
                                            merge_key: Some(ext_id.clone()),
                                            source_path_hash: Some(crate::redaction::sha256_string(&manifest_path.to_string_lossy())),
                                            source_path_redacted: Some(crate::redaction::redact_path_for_ui(&manifest_path.to_string_lossy())),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(evidence)
}

fn get_browser_extension_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    #[cfg(target_os = "windows")]
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        paths.push(PathBuf::from(&localappdata).join("Google").join("Chrome").join("User Data").join("Default").join("Extensions"));
        paths.push(PathBuf::from(&localappdata).join("Microsoft").join("Edge").join("User Data").join("Default").join("Extensions"));
    }

    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(&home).join("Library").join("Application Support").join("Google").join("Chrome").join("Default").join("Extensions"));
    }

    #[cfg(target_os = "linux")]
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(&home).join(".config").join("google-chrome").join("Default").join("Extensions"));
    }

    paths
}
