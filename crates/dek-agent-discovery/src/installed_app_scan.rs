// SPDX-License-Identifier: Apache-2.0
use crate::model::{DiscoveryEvidenceV2, EvidenceSource, PrivacyClass};
use dek_fingerprint_defs::model::InstalledAppSignatureDef;
use serde_json::json;

pub fn scan_installed_apps(
    signatures: &[InstalledAppSignatureDef],
) -> anyhow::Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();
    let current_os = std::env::consts::OS;

    for sig in signatures {
        for marker in &sig.markers {
            // Match OS or wildcard
            if let Some(os) = &marker.os {
                if os != current_os && os != "all" {
                    continue;
                }
            }

            let mut found = false;
            let mut matched_path = String::new();

            // Check if path exists
            let expanded = expand_path(&marker.path);
            if std::path::Path::new(&expanded).exists() {
                found = true;
                matched_path = expanded;
            }

            if found {
                evidence.push(DiscoveryEvidenceV2 {
                    evidence_id: format!("ev_inst_{}", uuid::Uuid::new_v4()),
                    source: EvidenceSource::InstalledAppScan,
                    confidence: 1.0,
                    observed_at: chrono::Utc::now().to_rfc3339(),
                    privacy_class: PrivacyClass::PublicMetadata,
                    redacted: false,
                    merge_key: Some(crate::identity_key::identity_key(
                        None,
                        Some(&sig.vendor),
                        Some(&sig.product),
                        None,
                        &sig.name,
                    )),
                    source_path_hash: Some(crate::redaction::sha256_string(&matched_path)),
                    source_path_redacted: Some(crate::redaction::redact_path_for_ui(&matched_path)),
                    data: json!({
                        "name": sig.name,
                        "vendor": sig.vendor,
                        "product": sig.product,
                        "agent_type": sig.agent_type,
                        "capability_tags": sig.capability_tags,
                        "matched_path": crate::redaction::redact_path_for_ui(&matched_path)
                    }),
                });
                break; // Only one evidence per signature is enough
            }
        }
    }

    Ok(evidence)
}

fn expand_path(p: &str) -> String {
    if let Some(stripped) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}/{}", home.display(), stripped);
        }
    } else if let Some(stripped) = p.strip_prefix("%LOCALAPPDATA%\\") {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            return format!("{}\\{}", appdata, stripped);
        }
    } else if let Some(stripped) = p.strip_prefix("%APPDATA%\\") {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return format!("{}\\{}", appdata, stripped);
        }
    }
    p.to_string()
}
