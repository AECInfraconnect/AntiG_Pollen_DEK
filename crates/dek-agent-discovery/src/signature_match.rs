use dek_fingerprint_defs::model::AgentSignatureV2;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct ProcessFacts {
    pub name: String,
    pub exe_path: Option<String>,
    pub cmdline: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentMatch {
    pub agent_id: String,
    pub agent_type: String,
    pub display_name: String,
    pub vendor: Option<String>,
    pub capability_tags: Vec<String>,
    pub confidence: f64,
}

pub fn match_process(facts: &ProcessFacts, catalog: &[AgentSignatureV2]) -> Option<AgentMatch> {
    for sig in catalog {
        let mut score = 0.0;
        let mut match_found = false;

        // 1. match process_names
        if sig
            .process_names
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&facts.name))
        {
            score += 0.5;
            match_found = true;
        }

        // 2. match exe_path_patterns
        if let Some(ref path) = facts.exe_path {
            if sig
                .exe_path_patterns
                .iter()
                .any(|pat| glob_match(pat, path))
            {
                score += 0.4;
                match_found = true;
            }
        }

        // 3. match cmd_patterns
        if let Some(ref cmd) = facts.cmdline {
            if sig.cmd_patterns.iter().any(|pat| glob_match(pat, cmd)) {
                score += 0.4;
                match_found = true;
            }
        }

        if match_found {
            return Some(AgentMatch {
                agent_id: sig.id.clone(),
                agent_type: sig.agent_type.clone(),
                display_name: sig.display_name.clone(),
                vendor: sig.vendor.clone(),
                capability_tags: sig.capability_tags.clone(),
                confidence: f64::min(score, 1.0),
            });
        }
    }
    None
}

pub fn glob_match(pattern: &str, target: &str) -> bool {
    let target_norm = target.replace('\\', "/");
    let re_str = pattern
        .replace("**/", "__GLOB_DIR_START__")
        .replace("/**", "__GLOB_DIR_END__")
        .replace("**", "__GLOB_STAR_STAR__")
        .replace('*', "[^/]*")
        .replace("__GLOB_DIR_START__", "(?:.*/)?")
        .replace("__GLOB_DIR_END__", "(?:/.*)?")
        .replace("__GLOB_STAR_STAR__", ".*");
    match Regex::new(&format!("(?i)^{re_str}$")) {
        Ok(re) => re.is_match(&target_norm),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match(
            "**/WindowsApps/OpenAI.Codex_*/**",
            "C:\\Program Files\\WindowsApps\\OpenAI.Codex_1.2.3_x64\\app.exe"
        ));
        assert!(!glob_match(
            "**/WindowsApps/OpenAI.Codex_*/**",
            "C:\\Program Files\\WindowsApps\\OtherApp_1.2.3_x64\\app.exe"
        ));
        assert!(glob_match(
            "**/Cursor/**",
            "/Applications/Cursor.app/Contents/MacOS/Cursor"
        ));
        assert!(glob_match("*openclaw*", "node openclaw-agent.js"));
    }
}
