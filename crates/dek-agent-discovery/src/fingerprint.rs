use crate::model::*;

pub fn infer_agent_type_from_name(name: &str) -> InferredAgentType {
    let lower = name.to_ascii_lowercase();
    if lower.contains("claude") {
        InferredAgentType::DesktopAgent
    } else if lower.contains("cursor") || lower.contains("code") || lower.contains("windsurf") {
        InferredAgentType::IdeAgent
    } else if lower.contains("ollama") || lower.contains("lmstudio") {
        InferredAgentType::LocalModelServer
    } else {
        InferredAgentType::UnknownAiProcess
    }
}

pub fn fingerprint_process(process_name: &str) -> f64 {
    let lower = process_name.to_ascii_lowercase();
    if lower.contains("claude") || lower.contains("cursor") || lower.contains("windsurf") {
        0.85
    } else if lower.contains("ollama") {
        0.95
    } else if lower.contains("code") {
        0.65
    } else {
        0.30
    }
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
        assert_eq!(fingerprint_process("Claude"), 0.85);
        assert_eq!(fingerprint_process("Ollama"), 0.95);
        assert_eq!(fingerprint_process("Code"), 0.65);
        assert_eq!(fingerprint_process("NotAnAgent"), 0.30);
    }
}
