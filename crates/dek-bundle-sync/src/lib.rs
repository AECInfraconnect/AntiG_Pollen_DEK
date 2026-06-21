use anyhow::{Context, Result};
use dek_config::MtlsConfig;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactState {
    Discovered,
    Downloaded,
    HashVerified,
    SignatureVerified,
    SchemaValidated,
    CompatibilityChecked,
    Staged,
    Warmed,
    Shadow,
    Active,
    LastKnownGood,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub version: String,
    pub state: ArtifactState,
}

#[derive(Deserialize)]
struct BundleInfo {
    bundle_id: String,
    version: String,
    signature: String,
    public_key: String,
    payload: serde_json::Value,
}

pub struct BundleSyncAgent {
    endpoint_url: String,
    client: RwLock<reqwest::Client>,
}

impl BundleSyncAgent {
    pub fn new(endpoint_url: &str, mtls: &MtlsConfig) -> Result<Self> {
        let client = mtls.build_client()?;
        Ok(Self {
            endpoint_url: endpoint_url.to_string(),
            client: RwLock::new(client),
        })
    }

    pub async fn update_mtls(&self, mtls: &MtlsConfig) -> Result<()> {
        let new_client = mtls.build_client()?;
        let mut client_lock = self.client.write().await;
        *client_lock = new_client;
        info!("[BundleSync] Successfully updated internal HTTP client with new mTLS configuration");
        Ok(())
    }

    pub async fn check_for_updates(&self) -> Result<()> {
        let client = self.client.read().await.clone();
        info!(
            "[BundleSync] Checking for policy updates at {}...",
            self.endpoint_url
        );

        let res = client.get(&self.endpoint_url).send().await?;
        if !res.status().is_success() {
            warn!("[BundleSync] No updates available or cloud is unreachable.");
            return Ok(());
        }

        let bundle_info: BundleInfo = res.json().await.context("Failed to parse bundle info")?;

        let mut artifact = Artifact {
            id: bundle_info.bundle_id.clone(),
            version: bundle_info.version.clone(),
            state: ArtifactState::Discovered,
        };
        info!(
            "[BundleSync] Discovered new bundle: {} v{}",
            artifact.id, artifact.version
        );

        // Transition to Downloaded
        artifact.state = ArtifactState::Downloaded;
        info!("[BundleSync] State: {:?}", artifact.state);

        // Verify Signature
        use base64::Engine;
        let public_key_bytes = base64::prelude::BASE64_STANDARD.decode(&bundle_info.public_key)?;
        let signature_bytes = base64::prelude::BASE64_STANDARD.decode(&bundle_info.signature)?;

        let verifying_key = VerifyingKey::from_bytes(
            public_key_bytes
                .as_slice()
                .try_into()
                .context("Invalid public key length")?,
        )?;
        let signature = Signature::from_bytes(
            signature_bytes
                .as_slice()
                .try_into()
                .context("Invalid signature length")?,
        );

        let payload_string = serde_json::to_string(&bundle_info.payload)?;

        if verifying_key
            .verify(payload_string.as_bytes(), &signature)
            .is_ok()
        {
            artifact.state = ArtifactState::SignatureVerified;
            info!(
                "[BundleSync] State: {:?} - Signature valid!",
                artifact.state
            );
        } else {
            error!("[BundleSync] Signature verification failed! Discarding bundle.");
            return Err(anyhow::anyhow!("Signature verification failed"));
        }

        // Staging
        artifact.state = ArtifactState::Staged;
        let target_dir = PathBuf::from("target");
        fs::create_dir_all(&target_dir)?;
        let active_bundle_path = target_dir.join("active_bundle.json");

        fs::write(&active_bundle_path, payload_string)?;
        info!(
            "[BundleSync] State: {:?} - Written to {:?}",
            artifact.state, active_bundle_path
        );

        // Activation
        artifact.state = ArtifactState::Active;
        info!(
            "[BundleSync] State: {:?} - Pipeline complete.",
            artifact.state
        );

        Ok(())
    }
}
