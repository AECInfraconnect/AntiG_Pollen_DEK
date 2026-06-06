use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use dek_config::BootstrapConfig;
use dek_telemetry::CloudTelemetrySink;
use dek_bundle_sync::BundleSyncAgent;
use metrics_exporter_prometheus::PrometheusHandle;
use std::time::Instant;

// Import logic originally in main.rs (now we just assume it's here or in main)
// MOVE: The spawn_metrics_push_task, spawn_bundle_sync_task, and spawn_ipc_server_task
// should ideally be moved here, but since the user requested "Supervisor struct that holds JoinSet",
// we will just define Supervisor here and import those fns if they stay in main.rs, or move them here.

// For now, let's keep it simple. Supervisor orchestrates the startup sequence.

pub struct Supervisor {
    pub cancel_token: CancellationToken,
    pub join_set: tokio::task::JoinSet<()>,
}

impl Supervisor {
    pub fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            join_set: tokio::task::JoinSet::new(),
        }
    }

    pub fn shutdown(&mut self) {
        self.cancel_token.cancel();
    }

    pub async fn wait_for_shutdown(mut self) {
        // Wait for active handlers to finish
        match timeout(Duration::from_secs(15), async {
            while let Some(res) = self.join_set.join_next().await {
                if let Err(e) = res {
                    warn!("Active task panicked during shutdown: {}", e);
                }
            }
        })
        .await
        {
            Ok(_) => info!("All supervisor tasks closed gracefully."),
            Err(_) => warn!(
                "Grace period expired! Forcefully terminating remaining active supervisor tasks."
            ),
        }
    }
}
