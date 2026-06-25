const fs = require("fs");
const file = "crates/local-control-plane/src/policy_first_api.rs";
let content = fs.readFileSync(file, "utf8");

content = content.replace(
    /struct FeasibilityRequest \{\s*policy: serde_json::Value,\s*requested_level: ControlLevel,\s*\}/g,
    `struct FeasibilityRequest {
    candidate: dek_agent_discovery::model::DiscoveredAgentCandidateV2,
    requested_level: ControlLevel,
}`
);

content = content.replace(
    /async fn evaluate_feasibility\([\s\S]*?let res = assess_feasibility\(&pol, &snap\);\s*Ok\(\(StatusCode::OK, Json\(res\)\)\)\s*\}/g,
    `async fn evaluate_feasibility(
    Path(_tenant): Path<String>,
    Json(req): Json<FeasibilityRequest>,
) -> ApiResult<(StatusCode, Json<PolicyFeasibilityResult>)> {
    let snap = get_current_snapshot();
    let res = dek_enforcement_api::feasibility::assess(&req.candidate, req.requested_level, &snap);
    Ok((StatusCode::OK, Json(res)))
}`
);

// We should also replace create_deploy_session to use candidate instead of policy
content = content.replace(
    /struct CreateDeployRequest \{\s*policy: serde_json::Value,\s*agents: Vec<String>,\s*requested_level: ControlLevel,\s*\}/g,
    `struct CreateDeployRequest {
    candidate: dek_agent_discovery::model::DiscoveredAgentCandidateV2,
    requested_level: ControlLevel,
}`
);

content = content.replace(
    /async fn create_deploy_session\([\s\S]*?let res = assess_feasibility\(&pol, &snap\);/g,
    `async fn create_deploy_session(
    Path(_tenant): Path<String>,
    Json(req): Json<CreateDeployRequest>,
) -> ApiResult<(StatusCode, Json<DeploySession>)> {
    let snap = get_current_snapshot();
    let res = dek_enforcement_api::feasibility::assess(&req.candidate, req.requested_level, &snap);`
);

content = content.replace(
    /async fn confirm_deploy_session\([\s\S]*?let res = assess_feasibility\(&pol, &snap\);/g,
    `async fn confirm_deploy_session(
    Path((_tenant, _id)): Path<(String, String)>,
) -> ApiResult<(
    StatusCode,
    Json<dek_enforcement_api::planner::ControlMethodPlan>,
)> {
    // In a real app we would load the session and candidate from DB.
    // Here we just mock it for compilation based on previous stub logic.
    let snap = get_current_snapshot();
    let candidate = dek_agent_discovery::model::DiscoveredAgentCandidateV2 {
        id: "mock".into(),
        name: "mock".into(),
        agent_type: "mock".into(),
        vendor: "mock".into(),
        confidence: 1.0,
        pid: None,
        paths: vec![],
        discovered_mcp_servers: vec![],
        suggested_registration: dek_agent_discovery::model::SuggestedAgentRegistration {
            tags: vec![],
            is_ai_agent: true,
            auto_register: false,
        },
        suggested_observation_profile: dek_agent_discovery::model::ObservationProfile {
            enabled_layers: vec![],
            sensitivity: "medium".into(),
        },
        suggested_control_bindings: vec![],
        telemetry_plan: dek_agent_discovery::model::TelemetryPlan {
            events_endpoint: "none".into(),
        },
        labels: std::collections::BTreeMap::new(),
    };
    let res = dek_enforcement_api::feasibility::assess(&candidate, ControlLevel::Enforce, &snap);`
);

fs.writeFileSync(file, content);
console.log("Updated policy_first_api.rs");

