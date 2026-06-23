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
        EvidenceSource::BrowserSession | EvidenceSource::BrowserHistory => Some(IdentityHint {
            name: s("name"), vendor: s("vendor"),
            agent_type: Some(InferredAgentType::WebAIApp),
            capability_tags: vec!["web.chat".into(), "net.egress.llm".into()],
            confidence: ev.confidence, ..Default::default()
        }),
        EvidenceSource::NetworkSni | EvidenceSource::NetworkEgress => Some(IdentityHint {
            name: s("name").or_else(|| s("sni_host")), vendor: s("vendor"),
            agent_type: Some(InferredAgentType::WebAIApp),
            capability_tags: vec!["net.egress.llm".into()],
            confidence: ev.confidence, ..Default::default()
        }),
        EvidenceSource::CliAgent => Some(IdentityHint {
            name: s("name").or_else(|| s("cli_name")), vendor: s("vendor"),
            product: s("product"),
            agent_type: Some(InferredAgentType::CliAgent),
            capability_tags: vec!["code.agentic".into()],
            confidence: ev.confidence, ..Default::default()
        }),
        EvidenceSource::Container => Some(IdentityHint {
            name: s("name").or_else(|| s("image")), vendor: s("vendor"),
            agent_type: Some(InferredAgentType::AutomationAgent),
            confidence: ev.confidence, ..Default::default()
        }),
        EvidenceSource::PythonFramework => Some(IdentityHint {
            name: s("framework").or_else(|| s("name")), vendor: s("vendor"),
            agent_type: Some(InferredAgentType::AutomationAgent),
            capability_tags: vec!["automation".into(), "tool.use".into()],
            confidence: ev.confidence, ..Default::default()
        }),
        EvidenceSource::InstalledAppScan | EvidenceSource::BrowserExtension => Some(IdentityHint {
            name: s("name"), vendor: s("vendor"),
            confidence: ev.confidence, ..Default::default()
        }),
        EvidenceSource::UserConfirmation => Some(IdentityHint {
            name: s("display_name"), vendor: s("vendor"), product: s("product"),
            confidence: 1.0, ..Default::default()
        }),
        _ => None,
    }
}
