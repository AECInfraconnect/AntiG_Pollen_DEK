// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use anyhow::Result;
use dek_domain_schema::CompiledNetworkRules;
use tracing::{info, warn};

// Stub implementation for cross-compilation testing.
// In reality, this communicates with the macOS Network Extension via XPC or Unix Sockets.

#[derive(Debug, Default)]
pub struct NeFilterClient {
    connected: bool,
}

impl NeFilterClient {
    pub fn new() -> Self {
        Self { connected: false }
    }

    pub fn connect(&mut self) -> Result<()> {
        info!("Connecting to PollenDEKNetworkExtension via IPC");
        self.connected = true;
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from PollenDEKNetworkExtension");
        self.connected = false;
        Ok(())
    }

    pub fn push_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
        if !self.connected {
            warn!("Cannot push rules; NEFilterClient is not connected.");
            return Ok(());
        }

        info!(
            "Pushing compiled rules to macOS Network Extension: {} (v{})",
            rules.policy_id, rules.version
        );

        Ok(())
    }

    pub fn clear_rules(&self) -> Result<()> {
        info!("Clearing rules in macOS Network Extension");
        Ok(())
    }
}

