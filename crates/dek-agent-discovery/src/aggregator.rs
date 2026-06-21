use crate::model::*;
use std::collections::{BTreeMap, HashMap};

pub fn aggregate_evidence(
    tenant_id: &str,
    mut evidence: Vec<DiscoveryEvidenceV2>,
) -> Vec<DiscoveredAgentCandidateV2> {
    // Group evidence by merge_key
    let mut groups: HashMap<String, Vec<DiscoveryEvidenceV2>> = HashMap::new();

    for ev in evidence.drain(..) {
        let key = ev
            .merge_key
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        groups.entry(key).or_default().push(ev);
    }

    let mut candidates = Vec::new();

    for (_key, group) in groups {
        let mut max_confidence = 0.0;
        let risk_score = 10;
        let mut agent_type = InferredAgentType::UnknownAiProcess;
        let mut name = "Unknown Agent".to_string();

        let mut process_hash = None;
        let mut mcp_servers = Vec::new();
        let mut endpoints = Vec::new();
        let mut redacted_env_keys = Vec::new();

        let mut signals = crate::fingerprint::FingerprintSignals {
            matched_process_name: None,
            matched_config_path: None,
            matched_port: None,
            has_mcp_servers: false,
        };

        for ev in &group {
            if ev.confidence > max_confidence {
                max_confidence = ev.confidence;
            }

            match ev.source {
                EvidenceSource::ProcessScan => {
                    name = ev.source_path_redacted.clone().unwrap_or(name);
                    agent_type = crate::fingerprint::infer_agent_type_from_name(&name);
                    process_hash = ev.source_path_hash.clone();
                    signals.matched_process_name = Some(name.clone());
                }
                EvidenceSource::McpConfig => {
                    if agent_type == InferredAgentType::UnknownAiProcess {
                        agent_type = InferredAgentType::DesktopAgent;
                        name = "MCP Capable Agent".to_string();
                    }
                    signals.matched_config_path = ev.source_path_redacted.clone();

                    if let Some(transport) = ev.data.get("transport").and_then(|v| v.as_str()) {
                        let server_name = ev
                            .data
                            .get("server_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let command = ev
                            .data
                            .get("command_template")
                            .and_then(|v| v.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        mcp_servers.push(DiscoveredMcpServerRef {
                            server_name: server_name.clone(),
                            transport: transport.to_string(),
                            command,
                        });

                        signals.has_mcp_servers = true;

                        if let Some(env_keys) =
                            ev.data.get("env_key_names").and_then(|v| v.as_array())
                        {
                            for key in env_keys {
                                if let Some(k) = key.as_str() {
                                    if !redacted_env_keys.contains(&k.to_string()) {
                                        redacted_env_keys.push(k.to_string());
                                    }
                                }
                            }
                        }
                    } else if let Some(data) = ev.data.get("servers") {
                        // Fallback for older mock tests
                        if let Some(obj) = data.as_object() {
                            for (k, v) in obj {
                                mcp_servers.push(DiscoveredMcpServerRef {
                                    server_name: k.to_string(),
                                    transport: "stdio".into(),
                                    command: v
                                        .get("command")
                                        .and_then(|c| c.as_str())
                                        .map(|s| s.to_string()),
                                });
                                signals.has_mcp_servers = true;
                            }
                        }
                    }
                }
                EvidenceSource::LocalModelServer => {
                    agent_type = InferredAgentType::LocalModelServer;
                    name = "Local Model Server".into();
                    if let Some(key_url) = &ev.merge_key {
                        endpoints.push(DiscoveredEndpointRef {
                            url: key_url.clone(),
                            protocol: "http".into(),
                        });
                    }
                    signals.matched_port = Some(80); // Just a dummy port to indicate port matched
                }
                EvidenceSource::PortProbe => {
                    agent_type = InferredAgentType::LocalModelServer;
                    name = "Local Server".into();
                    if let Some(key_url) = &ev.merge_key {
                        endpoints.push(DiscoveredEndpointRef {
                            url: key_url.clone(),
                            protocol: "sse".into(),
                        });

                        mcp_servers.push(DiscoveredMcpServerRef {
                            server_name: "sse_server".into(),
                            transport: "sse".into(),
                            command: None,
                        });
                        signals.has_mcp_servers = true;
                    }
                    signals.matched_port = Some(80);
                }
                EvidenceSource::IdeExtension => {
                    agent_type = InferredAgentType::IdeExtension;
                    name = "IDE Extension".into();
                    signals.matched_process_name = Some(name.clone());
                }
                _ => {}
            }
        }

        let computed_confidence = crate::fingerprint::compute_confidence(&signals);
        max_confidence = f64::max(max_confidence, computed_confidence);

        let mut control_bindings = Vec::new();
        let cand_id = format!("cand_{}", uuid::Uuid::new_v4());

        for server in &mcp_servers {
            let binding_id = format!("bind_{}", uuid::Uuid::new_v4());
            if server.transport == "stdio" {
                control_bindings.push(ControlBindingPlan {
                    binding_id,
                    kind: ControlBindingKind::McpStdioWrapper,
                    target_candidate_id: cand_id.clone(),
                    target_config_hash: None, // In real scenario, map from config evidence
                    action: ControlBindingAction::Wrap,
                    requires_user_approval: true,
                    risk: "medium".to_string(),
                    reversible: true,
                    backup_path_hash: None,
                    summary: format!("Wrap stdio MCP server: {}", server.server_name),
                });
            } else if server.transport == "http" || server.transport == "sse" {
                control_bindings.push(ControlBindingPlan {
                    binding_id,
                    kind: ControlBindingKind::McpHttpProxy,
                    target_candidate_id: cand_id.clone(),
                    target_config_hash: None,
                    action: ControlBindingAction::Proxy,
                    requires_user_approval: false,
                    risk: "low".to_string(),
                    reversible: true,
                    backup_path_hash: None,
                    summary: format!("Proxy HTTP/SSE MCP server: {}", server.server_name),
                });
            }
        }

        candidates.push(DiscoveredAgentCandidateV2 {
            schema_version: "pollen.agent_discovery_candidate.v2".into(),
            candidate_id: cand_id,
            tenant_id: tenant_id.to_string(),
            device_id: "device-local".into(),
            status: DiscoveryStatus::Discovered,
            display_name: name.clone(),
            vendor: None,
            product: None,
            inferred_agent_type: agent_type.clone(),
            confidence: max_confidence,
            risk_score,
            first_seen: chrono::Utc::now().to_rfc3339(),
            last_seen: chrono::Utc::now().to_rfc3339(),
            evidence: group,
            discovered_configs: vec![],
            discovered_endpoints: endpoints,
            discovered_mcp_servers: mcp_servers,
            suggested_registration: SuggestedAgentRegistration {
                agent_id: format!("agent_{}", uuid::Uuid::new_v4()),
                name: name.clone(),
                agent_type: format!("{:?}", agent_type),
                runtime_name: "native".into(),
                process_path_hash: process_hash,
                executable_signer: None,
                declared_tools: vec![],
                declared_resources: vec![],
                trust_level: "medium".into(),
                initial_status: "pending_approval".into(),
            },
            suggested_observation_profile: ObservationProfile {
                mode: ObservationMode::ObserveOnly,
                collect_process_metadata: true,
                collect_network_metadata: true,
                collect_mcp_tool_metadata: false,
                collect_token_usage: false,
                collect_file_metadata: false,
                collect_raw_prompt: false,
                collect_raw_response: false,
                retention_days: 30,
            },
            suggested_control_bindings: control_bindings,
            telemetry_plan: TelemetryPlan {
                events_endpoint: "/v1/telemetry/events".into(),
                metrics_endpoint: "/v1/metrics".into(),
                capture_tool_calls: true,
                capture_arguments: true,
                redact_env_keys: redacted_env_keys,
                risk_signals: vec!["mcp_active".into()],
            },
            labels: BTreeMap::new(),
        });
    }

    candidates
}
