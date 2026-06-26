// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentIdentityBindingStatus {
    LocalOnly,
    ReadyForCloudEnrollment,
    CloudBound,
    NeedsSpiffeAgent,
    NeedsOauthEnrollment,
    Suspended,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenBindingKind {
    OAuthAccessToken,
    OAuthRefreshToken,
    OidcIdToken,
    JwtSvid,
    X509Svid,
    ApiKeyReference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenStorageLocation {
    OsKeyStore,
    BrowserProfile,
    AgentRuntime,
    SpireWorkloadApi,
    ExternalSecretManager,
    NotStoredByPollek,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TokenBindingReference {
    pub kind: TokenBindingKind,
    pub issuer: Option<String>,
    pub audience: Vec<String>,
    pub subject_hint: Option<String>,
    pub storage_location: TokenStorageLocation,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RegisteredAgentIdentityBinding {
    pub schema_version: String,
    pub tenant_id: String,
    pub device_id: String,
    pub agent_id: String,
    pub agent_label: String,
    pub spiffe_id: Option<String>,
    pub spiffe_trust_domain: Option<String>,
    pub spiffe_selector_hashes: Vec<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_issuer: Option<String>,
    pub token_bindings: Vec<TokenBindingReference>,
    pub status: AgentIdentityBindingStatus,
    pub local_dashboard_available: bool,
    pub cloud_enrollment_required: bool,
    pub cloud_enrolled: bool,
    pub evidence_ids: Vec<String>,
    pub friendly_summary_en: String,
    pub friendly_summary_th: String,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_identity_binding_does_not_require_token_secret() {
        let binding = RegisteredAgentIdentityBinding {
            schema_version: "registered-agent-identity-binding.v1".into(),
            tenant_id: "local".into(),
            device_id: "dev_test".into(),
            agent_id: "agent_test".into(),
            agent_label: "Test Agent".into(),
            spiffe_id: Some("spiffe://local.pollek.dev/tenant/local/agent/agent_test".into()),
            spiffe_trust_domain: Some("local.pollek.dev".into()),
            spiffe_selector_hashes: vec!["sha256:selector".into()],
            oauth_client_id: None,
            oauth_issuer: None,
            token_bindings: vec![TokenBindingReference {
                kind: TokenBindingKind::JwtSvid,
                issuer: Some("spiffe://local.pollek.dev".into()),
                audience: vec!["pollek-local-control-plane".into()],
                subject_hint: Some("agent_test".into()),
                storage_location: TokenStorageLocation::SpireWorkloadApi,
                expires_at: None,
                last_verified_at: None,
            }],
            status: AgentIdentityBindingStatus::LocalOnly,
            local_dashboard_available: true,
            cloud_enrollment_required: false,
            cloud_enrolled: false,
            evidence_ids: vec![],
            friendly_summary_en: "Local workload identity is available.".into(),
            friendly_summary_th: "มี workload identity สำหรับ local แล้ว".into(),
            updated_at: Utc::now(),
        };

        assert!(binding.local_dashboard_available);
        assert!(!binding.cloud_enrollment_required);
        assert_eq!(
            binding.token_bindings[0].storage_location,
            TokenStorageLocation::SpireWorkloadApi
        );
    }
}
