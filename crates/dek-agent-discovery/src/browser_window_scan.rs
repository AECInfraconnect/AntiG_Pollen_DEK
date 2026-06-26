use crate::model::{DiscoveryEvidenceV2, EvidenceSource, PrivacyClass};
use anyhow::Result;
use dek_fingerprint_defs::model::{BrowserProcessDef, WebAiSignatureDef};
#[cfg(target_os = "windows")]
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserWindow {
    pub pid: u32,
    pub process_name: String,
    pub title: String,
    pub cmdline: String,
}

pub fn scan_browser_windows(
    catalog: &[WebAiSignatureDef],
    browsers: &[BrowserProcessDef],
) -> Result<Vec<DiscoveryEvidenceV2>> {
    let windows = enumerate_browser_windows(browsers);
    Ok(evidence_from_browser_windows(&windows, catalog))
}

pub(crate) fn evidence_from_browser_windows(
    windows: &[BrowserWindow],
    catalog: &[WebAiSignatureDef],
) -> Vec<DiscoveryEvidenceV2> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for window in windows {
        for sig in catalog {
            let by_title = title_matches_signature(&window.title, sig);
            let by_app = app_cmdline_matches_signature(&window.cmdline, sig);
            if !by_title && !by_app {
                continue;
            }

            let matched_domain = sig.domains().into_iter().find(|domain| {
                let domain_l = domain.to_ascii_lowercase();
                window.title.to_ascii_lowercase().contains(&domain_l)
                    || window.cmdline.to_ascii_lowercase().contains(&domain_l)
            });
            let matched_domain = matched_domain.unwrap_or(sig.domain.as_str());
            let browser_name = browser_display_name(&window.process_name);
            let browser_id = browser_id(&window.process_name);
            let key = format!("webai:{}:{}", sig.stable_id(), browser_id);

            if !seen.insert(key.clone()) {
                continue;
            }

            let detected_via = if by_app {
                "browser_app_cmdline"
            } else {
                "browser_window_title"
            };
            let confidence = if by_app { 0.95 } else { 0.85 };

            out.push(DiscoveryEvidenceV2 {
                evidence_id: uuid::Uuid::new_v4().to_string(),
                source: EvidenceSource::BrowserWindow,
                confidence,
                observed_at: chrono::Utc::now().to_rfc3339(),
                privacy_class: PrivacyClass::InternalMetadata,
                redacted: true,
                data: serde_json::json!({
                    "origin": format!("https://{}", matched_domain),
                    "name": browser_scoped_ai_name(&sig.name, browser_name),
                    "base_name": sig.name.clone(),
                    "vendor": sig.vendor.clone(),
                    "domain": sig.domain.clone(),
                    "matched_domain": matched_domain,
                    "browser": window.process_name.clone(),
                    "browser_id": browser_id,
                    "browser_name": browser_name,
                    "pid": window.pid,
                    "detected_via": detected_via,
                    "capability_tags": sig.capability_tags.clone(),
                }),
                merge_key: Some(key),
                source_path_hash: None,
                source_path_redacted: Some(window.process_name.clone()),
            });
        }
    }

    out
}

pub fn default_browser_processes() -> Vec<BrowserProcessDef> {
    vec![
        browser("chromium", &["chrome.exe", "chrome", "google chrome"]),
        browser("chromium", &["msedge.exe", "msedge", "microsoft edge"]),
        browser("chromium", &["brave.exe", "brave", "brave browser"]),
        browser("chromium", &["opera.exe", "opera", "opera gx"]),
        browser("chromium", &["vivaldi.exe", "vivaldi"]),
        browser("chromium", &["chromium.exe", "chromium"]),
        browser("chromium", &["arc.exe", "arc"]),
        browser("firefox", &["firefox.exe", "firefox"]),
        browser("webkit", &["safari"]),
    ]
}

pub fn is_browser_process(process_name: &str, browsers: &[BrowserProcessDef]) -> bool {
    let defs;
    let browsers = if browsers.is_empty() {
        defs = default_browser_processes();
        defs.as_slice()
    } else {
        browsers
    };

    let normalized = normalize_process_name(process_name);
    browsers.iter().any(|browser| {
        browser
            .process_names
            .iter()
            .any(|name| normalize_process_name(name) == normalized)
    })
}

pub fn browser_display_name(process_name: &str) -> &'static str {
    match browser_id(process_name) {
        "chrome" => "Chrome",
        "edge" => "Edge",
        "brave" => "Brave",
        "opera" => "Opera",
        "vivaldi" => "Vivaldi",
        "chromium" => "Chromium",
        "arc" => "Arc",
        "firefox" => "Firefox",
        "safari" => "Safari",
        _ => "Browser",
    }
}

pub fn browser_id(process_name: &str) -> &'static str {
    let normalized = normalize_process_name(process_name);
    match normalized.as_str() {
        "chrome" | "google chrome" => "chrome",
        "msedge" | "microsoft edge" => "edge",
        "brave" | "brave browser" => "brave",
        "opera" | "opera gx" => "opera",
        "vivaldi" => "vivaldi",
        "chromium" => "chromium",
        "arc" => "arc",
        "firefox" => "firefox",
        "safari" => "safari",
        _ => "browser",
    }
}

pub fn browser_scoped_ai_name(base_name: &str, browser_name: &str) -> String {
    let base = base_name.strip_suffix(" (Web)").unwrap_or(base_name).trim();
    format!("{base} ({browser_name})")
}

fn browser(engine: &str, process_names: &[&str]) -> BrowserProcessDef {
    BrowserProcessDef {
        engine: engine.to_string(),
        process_names: process_names.iter().map(|s| s.to_string()).collect(),
    }
}

fn enumerate_browser_windows(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
    let mut windows = enumerate_browser_process_cmdlines(browsers);
    merge_platform_window_titles(&mut windows, browsers);
    windows
}

fn enumerate_browser_process_cmdlines(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut out = Vec::new();
    for (pid, process) in sys.processes() {
        let process_name = process.name().to_string_lossy().to_string();
        if !is_browser_process(&process_name, browsers) {
            continue;
        }
        let cmdline = process
            .cmd()
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        out.push(BrowserWindow {
            pid: pid.as_u32(),
            process_name,
            title: String::new(),
            cmdline,
        });
    }
    out
}

#[cfg(target_os = "windows")]
fn merge_platform_window_titles(windows: &mut Vec<BrowserWindow>, browsers: &[BrowserProcessDef]) {
    let mut by_pid: std::collections::HashMap<u32, usize> = windows
        .iter()
        .enumerate()
        .map(|(idx, window)| (window.pid, idx))
        .collect();

    for title in powershell_window_titles() {
        if !is_browser_process(&title.process_name, browsers) {
            continue;
        }
        if let Some(idx) = by_pid.get(&title.pid).copied() {
            if windows[idx].title.is_empty() {
                windows[idx].title = title.title;
            }
        } else {
            let idx = windows.len();
            by_pid.insert(title.pid, idx);
            windows.push(BrowserWindow {
                pid: title.pid,
                process_name: title.process_name,
                title: title.title,
                cmdline: String::new(),
            });
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn merge_platform_window_titles(
    _windows: &mut Vec<BrowserWindow>,
    _browsers: &[BrowserProcessDef],
) {
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WinProcessTitle {
    id: u32,
    process_name: String,
    main_window_title: Option<String>,
}

#[cfg(target_os = "windows")]
struct WindowTitle {
    pid: u32,
    process_name: String,
    title: String,
}

#[cfg(target_os = "windows")]
fn powershell_window_titles() -> Vec<WindowTitle> {
    let script = "Get-Process | Where-Object {$_.MainWindowTitle} | Select-Object Id,ProcessName,MainWindowTitle | ConvertTo-Json -Compress";
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() || output.stdout.is_empty() {
        return Vec::new();
    }

    let value = match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let rows = if value.is_array() {
        value.as_array().cloned().unwrap_or_default()
    } else {
        vec![value]
    };

    rows.into_iter()
        .filter_map(|row| serde_json::from_value::<WinProcessTitle>(row).ok())
        .filter_map(|row| {
            let title = row.main_window_title.unwrap_or_default();
            if title.trim().is_empty() {
                None
            } else {
                Some(WindowTitle {
                    pid: row.id,
                    process_name: row.process_name,
                    title,
                })
            }
        })
        .collect()
}

fn title_matches_signature(title: &str, sig: &WebAiSignatureDef) -> bool {
    let normalized_title = title.to_ascii_lowercase();
    if normalized_title.trim().is_empty() {
        return false;
    }

    let mut patterns: Vec<&str> = sig.title_patterns.iter().map(String::as_str).collect();
    if patterns.is_empty() {
        patterns.push(&sig.name);
        patterns.push(&sig.domain);
    }

    patterns.into_iter().any(|pattern| {
        let pattern = pattern.trim().to_ascii_lowercase();
        !pattern.is_empty()
            && (normalized_title.contains(&pattern)
                || normalized_title
                    .split(['-', '|', ':'])
                    .any(|segment| segment.trim() == pattern))
    })
}

fn app_cmdline_matches_signature(cmdline: &str, sig: &WebAiSignatureDef) -> bool {
    let normalized_cmd = cmdline.to_ascii_lowercase();
    if !normalized_cmd.contains("--app=") {
        return false;
    }

    let mut patterns: Vec<String> = sig
        .app_cmdline_patterns
        .iter()
        .map(|p| p.to_ascii_lowercase())
        .collect();
    if patterns.is_empty() {
        patterns.extend(
            sig.domains()
                .into_iter()
                .map(|domain| format!("*--app=*{}*", domain.to_ascii_lowercase())),
        );
    }

    patterns
        .into_iter()
        .any(|pattern| glob_match(&pattern, &normalized_cmd))
}

fn glob_match(pattern: &str, text: &str) -> bool {
    match glob::Pattern::new(pattern) {
        Ok(pattern) if pattern.matches(text) => true,
        Ok(_) => {
            let wrapped = format!("*{}*", pattern.trim_matches('*'));
            glob::Pattern::new(&wrapped)
                .map(|pattern| pattern.matches(text))
                .unwrap_or(false)
        }
        Err(_) => text.contains(pattern.trim_matches('*')),
    }
}

fn normalize_process_name(name: &str) -> String {
    let lower = name.trim().to_ascii_lowercase();
    lower.trim_end_matches(".exe").replace(['-', '_'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn web_sig(id: &str, domain: &str, name: &str, title: &[&str]) -> WebAiSignatureDef {
        WebAiSignatureDef {
            id: id.to_string(),
            domain: domain.to_string(),
            alias_domains: Vec::new(),
            name: name.to_string(),
            vendor: "Test".to_string(),
            title_patterns: title.iter().map(|v| v.to_string()).collect(),
            app_cmdline_patterns: vec![format!("--app=*{}*", domain)],
            capability_tags: vec!["llm.chat".to_string()],
            risk_weight: 0.5,
        }
    }

    #[test]
    fn title_and_app_mode_create_distinct_web_ai_candidates() {
        let catalog = vec![
            web_sig("chatgpt_web", "chatgpt.com", "ChatGPT (Web)", &["ChatGPT"]),
            web_sig("claude_web", "claude.ai", "Claude (Web)", &["Claude"]),
            web_sig(
                "gemini_web",
                "gemini.google.com",
                "Gemini (Web)",
                &["Gemini"],
            ),
            web_sig(
                "deepseek_web",
                "chat.deepseek.com",
                "DeepSeek (Web)",
                &["DeepSeek"],
            ),
        ];
        let windows = vec![
            BrowserWindow {
                pid: 1,
                process_name: "chrome.exe".into(),
                title: "ChatGPT - Google Chrome".into(),
                cmdline: String::new(),
            },
            BrowserWindow {
                pid: 2,
                process_name: "chrome.exe".into(),
                title: "Claude".into(),
                cmdline: String::new(),
            },
            BrowserWindow {
                pid: 3,
                process_name: "chrome.exe".into(),
                title: "Google Gemini".into(),
                cmdline: String::new(),
            },
            BrowserWindow {
                pid: 4,
                process_name: "msedge.exe".into(),
                title: String::new(),
                cmdline: "msedge.exe --app=https://chat.deepseek.com".into(),
            },
        ];

        let evidence = evidence_from_browser_windows(&windows, &catalog);
        let names = evidence
            .iter()
            .filter_map(|ev| ev.data.get("name").and_then(|v| v.as_str()))
            .collect::<HashSet<_>>();

        assert_eq!(names.len(), 4);
        assert!(names.contains("ChatGPT (Chrome)"));
        assert!(names.contains("Claude (Chrome)"));
        assert!(names.contains("Gemini (Chrome)"));
        assert!(names.contains("DeepSeek (Edge)"));
    }

    #[test]
    fn same_web_ai_in_multiple_browsers_creates_browser_scoped_evidence() {
        let catalog = vec![web_sig(
            "chatgpt_web",
            "chatgpt.com",
            "ChatGPT (Web)",
            &["ChatGPT"],
        )];
        let windows = vec![
            BrowserWindow {
                pid: 1,
                process_name: "chrome.exe".into(),
                title: "ChatGPT - Google Chrome".into(),
                cmdline: "chrome.exe https://chatgpt.com".into(),
            },
            BrowserWindow {
                pid: 2,
                process_name: "msedge.exe".into(),
                title: "ChatGPT - Microsoft Edge".into(),
                cmdline: "msedge.exe https://chatgpt.com".into(),
            },
        ];

        let evidence = evidence_from_browser_windows(&windows, &catalog);
        let names = evidence
            .iter()
            .filter_map(|ev| ev.data.get("name").and_then(|v| v.as_str()))
            .collect::<HashSet<_>>();
        let merge_keys = evidence
            .iter()
            .filter_map(|ev| ev.merge_key.as_deref())
            .collect::<HashSet<_>>();

        assert_eq!(evidence.len(), 2);
        assert!(names.contains("ChatGPT (Chrome)"));
        assert!(names.contains("ChatGPT (Edge)"));
        assert!(merge_keys.contains("webai:chatgpt_web:chrome"));
        assert!(merge_keys.contains("webai:chatgpt_web:edge"));
    }

    #[test]
    fn browser_process_matching_uses_definition_aliases() {
        let browsers = vec![browser("chromium", &["msedge.exe", "Google Chrome"])];

        assert!(is_browser_process("msedge", &browsers));
        assert!(is_browser_process("google chrome", &browsers));
        assert!(!is_browser_process("codex.exe", &browsers));
    }
}
