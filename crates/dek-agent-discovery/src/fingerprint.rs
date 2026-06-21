use crate::model::*;

pub struct FingerprintSignals {
    pub matched_process_name: Option<String>,
    pub matched_config_path: Option<String>,
    pub matched_port: Option<u16>,
    pub has_mcp_servers: bool,
}

pub fn compute_confidence(signals: &FingerprintSignals) -> f64 {
    let mut score: f64 = 0.0;
    if signals.matched_process_name.is_some() {
        score += 0.4;
    }
    if signals.matched_config_path.is_some() {
        score += 0.4;
    }
    if signals.matched_port.is_some() {
        score += 0.4;
    }
    if signals.has_mcp_servers {
        score += 0.2;
    }
    score.min(1.0)
}

pub fn infer_agent_type_from_name(name: &str) -> InferredAgentType {
    let lower = name.to_ascii_lowercase();
    if lower.contains("claude") {
        InferredAgentType::DesktopAgent
    } else if lower.contains("cursor")
        || lower.contains("code")
        || lower.contains("windsurf")
        || lower.contains("zed")
    {
        InferredAgentType::IdeAgent
    } else if lower.contains("ollama")
        || lower.contains("lmstudio")
        || lower.contains("vllm")
        || lower.contains("llama")
    {
        InferredAgentType::LocalModelServer
    } else if lower.contains("python") || lower.contains("node") || lower.contains("n8n") {
        InferredAgentType::AutomationAgent
    } else {
        InferredAgentType::UnknownAiProcess
    }
}

pub fn fingerprint_process(process_name: &str) -> f64 {
    let signals = FingerprintSignals {
        matched_process_name: Some(process_name.to_string()),
        matched_config_path: None,
        matched_port: None,
        has_mcp_servers: false,
    };
    compute_confidence(&signals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_agent_type() {
        assert_eq!(
            infer_agent_type_from_name("Claude"),
            InferredAgentType::DesktopAgent
        );
        assert_eq!(
            infer_agent_type_from_name("Cursor"),
            InferredAgentType::IdeAgent
        );
        assert_eq!(
            infer_agent_type_from_name("Ollama"),
            InferredAgentType::LocalModelServer
        );
        assert_eq!(
            infer_agent_type_from_name("NotAnAgent"),
            InferredAgentType::UnknownAiProcess
        );
    }

    #[test]
    fn test_fingerprint_process() {
        assert_eq!(fingerprint_process("Claude"), 0.4);
        assert_eq!(fingerprint_process("Ollama"), 0.4);
        assert_eq!(fingerprint_process("Code"), 0.4);
    }
}
