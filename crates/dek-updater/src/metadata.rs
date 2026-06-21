use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMetadata {
    pub schema_version: String,
    pub channel: String,
    pub version: String,
    pub published_at: String,
    pub expires_at: String,
    pub min_supported_version: Option<String>,
    pub rollout: RolloutConfig,
    pub artifacts: Vec<UpdateArtifact>,
    pub security: SecurityConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RolloutConfig {
    pub percentage: u8,
    pub tenant_allowlist: Vec<String>,
    pub device_allowlist: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateArtifact {
    pub os: String,
    pub arch: String,
    pub format: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub signature: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub signing_key_id: String,
    pub threshold: u8,
    pub signatures: Vec<String>,
}

impl UpdateMetadata {
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let metadata: Self = serde_json::from_str(content)?;
        Ok(metadata)
    }

    pub fn is_expired(&self) -> bool {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            return expires.with_timezone(&chrono::Utc) < chrono::Utc::now();
        }
        true // fail closed if parsing fails
    }
}
