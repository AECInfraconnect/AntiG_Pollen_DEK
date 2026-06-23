use crate::model::*;
use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;

pub struct SniFlow {
    pub browser_pid: Option<u32>,
    pub sni_host: String,
    pub ts: u64,
}

pub trait SniFlowSource: Send + Sync {
    /// คืน flow ที่ DEK เห็นในหน้าต่างเวลาล่าสุด
    fn recent_flows(&self, since: Duration) -> Vec<SniFlow>;
}

pub struct WebAiSignature {
    pub domain: &'static str,
    pub name: &'static str,
    pub vendor: &'static str,
}

pub const WEB_AI_CATALOG: &[WebAiSignature] = &[
    WebAiSignature {
        domain: "chatgpt.com",
        name: "ChatGPT in Browser",
        vendor: "OpenAI",
    },
    WebAiSignature {
        domain: "claude.ai",
        name: "Claude in Browser",
        vendor: "Anthropic",
    },
    WebAiSignature {
        domain: "chat.deepseek.com",
        name: "Deepseek in Browser",
        vendor: "DeepSeek",
    },
    WebAiSignature {
        domain: "perplexity.ai",
        name: "Perplexity",
        vendor: "Perplexity",
    },
    WebAiSignature {
        domain: "poe.com",
        name: "Poe",
        vendor: "Quora",
    },
    WebAiSignature {
        domain: "gemini.google.com",
        name: "Gemini",
        vendor: "Google",
    },
    WebAiSignature {
        domain: "copilot.microsoft.com",
        name: "Copilot",
        vendor: "Microsoft",
    },
    WebAiSignature {
        domain: "huggingface.co/chat",
        name: "HuggingChat",
        vendor: "HuggingFace",
    },
    WebAiSignature {
        domain: "grok.com",
        name: "Grok",
        vendor: "xAI",
    },
    WebAiSignature {
        domain: "chat.mistral.ai",
        name: "Mistral",
        vendor: "Mistral AI",
    },
];

pub fn scan_web_ai(
    sni_source: Option<&dyn SniFlowSource>,
    config: &crate::config::DiscoveryConfig,
) -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();

    if config.enable_browser_history_scan {
        if let Ok(mut hist) = scan_history() {
            evidence.append(&mut hist);
        }
    }

    if config.enable_browser_session_scan {
        if let Ok(mut sess) = scan_sessions() {
            evidence.append(&mut sess);
        }
    }

    if config.enable_network_sni_scan {
        if let Some(source) = sni_source {
            if let Ok(mut net) = scan_network_sni(source) {
                evidence.append(&mut net);
            }
        }
    }

    Ok(evidence)
}

fn scan_history() -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();
    let history_paths = get_browser_history_paths();

    for path in history_paths {
        if !path.exists() {
            continue;
        }

        let temp_path = path.with_extension(format!("temp_{}", uuid::Uuid::new_v4()));
        // Copy to avoid SQLite lock
        if std::fs::copy(&path, &temp_path).is_err() {
            continue;
        }

        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &temp_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            let stmt = conn.prepare("SELECT url, title, last_visit_time, visit_count FROM urls ORDER BY last_visit_time DESC LIMIT 1000");
            if let Ok(mut stmt) = stmt {
                let url_iter = stmt.query_map([], |row| row.get::<_, String>(0));

                if let Ok(url_iter) = url_iter {
                    for url_result in url_iter.flatten() {
                        if let Ok(parsed_url) = url::Url::parse(&url_result) {
                            if let Some(host) = parsed_url.host_str() {
                                for sig in WEB_AI_CATALOG {
                                    if host.ends_with(sig.domain) {
                                        let origin = format!("{}://{}", parsed_url.scheme(), host);

                                        evidence.push(DiscoveryEvidenceV2 {
                                            evidence_id: uuid::Uuid::new_v4().to_string(),
                                            source: EvidenceSource::BrowserHistory,
                                            confidence: 0.6,
                                            observed_at: chrono::Utc::now().to_rfc3339(),
                                            privacy_class: PrivacyClass::InternalMetadata,
                                            redacted: true,
                                            data: serde_json::json!({
                                                "origin": origin,
                                                "name": sig.name,
                                                "vendor": sig.vendor,
                                            }),
                                            merge_key: Some(sig.domain.to_string()),
                                            source_path_hash: Some(
                                                crate::redaction::sha256_string(
                                                    &path.to_string_lossy(),
                                                ),
                                            ),
                                            source_path_redacted: Some(
                                                crate::redaction::redact_path_for_ui(
                                                    &path.to_string_lossy(),
                                                ),
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = std::fs::remove_file(temp_path);
    }

    Ok(evidence)
}

fn scan_sessions() -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();
    let session_paths = get_browser_session_paths();

    for path in session_paths {
        if !path.exists() {
            continue;
        }

        // Since session paths can be directories (like `Sessions` folder in newer Chrome), we should read all files in it if it's a dir.
        let mut files_to_scan = Vec::new();
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let file_type = entry.file_type();
                    let is_file = match file_type {
                        Ok(ft) => ft.is_file(),
                        Err(_) => {
                            if let Ok(meta) = std::fs::metadata(entry.path()) {
                                meta.file_type().is_file()
                            } else {
                                false
                            }
                        }
                    };

                    if is_file {
                        files_to_scan.push(entry.path());
                    }
                }
            }
        } else {
            files_to_scan.push(path);
        }

        for file_path in files_to_scan {
            if let Ok(content) = std::fs::read(&file_path) {
                let content_str = String::from_utf8_lossy(&content);
                for sig in WEB_AI_CATALOG {
                    if content_str.contains(sig.domain) {
                        evidence.push(DiscoveryEvidenceV2 {
                            evidence_id: uuid::Uuid::new_v4().to_string(),
                            source: EvidenceSource::BrowserSession,
                            confidence: 0.8,
                            observed_at: chrono::Utc::now().to_rfc3339(),
                            privacy_class: PrivacyClass::InternalMetadata,
                            redacted: true,
                            data: serde_json::json!({
                                "origin": format!("https://{}", sig.domain),
                                "name": sig.name,
                                "vendor": sig.vendor,
                            }),
                            merge_key: Some(sig.domain.to_string()),
                            source_path_hash: Some(crate::redaction::sha256_string(
                                &file_path.to_string_lossy(),
                            )),
                            source_path_redacted: Some(crate::redaction::redact_path_for_ui(
                                &file_path.to_string_lossy(),
                            )),
                        });
                    }
                }
            }
        }
    }

    Ok(evidence)
}

fn scan_network_sni(source: &dyn SniFlowSource) -> Result<Vec<DiscoveryEvidenceV2>> {
    let mut evidence = Vec::new();

    // Query recent flows from the injected source (e.g., from spool or eBPF directly)
    let recent_snis = source.recent_flows(Duration::from_secs(3600));

    for flow in recent_snis {
        for sig in WEB_AI_CATALOG {
            if flow.sni_host.ends_with(sig.domain) {
                evidence.push(DiscoveryEvidenceV2 {
                    evidence_id: uuid::Uuid::new_v4().to_string(),
                    source: EvidenceSource::NetworkSni,
                    confidence: 1.0,
                    observed_at: chrono::Utc::now().to_rfc3339(),
                    privacy_class: PrivacyClass::InternalMetadata,
                    redacted: true,
                    data: serde_json::json!({
                        "origin": format!("https://{}", sig.domain),
                        "name": sig.name,
                        "vendor": sig.vendor,
                        "browser_pid": flow.browser_pid,
                    }),
                    merge_key: Some(sig.domain.to_string()),
                    source_path_hash: None,
                    source_path_redacted: Some("network:sni".to_string()),
                });
            }
        }
    }

    Ok(evidence)
}

fn get_browser_history_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        if let Ok(entries) = std::fs::read_dir(
            PathBuf::from(&appdata)
                .join("Mozilla")
                .join("Firefox")
                .join("Profiles"),
        ) {
            for entry in entries.flatten() {
                paths.push(entry.path().join("places.sqlite"));
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(&localappdata)
                .join("Google")
                .join("Chrome")
                .join("User Data")
                .join("Default")
                .join("History"),
        );
        paths.push(
            PathBuf::from(&localappdata)
                .join("Microsoft")
                .join("Edge")
                .join("User Data")
                .join("Default")
                .join("History"),
        );
    }

    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        paths.push(
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome")
                .join("Default")
                .join("History"),
        );
        if let Ok(entries) = std::fs::read_dir(
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("Firefox")
                .join("Profiles"),
        ) {
            for entry in entries.flatten() {
                paths.push(entry.path().join("places.sqlite"));
            }
        }
    }

    #[cfg(target_os = "linux")]
    if let Ok(home) = std::env::var("HOME") {
        paths.push(
            PathBuf::from(&home)
                .join(".config")
                .join("google-chrome")
                .join("Default")
                .join("History"),
        );
        if let Ok(entries) =
            std::fs::read_dir(PathBuf::from(&home).join(".mozilla").join("firefox"))
        {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    paths.push(entry.path().join("places.sqlite"));
                }
            }
        }
    }

    paths
}

fn get_browser_session_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        if let Ok(entries) = std::fs::read_dir(
            PathBuf::from(&appdata)
                .join("Mozilla")
                .join("Firefox")
                .join("Profiles"),
        ) {
            for entry in entries.flatten() {
                paths.push(entry.path().join("sessionstore.jsonlz4"));
                paths.push(
                    entry
                        .path()
                        .join("sessionstore-backups")
                        .join("recovery.jsonlz4"),
                );
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(&localappdata)
                .join("Google")
                .join("Chrome")
                .join("User Data")
                .join("Default")
                .join("Sessions"),
        );
        paths.push(
            PathBuf::from(&localappdata)
                .join("Microsoft")
                .join("Edge")
                .join("User Data")
                .join("Default")
                .join("Sessions"),
        );
    }

    #[cfg(target_os = "macos")]
    if let Ok(home) = std::env::var("HOME") {
        paths.push(
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome")
                .join("Default")
                .join("Sessions"),
        );
        if let Ok(entries) = std::fs::read_dir(
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("Firefox")
                .join("Profiles"),
        ) {
            for entry in entries.flatten() {
                paths.push(entry.path().join("sessionstore.jsonlz4"));
                paths.push(
                    entry
                        .path()
                        .join("sessionstore-backups")
                        .join("recovery.jsonlz4"),
                );
            }
        }
    }

    #[cfg(target_os = "linux")]
    if let Ok(home) = std::env::var("HOME") {
        paths.push(
            PathBuf::from(&home)
                .join(".config")
                .join("google-chrome")
                .join("Default")
                .join("Sessions"),
        );
        if let Ok(entries) =
            std::fs::read_dir(PathBuf::from(&home).join(".mozilla").join("firefox"))
        {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    paths.push(entry.path().join("sessionstore.jsonlz4"));
                    paths.push(
                        entry
                            .path()
                            .join("sessionstore-backups")
                            .join("recovery.jsonlz4"),
                    );
                }
            }
        }
    }

    paths
}
