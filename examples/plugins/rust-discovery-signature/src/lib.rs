// SPDX-License-Identifier: Apache-2.0
// Example Pollek discovery signature plugin implementation sketch.
//
// A production plugin should generate WIT bindings for
// pollek:discovery/discovery-plugin@0.1.0 and export the requested world.
// This file stays dependency-light so the example can be read without a full
// cargo-component setup.

pub struct SignatureMatch {
    pub provider: &'static str,
    pub display_name: &'static str,
    pub confidence: u8,
    pub capabilities: &'static [&'static str],
}

pub fn match_process_or_url(value: &str) -> Option<SignatureMatch> {
    let lower = value.to_ascii_lowercase();
    if lower.contains("chatgpt") || lower.contains("openai.com") {
        return Some(SignatureMatch {
            provider: "openai",
            display_name: "ChatGPT",
            confidence: 85,
            capabilities: &["llm.chat", "web.chat", "browser_agent"],
        });
    }
    if lower.contains("claude") || lower.contains("anthropic") {
        return Some(SignatureMatch {
            provider: "anthropic",
            display_name: "Claude",
            confidence: 84,
            capabilities: &["llm.chat", "web.chat"],
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_ai_surface() {
        let matched = match_process_or_url("https://chatgpt.com/c/123");
        assert_eq!(matched.map(|item| item.provider), Some("openai"));
    }
}
