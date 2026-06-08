use dek_domain_schema::{
    CompiledNetworkRules, NetworkConditions, NetworkDestination, NetworkFallback,
    NetworkGuardrailEffect, NetworkTargets,
};
use dek_windows_wfp::WfpFilterManager;

#[test]
fn test_wfp_bypass_prevention_logic() {
    let mut manager = WfpFilterManager::new();
    manager.start().unwrap();

    let rules = CompiledNetworkRules {
        policy_id: "pol-001".to_string(),
        policy_type: "NETWORK_EGRESS_GUARDRAIL".to_string(),
        version: 1,
        risk_tier: "high".to_string(),
        targets: NetworkTargets {
            devices: vec!["*".to_string()],
            ..Default::default()
        },
        conditions: NetworkConditions {
            destinations: vec![NetworkDestination {
                r#type: "domain".to_string(),
                value: serde_json::json!("malicious.com"),
            }],
            protocols: vec!["TCP".to_string()],
            time_window: None,
        },
        effect: NetworkGuardrailEffect::Deny,
        obligations: vec![],
        fallback: NetworkFallback {
            cloud_unavailable: "FAIL_CLOSED".to_string(),
            policy_stale: "FAIL_CLOSED".to_string(),
        },
    };

    // Apply rule that explicitly blocks malicious.com.
    // In actual kernel integration, this would ensure that direct TCP connections
    // bypassing the user-mode HTTP proxy are caught by the kernel WFP filter.
    assert!(manager.apply_rules(&rules).is_ok());

    // We verify the stub works properly
    assert!(manager.clear_rules().is_ok());
    manager.stop().unwrap();
}

#[test]
fn test_fail_closed_watchdog() {
    use dek_windows_wfp::watchdog::WfpWatchdog;

    let mut watchdog = WfpWatchdog::new();

    let rules = CompiledNetworkRules {
        policy_id: "pol-002".to_string(),
        policy_type: "NETWORK_EGRESS_GUARDRAIL".to_string(),
        version: 1,
        risk_tier: "high".to_string(),
        targets: NetworkTargets {
            devices: vec!["*".to_string()],
            ..Default::default()
        },
        conditions: NetworkConditions {
            destinations: vec![],
            protocols: vec![],
            time_window: None,
        },
        effect: NetworkGuardrailEffect::Deny,
        obligations: vec![],
        fallback: NetworkFallback {
            cloud_unavailable: "FAIL_CLOSED".to_string(),
            policy_stale: "FAIL_CLOSED".to_string(),
        },
    };

    let result = watchdog.apply_with_fallback(rules.clone(), |_r| {
        // Simulate immediate driver panic or error
        anyhow::bail!("Simulated driver failure")
    });

    // Should fail and trigger fail closed
    assert!(result.is_err());
}
