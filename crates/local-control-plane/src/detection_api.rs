use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use dek_capability::{derive_requirements, AchievedLevel, HostFacts, Os, Sensor};
use dek_detection::{
    build_coverage, evaluate, verify_and_load_pack, Detection, ObservedEvent, PackManifest,
    RuleSpec,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path as FsPath, PathBuf};

use crate::{enforcement_plan_api::HostCapabilities, error::ApiResult, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/detections/coverage", get(get_coverage))
        .route("/v1/tenants/:tenant/detections/rules", get(list_rules))
        .route(
            "/v1/tenants/:tenant/detections/evaluate",
            post(evaluate_events),
        )
        .route("/v1/tenants/:tenant/detections/sensors", get(list_sensors))
        .route(
            "/v1/tenants/:tenant/detections/sensors/:sensor_id/preflight",
            post(preflight_sensor),
        )
        .route(
            "/v1/tenants/:tenant/detections/sensors/:sensor_id/consent",
            post(record_sensor_consent),
        )
        .route(
            "/v1/tenants/:tenant/detections/sensors/:sensor_id/install",
            post(request_sensor_install),
        )
}

#[derive(Debug, Serialize)]
struct DetectionRuleSummary {
    id: String,
    name: String,
    severity: String,
    confidence: String,
    maturity: String,
    detect_type: String,
    default_response: String,
    enforce_if_capable: Option<String>,
    observe_only_fallback: bool,
    user_message: String,
    maps: Value,
    setup_requirements: Vec<String>,
    can_stop_next_time: bool,
    privacy_note: String,
}

#[derive(Debug, Serialize)]
struct DetectionCoverageResponse {
    schema_version: &'static str,
    tenant_id: String,
    generated_at: String,
    pack_id: String,
    pack_version: String,
    manifest_integrity: &'static str,
    rule_count: usize,
    coverage: dek_detection::Coverage,
    rules: Vec<DetectionRuleSummary>,
    sensors: Vec<ObserveSensor>,
    research_basis: Vec<ResearchBasis>,
    privacy_guards: Vec<String>,
    limitations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResearchBasis {
    framework: &'static str,
    source: &'static str,
    implementation_use: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct ObserveSensor {
    id: String,
    title: String,
    os: Vec<String>,
    domains: Vec<String>,
    layer: String,
    status: String,
    achieved_level: String,
    achievable_level: String,
    deterministic_decision: String,
    evidence_sources: Vec<String>,
    missing_requirements: Vec<Value>,
    remediation: Vec<Value>,
    can_observe: bool,
    can_enforce: bool,
    requires_admin: bool,
    user_consent_required: bool,
    setup_action: String,
    reason: String,
    fallback: String,
    package_path: Option<String>,
    setup_state: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct EvaluateRequest {
    events: Vec<ObservedEvent>,
}

#[derive(Debug, Serialize)]
struct DetectionHit {
    rule: DetectionRuleSummary,
    matched_event_ids: Vec<String>,
    agent_id: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct EvaluateResponse {
    schema_version: &'static str,
    tenant_id: String,
    evaluated_events: usize,
    fired: Vec<DetectionHit>,
}

#[derive(Debug, Deserialize)]
struct SensorConsentRequest {
    accepted: bool,
    #[serde(default)]
    scopes: Vec<String>,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SensorInstallRequest {
    accepted: bool,
    #[serde(default)]
    requested_level: Option<String>,
}

async fn get_coverage(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
) -> ApiResult<Json<DetectionCoverageResponse>> {
    let rules = verified_rules()?;
    let coverage = build_coverage(&rules);
    let manifest = load_manifest()?;
    let sensors = observe_sensors(&state, &tenant).await;

    Ok(Json(DetectionCoverageResponse {
        schema_version: "pollek.detection.coverage.v1",
        tenant_id: tenant,
        generated_at: Utc::now().to_rfc3339(),
        pack_id: manifest.pack_id,
        pack_version: manifest.version,
        manifest_integrity: "verified",
        rule_count: rules.len(),
        coverage,
        rules: rules.iter().map(rule_summary).collect(),
        sensors,
        research_basis: research_basis(),
        privacy_guards: privacy_guards(),
        limitations: detection_limitations(),
    }))
}

async fn list_rules(Path(_tenant): Path<String>) -> ApiResult<Json<Value>> {
    let rules = verified_rules()?;
    Ok(Json(json!({
        "schema_version": "pollek.detection.rules.v1",
        "pack": load_manifest()?,
        "items": rules.iter().map(rule_summary).collect::<Vec<_>>()
    })))
}

async fn evaluate_events(
    Path(tenant): Path<String>,
    Json(req): Json<EvaluateRequest>,
) -> ApiResult<Json<EvaluateResponse>> {
    let rules = verified_rules()?;
    let mut groups: BTreeMap<(String, String), Vec<ObservedEvent>> = BTreeMap::new();
    let evaluated_events = req.events.len();

    for event in req.events {
        groups
            .entry((event.agent_id.clone(), event.session_id.clone()))
            .or_default()
            .push(event);
    }

    let mut fired = Vec::new();
    for ((agent_id, session_id), mut events) in groups {
        events.sort_by_key(|event| event.ts_ms);
        for rule in &rules {
            if let Some(hit) = evaluate(rule, &events) {
                fired.push(hit_response(rule, hit, &agent_id, &session_id));
            }
        }
    }

    Ok(Json(EvaluateResponse {
        schema_version: "pollek.detection.evaluate.v1",
        tenant_id: tenant,
        evaluated_events,
        fired,
    }))
}

async fn list_sensors(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
) -> ApiResult<Json<Value>> {
    let sensors = observe_sensors(&state, &tenant).await;
    Ok(Json(json!({
        "schema_version": "pollek.observe.sensors.v1",
        "tenant_id": tenant,
        "generated_at": Utc::now().to_rfc3339(),
        "items": sensors
    })))
}

async fn preflight_sensor(
    State(state): State<AppState>,
    Path((tenant, sensor_id)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let sensor = observe_sensor(&state, &tenant, &sensor_id).await;
    Ok(Json(json!({
        "schema_version": "pollek.observe.sensor.preflight.v1",
        "tenant_id": tenant,
        "sensor": sensor,
        "checked_at": Utc::now().to_rfc3339()
    })))
}

async fn record_sensor_consent(
    State(state): State<AppState>,
    Path((tenant, sensor_id)): Path<(String, String)>,
    Json(req): Json<SensorConsentRequest>,
) -> ApiResult<Json<Value>> {
    let record = json!({
        "schema_version": "pollek.observe.sensor.consent.v1",
        "tenant_id": tenant,
        "sensor_id": sensor_id,
        "accepted": req.accepted,
        "scopes": req.scopes,
        "note": req.note,
        "raw_content_stored": false,
        "accepted_at": if req.accepted { Some(Utc::now().to_rfc3339()) } else { None },
        "updated_at": Utc::now().to_rfc3339()
    });
    state
        .registry_store
        .upsert_raw(&tenant, "observe_sensor_consent", &sensor_id, &record)
        .await
        .map_err(crate::error::ApiError::Internal)?;

    Ok(Json(json!({
        "status": if req.accepted { "accepted" } else { "declined" },
        "record": record
    })))
}

async fn request_sensor_install(
    State(state): State<AppState>,
    Path((tenant, sensor_id)): Path<(String, String)>,
    Json(req): Json<SensorInstallRequest>,
) -> ApiResult<Json<Value>> {
    if !req.accepted {
        return Err(crate::error::ApiError::BadRequest(
            "User consent is required before starting sensor setup".into(),
        ));
    }

    let sensor = observe_sensor(&state, &tenant, &sensor_id).await;
    let install_status = install_status_for_sensor(&sensor);
    let record = json!({
        "schema_version": "pollek.observe.sensor.setup.v1",
        "tenant_id": tenant,
        "sensor_id": sensor_id,
        "requested_level": req.requested_level.unwrap_or_else(|| "observe".to_string()),
        "status": install_status,
        "can_observe": sensor.can_observe,
        "can_enforce": sensor.can_enforce,
        "requires_admin": sensor.requires_admin,
        "raw_content_stored": false,
        "setup_action": sensor.setup_action,
        "fallback": sensor.fallback,
        "requested_at": Utc::now().to_rfc3339(),
        "updated_at": Utc::now().to_rfc3339()
    });
    state
        .registry_store
        .upsert_raw(&tenant, "observe_sensor_setup", &sensor_id, &record)
        .await
        .map_err(crate::error::ApiError::Internal)?;

    Ok(Json(json!({
        "status": install_status,
        "record": record,
        "sensor": sensor
    })))
}

fn hit_response(
    rule: &RuleSpec,
    detection: Detection,
    agent_id: &str,
    session_id: &str,
) -> DetectionHit {
    DetectionHit {
        rule: rule_summary(rule),
        matched_event_ids: detection.matched_event_ids,
        agent_id: agent_id.to_string(),
        session_id: session_id.to_string(),
    }
}

fn verified_rules() -> ApiResult<Vec<RuleSpec>> {
    let pack_dir = detection_pack_dir();
    verify_and_load_pack(pack_dir, |_manifest, _dir| Ok(()))
        .map_err(|err| crate::error::ApiError::Internal(anyhow::anyhow!(err)))
}

fn load_manifest() -> ApiResult<PackManifest> {
    let path = detection_pack_dir().join("manifest.json");
    let text = std::fs::read_to_string(&path).map_err(|err| {
        crate::error::ApiError::Internal(anyhow::anyhow!(
            "failed to read detection manifest {}: {err}",
            path.display()
        ))
    })?;
    serde_json::from_str(&text).map_err(|err| {
        crate::error::ApiError::Internal(anyhow::anyhow!(
            "failed to parse detection manifest {}: {err}",
            path.display()
        ))
    })
}

fn detection_pack_dir() -> PathBuf {
    if let Ok(path) = std::env::var("POLLEK_DETECTION_PACK_DIR") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("contracts/detections/packs/core-v1"));
        candidates.push(cwd.join("../contracts/detections/packs/core-v1"));
    }
    candidates.push(
        FsPath::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/detections/packs/core-v1"),
    );

    candidates
        .iter()
        .find(|path| path.join("manifest.json").exists())
        .cloned()
        .unwrap_or_else(|| {
            FsPath::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/detections/packs/core-v1")
        })
}

fn rule_summary(rule: &RuleSpec) -> DetectionRuleSummary {
    let enforce = rule.response.enforce_if_capable;
    DetectionRuleSummary {
        id: rule.id.clone(),
        name: rule.name.clone(),
        severity: enum_name(&rule.severity),
        confidence: enum_name(&rule.confidence),
        maturity: enum_name(&rule.maturity),
        detect_type: enum_name(&rule.detect.detect_type),
        default_response: enum_name(&rule.response.default),
        enforce_if_capable: enforce.map(|action| enum_name(&action)),
        observe_only_fallback: rule.response.observe_only_fallback,
        user_message: rule.response.user_message.clone(),
        maps: serde_json::to_value(&rule.maps).unwrap_or_else(|_| json!({})),
        setup_requirements: setup_requirements_for_rule(rule),
        can_stop_next_time: enforce.is_some(),
        privacy_note: "Detection uses redacted metadata and rule IDs. It does not store raw prompt, response, email body, or file content.".into(),
    }
}

fn enum_name<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}").to_ascii_lowercase()
}

fn setup_requirements_for_rule(rule: &RuleSpec) -> Vec<String> {
    let mut out = Vec::new();
    for step in &rule.detect.steps {
        if let Some(activity) = &step.activity {
            match activity.as_str() {
                "FileRead" | "PackageInstall" => out.push(
                    "File/process visibility needs local OS metadata, MCP wrapper, SDK wrapper, or structured agent logs.".into(),
                ),
                "WebUpload" | "WebVisit" => out.push(
                    "Browser or network visibility needs browser connector, HTTP/MCP proxy, WFP, Network Extension, or eBPF.".into(),
                ),
                "ShellCommand" => out.push(
                    "Command execution visibility needs terminal wrapper, MCP tool proxy, process audit, or agent SDK hook.".into(),
                ),
                "LlmApiCall" => out.push(
                    "LLM usage visibility needs provider usage object, SDK wrapper, browser connector, MCP/HTTP proxy, or local log source.".into(),
                ),
                _ => {}
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

async fn observe_sensors(state: &AppState, tenant: &str) -> Vec<ObserveSensor> {
    let ids = [
        "mcp_proxy",
        "http_gateway",
        "browser_ai_extension",
        "windows_wfp_driver",
        "linux_ebpf",
        "linux_fanotify",
        "macos_endpoint_security",
        "macos_network_extension",
    ];
    let mut sensors = Vec::new();
    for id in ids {
        sensors.push(observe_sensor(state, tenant, id).await);
    }
    sensors
}

async fn observe_sensor(state: &AppState, tenant: &str, sensor_id: &str) -> ObserveSensor {
    let setup_state = state
        .registry_store
        .get_raw(tenant, "observe_sensor_setup", sensor_id)
        .await
        .ok()
        .flatten();
    let consent_state = state
        .registry_store
        .get_raw(tenant, "observe_sensor_consent", sensor_id)
        .await
        .ok()
        .flatten();
    let persisted_state = setup_state.or(consent_state);
    let host = crate::enforcement_plan_api::detect_host();
    let os = host.os.as_str();
    let extension_package = browser_extension_package_path();
    let capability = capability_matrix(sensor_id, &host);

    let mut sensor = match sensor_id {
        "mcp_proxy" => ObserveSensor {
            id: sensor_id.into(),
            title: "MCP proxy and tool wrapper".into(),
            os: all_os(),
            domains: vec!["tools".into(), "files".into(), "commands".into()],
            layer: "application".into(),
            status: "ready".into(),
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: true,
            can_enforce: true,
            requires_admin: false,
            user_consent_required: true,
            setup_action: "Route AI tools through the Pollek MCP proxy or wrapper.".into(),
            reason: "MCP traffic is plaintext at the tool boundary, so Pollek can observe and block before the tool runs.".into(),
            fallback: "If the agent cannot use MCP, keep OS/process observation and configure the AI app's own permissions.".into(),
            package_path: None,
            setup_state: persisted_state,
        },
        "http_gateway" => ObserveSensor {
            id: sensor_id.into(),
            title: "HTTP or SDK gateway".into(),
            os: all_os(),
            domains: vec!["web".into(), "llm_api".into(), "cost".into()],
            layer: "application".into(),
            status: "ready".into(),
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: true,
            can_enforce: true,
            requires_admin: false,
            user_consent_required: true,
            setup_action: "Point supported SDKs or local agents at the Pollek gateway endpoint.".into(),
            reason: "Gateway integration can see request metadata, provider usage fields, and policy decisions before egress.".into(),
            fallback: "For agents that cannot use a gateway, use browser extension, MCP proxy, or OS network metadata.".into(),
            package_path: None,
            setup_state: persisted_state,
        },
        "browser_ai_extension" => ObserveSensor {
            id: sensor_id.into(),
            title: "Browser AI connector".into(),
            os: all_os(),
            domains: vec!["web".into(), "prompts".into(), "uploads".into(), "safety".into()],
            layer: "browser".into(),
            status: if extension_package.is_some() {
                "package_available_user_install_required".into()
            } else {
                "source_available_build_required".into()
            },
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: true,
            can_enforce: true,
            requires_admin: false,
            user_consent_required: true,
            setup_action: "Build or install the browser connector, then approve it in Chrome, Edge, or Safari.".into(),
            reason: "Browsers do not permit silent local extension install. User approval or enterprise browser policy is required.".into(),
            fallback: "Without the extension, Pollek can still observe browser windows, domains, and process metadata, but not exact prompt/session fields.".into(),
            package_path: extension_package,
            setup_state: persisted_state,
        },
        "windows_wfp_driver" => ObserveSensor {
            id: sensor_id.into(),
            title: "Windows WFP network driver".into(),
            os: vec!["windows".into()],
            domains: vec!["network".into(), "dns".into(), "egress".into()],
            layer: "kernel".into(),
            status: native_status(os == "windows", host.windows_wfp, "signed_driver_required"),
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: os == "windows" && host.windows_wfp,
            can_enforce: os == "windows" && host.windows_wfp,
            requires_admin: true,
            user_consent_required: true,
            setup_action: "Install the signed Pollek WFP service/driver and approve the Windows administrator prompt.".into(),
            reason: "Windows network blocking requires a running WFP callout/service plus OS approval.".into(),
            fallback: "Use MCP/HTTP gateway enforcement or observe-only network metadata until WFP is active.".into(),
            package_path: Some("crates/dek-windows-wfp/driver".into()),
            setup_state: persisted_state,
        },
        "linux_ebpf" => ObserveSensor {
            id: sensor_id.into(),
            title: "Linux eBPF network sensor".into(),
            os: vec!["linux".into()],
            domains: vec!["network".into(), "dns".into(), "egress".into()],
            layer: "kernel".into(),
            status: native_status(os == "linux", host.linux_ebpf, "root_or_cap_bpf_required"),
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: os == "linux" && host.linux_ebpf,
            can_enforce: os == "linux" && host.linux_ebpf,
            requires_admin: true,
            user_consent_required: true,
            setup_action: "Run the eBPF observer with CAP_BPF/CAP_NET_ADMIN and a kernel with BTF support.".into(),
            reason: "eBPF can observe or enforce network metadata only when kernel support and privileges are present.".into(),
            fallback: "Use gateway or MCP enforcement until eBPF warm checks pass.".into(),
            package_path: Some("crates/dek-ebpfd".into()),
            setup_state: persisted_state,
        },
        "linux_fanotify" => ObserveSensor {
            id: sensor_id.into(),
            title: "Linux fanotify file permission sensor".into(),
            os: vec!["linux".into()],
            domains: vec!["files".into(), "folders".into()],
            layer: "kernel".into(),
            status: if os == "linux" {
                "needs_root_or_capability".into()
            } else {
                "not_available_on_this_os".into()
            },
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: os == "linux",
            can_enforce: false,
            requires_admin: true,
            user_consent_required: true,
            setup_action: "Grant fanotify permissions and mount coverage for paths the user wants Pollek to watch.".into(),
            reason: "fanotify can provide file event metadata on Linux, but active permission enforcement depends on privileges and mount scope.".into(),
            fallback: "Use MCP/SDK wrappers or structured agent logs for exact file activity if fanotify is not available.".into(),
            package_path: None,
            setup_state: persisted_state,
        },
        "macos_endpoint_security" => ObserveSensor {
            id: sensor_id.into(),
            title: "macOS Endpoint Security sensor".into(),
            os: vec!["macos".into()],
            domains: vec!["process".into(), "files".into(), "commands".into()],
            layer: "system_extension".into(),
            status: if os == "macos" {
                "requires_apple_entitlement_and_user_approval".into()
            } else {
                "not_available_on_this_os".into()
            },
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: os == "macos",
            can_enforce: false,
            requires_admin: true,
            user_consent_required: true,
            setup_action: "Install an Endpoint Security system extension and approve it in macOS Privacy & Security.".into(),
            reason: "macOS Endpoint Security requires Apple entitlement and explicit user approval.".into(),
            fallback: "Use browser/MCP/SDK observation until an approved system extension is installed.".into(),
            package_path: None,
            setup_state: persisted_state,
        },
        "macos_network_extension" => ObserveSensor {
            id: sensor_id.into(),
            title: "macOS Network Extension".into(),
            os: vec!["macos".into()],
            domains: vec!["network".into(), "dns".into(), "egress".into()],
            layer: "system_extension".into(),
            status: native_status(os == "macos", host.macos_nefilter, "network_extension_approval_required"),
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: os == "macos" && host.macos_nefilter,
            can_enforce: os == "macos" && host.macos_nefilter,
            requires_admin: true,
            user_consent_required: true,
            setup_action: "Install and approve the Pollek Network Extension system extension.".into(),
            reason: "macOS network filtering requires a signed Network Extension approved by the user or MDM.".into(),
            fallback: "Use browser/MCP/HTTP gateway observation until Network Extension warm checks pass.".into(),
            package_path: None,
            setup_state: persisted_state,
        },
        _ => ObserveSensor {
            id: sensor_id.into(),
            title: "Unknown observe sensor".into(),
            os: all_os(),
            domains: vec![],
            layer: "unknown".into(),
            status: "unknown".into(),
            achieved_level: String::new(),
            achievable_level: String::new(),
            deterministic_decision: String::new(),
            evidence_sources: vec![],
            missing_requirements: vec![],
            remediation: vec![],
            can_observe: false,
            can_enforce: false,
            requires_admin: false,
            user_consent_required: true,
            setup_action: "No setup action is registered for this sensor.".into(),
            reason: "The requested sensor id is not in the local capability catalog.".into(),
            fallback: "Use supported MCP, HTTP, browser, or OS sensors.".into(),
            package_path: None,
            setup_state: persisted_state,
        },
    };

    if let Some(capability) = capability {
        sensor.achievable_level = level_name(capability.achievable);
        sensor.achieved_level = level_name(achieved_level_for_setup(
            &sensor,
            capability.achieved_after_consent,
        ));
        sensor.missing_requirements = capability.missing;
        sensor.remediation = capability.remediation;
        sensor.deterministic_decision = deterministic_decision_text(&sensor);
        sensor.evidence_sources = evidence_sources_for_sensor(&sensor);
    }

    sensor
}

struct CapabilityMatrix {
    achievable: AchievedLevel,
    achieved_after_consent: AchievedLevel,
    missing: Vec<Value>,
    remediation: Vec<Value>,
}

fn capability_matrix(sensor_id: &str, host: &HostCapabilities) -> Option<CapabilityMatrix> {
    let sensor = capability_sensor(sensor_id)?;
    let facts = host_facts(host);
    let report = derive_requirements(sensor, &facts);
    let achievable = report.achievable_level();
    let missing: Vec<Value> = report
        .missing()
        .into_iter()
        .filter_map(|requirement| serde_json::to_value(requirement).ok())
        .collect();
    let remediation: Vec<Value> = report
        .missing()
        .into_iter()
        .filter_map(|requirement| requirement.remediation.as_ref())
        .filter_map(|remediation| serde_json::to_value(remediation).ok())
        .collect();
    let achieved_after_consent = if report.observe_supported {
        achievable
    } else {
        AchievedLevel::None
    };
    Some(CapabilityMatrix {
        achievable,
        achieved_after_consent,
        missing,
        remediation,
    })
}

fn capability_sensor(sensor_id: &str) -> Option<Sensor> {
    match sensor_id {
        "mcp_proxy" => Some(Sensor::McpProxy),
        "http_gateway" => Some(Sensor::Content),
        "browser_ai_extension" => Some(Sensor::Browser),
        "windows_wfp_driver" | "linux_ebpf" | "macos_network_extension" => Some(Sensor::Network),
        "linux_fanotify" | "macos_endpoint_security" => Some(Sensor::File),
        _ => None,
    }
}

fn host_facts(host: &HostCapabilities) -> HostFacts {
    HostFacts {
        os: os_for_name(&host.os),
        is_admin_or_root: host.windows_wfp || host.linux_ebpf || host.macos_nefilter,
        win_driver_signed: host.windows_wfp,
        win_test_signing: false,
        mac_es_entitlement_present: false,
        mac_system_extension_approved: host.macos_nefilter,
        mac_full_disk_access: host.macos_nefilter,
        mac_notarized: host.macos_nefilter,
        linux_kernel_supports_ebpf: host.linux_ebpf,
        linux_kernel_supports_fanotify: host.os == "linux",
        linux_has_cap_sys_admin: host.linux_ebpf,
    }
}

fn os_for_name(value: &str) -> Option<Os> {
    match value {
        "windows" => Some(Os::Windows),
        "macos" => Some(Os::Macos),
        "linux" => Some(Os::Linux),
        _ => None,
    }
}

fn level_name(level: AchievedLevel) -> String {
    match level {
        AchievedLevel::None => "none",
        AchievedLevel::ObserveOnly => "observe_only",
        AchievedLevel::Enforce => "enforce",
    }
    .into()
}

fn achieved_level_for_setup(sensor: &ObserveSensor, level: AchievedLevel) -> AchievedLevel {
    let Some(status) = sensor
        .setup_state
        .as_ref()
        .and_then(|state| state.get("status"))
        .and_then(Value::as_str)
    else {
        return AchievedLevel::None;
    };

    match status {
        "ready" | "observe_ready" => level,
        _ => AchievedLevel::None,
    }
}

fn deterministic_decision_text(sensor: &ObserveSensor) -> String {
    let setup_waiting = sensor
        .setup_state
        .as_ref()
        .and_then(|state| state.get("status"))
        .and_then(Value::as_str)
        .map(|status| status.starts_with("waiting_"))
        .unwrap_or(false);
    let setup_required = sensor.status.contains("required")
        || sensor.status.contains("approval")
        || sensor.status.contains("package_available")
        || sensor.status.contains("source_available");

    if (sensor.can_enforce || sensor.achieved_level == "enforce")
        && !setup_waiting
        && !setup_required
    {
        return "This source can contribute enforce decisions. Other evidence sources still cross-check activity so policy does not depend on this source alone.".into();
    }
    if sensor.can_observe
        || sensor.achievable_level == "observe_only"
        || sensor.achievable_level == "enforce"
    {
        return "This source contributes deterministic observe evidence. If it fails, Pollek keeps deciding from the remaining evidence matrix and lowers confidence/control level instead of blocking the whole flow.".into();
    }
    "This source is unavailable on this host. Pollek excludes it from the current decision matrix and uses MCP, gateway, browser, process, local log, or registry evidence when available.".into()
}

fn evidence_sources_for_sensor(sensor: &ObserveSensor) -> Vec<String> {
    let mut sources = vec![sensor.layer.clone()];
    sources.extend(
        sensor
            .domains
            .iter()
            .map(|domain| format!("domain:{domain}")),
    );
    if sensor.can_enforce {
        sources.push("enforce-capable".into());
    } else if sensor.can_observe {
        sources.push("observe-capable".into());
    } else {
        sources.push("unavailable-excluded".into());
    }
    sources
}

fn native_status(current_os: bool, ready: bool, missing_reason: &str) -> String {
    if ready {
        "ready".into()
    } else if current_os {
        missing_reason.into()
    } else {
        "not_available_on_this_os".into()
    }
}

fn all_os() -> Vec<String> {
    vec!["windows".into(), "macos".into(), "linux".into()]
}

fn install_status_for_sensor(sensor: &ObserveSensor) -> &'static str {
    if sensor.layer == "browser" && sensor.status != "ready" {
        return "waiting_for_browser_user_approval";
    }
    if sensor.can_enforce {
        "ready"
    } else if sensor.layer == "kernel" || sensor.layer == "system_extension" {
        "waiting_for_os_privilege_or_signed_component"
    } else if sensor.can_observe {
        "observe_ready"
    } else {
        "observe_only_fallback"
    }
}

fn browser_extension_package_path() -> Option<String> {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(
            cwd.join(
                "apps/prompt-guard-browser-extension/packages/pollek-prompt-guard-chromium.zip",
            ),
        );
        candidates.push(cwd.join("apps/prompt-guard-browser-extension"));
    }
    candidates.push(FsPath::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../apps/prompt-guard-browser-extension/packages/pollek-prompt-guard-chromium.zip",
    ));
    candidates.push(
        FsPath::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/prompt-guard-browser-extension"),
    );

    candidates
        .iter()
        .find(|path| path.exists())
        .map(|path| path.display().to_string())
}

fn research_basis() -> Vec<ResearchBasis> {
    vec![
        ResearchBasis {
            framework: "OWASP Top 10 for LLM Applications",
            source: "https://genai.owasp.org/llm-top-10/",
            implementation_use: "Rule mappings for prompt injection, sensitive disclosure, supply chain, excessive agency, and unbounded consumption.",
        },
        ResearchBasis {
            framework: "NIST AI RMF / Generative AI Profile",
            source: "https://doi.org/10.6028/NIST.AI.600-1",
            implementation_use: "Risk mapping, measurement, governance traceability, and user disclosure for AI activity monitoring.",
        },
        ResearchBasis {
            framework: "NIST SSDF",
            source: "https://csrc.nist.gov/Projects/ssdf",
            implementation_use: "Signed content, manifest integrity, and repeatable CI checks for detection rule packs.",
        },
        ResearchBasis {
            framework: "EDR-style local sensors",
            source: "OS vendor APIs: WFP, Endpoint Security, Network Extension, eBPF, fanotify, and browser extensions.",
            implementation_use: "Capability probes and setup gates keep OS-level enforcement honest and observable-first.",
        },
    ]
}

fn privacy_guards() -> Vec<String> {
    vec![
        "No raw prompt, response, email body, or file content is stored by detection rules.".into(),
        "Rules operate on redacted metadata, classifications, hashes, timestamps, and provenance tags.".into(),
        "Browser prompt checking requires explicit extension approval; exact body capture is not enabled silently.".into(),
        "Enterprise third-party NER remains an explicit future routing point with provider consent and audit metadata.".into(),
    ]
}

fn detection_limitations() -> Vec<String> {
    vec![
        "Kernel-level enforcement depends on OS support, signed components, user or admin approval, and warm checks.".into(),
        "Encrypted HTTPS metadata alone cannot reveal prompt or response bodies; use browser extension, SDK wrapper, MCP proxy, or gateway for plaintext guard paths.".into(),
        "Observe-only fallback stays available when a native driver, extension, entitlement, or privilege is missing.".into(),
        "A local dashboard can guide installation and record consent, but browsers and operating systems intentionally require their own approval prompts for privileged components.".into(),
    ]
}
