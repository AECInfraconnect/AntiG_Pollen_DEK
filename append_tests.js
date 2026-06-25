
const fs = require("fs");
const file = "crates/acceptance-tests/tests/policy_first_ux_tests.rs";
let content = fs.readFileSync(file, "utf8");

const newTests = `
#[test]
fn e2e_happy_path_scan_deploy_active() {
    let fixture = load_fixture("windows_mcp_stdio_no_wfp");
    let intent = PolicyIntent::ApproveRiskyToolCalls;
    
    // 1. scan -> suggestion (mocked)
    // 2. feasibility
    let feasibility = evaluate_policy(fixture, intent);
    assert_eq!(feasibility.status, PolicyFeasibilityStatus::CanEnforceAfterApproval);
    
    // 3. deploy (mocked user approval)
    let requested = ControlLevel::Enforce;
    let effective = ControlLevel::Enforce; // Assuming user approved
    
    // 4. warm check
    let warm_check = WarmCheckResult { ok: true };
    
    // 5. active
    let final_status = status_after_warm_check(requested, effective, &warm_check);
    assert_eq!(final_status, DeploymentStatus::Active);
}

#[test]
fn e2e_fallback_path_scan_observe_only() {
    let fixture = load_fixture("linux_no_ebpf_permission");
    let intent = PolicyIntent::BlockUnknownNetworkDestinations;
    
    // 1. scan -> suggestion (mocked)
    // 2. feasibility
    let feasibility = evaluate_policy(fixture, intent);
    assert_eq!(feasibility.status, PolicyFeasibilityStatus::NeedsSetup);
    
    // 3. fallback to observe
    let requested = ControlLevel::Enforce;
    let effective = ControlLevel::Observe;
    
    // 4. warm check
    let warm_check = WarmCheckResult { ok: true };
    
    // 5. active_observe_only
    let final_status = status_after_warm_check(requested, effective, &warm_check);
    assert_eq!(final_status, DeploymentStatus::ActiveObserveOnly);
}
`;

if (!content.includes("e2e_happy_path_scan_deploy_active")) {
    fs.appendFileSync(file, newTests);
    console.log("E2E tests added.");
}

