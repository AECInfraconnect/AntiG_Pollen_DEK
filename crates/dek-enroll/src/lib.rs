//! dek-enroll — first-run device enrollment for Pollen DEK.
//!
//! Implements the OAuth 2.0 Device Authorization Grant (RFC 8628): the DEK is a
//! headless daemon that cannot host a browser callback, so the user authorizes
//! it on a separate device by entering a short `user_code`. After the user
//! approves (binding this device to their user/tenant in Pollen Cloud), the DEK
//! exchanges the resulting access token at `/enroll` for a one-time SPIRE join
//! token + trust bundle + pinned bundle public key + cloud URLs.
//!
//! This crate does NOT touch the filesystem or crypto — it returns an
//! [`Enrollment`]; the caller (`dekctl enroll`) drives SPIRE attestation and
//! writes `bootstrap.json`. Keeping it side-effect-free makes it testable.

use serde::Deserialize;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum EnrollError {
    #[error("network error talking to {0}: {1}")]
    Network(String, String),
    #[error("device authorization rejected: {0}")]
    DeviceAuth(String),
    #[error("the user did not approve before the code expired")]
    Expired,
    #[error("authorization was denied by the user")]
    AccessDenied,
    #[error("enrollment endpoint failed: HTTP {0}")]
    EnrollHttp(u16),
    #[error("malformed response from {0}")]
    BadResponse(String),
}

/// What the caller needs to complete enrollment (drive SPIRE + write bootstrap).
#[derive(Debug, Clone)]
pub struct Enrollment {
    pub join_token: String,
    pub spire_endpoint: String,
    /// Trust anchor (root CA) for mTLS, PEM.
    pub trust_bundle_pem: String,
    /// ed25519 public key used to verify signed policy bundles, base64.
    pub pinned_bundle_public_key: String,
    pub tenant_id: String,
    pub device_id: String,
    /// SPIFFE ID the server intends to assign (informational; the SVID is
    /// authoritative once issued).
    pub spiffe_id_hint: Option<String>,
    /// The base URL the DEK should persist and use for all future calls.
    pub cloud_url: String,
}

pub struct EnrollClient {
    cloud_url: String,
    client_id: String,
    scope: String,
    http: reqwest::Client,
}

impl EnrollClient {
    pub fn new(cloud_url: &str, client_id: &str, scope: &str) -> Self {
        let cloud_url = cloud_url.trim_end_matches('/').to_string();
        
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(15));
            
        // For local mock testing, allow invalid certs
        if cloud_url.contains("127.0.0.1") || cloud_url.contains("localhost") {
            builder = builder.danger_accept_invalid_certs(true);
        }

        Self {
            cloud_url,
            client_id: client_id.to_string(),
            scope: scope.to_string(),
            http: builder.build().expect("build http client"),
        }
    }

    /// Run the full device flow and return the [`Enrollment`].
    /// `display` is invoked once with the user-facing instructions.
    pub async fn run<F: Fn(&UserPrompt)>(&self, display: F) -> Result<Enrollment, EnrollError> {
        let auth = self.request_device_code().await?;
        display(&UserPrompt {
            verification_uri: auth.verification_uri.clone(),
            verification_uri_complete: auth.verification_uri_complete.clone(),
            user_code: auth.user_code.clone(),
            expires_in: auth.expires_in,
        });
        let access_token = self.poll_for_token(&auth).await?;
        self.enroll(&access_token).await
    }

    async fn request_device_code(&self) -> Result<DeviceAuthResp, EnrollError> {
        let url = format!("{}/oauth/device_authorization", self.cloud_url);
        let resp = self
            .http
            .post(&url)
            .form(&[("client_id", self.client_id.as_str()), ("scope", self.scope.as_str())])
            .send()
            .await
            .map_err(|e| EnrollError::Network(url.clone(), e.to_string()))?;
        if !resp.status().is_success() {
            return Err(EnrollError::DeviceAuth(format!("HTTP {}", resp.status())));
        }
        resp.json::<DeviceAuthResp>()
            .await
            .map_err(|_| EnrollError::BadResponse("device_authorization".into()))
    }

    async fn poll_for_token(&self, auth: &DeviceAuthResp) -> Result<String, EnrollError> {
        let url = format!("{}/oauth/token", self.cloud_url);
        let deadline = Instant::now() + Duration::from_secs(auth.expires_in);
        // RFC 8628 §3.5: default minimum polling interval is 5s.
        let mut interval = Duration::from_secs(auth.interval.unwrap_or(5));

        info!("waiting for user authorization (polling every {}s)...", interval.as_secs());
        loop {
            if Instant::now() >= deadline {
                return Err(EnrollError::Expired);
            }
            tokio::time::sleep(interval).await;

            let resp = self
                .http
                .post(&url)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", auth.device_code.as_str()),
                    ("client_id", self.client_id.as_str()),
                ])
                .send()
                .await
                .map_err(|e| EnrollError::Network(url.clone(), e.to_string()))?;

            // Token endpoint returns 400 with an `error` field while pending.
            let body: TokenResp = resp
                .json()
                .await
                .map_err(|_| EnrollError::BadResponse("token".into()))?;

            if let Some(token) = body.access_token {
                info!("authorization granted");
                return Ok(token);
            }
            match body.error.as_deref() {
                Some("authorization_pending") => { /* keep polling */ }
                Some("slow_down") => {
                    // RFC 8628 §3.5: increase the interval by 5s on slow_down.
                    interval += Duration::from_secs(5);
                    warn!("server asked to slow down; interval now {}s", interval.as_secs());
                }
                Some("access_denied") => return Err(EnrollError::AccessDenied),
                Some("expired_token") => return Err(EnrollError::Expired),
                Some(other) => return Err(EnrollError::DeviceAuth(other.to_string())),
                None => return Err(EnrollError::BadResponse("token (no token, no error)".into())),
            }
        }
    }

    async fn enroll(&self, access_token: &str) -> Result<Enrollment, EnrollError> {
        let url = format!("{}/enroll", self.cloud_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| EnrollError::Network(url.clone(), e.to_string()))?;
        if !resp.status().is_success() {
            return Err(EnrollError::EnrollHttp(resp.status().as_u16()));
        }
        let r: EnrollResp = resp
            .json()
            .await
            .map_err(|_| EnrollError::BadResponse("enroll".into()))?;
        Ok(Enrollment {
            join_token: r.join_token,
            spire_endpoint: r.spire_endpoint,
            trust_bundle_pem: r.trust_bundle_pem,
            pinned_bundle_public_key: r.pinned_bundle_public_key,
            tenant_id: r.tenant_id,
            device_id: r.device_id,
            spiffe_id_hint: r.spiffe_id,
            cloud_url: r.cloud_url.unwrap_or_else(|| self.cloud_url.clone()),
        })
    }
}

/// User-facing instructions emitted once during the flow.
pub struct UserPrompt {
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub user_code: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct DeviceAuthResp {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    expires_in: u64,
    #[serde(default)]
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenResp {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EnrollResp {
    join_token: String,
    spire_endpoint: String,
    trust_bundle_pem: String,
    pinned_bundle_public_key: String,
    tenant_id: String,
    device_id: String,
    #[serde(default)]
    spiffe_id: Option<String>,
    #[serde(default)]
    cloud_url: Option<String>,
}
