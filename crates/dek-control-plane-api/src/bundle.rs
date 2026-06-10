use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PollenPolicyBundleManifest {
    pub bundle_version: String,
    pub bundle_id: String,
    pub tenant_id: String,
    pub workspace_id: String,
    pub environment_id: String,
    pub build_number: u64,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub created_by: String,
    pub registry_snapshot_sha256: String,
    pub router_config_sha256: String,
    pub artifacts: Vec<BundleArtifact>,
    pub signatures: Vec<BundleSignature>,
    pub min_dek_version: String,
    pub rollback_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleArtifact {
    pub artifact_id: String,
    pub adapter_id: String,
    pub artifact_type: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BundleSignature {
    pub signature_id: String,
    pub signature_type: String,
    pub payload: String,
    pub public_key_fingerprint: String,
}
