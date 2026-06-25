use crate::model::*;
use std::collections::{BTreeMap, HashMap};

pub fn aggregate_evidence(
    tenant_id: &str,
    device_id: &str,
    evidence: Vec<DiscoveryEvidenceV2>,
) -> Vec<DiscoveredAgentCandidateV2> {
    let raw = aggregate_by_merge_key(tenant_id, device_id, evidence);
    coalesce_by_identity(tenant_id, raw)
}

fn coalesce_by_identity(
    tenant: &str,
    raw: Vec<DiscoveredAgentCandidateV2>,
) -> Vec<DiscoveredAgentCandidateV2> {
    use std::collections::HashMap;
    let mut by_key: HashMap<String, DiscoveredAgentCandidateV2> = HashMap::new();

    for mut c in raw {
        let key = crate::identity_key::identity_key(
            c.matched_signature_id.as_deref(),
            c.vendor.as_deref(),
            c.product.as_deref(),
            c.suggested_registration.process_path_hash.as_deref(),
            &c.display_name,
        );
        c.candidate_id = crate::identity_key::deterministic_candidate_id(tenant, &key);
        // Also update the target_candidate_id in suggested control bindings
        for cb in &mut c.suggested_control_bindings {
            cb.target_candidate_id = c.candidate_id.clone();
        }

        match by_key.get_mut(&key) {
            Some(existing) => {
                existing.evidence.extend(std::mem::take(&mut c.evidence));
                existing.confidence = existing.confidence.max(c.confidence);
                existing.risk_score = existing.risk_score.max(c.risk_score);
                existing.instance_count = existing.instance_count.saturating_add(1);

                for _cap in c
                    .suggested_registration
                    .declared_tools
                    .iter()
                    .chain(c.labels.keys())
                {
                    // Not strictly capabilities but labels could be merged.
                    // We'll merge labels.
                    for (k, v) in c.labels.iter() {
                        if !existing.labels.contains_key(k) {
                            existing.labels.insert(k.clone(), v.clone());
                        }
                    }
                }

                if is_better_name(&c.display_name, &existing.display_name) {
                    existing.display_name = c.display_name;
                    existing.vendor = c.vendor.or(existing.vendor.take());
                    existing.product = c.product.or(existing.product.take());
                    existing.inferred_agent_type = c.inferred_agent_type;
                }

                existing.first_seen = std::cmp::min(existing.first_seen.clone(), c.first_seen);
                existing.last_seen = std::cmp::max(existing.last_seen.clone(), c.last_seen);
            }
            None => {
                c.instance_count = 1;
                by_key.insert(key, c);
            }
        }
    }
    by_key.into_values().collect()
}

fn is_better_name(new: &str, old: &str) -> bool {
    let bad = |s: &str| {
        s == "Unknown Agent" || s.contains("unconfirmed") || basename_no_ext(s) == s && s.len() > 15
    };
    bad(old) && !bad(new)
}

fn aggregate_by_merge_key(
    tenant_id: &str,
    device_id: &str,
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
        let mut risk_score = 0;
        let mut agent_type = InferredAgentType::UnknownAiProcess;
        let mut name = "Unknown Agent".to_string();
        let mut vendor = None;
        let mut product = None;
        let mut matched_signature_id: Option<String> = None;
        let mut capability_tags = Vec::new();
        let mut matched_signals = Vec::new();
        let mut status = DiscoveryStatus::Discovered;

        let has_confirmed = group.iter().any(|e| {
            e.data
                .get("confirmed")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
        });
        if !has_confirmed {
            status = DiscoveryStatus::Unconfirmed;
        }

        let mut process_hash = None;
        let mut mcp_servers = Vec::new();
        let mut endpoints = Vec::new();
        let mut redacted_env_keys = Vec::new();

        let mut ctx = crate::identity::ResolutionContext::default();
        let mut best_hint = crate::identity_hint::IdentityHint::default();

        for ev in &group {
            if let Some(hint) = crate::identity_hint::extract_identity_hint(ev) {
                if hint.confidence >= best_hint.confidence {
                    best_hint = hint;
                }
            }

            if ev.confidence > max_confidence {
                max_confidence = ev.confidence;
            }

            match ev.source {
                EvidenceSource::ProcessScan => {
                    if let Some(r_name) = ev.data.get("resolved_name").and_then(|v| v.as_str()) {
                        name = r_name.to_string();
                    }
                    if let Some(r_vendor) = ev.data.get("vendor").and_then(|v| v.as_str()) {
                        vendor = Some(r_vendor.to_string());
                    }
                    if let Some(sig_id) =
                        ev.data.get("matched_signature_id").and_then(|v| v.as_str())
                    {
                        matched_signature_id = Some(sig_id.to_string());
                    }

                    let process_data = ev.data.get("process").unwrap_or(&ev.data);
                    if let Ok(p) = serde_json::from_value::<crate::process_scan::ProcessEvidence>(
                        process_data.clone(),
                    ) {
                        ctx.process_name = p.process_name.clone();
                        ctx.cmd_redacted = p.cmd_template.join(" ");
                        ctx.exe_path_norm = p.exe_path_redacted.clone();
                        ctx.binary_hash = p.exe_path_hash.clone();
                        process_hash = p.exe_path_hash.clone();

                        if let Some(exe) = &p.exe_path_redacted {
                            ctx.cli_on_path.push(basename_no_ext(exe));
                        }
                        if let Some(pkg) = npm_pkg_from_argv(&p.cmd_template) {
                            ctx.packages.push(("npm".into(), pkg.clone()));
                            ctx.cli_on_path.push(pkg);
                        }
                    }
                }
                EvidenceSource::McpConfig => {
                    if let Some(path) = &ev.source_path_redacted {
                        ctx.present_paths.push(path.clone());
                    }
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

                        if let Some(env_keys) =
                            ev.data.get("env_key_names").and_then(|v| v.as_array())
                        {
                            for key in env_keys {
                                if let Some(k) = key.as_str() {
                                    ctx.env_present.push(k.to_string());
                                    if !redacted_env_keys.contains(&k.to_string()) {
                                        redacted_env_keys.push(k.to_string());
                                    }
                                }
                            }
                        }
                    } else if let Some(data) = ev.data.get("servers") {
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
                            }
                        }
                    }
                }
                EvidenceSource::LocalModelServer => {
                    if let Some(key_url) = &ev.merge_key {
                        endpoints.push(DiscoveredEndpointRef {
                            url: key_url.clone(),
                            protocol: "http".into(),
                        });
                    }
                    if let Some(port) = ev.data.get("port").and_then(|v| v.as_u64()) {
                        ctx.listening_ports.push(port as u16);
                    } else {
                        ctx.listening_ports.push(80);
                    }

                    if let Some(models_val) = ev.data.get("models") {
                        if let Some(arr) = models_val.as_array() {
                            if let Some(clf_def) =
                                &dek_fingerprint_defs::load_latest_baseline().model_classifier
                            {
                                let clf =
                                    dek_fingerprint_defs::model_classifier::ModelClassifier::new(
                                        clf_def,
                                    );
                                for v in arr {
                                    if let Some(m_name) = v.as_str() {
                                        let mc = clf.classify(m_name);
                                        for cap in mc.capability_tags {
                                            if !capability_tags.contains(&cap) {
                                                capability_tags.push(cap);
                                            }
                                        }
                                        let r = (mc.risk_score * 100.0) as u32;
                                        if r > risk_score {
                                            risk_score = r;
                                        }
                                        if mc.needs_human {
                                            status = DiscoveryStatus::PendingApproval;
                                        }
                                    }
                                }
                            }
                            if !capability_tags.contains(&"model.server".to_string()) {
                                capability_tags.push("model.server".to_string());
                            }
                        }
                    }
                }
                EvidenceSource::PortProbe => {
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
                    }
                    if let Some(port) = ev.data.get("port").and_then(|v| v.as_u64()) {
                        ctx.listening_ports.push(port as u16);
                    } else {
                        ctx.listening_ports.push(80);
                    }
                }
                EvidenceSource::IdeExtension => {
                    // Not fully utilizing this signal yet in identity.rs
                }
                EvidenceSource::InstalledAppScan => {
                    if let Some(path) = ev.data.get("path").and_then(|v| v.as_str()) {
                        let sigs =
                            &dek_fingerprint_defs::load_latest_baseline().installed_app_signatures;
                        if let Some(am) = crate::identity::resolve_installed_app(path, sigs) {
                            name = am.display_name.clone();
                            vendor = am.vendor.clone();
                            product = am.product.clone();
                            agent_type = match am.agent_type.as_str() {
                                "desktop_agent" => InferredAgentType::DesktopAgent,
                                "ide_agent" => InferredAgentType::IdeAgent,
                                _ => InferredAgentType::DesktopAgent,
                            };
                            for cap in &am.capability_tags {
                                if !capability_tags.contains(cap) {
                                    capability_tags.push(cap.clone());
                                }
                            }
                        } else {
                            name = path.to_string();
                            agent_type = InferredAgentType::DesktopAgent;
                        }
                    }
                }
                EvidenceSource::BrowserSession
                | EvidenceSource::BrowserHistory
                | EvidenceSource::NetworkSni => {
                    if let Some(url) = ev
                        .data
                        .get("url")
                        .and_then(|v| v.as_str())
                        .or_else(|| ev.data.get("sni").and_then(|v| v.as_str()))
                    {
                        let sigs = &dek_fingerprint_defs::load_latest_baseline().web_ai_signatures;
                        let mut found = false;
                        for w_sig in sigs {
                            if url.contains(&w_sig.domain) {
                                name = w_sig.name.clone();
                                vendor = Some(w_sig.vendor.clone());
                                for cap in &w_sig.capability_tags {
                                    if !capability_tags.contains(cap) {
                                        capability_tags.push(cap.clone());
                                    }
                                }
                                agent_type = InferredAgentType::WebAIApp;
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            name = url.to_string();
                            agent_type = InferredAgentType::WebAIApp;
                        }
                    }
                }
                _ => {}
            }
        }

        let baseline = dek_fingerprint_defs::load_latest_baseline();
        let signatures = baseline.signatures.clone();
        let mut decision = crate::identity::resolve(&ctx, &signatures);

        if decision.best.is_none() {
            if let Some(ref exe) = ctx.exe_path_norm {
                if let Some(id) = crate::fingerprint::resolve_by_install_path(exe, &baseline) {
                    decision.best = Some(id);
                    decision.needs_human = false;
                }
            }
        }

        // New signature match logic using process_names, cmd_patterns, etc.
        if decision.best.is_none() {
            let facts = crate::signature_match::ProcessFacts {
                process_name: &ctx.process_name,
                exe_path: ctx.exe_path_norm.as_deref().unwrap_or(""),
                cmdline: &ctx.cmd_redacted,
                installed_paths: &ctx.present_paths,
            };
            if let Some(am) = crate::signature_match::match_process(
                &facts,
                &signatures,
                &baseline.installed_app_signatures,
            ) {
                decision.best = Some(crate::identity::AgentMatch {
                    signature_id: am.id,
                    display_name: am.display_name,
                    vendor: am.vendor,
                    product: None,
                    agent_type: am.agent_type,
                    confidence: am.confidence,
                    capability_tags: am.capability_tags,
                    matched_signals: vec![crate::identity::MatchedSignal {
                        kind: am.matched_by.to_string(),
                        detail: "signature_match".into(),
                        weight: am.confidence,
                    }],
                });
                decision.needs_human = false;
            }
        }

        // If unknown, run claw family heuristic
        if decision.best.is_none() {
            if let Some(claw_match) = crate::identity::claw_family_heuristic(&ctx) {
                decision.best = Some(claw_match);
                decision.needs_human = true;
            }
        }

        let resolved_by_signature = decision.best.is_some();

        if let Some(best) = decision.best {
            matched_signature_id = Some(best.signature_id.clone());
            name = best.display_name.clone();
            vendor = best.vendor.clone();
            product = best.product.clone();
            matched_signals = best
                .matched_signals
                .iter()
                .map(|s| MatchedSignal {
                    kind: s.kind.clone(),
                    detail: s.detail.clone(),
                    weight: s.weight,
                })
                .collect();
            // Map agent_type string to enum
            agent_type = match best.agent_type.as_str() {
                "desktop_agent" => InferredAgentType::DesktopAgent,
                "ide" | "ide_agent" => InferredAgentType::IdeAgent,
                "cli_agent" => InferredAgentType::CliAgent,
                "browser_agent" => InferredAgentType::BrowserAgent,
                "web_ai" | "web_agent" | "chat_ui" => InferredAgentType::WebAIApp,
                "local_model" | "local_model_server" => InferredAgentType::LocalModelServer,
                "automation_agent" => InferredAgentType::AutomationAgent,
                "mcp_server" => InferredAgentType::McpServer,
                "mcp_client" => InferredAgentType::McpClient,
                _ => InferredAgentType::UnknownAiProcess,
            };
            max_confidence = f64::max(max_confidence, best.confidence);
            for cap in best.capability_tags {
                if !capability_tags.contains(&cap) {
                    capability_tags.push(cap);
                }
            }
        }

        if name == "Unknown Agent" && !ctx.process_name.is_empty() {
            name = format!("Unknown Agent ({})", ctx.process_name);
        }

        if !resolved_by_signature || best_hint.confidence >= 1.0 {
            if let Some(n) = best_hint
                .name
                .filter(|n| !n.is_empty() && n != "Unknown Agent")
            {
                let n_lower = n.to_lowercase();
                if let Some(sig) = signatures.iter().find(|s| {
                    s.display_name.to_lowercase() == n_lower
                        || s.process_names.iter().any(|pn| {
                            let pn_lower = pn.to_lowercase();
                            let generic = [
                                "node",
                                "node.exe",
                                "python",
                                "python.exe",
                                "chrome",
                                "msedge",
                                "firefox",
                                "safari",
                                "brave",
                                "code",
                            ];
                            !generic.contains(&pn_lower.as_str()) && n_lower.contains(&pn_lower)
                        })
                        || s.id.to_lowercase().replace("_", " ") == n_lower
                }) {
                    name = sig.display_name.clone();
                    vendor = sig.vendor.clone();
                    product = sig.product.clone();
                    agent_type = match sig.agent_type.as_str() {
                        "desktop_agent" => InferredAgentType::DesktopAgent,
                        "ide_agent" => InferredAgentType::IdeAgent,
                        "cli_agent" => InferredAgentType::CliAgent,
                        "browser_agent" => InferredAgentType::BrowserAgent,
                        "mcp_server" => InferredAgentType::McpServer,
                        "mcp_client" => InferredAgentType::McpClient,
                        _ => InferredAgentType::AutomationAgent,
                    };
                    max_confidence = f64::max(max_confidence, f64::max(0.8, best_hint.confidence));
                    for cap in &sig.capability_tags {
                        if !capability_tags.contains(cap) {
                            capability_tags.push(cap.clone());
                        }
                    }
                    matched_signals.push(MatchedSignal {
                        kind: "identity_hint_signature".into(),
                        detail: sig.id.clone(),
                        weight: best_hint.confidence,
                    });
                    matched_signature_id = Some(sig.id.clone());
                } else {
                    name = n;
                    if vendor.is_none() {
                        vendor = best_hint.vendor;
                    }
                    if product.is_none() {
                        product = best_hint.product;
                    }
                    if let Some(t) = best_hint.agent_type {
                        agent_type = t;
                    }
                    max_confidence = f64::max(max_confidence, best_hint.confidence);
                    for cap in best_hint.capability_tags {
                        if !capability_tags.contains(&cap) {
                            capability_tags.push(cap);
                        }
                    }
                    matched_signals.push(MatchedSignal {
                        kind: "identity_hint".into(),
                        detail: "non-signature discovery hint".into(),
                        weight: best_hint.confidence,
                    });
                }
            }
        }

        let mut computed_risk = 0;
        for cap in &capability_tags {
            let cap_score = match cap.as_str() {
                "browser.control" => 60,
                "automation" => 40,
                "fs.write" => 50,
                "code.exec" => 80,
                "llm.call" => 20,
                "web.search" => 10,
                "net.egress" => 30,
                "net.egress.llm" => 30,
                "tool.use" => 30,
                "model.server" => 40,
                "code.agentic" => 70,
                "web.chat" => 20,
                _ => 10,
            };
            computed_risk += cap_score;
        }
        if computed_risk > risk_score {
            risk_score = std::cmp::min(100, computed_risk);
        }

        if name == "Unknown Agent" {
            status = DiscoveryStatus::PendingApproval;
        }

        if decision.needs_human {
            status = DiscoveryStatus::PendingApproval;
        }

        let mut control_bindings = Vec::new();
        let cand_id = String::new();

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

        let preset_id =
            dek_policy_presets::catalog::preset_for_capabilities(&capability_tags, max_confidence);
        let mut labels = BTreeMap::new();
        for tag in &capability_tags {
            labels.insert(format!("capability:{}", tag), "true".into());
        }
        labels.insert("suggested_preset".into(), preset_id.to_string());

        capability_tags.sort();
        capability_tags.dedup();
        matched_signals.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut mcp_stdio_paths = vec![];
        let mut mcp_http_urls = vec![];
        let mut local_model_urls = vec![];
        let mut browser_ext_evidence = vec![];

        for ev in &group {
            match ev.source {
                EvidenceSource::McpConfig => {
                    if let Some(path) = ev.source_path_redacted.clone() {
                        mcp_stdio_paths.push(path);
                    }
                }
                EvidenceSource::LocalModelServer => {
                    if let Some(obj) = ev.data.as_object() {
                        if let Some(url) = obj.get("endpoint").and_then(|v| v.as_str()) {
                            local_model_urls.push(url.to_string());
                        }
                    }
                }
                EvidenceSource::BrowserExtension => {
                    if let Some(obj) = ev.data.as_object() {
                        if let Some(ext_id) = obj.get("extension_id").and_then(|v| v.as_str()) {
                            browser_ext_evidence.push(ext_id.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        for server in &mcp_servers {
            if server.transport == "http" {
                mcp_http_urls.push(server.server_name.clone());
            }
        }

        candidates.push(DiscoveredAgentCandidateV2 {
            schema_version: "pollen.agent_discovery_candidate.v2".into(),
            candidate_id: cand_id,
            tenant_id: tenant_id.to_string(),
            device_id: device_id.to_string(),
            status,
            instance_count: 1,
            matched_signature_id,
            display_name: name.clone(),
            vendor,
            product,
            inferred_agent_type: agent_type.clone(),
            confidence: max_confidence,
            risk_score,
            capability_tags: capability_tags.clone(),
            matched_signals,
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
                mcp_stdio_config_paths: mcp_stdio_paths,
                mcp_http_urls,
                local_model_endpoints: local_model_urls,
                browser_extension_evidence: browser_ext_evidence,
                trust_level: "Unknown".into(),
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
            labels,
        });
    }

    let mut final_candidates: std::collections::HashMap<String, DiscoveredAgentCandidateV2> =
        std::collections::HashMap::new();
    for cand in candidates {
        let key = cand
            .matched_signature_id
            .clone()
            .unwrap_or_else(|| cand.candidate_id.clone());
        if let Some(existing) = final_candidates.get_mut(&key) {
            existing.evidence.extend(cand.evidence.clone());
            existing.confidence = f64::max(existing.confidence, cand.confidence);
            existing.labels.extend(cand.labels.clone());
            existing
                .discovered_endpoints
                .extend(cand.discovered_endpoints.clone());
            existing
                .discovered_mcp_servers
                .extend(cand.discovered_mcp_servers.clone());
            for cap in cand.capability_tags {
                if !existing.capability_tags.contains(&cap) {
                    existing.capability_tags.push(cap);
                }
            }
            existing.capability_tags.sort();
            existing.capability_tags.dedup();
            existing
                .matched_signals
                .extend(cand.matched_signals.clone());
            existing.matched_signals.sort_by(|a, b| {
                b.weight
                    .partial_cmp(&a.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            existing.matched_signals.dedup_by(|a, b| {
                a.kind == b.kind
                    && a.detail == b.detail
                    && (a.weight - b.weight).abs() < f64::EPSILON
            });
            existing
                .suggested_registration
                .mcp_stdio_config_paths
                .extend(cand.suggested_registration.mcp_stdio_config_paths.clone());
            existing
                .suggested_registration
                .mcp_http_urls
                .extend(cand.suggested_registration.mcp_http_urls.clone());
            existing
                .suggested_registration
                .local_model_endpoints
                .extend(cand.suggested_registration.local_model_endpoints.clone());
            existing
                .suggested_registration
                .browser_extension_evidence
                .extend(
                    cand.suggested_registration
                        .browser_extension_evidence
                        .clone(),
                );
            if existing.status == crate::model::DiscoveryStatus::Unconfirmed
                && cand.status == crate::model::DiscoveryStatus::Discovered
            {
                existing.status = crate::model::DiscoveryStatus::Discovered;
            }
        } else {
            final_candidates.insert(key, cand);
        }
    }

    final_candidates.into_values().collect()
}

fn npm_pkg_from_argv(argv: &[String]) -> Option<String> {
    argv.iter().find_map(|a| {
        let a = a.replace('\\', "/");
        a.split("node_modules/")
            .nth(1)
            .map(|rest| rest.split('/').next().unwrap_or("").to_string())
            .filter(|p| !p.is_empty())
    })
}

fn basename_no_ext(p: &str) -> String {
    std::path::Path::new(p)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}
