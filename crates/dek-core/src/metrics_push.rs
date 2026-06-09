// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;
use tracing::{debug, info, warn};

#[allow(dead_code)]
pub async fn run(
    shutdown: Arc<Notify>,
    metrics_client: Arc<RwLock<reqwest::Client>>,
    metrics_push_url: String,
    prometheus_handle: PrometheusHandle,
) {
    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                info!("Metrics Push task shutting down gracefully.");
                break;
            }
            _ = sleep(Duration::from_secs(10)) => {
                let metrics_text = prometheus_handle.render();

                let strategy = ExponentialBackoff::from_millis(500)
                    .factor(2)
                    .max_delay(Duration::from_secs(2))
                    .take(4);

                let res = Retry::spawn(strategy, || async {
                    let client = metrics_client.read().await.clone();
                    let push_res = client
                        .post(&metrics_push_url)
                        .body(metrics_text.clone())
                        .send()
                        .await;

                    match push_res {
                        Ok(r) if r.status().is_success() => Ok(()),
                        Ok(r) => {
                            warn!("Failed to push metrics, status: {}", r.status());
                            Err(anyhow::anyhow!("HTTP Status: {}", r.status()))
                        },
                        Err(e) => {
                            warn!("Error pushing metrics: {}", e);
                            Err(anyhow::anyhow!("Request error: {}", e))
                        }
                    }
                }).await;

                if res.is_ok() {
                    debug!("Successfully pushed metrics to {}", metrics_push_url);
                } else {
                    warn!("Failed to push metrics after retries");
                }
            }
        }
    }
}

