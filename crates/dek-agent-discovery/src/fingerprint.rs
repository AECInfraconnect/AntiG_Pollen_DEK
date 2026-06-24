use crate::model::*;

pub struct FingerprintSignals {
    pub matched_process_name: Option<String>,
    pub matched_config_path: Option<String>,
    pub matched_port: Option<u16>,
    pub has_mcp_servers: bool,
}

pub fn compute_confidence(
    signals: &FingerprintSignals,
    weights: &std::collections::HashMap<String, f64>,
) -> f64 {
    let mut score: f64 = 0.0;
    if signals.matched_process_name.is_some() {
        score += weights.get("process_name").copied().unwrap_or(0.5);
    }
    if signals.matched_config_path.is_some() {
        score += weights.get("config_path").copied().unwrap_or(0.3);
    }
    if signals.matched_port.is_some() {
        score += weights.get("port").copied().unwrap_or(0.2);
    }
    if signals.has_mcp_servers {
        score += weights.get("mcp_servers").copied().unwrap_or(0.2);
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
    let agent_type = infer_agent_type_from_name(process_name);
    let matched_process_name = if agent_type != InferredAgentType::UnknownAiProcess {
        Some(process_name.to_string())
    } else {
        None
    };

    let signals = FingerprintSignals {
        matched_process_name,
        matched_config_path: None,
        matched_port: None,
        has_mcp_servers: false,
    };
    let mut default_weights = std::collections::HashMap::new();
    default_weights.insert("process_name".to_string(), 0.6);
    default_weights.insert("config_path".to_string(), 0.4);
    default_weights.insert("port".to_string(), 0.4);
    default_weights.insert("mcp_servers".to_string(), 0.2);

    compute_confidence(&signals, &default_weights)
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
        assert_eq!(fingerprint_process("Claude"), 0.6);
        assert_eq!(fingerprint_process("Ollama"), 0.6);
        assert_eq!(fingerprint_process("Code"), 0.6);
        assert_eq!(fingerprint_process("NotAnAgent"), 0.0);
    }
}
