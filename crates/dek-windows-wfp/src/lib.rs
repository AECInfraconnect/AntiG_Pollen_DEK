// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

pub mod watchdog;

use anyhow::Result;
use dek_domain_schema::CompiledNetworkRules;
use dek_enforcement_api::NetworkEnforcer;
use tracing::{info, warn};

// In a real Windows environment, this would call Windows Filtering Platform (WFP) APIs.
// For now, we stub it to allow cross-compilation tests and development without Windows APIs failing.

#[derive(Debug, Default)]
pub struct WfpFilterManager {
    is_active: bool,
}

impl WfpFilterManager {
    pub fn new() -> Self {
        Self { is_active: false }
    }
}

impl NetworkEnforcer for WfpFilterManager {
    fn start(&mut self) -> Result<()> {
        info!("Starting Windows Filtering Platform (WFP) provider (observe-only prototype)");
        self.is_active = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Stopping Windows Filtering Platform (WFP) provider");
        self.is_active = false;
        Ok(())
    }

    fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
        if !self.is_active {
            warn!("Attempted to apply rules, but WFP manager is not active.");
            return Ok(());
        }

        info!(
            "Applying WFP filters for policy: {} (v{})",
            rules.policy_id, rules.version
        );

        Ok(())
    }

    fn clear_rules(&self) -> Result<()> {
        info!("Clearing all active WFP filters");
        Ok(())
    }
}
