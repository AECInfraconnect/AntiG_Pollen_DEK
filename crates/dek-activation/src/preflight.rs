use crate::ActivationError;
use dek_config::DekConfig;
use dek_policy_router::PolicyRouter;
use std::sync::Arc;
use tracing::{error, info};

pub async fn run_preflight_tests(config: &DekConfig, router: Arc<PolicyRouter>) -> Result<(), ActivationError> {
    if config.preflight_tests.is_empty() {
        // Warning: Regulated profiles might require at least 1 preflight test, but we allow empty for now.
        info!("No preflight tests found, skipping.");
        return Ok(());
    }

    info!("Running {} preflight tests...", config.preflight_tests.len());
    for test in &config.preflight_tests {
        match router.authorize(test.input.clone()).await {
            Ok(decision) => {
                if decision.decision != test.expected_decision {
                    let msg = format!("Preflight test '{}' failed: expected {}, got {}", test.name, test.expected_decision, decision.decision);
                    error!("{}", msg);
                    return Err(ActivationError::PreflightFailed(msg));
                }
            }
            Err(e) => {
                let msg = format!("Preflight test '{}' failed with execution error: {}", test.name, e);
                error!("{}", msg);
                return Err(ActivationError::PreflightFailed(msg));
            }
        }
    }
    
    info!("All preflight tests passed.");
    Ok(())
}
