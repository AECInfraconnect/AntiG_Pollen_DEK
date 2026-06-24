use crate::model::*;
use crate::process_scan::ProcessEvidence;
use dek_fingerprint_defs::model::AgentSignatureV2;
use regex::Regex;

/// Quick filter for process scan events before sending to aggregator.
/// Returns a basic confidence score. If > config.min_fingerprint_confidence, the event is kept.
pub fn fingerprint_process(evidence: &ProcessEvidence, signatures: &[AgentSignatureV2]) -> f64 {
    let name_lower = evidence.process_name.to_ascii_lowercase();

    // 1. Check if process_name matches any known signature's process_names
    for sig in signatures {
        if sig
            .process_names
            .iter()
            .any(|n| n.to_ascii_lowercase() == name_lower)
        {
            return 0.6;
        }
    }

    let cmd_joined = evidence.cmd_template.join(" ");

    // 2. Check if cmd_template matches any cmd_patterns
    for sig in signatures {
        for pat in &sig.cmd_patterns {
            if let Ok(re) = Regex::new(pat) {
                if re.is_match(&cmd_joined) {
                    return 0.8;
                }
            }
        }
    }

    // 3. Check if exe_path matches any exe_path_patterns
    if let Some(exe_path) = &evidence.exe_path_redacted {
        for sig in signatures {
            for pat in &sig.exe_path_patterns {
                if let Ok(pattern) = glob::Pattern::new(pat) {
                    if pattern.matches(exe_path) {
                        return 0.8;
                    }
                }
            }
        }
    }

    // 4. Check if cli_binaries match the process name
    for sig in signatures {
        if sig
            .cli_binaries
            .iter()
            .any(|c| c.to_ascii_lowercase() == name_lower)
        {
            return 0.7;
        }
    }

    // Fallback heuristic for common engines (Node/Python) that might be running an unknown script.
    // They are passed to the aggregator with a low score, where identity::resolve will attempt deep matching.
    if name_lower.contains("python") || name_lower.contains("node") || name_lower.contains("n8n") {
        return 0.1;
    }

    0.0
}

pub fn resolve_by_install_path(
    exe_path: &str,
    defs: &dek_fingerprint_defs::model::FingerprintDefinition,
) -> Option<crate::identity::AgentMatch> {
    let p = exe_path.replace('\\', "/").to_lowercase();
    for app in &defs.installed_app_signatures {
        for marker in &app.markers {
            for path in &marker.paths {
                let needle = path
                    .replace("**", "")
                    .replace("*", "")
                    .replace("//", "/")
                    .replace('\\', "/")
                    .to_lowercase();
                if !needle.is_empty() && p.contains(&needle) {
                    return Some(crate::identity::AgentMatch {
                        signature_id: app.id.clone(),
                        display_name: app.name.clone(),
                        vendor: Some(app.vendor.clone()),
                        product: Some(app.product.clone()),
                        agent_type: app.agent_type.clone(),
                        confidence: 0.95,
                        matched_signals: vec![crate::identity::MatchedSignal {
                            kind: "install_path".into(),
                            detail: path.clone(),
                            weight: 0.95,
                        }],
                        capability_tags: app.capability_tags.clone(),
                    });
                }
            }
        }
        let exe_path_std = std::path::Path::new(exe_path);
        if let Some(pn) = exe_path_std.file_name().and_then(|n| n.to_str()) {
            for marker in &app.markers {
                if marker
                    .process_names
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(pn))
                {
                    return Some(crate::identity::AgentMatch {
                        signature_id: app.id.clone(),
                        display_name: app.name.clone(),
                        vendor: Some(app.vendor.clone()),
                        product: Some(app.product.clone()),
                        agent_type: app.agent_type.clone(),
                        confidence: 0.90,
                        matched_signals: vec![crate::identity::MatchedSignal {
                            kind: "process_name".into(),
                            detail: pn.to_string(),
                            weight: 0.90,
                        }],
                        capability_tags: app.capability_tags.clone(),
                    });
                }
            }
        }
    }
    None
}
