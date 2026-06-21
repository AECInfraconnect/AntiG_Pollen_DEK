use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfigEvidence {
    pub config_path_hash: String,
    pub config_path_redacted: String,
    pub client_hint: String,
    pub server_name: String,
    pub transport: String,
    pub command_template: Option<Vec<String>>,
    pub endpoint_domain: Option<String>,
    pub env_key_names: Vec<String>,
    pub has_auth_header: bool,
}

#[derive(Debug, Clone)]
pub struct McpServerEntry {
    pub name: String,
    pub transport: McpTransport,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env_keys: Vec<String>,
    pub url: Option<String>,
    pub has_auth_header: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum McpTransport {
    Stdio,
    Http,
    Sse,
    Unknown,
}

pub fn parse_mcp_config(_parser_id: &str, content: &str) -> Vec<McpServerEntry> {
    let json: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let node = json
        .get("mcpServers")
        .or_else(|| json.get("servers"))
        .or_else(|| json.get("context_servers"));
    let Some(node) = node else {
        return vec![];
    };

    let mut out = Vec::new();
    let entries: Vec<(String, &serde_json::Value)> = match node {
        serde_json::Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v)).collect(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(|v| {
                (
                    v.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unnamed")
                        .to_string(),
                    v,
                )
            })
            .collect(),
        _ => return vec![],
    };

    for (name, cfg) in entries {
        let url = cfg
            .get("url")
            .or_else(|| cfg.get("serverUrl"))
            .and_then(|v| v.as_str())
            .map(String::from);
        let command = cfg
            .get("command")
            .and_then(|v| v.as_str())
            .map(String::from);
        let transport = match (cfg.get("type").and_then(|v| v.as_str()), &url, &command) {
            (Some("http"), _, _) => McpTransport::Http,
            (Some("sse"), _, _) => McpTransport::Sse,
            (_, Some(u), _) if u.contains("/sse") => McpTransport::Sse,
            (_, Some(_), _) => McpTransport::Http,
            (_, _, Some(_)) => McpTransport::Stdio,
            _ => McpTransport::Unknown,
        };
        let env_keys = cfg
            .get("env")
            .and_then(|e| e.as_object())
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        let has_auth = cfg
            .get("headers")
            .and_then(|h| h.as_object())
            .map(|o| {
                o.keys().any(|k| {
                    k.eq_ignore_ascii_case("authorization")
                        || k.to_lowercase().contains("api-key")
                        || k.to_lowercase().contains("token")
                })
            })
            .unwrap_or(false);
        let args = cfg
            .get("args")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        out.push(McpServerEntry {
            name,
            transport,
            command,
            args,
            env_keys,
            url,
            has_auth_header: has_auth,
        });
    }
    out
}

pub fn discover_mcp_configs(paths: &[PathBuf]) -> Result<Vec<McpServerConfigEvidence>> {
    let mut out = vec![];
    for p in paths {
        if !p.exists() || !p.is_file() {
            continue;
        }
        let text = match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let client_hint = infer_client_from_path(p);
        let parser_id = match client_hint.as_str() {
            "vscode" => "mcp_servers_vscode",
            "zed" => "mcp_servers_zed",
            "continue" => "mcp_servers_continue_array",
            _ => "mcp_servers_object",
        };

        let entries = parse_mcp_config(parser_id, &text);

        for entry in entries {
            let transport_str = match entry.transport {
                McpTransport::Stdio => "stdio",
                McpTransport::Http => "http",
                McpTransport::Sse => "sse",
                McpTransport::Unknown => "unknown",
            };

            let command_template = entry.command.map(|c| {
                std::iter::once(crate::redaction::redact_arg(&c))
                    .chain(entry.args.iter().map(|a| crate::redaction::redact_arg(a)))
                    .collect()
            });

            let endpoint_domain = entry
                .url
                .and_then(|u| url::Url::parse(&u).ok())
                .and_then(|u| u.host_str().map(|s| s.to_string()));

            out.push(McpServerConfigEvidence {
                config_path_hash: crate::redaction::sha256_string(&p.to_string_lossy()),
                config_path_redacted: crate::redaction::redact_path_for_ui(&p.to_string_lossy()),
                client_hint: client_hint.clone(),
                server_name: entry.name,
                transport: transport_str.into(),
                command_template,
                endpoint_domain,
                env_key_names: entry.env_keys,
                has_auth_header: entry.has_auth_header,
            });
        }
    }
    Ok(out)
}

fn infer_client_from_path(path: &std::path::Path) -> String {
    let s = path.to_string_lossy().to_ascii_lowercase();
    if s.contains("claude") {
        "claude-desktop".into()
    } else if s.contains("cursor") {
        "cursor".into()
    } else if s.contains("windsurf") {
        "windsurf".into()
    } else if s.contains("code") || s.contains("vscode") {
        "vscode".into()
    } else if s.contains("zed") {
        "zed".into()
    } else if s.contains("continue") {
        "continue".into()
    } else {
        "unknown".into()
    }
}
