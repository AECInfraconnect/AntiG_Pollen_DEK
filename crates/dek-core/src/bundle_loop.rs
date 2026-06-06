use anyhow::Result;
use dek_config::DekConfig;
use metrics::counter;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::{Retry, RetryIf};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, Instrument};

pub async fn run_sync_pipeline_with_retry(
    sync_agent: &dek_bundle_sync::BundleSyncAgent,
) -> Result<DekConfig> {
    info!("Running Unified Sync Pipeline...");
    let strategy = ExponentialBackoff::from_millis(2000)
        .factor(2)
        .max_delay(Duration::from_secs(30))
        .take(10);

    RetryIf::spawn(
        strategy,
        || async {
            match sync_agent.run_pipeline().await {
                Ok(c) => Ok(c),
                Err(e) => {
                    warn!("Pipeline run failed: {}. Retrying...", e);
                    counter!("dek_core_config_fetch_errors_total").increment(1);
                    Err(e)
                }
            }
        },
        |e: &anyhow::Error| {
            if let Some(reqwest_err) = e.downcast_ref::<reqwest::Error>() {
                if let Some(status) = reqwest_err.status() {
                    if status.is_client_error() {
                        error!(
                            "Fatal HTTP client error running pipeline: {}. Aborting startup.",
                            status
                        );
                        return false;
                    }
                }
                if reqwest_err.is_builder() || reqwest_err.is_request() {
                    return false;
                }
            }
            true
        },
    )
    .await
}

pub fn spawn_bundle_sync_task(
    cancel_token: CancellationToken,
    sync_agent: Arc<dek_bundle_sync::BundleSyncAgent>,
    bundle_sync_interval: u64,
    metrics_client: Arc<RwLock<reqwest::Client>>,
    pinned_key: String,
) -> JoinHandle<()> {
    tokio::spawn(
        async move {
            let mut current_version = String::new();
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("Bundle Sync task shutting down gracefully.");
                        break;
                    }
                    _ = sleep(Duration::from_secs(bundle_sync_interval)) => {
                        debug!("Running unified bundle sync pipeline...");
                        match timeout(Duration::from_secs(30), sync_agent.run_pipeline()).await {
                            Ok(Ok(new_config)) => {
                                counter!("dek_core_bundle_sync_success_total").increment(1);
                                if let Some(update) = new_config.update_config {
                                    if update.version != current_version {
                                        info!("New binary update found: version {}", update.version);
                                        let client = metrics_client.read().await.clone();
                                        match crate::updater::run_update(&client, &update.download_url, &update.signature_b64, &pinned_key).await {
                                            Ok(_) => {
                                                info!("Update applied successfully. Version updated to {}", update.version);
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
            }
        }
        .instrument(tracing::info_span!("bundle_sync")),
    )
}
