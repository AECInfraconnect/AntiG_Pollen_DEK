use anyhow::Result;
use dek_config::MtlsConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::Retry;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryEnvelope {
    pub ts: String,
    pub tenant_id: Option<String>,
    pub device_id: String,
    pub spiffe_id: String,
    pub dek_version: String,
    pub os: String,
    pub event_type: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egress: Option<EgressLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpDecisionLog>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EgressLog {
    pub dest_ip: String,
    pub dest_port: u16,
    pub fqdn: Option<String>,
    pub cgroup_id: u64,
    pub pid: u32,
    pub verdict: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpDecisionLog {
    pub principal: String,
    pub tool: String,
    pub method: String,
    pub engine: String,
    pub verdict: String,
    pub reason: String,
    pub request_id: String,
}

pub struct CloudTelemetrySink {
    sender: mpsc::Sender<Value>,
    client: Arc<RwLock<reqwest::Client>>,
}

impl CloudTelemetrySink {
    pub fn new(endpoint_url: &str, mtls: &MtlsConfig, client_key_override: Option<&[u8]>) -> Result<Self> {
        let client = Arc::new(RwLock::new(mtls.build_client(client_key_override)?));

        // MPSC channel with buffer size of 1024
        let (tx, mut rx) = mpsc::channel::<Value>(1024);

        let bg_client = client.clone();
        let bg_url = endpoint_url.to_string();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let strategy = ExponentialBackoff::from_millis(500)
                    .factor(2)
                    .max_delay(Duration::from_secs(5))
                    .take(7);

                let res = Retry::spawn(strategy, || async {
                    let c = bg_client.read().await.clone();
                    match c.post(&bg_url).json(&event).send().await {
                        Ok(res) if res.status().is_success() => Ok(()),
                        Ok(res) => {
                            warn!(
                                "[Telemetry] Failed to send event. Status: {}. Retrying...",
                                res.status()
                            );
                            Err(anyhow::anyhow!("Status {}", res.status()))
                        }
                        Err(e) => {
                            warn!("[Telemetry] Request error: {}. Retrying...", e);
                            Err(e.into())
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
        let new_client = mtls.build_client(None)?;
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
