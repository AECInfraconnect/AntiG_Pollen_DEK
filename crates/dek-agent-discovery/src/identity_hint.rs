// SPDX-License-Identifier: Apache-2.0

use crate::model::*;

/// Identity hint extracted from discovery evidence before full signature resolution
#[derive(Debug, Clone, Default)]
pub struct IdentityHint {
    pub name: Option<String>,
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub agent_type: Option<InferredAgentType>,
    pub capability_tags: Vec<String>,
    pub confidence: f64,
}

pub fn extract_identity_hint(ev: &DiscoveryEvidenceV2) -> Option<IdentityHint> {
    let d = &ev.data;
    let s = |k: &str| d.get(k).and_then(|v| v.as_str()).map(String::from);

    match ev.source {
        EvidenceSource::BrowserSession
        | EvidenceSource::BrowserWindow
        | EvidenceSource::BrowserHistory => Some(IdentityHint {
            name: s("name"),
            vendor: s("vendor"),
            agent_type: Some(InferredAgentType::WebAIApp),
            capability_tags: capability_tags_from_data(d, &["web.chat", "net.egress.llm"]),
            confidence: ev.confidence,
            ..Default::default()
        }),
        EvidenceSource::NetworkSni | EvidenceSource::NetworkEgress => Some(IdentityHint {
            name: s("name").or_else(|| s("sni_host")),
            vendor: s("vendor"),
            agent_type: Some(InferredAgentType::WebAIApp),
            capability_tags: capability_tags_from_data(d, &["net.egress.llm"]),
            confidence: ev.confidence,
            ..Default::default()
        }),
        EvidenceSource::CliAgent => Some(IdentityHint {
            name: s("name")
                .or_else(|| s("cli_name"))
                .or_else(|| s("cli_agent")),
            vendor: s("vendor"),
            product: s("product"),
            agent_type: Some(InferredAgentType::CliAgent),
            capability_tags: capability_tags_from_data(d, &["code.agentic", "tool.use"]),
            confidence: ev.confidence,
        }),
        EvidenceSource::Container => Some(IdentityHint {
            name: s("name").or_else(|| s("image")),
            vendor: s("vendor"),
            agent_type: Some(InferredAgentType::AutomationAgent),
            confidence: ev.confidence,
            ..Default::default()
        }),
        EvidenceSource::PythonFramework => Some(IdentityHint {
            name: s("framework").or_else(|| s("name")),
            vendor: s("vendor"),
            agent_type: Some(InferredAgentType::AutomationAgent),
            capability_tags: vec!["automation".into(), "tool.use".into()],
            confidence: ev.confidence,
            ..Default::default()
        }),
        EvidenceSource::IdeExtension => Some(IdentityHint {
            name: s("name"),
            vendor: s("vendor"),
            product: s("product"),
            agent_type: Some(InferredAgentType::IdeExtension),
            capability_tags: capability_tags_from_data(d, &["ide.extension", "tool.use"]),
            confidence: ev.confidence,
        }),
        EvidenceSource::InstalledAppScan => {
            let agent_type = s("agent_type").map(|at| match at.as_str() {
                "desktop_agent" => InferredAgentType::DesktopAgent,
                "ide_agent" => InferredAgentType::IdeAgent,
                "cli_agent" => InferredAgentType::CliAgent,
                "browser_agent" => InferredAgentType::BrowserAgent,
                _ => InferredAgentType::AutomationAgent,
            });
            let capability_tags = d
                .get("capability_tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            Some(IdentityHint {
                name: s("name"),
                vendor: s("vendor"),
                product: s("product"),
                agent_type,
                capability_tags,
                confidence: ev.confidence,
            })
        }
        EvidenceSource::BrowserExtension => Some(IdentityHint {
            name: s("name"),
            vendor: s("vendor"),
            confidence: ev.confidence,
            ..Default::default()
        }),
        EvidenceSource::UserConfirmation => Some(IdentityHint {
            name: s("display_name"),
            vendor: s("vendor"),
            product: s("product"),
            confidence: 1.0,
            ..Default::default()
        }),
        _ => None,
    }
}

fn capability_tags_from_data(data: &serde_json::Value, defaults: &[&str]) -> Vec<String> {
    let mut tags: Vec<String> = data
        .get("capability_tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    for default in defaults {
        let default = default.to_string();
        if !tags.contains(&default) {
            tags.push(default);
        }
    }

    tags
}
