use crate::model::InferredAgentType;
use crate::process_scan::ProcessEvidence;
use dek_fingerprint_defs::model::{AgentSignatureV2, InstalledAppSignatureDef};
use regex::Regex;

/// Quick filter for process scan events before sending to aggregator.
/// Returns a basic confidence score. If > config.min_fingerprint_confidence, the event is kept.
#[deprecated(note = "Use fingerprint_process_v2 instead which supports signature matching")]
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

pub struct ProcessFacts<'a> {
    pub process_name: &'a str,
    pub exe_path: &'a str,
    pub cmdline: &'a str,
}

pub struct ResolvedAgent {
    pub confidence: f64,
    pub display_name: Option<String>,
    pub vendor: Option<String>,
    pub matched_signature_id: Option<String>,
    pub inferred_type: InferredAgentType,
}

fn glob_match(pattern: &str, text: &str) -> bool {
    if let Ok(p) = glob::Pattern::new(pattern) {
        p.matches(text)
    } else {
        false
    }
}

fn strip_ext(name: &str) -> String {
    if let Some(pos) = name.rfind('.') {
        name[..pos].to_string()
    } else {
        name.to_string()
    }
}

fn basename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

fn map_type(t: &str) -> InferredAgentType {
    match t {
        "desktop_agent" => InferredAgentType::DesktopAgent,
        "ide_agent" => InferredAgentType::IdeAgent,
        "cli_agent" => InferredAgentType::CliAgent,
        "browser_agent" => InferredAgentType::BrowserAgent,
        "mcp_server" => InferredAgentType::McpServer,
        "mcp_client" => InferredAgentType::McpClient,
        _ => InferredAgentType::AutomationAgent,
    }
}

fn looks_ai_ish(name: &str, cmd: &str) -> bool {
    const HINTS: &[&str] = &[
        "ai",
        "agent",
        "llm",
        "gpt",
        "claude",
        "gemini",
        "copilot",
        "ollama",
        "mcp",
        "langchain",
        "crew",
        "auto",
    ];
    HINTS.iter().any(|h| name.contains(h) || cmd.contains(h))
}

pub fn fingerprint_process_v2(
    facts: &ProcessFacts,
    sigs: &[AgentSignatureV2],
    apps: &[InstalledAppSignatureDef],
) -> ResolvedAgent {
    let pname = facts.process_name.to_lowercase();
    let exe = facts.exe_path.replace('\\', "/").to_lowercase();
    let cmd = facts.cmdline.to_lowercase();

    let mut best = ResolvedAgent {
        confidence: 0.0,
        display_name: None,
        vendor: None,
        matched_signature_id: None,
        inferred_type: InferredAgentType::UnknownAiProcess,
    };

    let mut consider = |conf: f64, s: &AgentSignatureV2| {
        if conf > best.confidence {
            best = ResolvedAgent {
                confidence: conf,
                display_name: Some(s.display_name.clone()),
                vendor: s.vendor.clone(),
                matched_signature_id: Some(s.id.clone()),
                inferred_type: map_type(&s.agent_type),
            };
        }
    };

    for s in sigs {
        if !exe.is_empty()
            && s.exe_path_patterns
                .iter()
                .any(|p| glob_match(&p.to_lowercase(), &exe))
        {
            consider(0.95, s);
        }
        if s.process_names.iter().any(|n| {
            n.eq_ignore_ascii_case(&pname) || strip_ext(&n.to_lowercase()) == strip_ext(&pname)
        }) {
            consider(0.9, s);
        }
        if !cmd.is_empty()
            && s.cmd_patterns
                .iter()
                .any(|p| glob_match(&p.to_lowercase(), &cmd))
        {
            consider(0.85, s);
        }
        if s.cli_binaries
            .iter()
            .any(|b| basename(&exe) == *b || cmd.split_whitespace().next() == Some(b.as_str()))
        {
            consider(0.8, s);
        }
    }

    for a in apps {
        if !exe.is_empty()
            && a.markers
                .iter()
                .any(|m| m.paths.iter().any(|p| glob_match(&p.to_lowercase(), &exe)))
            && 0.95 > best.confidence
        {
            best = ResolvedAgent {
                confidence: 0.95,
                display_name: Some(a.name.clone()),
                vendor: Some(a.vendor.clone()),
                matched_signature_id: Some(a.id.clone()),
                inferred_type: map_type(&a.agent_type),
            };
        }
    }

    if best.confidence == 0.0 && looks_ai_ish(&pname, &cmd) {
        best.confidence = 0.45;
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_desktop_passes_threshold() {
        let facts = ProcessFacts {
            process_name: "Codex.exe",
            exe_path: "C:/Program Files/WindowsApps/OpenAI.Codex_26.616.3767.0_x64__2p2nqsd0c76g0/app/Codex.exe",
            cmdline: "Codex.exe app-server",
        };
        let apps = vec![];
        let mut sigs = vec![];
        let sig: AgentSignatureV2 = serde_json::from_value(serde_json::json!({
            "id": "codex_desktop",
            "display_name": "OpenAI Codex (Desktop)",
            "vendor": "OpenAI",
            "product": "Codex",
            "agent_type": "desktop_agent",
            "process_names": ["Codex.exe"],
            "exe_path_patterns": ["**/*/OpenAI.Codex_*/**/Codex.exe"],
            "cmd_patterns": [],
            "cli_binaries": [],
            "capability_tags": [],
            "risk_weight": 0.8,
            "revision": 1,
            "detection_logic": "any_of",
            "meta": { "author": "system", "description": "", "added_in": "", "tags": [], "references": [] },
            "binary_hashes": [],
            "config_paths": {},
            "config_parsers": [],
            "ports": [],
            "control_strategies": []
        })).unwrap();
        sigs.push(sig);

        let r = fingerprint_process_v2(&facts, &sigs, &apps);
        assert!(r.confidence >= 0.9, "Codex must clear threshold");
        assert_eq!(r.display_name.as_deref(), Some("OpenAI Codex (Desktop)"));
    }

    #[test]
    fn unknown_ai_ish_process_emitted_as_unconfirmed() {
        let facts = ProcessFacts {
            process_name: "node.exe",
            exe_path: "C:/node/node.exe",
            cmdline: "node agent-server.js",
        };
        let r = fingerprint_process_v2(&facts, &[], &[]);
        assert_eq!(r.confidence, 0.45);
        assert_eq!(r.display_name, None);
    }
}
