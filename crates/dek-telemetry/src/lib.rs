use anyhow::Result;
use backoff::{future::retry, ExponentialBackoff};
use dek_config::MtlsConfig;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

pub struct CloudTelemetrySink {
    sender: mpsc::Sender<Value>,
    client: Arc<RwLock<reqwest::Client>>,
}

impl CloudTelemetrySink {
    pub fn new(endpoint_url: &str, mtls: &MtlsConfig) -> Result<Self> {
        let client = Arc::new(RwLock::new(mtls.build_client()?));

        // MPSC channel with buffer size of 1024
        let (tx, mut rx) = mpsc::channel::<Value>(1024);

        let bg_client = client.clone();
        let bg_url = endpoint_url.to_string();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let backoff = ExponentialBackoff {
                    max_elapsed_time: Some(Duration::from_secs(60)),
                    initial_interval: Duration::from_millis(500),
                    max_interval: Duration::from_secs(5),
                    multiplier: 2.0,
                    ..ExponentialBackoff::default()
                };

                let res = retry(backoff, || async {
                    let c = bg_client.read().await.clone();
                    match c.post(&bg_url).json(&event).send().await {
                        Ok(res) if res.status().is_success() => Ok(()),
                        Ok(res) => {
                            warn!(
                                "[Telemetry] Failed to send event. Status: {}. Retrying...",
                                res.status()
                            );
                            Err(backoff::Error::transient(anyhow::anyhow!(
                                "Status {}",
                                res.status()
                            )))
                        }
                        Err(e) => {
                            warn!("[Telemetry] Request error: {}. Retrying...", e);
                            Err(backoff::Error::transient(e.into()))
                        }
                    }
                })
                .await;

                if let Err(e) = res {
                    error!("[Telemetry] Dropped event after max retries: {}", e);
                } else {
                    info!("[Telemetry] Successfully sent event to cloud.");
                }
            }
        });

        Ok(Self { sender: tx, client })
    }

    pub async fn update_mtls(&self, mtls: &MtlsConfig) -> Result<()> {
        let new_client = mtls.build_client()?;
        let mut client_lock = self.client.write().await;
        *client_lock = new_client;
        info!("[Telemetry] Successfully updated internal HTTP client with new mTLS configuration");
        Ok(())
    }

    pub async fn emit_async(&self, event: Value) -> Result<()> {
        // Use try_send for non-blocking. If buffer is full, event is dropped.
        if let Err(e) = self.sender.try_send(event) {
            error!("[Telemetry] Buffer full or closed, dropping event: {}", e);
        }
        Ok(())
    }
}
