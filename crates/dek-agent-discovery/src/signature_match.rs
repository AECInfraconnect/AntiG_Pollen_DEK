use dek_fingerprint_defs::model::{AgentSignatureV2, InstalledAppSignatureDef};

pub struct ProcessFacts<'a> {
    pub process_name: &'a str,
    pub exe_path: &'a str,
    pub cmdline: &'a str,
    pub installed_paths: &'a [String],
}

pub struct SignatureMatch {
    pub id: String,
    pub display_name: String,
    pub vendor: Option<String>,
    pub agent_type: String,
    pub confidence: f64,
    pub matched_by: &'static str,
    pub capability_tags: Vec<String>,
}

pub fn match_process(
    facts: &ProcessFacts,
    sigs: &[AgentSignatureV2],
    apps: &[InstalledAppSignatureDef],
) -> Option<SignatureMatch> {
    let mut best: Option<SignatureMatch> = None;
    let pname = facts.process_name.to_lowercase();
    let exe = facts.exe_path.replace('\\', "/").to_lowercase();
    let cmd = facts.cmdline.to_lowercase();

    for s in sigs {
        let mut conf = 0.0f64;
        let mut by = "";

        if !pname.trim().is_empty()
            && s.process_names.iter().any(|n| {
                !n.trim().is_empty()
                    && (n.eq_ignore_ascii_case(&pname)
                        || strip_ext(&pname) == strip_ext(&n.to_lowercase()))
            })
        {
            conf = conf.max(0.9);
            by = "process_name";
        }

        if !exe.is_empty()
            && s.exe_path_patterns
                .iter()
                .any(|p| !p.trim().is_empty() && glob_match(&p.to_lowercase(), &exe))
        {
            conf = conf.max(0.95);
            by = "exe_path";
        }

        if !cmd.is_empty()
            && s.cmd_patterns
                .iter()
                .any(|p| !p.trim().is_empty() && glob_match(&p.to_lowercase(), &cmd))
        {
            conf = conf.max(0.85);
            by = "cmd_pattern";
        }

        if (!exe.trim().is_empty() || !cmd.trim().is_empty())
            && s.cli_binaries.iter().any(|b| {
                !b.trim().is_empty()
                    && (basename(&exe) == *b || cmd.split_whitespace().next() == Some(b))
            })
        {
            conf = conf.max(0.8);
            by = "cli_binary";
        }

        if s.install_markers.iter().any(|m| {
            facts.installed_paths.iter().any(|ip| {
                !m.path.trim().is_empty()
                    && glob_match(
                        &m.path.to_lowercase(),
                        &ip.replace('\\', "/").to_lowercase(),
                    )
            })
        }) {
            conf = conf.max(0.85);
            by = "install_marker";
        }

        if conf > 0.0 && best.as_ref().map(|b| conf > b.confidence).unwrap_or(true) {
            best = Some(SignatureMatch {
                id: s.id.clone(),
                display_name: s.display_name.clone(),
                vendor: s.vendor.clone(),
                agent_type: s.agent_type.clone(),
                confidence: conf,
                matched_by: leak(by),
                capability_tags: s.capability_tags.clone(),
            });
        }
    }

    for a in apps {
        let hit_path = a.markers.iter().any(|m| {
            m.paths
                .iter()
                .any(|path| !exe.is_empty() && glob_match(&path.to_lowercase(), &exe))
        });
        let hit_name = a.process_names().iter().any(|n| {
            !pname.trim().is_empty() && !n.trim().is_empty() && n.eq_ignore_ascii_case(&pname)
        });
        if hit_path || hit_name {
            let conf = if hit_path { 0.95 } else { 0.9 };
            if best.as_ref().map(|b| conf > b.confidence).unwrap_or(true) {
                best = Some(SignatureMatch {
                    id: a.id.clone(),
                    display_name: a.name.clone(),
                    vendor: Some(a.vendor.clone()),
                    agent_type: a.agent_type.clone(),
                    confidence: conf,
                    matched_by: if hit_path {
                        "install_path"
                    } else {
                        "process_name"
                    },
                    capability_tags: a.capability_tags.clone(),
                });
            }
        }
    }
    best
}

fn strip_ext(s: &str) -> String {
    s.rsplit_once('.')
        .map(|(a, _)| a.to_string())
        .unwrap_or_else(|| s.to_string())
}
fn basename(path: &str) -> String {
    path.rsplit('/').next().map(strip_ext).unwrap_or_default()
}
fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

pub fn glob_match(pat: &str, text: &str) -> bool {
    let pat = pat
        .replace("**", "\u{1}")
        .replace('*', "\u{2}")
        .replace('\u{1}', "*");
    fn rec(p: &[u8], t: &[u8]) -> bool {
        match p.first() {
            None => t.is_empty(),
            Some(b'*') => rec(&p[1..], t) || (!t.is_empty() && rec(p, &t[1..])),
            Some(&0x02) => (!t.is_empty() && t[0] != b'/' && rec(p, &t[1..])) || rec(&p[1..], t),
            Some(&c) => !t.is_empty() && t[0] == c && rec(&p[1..], &t[1..]),
        }
    }
    rec(pat.as_bytes(), text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glob_handles_windowsapps_codex() {
        assert!(glob_match("**/windowsapps/openai.codex_*/**",
            "c:/program files/windowsapps/openai.codex_26.616.3767.0_x64__2p2nqsd0c76g0/app/codex.exe"));
    }
}
