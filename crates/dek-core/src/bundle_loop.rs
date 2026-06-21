// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

//! bundle_loop.rs — periodic unified bundle-sync pipeline + binary auto-update.
//!
//! Lifted verbatim from `main.rs::spawn_bundle_sync_task`, made `pub`.
//! Each tick runs `BundleSyncAgent::run_pipeline()` (fetch -> verify ed25519 ->
//! merge -> stage active_bundle), and if the returned config carries an
//! `update_config` with a new version, triggers the health-gated A/B updater.

use dek_bundle_sync::BundleSyncAgent;
use metrics::counter;
use std::sync::Arc;
use tokio::sync::{RwLock, Notify};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, Instrument};

#[allow(dead_code)]
pub fn spawn_bundle_sync_task(
    cancel_token: CancellationToken,
    sync_agent: Arc<BundleSyncAgent>,
    bundle_sync_interval: u64,
    metrics_client: Arc<RwLock<reqwest::Client>>,
    pinned_key: String,
    reload_coordinator: Arc<crate::reload_coordinator::ReloadCoordinator>,
) -> JoinHandle<()> {
    let sse_notify = Arc::new(Notify::new());
    
    // Spawn SSE listener task
    let sse_sync_agent = sync_agent.clone();
    let sse_client = metrics_client.clone();
    let sse_token = cancel_token.clone();
    let sse_notifier = sse_notify.clone();
    
    tokio::spawn(async move {
        loop {
            if sse_token.is_cancelled() { break; }
            
            let url = format!("{}/v1/push", sse_sync_agent.cloud_url);
            let client = sse_client.read().await.clone();
            
            match client.get(&url).header("Accept", "text/event-stream").send().await {
                Ok(mut res) => {
                    if res.status().is_success() {
                        info!("Connected to SSE endpoint: {}", url);
                        while let Ok(Some(chunk)) = res.chunk().await {
                            if sse_token.is_cancelled() { break; }
                            if let Ok(text) = std::str::from_utf8(&chunk) {
                                if text.contains("data:") || text.contains("event:") {
                                    info!("SSE Push received, triggering bundle sync...");
                                    sse_notifier.notify_one();
                                }
                            }
                        }
                    } else {
                        warn!("SSE connection failed with status {}. Retrying in 10s...", res.status());
                    }
                }
                Err(e) => {
                    warn!("SSE request error: {}. Retrying in 10s...", e);
                }
            }
            
            tokio::select! {
                _ = sleep(Duration::from_secs(10)) => {}
                _ = sse_token.cancelled() => break,
            }
        }
    }.instrument(tracing::info_span!("sse_listener")));

    tokio::spawn(
        async move {
            let mut current_version = String::new();
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Bundle Sync task shutting down gracefully.");
                        break;
                    }
                    _ = sse_notify.notified() => {
                        debug!("SSE triggered unified bundle sync pipeline...");
                    }
                    _ = sleep(Duration::from_secs(bundle_sync_interval)) => {
                        debug!("Running unified bundle sync pipeline...");
                    }
                }

                match timeout(Duration::from_secs(30), sync_agent.run_pipeline()).await {
                    Ok(Ok((new_config, staged_path))) => {
                        counter!("dek_core_bundle_sync_success_total").increment(1);
                        if let Err(e) = reload_coordinator.process_staged_bundle(&new_config, &staged_path).await {
                            error!("Bundle Activation Failed: {}", e);
                            counter!("dek_core_bundle_activation_errors_total").increment(1);
                        } else {
                            // Enforce Enterprise Profiles
                            use dek_config::{EnterpriseProfile, ActivationMode};
                            if (new_config.enterprise_profile == EnterpriseProfile::Enterprise || new_config.enterprise_profile == EnterpriseProfile::Regulated)
                                && new_config.activation_mode != ActivationMode::Full {
                                    warn!("Enterprise Profile enforces 'Full' activation mode. Overriding '{}'", format!("{:?}", new_config.activation_mode));
                                }
                        }

                        if let Some(update) = new_config.update_config {
                            if update.version != current_version {
                                info!("New binary update found: version {}", update.version);
                                let client = metrics_client.read().await.clone();
                                match crate::updater::run_update(
                                    &client,
                                    &update.download_url,
                                    &update.signature_b64,
                                    &pinned_key,
                                ).await {
                                    Ok(_) => {
                                        info!("Update staged successfully. Version updated to {}", update.version);
                                        current_version = update.version;
                                    }
                                    Err(e) => {
                                        error!("Failed to apply binary update: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        warn!(error = %e, "Bundle sync pipeline failed");
                        counter!("dek_core_bundle_sync_errors_total").increment(1);
                    }
                    Err(_) => {
                        warn!("Bundle sync pipeline timed out after 30s");
                        counter!("dek_core_bundle_sync_timeout_total").increment(1);
                    }
                }
                counter!("dek_core_bundle_checks_total").increment(1);
            }
        }
        .instrument(tracing::info_span!("bundle_sync")),
    )
}
