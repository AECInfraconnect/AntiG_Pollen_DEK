use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use dek_secure_spool::{
    crypto::{AeadKey, EncryptedRecord, RecordAad},
    key_manager::OsKeyStore,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserveInputKind {
    ProviderUsageKey,
    LocalUsageLogPath,
    CloudReadRole,
    OauthReadToken,
    ProxyCaTrust,
    ProviderAdminWrite,
}

impl ObserveInputKind {
    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "provider_usage_key" => Some(Self::ProviderUsageKey),
            "local_usage_log_path" => Some(Self::LocalUsageLogPath),
            "cloud_read_role" => Some(Self::CloudReadRole),
            "oauth_read_token" => Some(Self::OauthReadToken),
            "proxy_ca_trust" => Some(Self::ProxyCaTrust),
            "provider_admin_write" => Some(Self::ProviderAdminWrite),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ProviderUsageKey => "provider_usage_key",
            Self::LocalUsageLogPath => "local_usage_log_path",
            Self::CloudReadRole => "cloud_read_role",
            Self::OauthReadToken => "oauth_read_token",
            Self::ProxyCaTrust => "proxy_ca_trust",
            Self::ProviderAdminWrite => "provider_admin_write",
        }
    }

    fn is_high_risk(self) -> bool {
        matches!(self, Self::ProxyCaTrust | Self::ProviderAdminWrite)
    }

    fn unlocks(self) -> Vec<String> {
        match self {
            Self::ProviderUsageKey => vec![
                "Billed usage reconciliation when a provider connector supports this key"
                    .to_string(),
                "Clear difference between estimated local spend and provider-billed spend"
                    .to_string(),
            ],
            Self::LocalUsageLogPath => vec![
                "Exact token/cost events when local session logs contain provider usage objects"
                    .to_string(),
                "Better per-agent attribution for Codex, Claude, Gemini, Cursor, and similar local logs"
                    .to_string(),
            ],
            Self::CloudReadRole => vec![
                "Read-only cloud-side activity proof for SaaS agents and provider dashboards"
                    .to_string(),
            ],
            Self::OauthReadToken => vec![
                "Read-only connector visibility for email, calendar, or workspace actions"
                    .to_string(),
            ],
            Self::ProxyCaTrust => vec![
                "Response usage capture for selected agent traffic routed through an approved proxy"
                    .to_string(),
            ],
            Self::ProviderAdminWrite => vec![
                "Provider-side allow/block settings where a provider supports admin controls"
                    .to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserveCredentialRequest {
    pub kind: ObserveInputKind,
    pub title: String,
    pub what_we_ask: String,
    pub why: String,
    pub unlocks: Vec<String>,
    pub risk_level: String,
    pub required_scope: String,
    pub least_privilege_tip: String,
    pub data_handling: Vec<String>,
    pub supported_now: bool,
}

impl ObserveCredentialRequest {
    fn for_kind(kind: ObserveInputKind) -> Self {
        let data_handling = vec![
            "Stored locally only; nothing is sent to Pollek Cloud by this endpoint".to_string(),
            "Secret-like values are encrypted at rest and never returned by the API".to_string(),
            "UI only shows a redacted preview and a revocation button".to_string(),
            "Consent metadata is kept so the user can audit what was connected".to_string(),
        ];
        match kind {
            ObserveInputKind::ProviderUsageKey => Self {
                kind,
                title: "Connect read-only provider usage key".to_string(),
                what_we_ask: "A read-only usage, billing, or cost API key. Do not enter a primary account password or write/admin key.".to_string(),
                why: "Provider billing APIs are the only way to reconcile local estimates with provider-billed totals when an AI app does not expose usage locally.".to_string(),
                unlocks: kind.unlocks(),
                risk_level: "medium".to_string(),
                required_scope: "usage:read or billing:read only".to_string(),
                least_privilege_tip: "Create a dedicated key scoped to usage/cost reporting and rotate it if it is exposed.".to_string(),
                data_handling,
                supported_now: true,
            },
            ObserveInputKind::LocalUsageLogPath => Self {
                kind,
                title: "Add local usage log path".to_string(),
                what_we_ask: "A file or folder path that contains local JSON, JSONL, NDJSON, or log files with provider usage objects.".to_string(),
                why: "Pollek can parse exact token usage from local logs without reading prompts or completions when the log contains a provider usage object.".to_string(),
                unlocks: kind.unlocks(),
                risk_level: "low".to_string(),
                required_scope: "Read access to the selected local path".to_string(),
                least_privilege_tip: "Prefer a narrow session or usage log folder instead of an entire home directory.".to_string(),
                data_handling,
                supported_now: true,
            },
            ObserveInputKind::CloudReadRole => Self {
                kind,
                title: "Connect cloud read-only role".to_string(),
                what_we_ask: "A read-only role, token, or service principal for a cloud provider or SaaS control plane.".to_string(),
                why: "Some agents run tasks in cloud systems that local OS sensors cannot fully see.".to_string(),
                unlocks: kind.unlocks(),
                risk_level: "medium".to_string(),
                required_scope: "audit:read, activity:read, or equivalent only".to_string(),
                least_privilege_tip: "Use a role limited to audit/activity APIs and the smallest tenant/project scope.".to_string(),
                data_handling,
                supported_now: false,
            },
            ObserveInputKind::OauthReadToken => Self {
                kind,
                title: "Connect OAuth read-only token".to_string(),
                what_we_ask: "A read-only OAuth grant for a specific connector such as email, calendar, or documents.".to_string(),
                why: "Connector-level observation needs provider consent and should not scrape user content without a clear purpose.".to_string(),
                unlocks: kind.unlocks(),
                risk_level: "medium".to_string(),
                required_scope: "read-only metadata/activity scopes only".to_string(),
                least_privilege_tip: "Choose metadata/activity scopes before content scopes. Avoid full mailbox or drive access unless required.".to_string(),
                data_handling,
                supported_now: false,
            },
            ObserveInputKind::ProxyCaTrust => Self {
                kind,
                title: "Proxy trust setup".to_string(),
                what_we_ask: "Explicit approval to trust a local proxy certificate for selected AI-agent traffic.".to_string(),
                why: "Traffic inspection can improve exact response-usage capture, but it is sensitive and must be scoped to the agent path.".to_string(),
                unlocks: kind.unlocks(),
                risk_level: "high".to_string(),
                required_scope: "Agent-scoped local proxy only".to_string(),
                least_privilege_tip: "Use a per-agent proxy profile. Do not install a broad device-wide CA unless you understand the risk.".to_string(),
                data_handling,
                supported_now: false,
            },
            ObserveInputKind::ProviderAdminWrite => Self {
                kind,
                title: "Provider admin write key".to_string(),
                what_we_ask: "A provider admin key that can change provider-side enforcement settings.".to_string(),
                why: "This is only for enterprise provider-side enforcement, not normal observe. Pollek should prefer read-only observe first.".to_string(),
                unlocks: kind.unlocks(),
                risk_level: "high".to_string(),
                required_scope: "Narrow policy/admin write scope for a test tenant only".to_string(),
                least_privilege_tip: "Avoid this unless provider-side enforcement is required and approved by an administrator.".to_string(),
                data_handling,
                supported_now: false,
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObserveInputRecord {
    pub input_id: String,
    pub kind: ObserveInputKind,
    pub label: String,
    pub provider: Option<String>,
    pub redacted_preview: String,
    pub fingerprint: String,
    pub scope_note: Option<String>,
    pub connected_at: String,
    pub updated_at: String,
    pub consent_statement: String,
    pub status: String,
    pub encrypted_value: EncryptedRecord,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObserveInputPublic {
    pub input_id: String,
    pub kind: ObserveInputKind,
    pub label: String,
    pub provider: Option<String>,
    pub redacted_preview: String,
    pub fingerprint: String,
    pub scope_note: Option<String>,
    pub connected_at: String,
    pub updated_at: String,
    pub consent_statement: String,
    pub status: String,
    pub unlocks: Vec<String>,
}

impl From<&ObserveInputRecord> for ObserveInputPublic {
    fn from(value: &ObserveInputRecord) -> Self {
        Self {
            input_id: value.input_id.clone(),
            kind: value.kind,
            label: value.label.clone(),
            provider: value.provider.clone(),
            redacted_preview: value.redacted_preview.clone(),
            fingerprint: value.fingerprint.clone(),
            scope_note: value.scope_note.clone(),
            connected_at: value.connected_at.clone(),
            updated_at: value.updated_at.clone(),
            consent_statement: value.consent_statement.clone(),
            status: value.status.clone(),
            unlocks: value.kind.unlocks(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct ObserveAccuracyState {
    inputs: Vec<ObserveInputRecord>,
}

#[derive(Clone)]
pub struct ObserveAccuracyStore {
    path: PathBuf,
    key: Arc<AeadKey>,
    state: Arc<RwLock<ObserveAccuracyState>>,
}

impl ObserveAccuracyStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let dir = data_dir.into().join("observe_accuracy");
        std::fs::create_dir_all(&dir)?;
        let key_path = dir.join("master.key");
        let os_store = dek_secure_spool::os::DefaultOsKeyStore::new(key_path);
        let key_bytes = os_store.load_or_create_master_key()?;
        let path = dir.join("inputs.json");
        let state = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            ObserveAccuracyState::default()
        };

        Ok(Self {
            path,
            key: Arc::new(AeadKey::new("observe-accuracy-local-v1", key_bytes)),
            state: Arc::new(RwLock::new(state)),
        })
    }

    pub async fn list_public(&self) -> Vec<ObserveInputPublic> {
        self.state
            .read()
            .await
            .inputs
            .iter()
            .map(ObserveInputPublic::from)
            .collect()
    }

    pub async fn local_usage_log_paths(&self) -> Vec<PathBuf> {
        let records: Vec<ObserveInputRecord> = self
            .state
            .read()
            .await
            .inputs
            .iter()
            .filter(|record| record.kind == ObserveInputKind::LocalUsageLogPath)
            .cloned()
            .collect();

        records
            .into_iter()
            .filter_map(|record| self.decrypt_value(&record).ok())
            .map(PathBuf::from)
            .collect()
    }

    pub async fn upsert(
        &self,
        tenant: &str,
        payload: StoreObserveInputRequest,
    ) -> ApiResult<ObserveInputPublic> {
        let request = ObserveCredentialRequest::for_kind(payload.kind);
        if !request.supported_now {
            return Err(ApiError::BadRequest(format!(
                "{} is reserved for a future connector/runtime and is not enabled yet",
                payload.kind.as_str()
            )));
        }
        if payload.kind.is_high_risk() {
            return Err(ApiError::BadRequest(
                "high-risk credential types require an enterprise connector workflow".to_string(),
            ));
        }
        if !payload.consent_ack {
            return Err(ApiError::BadRequest(
                "explicit user consent is required before storing an observe accuracy input"
                    .to_string(),
            ));
        }
        let value = payload.input_value.trim();
        if value.is_empty() {
            return Err(ApiError::BadRequest(
                "input value must not be empty".to_string(),
            ));
        }
        if matches!(payload.kind, ObserveInputKind::LocalUsageLogPath) {
            let path = PathBuf::from(value);
            if !path.exists() {
                return Err(ApiError::BadRequest(format!(
                    "local usage log path does not exist: {}",
                    redact_path_for_error(value)
                )));
            }
        }

        let now = Utc::now().to_rfc3339();
        let input_id = payload
            .input_id
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| format!("input_{}", Uuid::new_v4()));
        let encrypted_value = self.encrypt_value(tenant, &input_id, value.as_bytes())?;
        let record = ObserveInputRecord {
            input_id: input_id.clone(),
            kind: payload.kind,
            label: payload.label.unwrap_or_else(|| request.title.clone()),
            provider: payload.provider.filter(|value| !value.trim().is_empty()),
            redacted_preview: redacted_preview(payload.kind, value),
            fingerprint: fingerprint(payload.kind, value),
            scope_note: payload.scope_note.filter(|value| !value.trim().is_empty()),
            connected_at: now.clone(),
            updated_at: now,
            consent_statement: payload.consent_statement.unwrap_or_else(|| {
                format!(
                    "User approved local observe accuracy input with {} scope.",
                    request.required_scope
                )
            }),
            status: match payload.kind {
                ObserveInputKind::LocalUsageLogPath => "active_for_local_observe".to_string(),
                ObserveInputKind::ProviderUsageKey => {
                    "stored_for_read_only_billing_reconciliation".to_string()
                }
                _ => "stored".to_string(),
            },
            encrypted_value,
        };

        {
            let mut state = self.state.write().await;
            state.inputs.retain(|existing| {
                existing.input_id != input_id
                    && !(existing.kind == record.kind && existing.fingerprint == record.fingerprint)
            });
            state.inputs.push(record.clone());
        }
        self.save().await?;
        Ok(ObserveInputPublic::from(&record))
    }

    pub async fn revoke(&self, input_id: &str) -> ApiResult<bool> {
        let removed = {
            let mut state = self.state.write().await;
            let before = state.inputs.len();
            state.inputs.retain(|record| record.input_id != input_id);
            before != state.inputs.len()
        };
        if removed {
            self.save().await?;
        }
        Ok(removed)
    }

    async fn save(&self) -> ApiResult<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| ApiError::Internal(err.into()))?;
        }
        let snapshot = self.state.read().await.clone();
        let content =
            serde_json::to_vec_pretty(&snapshot).map_err(|err| ApiError::Internal(err.into()))?;
        tokio::fs::write(&self.path, content)
            .await
            .map_err(|err| ApiError::Internal(err.into()))
    }

    fn encrypt_value(
        &self,
        tenant: &str,
        input_id: &str,
        value: &[u8],
    ) -> ApiResult<EncryptedRecord> {
        let aad = RecordAad {
            schema: "pollek.observe_accuracy.input.v1".to_string(),
            tenant_id: tenant.to_string(),
            device_id: "local-device".to_string(),
            segment_id: input_id.to_string(),
            seq: 0,
            key_id: self.key.key_id().to_string(),
            alg: "AES-256-GCM".to_string(),
        };
        self.key.encrypt_record(aad, value).map_err(|err| {
            ApiError::Internal(anyhow::anyhow!("failed to encrypt observe input: {err}"))
        })
    }

    fn decrypt_value(&self, record: &ObserveInputRecord) -> ApiResult<String> {
        let bytes = self
            .key
            .decrypt_record(&record.encrypted_value)
            .map_err(|err| {
                ApiError::Internal(anyhow::anyhow!(
                    "failed to decrypt observe input {}: {err}",
                    record.input_id
                ))
            })?;
        String::from_utf8(bytes).map_err(|err| {
            ApiError::Internal(anyhow::anyhow!("invalid observe input utf-8: {err}"))
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct StoreObserveInputRequest {
    pub kind: ObserveInputKind,
    pub input_value: String,
    pub input_id: Option<String>,
    pub label: Option<String>,
    pub provider: Option<String>,
    pub scope_note: Option<String>,
    pub consent_ack: bool,
    pub consent_statement: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ObserveAccuracyResponse {
    pub schema_version: String,
    pub tenant_id: String,
    pub generated_at: String,
    pub active_level: String,
    pub active_level_label: String,
    pub ladder: Vec<serde_json::Value>,
    pub inputs: Vec<ObserveInputPublic>,
    pub available_requests: Vec<ObserveCredentialRequest>,
    pub suggested_local_log_paths: Vec<SuggestedLocalLogPath>,
    pub data_handling: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuggestedLocalLogPath {
    pub label: String,
    pub path: String,
    pub redacted_path: String,
    pub exists: bool,
    pub reason: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/observe/accuracy", get(get_accuracy))
        .route(
            "/v1/tenants/:tenant/observe/accuracy/requests/:kind",
            get(get_request),
        )
        .route(
            "/v1/tenants/:tenant/observe/accuracy/inputs",
            post(store_input),
        )
        .route(
            "/v1/tenants/:tenant/observe/accuracy/inputs/:input_id",
            delete(revoke_input),
        )
}

async fn get_accuracy(
    Path(tenant): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ObserveAccuracyResponse>> {
    let inputs = state.observe_accuracy_store.list_public().await;
    let has_provider_usage = inputs
        .iter()
        .any(|input| input.kind == ObserveInputKind::ProviderUsageKey);
    let has_log_path = inputs
        .iter()
        .any(|input| input.kind == ObserveInputKind::LocalUsageLogPath);
    let (active_level, active_level_label) = if has_provider_usage {
        (
            "provider_billed_ready".to_string(),
            "Ready for provider-billed reconciliation".to_string(),
        )
    } else if has_log_path {
        (
            "response_usage_plus_local_logs".to_string(),
            "Exact when local logs expose provider usage".to_string(),
        )
    } else {
        (
            "estimated_plus_response_usage".to_string(),
            "Estimated fallback; exact when response usage is observed".to_string(),
        )
    };

    Ok(Json(ObserveAccuracyResponse {
        schema_version: "pollek.observe_accuracy.v1".to_string(),
        tenant_id: tenant,
        generated_at: Utc::now().to_rfc3339(),
        active_level,
        active_level_label,
        ladder: accuracy_ladder(&inputs),
        inputs,
        available_requests: [
            ObserveInputKind::LocalUsageLogPath,
            ObserveInputKind::ProviderUsageKey,
            ObserveInputKind::CloudReadRole,
            ObserveInputKind::OauthReadToken,
            ObserveInputKind::ProxyCaTrust,
            ObserveInputKind::ProviderAdminWrite,
        ]
        .into_iter()
        .map(ObserveCredentialRequest::for_kind)
        .collect(),
        suggested_local_log_paths: suggested_local_log_paths(),
        data_handling: vec![
            "Observe first; ask the user only when extra input materially improves accuracy"
                .to_string(),
            "Use read-only scopes; never ask for primary passwords or broad admin keys".to_string(),
            "Secret-like values are encrypted locally and redacted from API responses and logs"
                .to_string(),
            "Each input is revocable from the local dashboard".to_string(),
        ],
        next_steps: next_steps(has_provider_usage, has_log_path),
    }))
}

async fn get_request(
    Path((_tenant, kind)): Path<(String, String)>,
) -> Result<Json<ObserveCredentialRequest>, (StatusCode, Json<serde_json::Value>)> {
    let Some(kind) = ObserveInputKind::from_str(&kind) else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "unknown observe input kind"})),
        ));
    };
    Ok(Json(ObserveCredentialRequest::for_kind(kind)))
}

async fn store_input(
    Path(tenant): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<StoreObserveInputRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let input = state
        .observe_accuracy_store
        .upsert(&tenant, payload)
        .await?;
    Ok(Json(json!({
        "schema_version": "pollek.observe_accuracy.input.v1",
        "input": input,
        "message": "Observe accuracy input saved locally. Run Observe Now to use newly added local log paths."
    })))
}

async fn revoke_input(
    Path((_tenant, input_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let removed = state.observe_accuracy_store.revoke(&input_id).await?;
    if !removed {
        return Err(ApiError::NotFound(format!(
            "observe accuracy input {input_id} was not found"
        )));
    }
    Ok(Json(json!({
        "schema_version": "pollek.observe_accuracy.revoke.v1",
        "input_id": input_id,
        "revoked": true
    })))
}

fn accuracy_ladder(inputs: &[ObserveInputPublic]) -> Vec<serde_json::Value> {
    let has_provider_usage = inputs
        .iter()
        .any(|input| input.kind == ObserveInputKind::ProviderUsageKey);
    let has_log_path = inputs
        .iter()
        .any(|input| input.kind == ObserveInputKind::LocalUsageLogPath);
    vec![
        json!({
            "level": "estimated",
            "label": "Estimated",
            "status": "available",
            "description": "Uses local metadata, response sizes, model/catalog prices, or presence signals when exact usage is not visible."
        }),
        json!({
            "level": "response_usage",
            "label": "Response usage",
            "status": if has_log_path { "strengthened" } else { "supported" },
            "description": "Exact when provider usage objects arrive through SDK wrappers, proxies, browser extensions, telemetry, or local usage logs."
        }),
        json!({
            "level": "provider_billed",
            "label": "Provider billed",
            "status": if has_provider_usage { "input_connected" } else { "needs_read_only_input" },
            "description": "Reconciles local usage with provider billing/usage APIs when a read-only provider usage source is connected."
        }),
    ]
}

fn next_steps(has_provider_usage: bool, has_log_path: bool) -> Vec<String> {
    let mut steps = Vec::new();
    if !has_log_path {
        steps.push(
            "Add a narrow local usage log path if your AI app writes JSON/JSONL usage metadata."
                .to_string(),
        );
    }
    if !has_provider_usage {
        steps.push(
            "Connect a read-only provider usage key when you need billed totals, not only local estimates."
                .to_string(),
        );
    }
    if steps.is_empty() {
        steps.push(
            "Run Observe Now, then inspect the usage ledger for exact vs estimated labels."
                .to_string(),
        );
    }
    steps
}

fn suggested_local_log_paths() -> Vec<SuggestedLocalLogPath> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    if home.is_empty() {
        return Vec::new();
    }
    let mut candidates: Vec<(&str, PathBuf, &str)> = vec![
        (
            "Codex sessions",
            PathBuf::from(&home).join(".codex").join("sessions"),
            "Codex often writes local session metadata here.",
        ),
        (
            "Codex logs",
            PathBuf::from(&home).join(".codex").join("logs"),
            "Codex or wrappers may write structured logs here.",
        ),
        (
            "ChatGPT desktop",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("ChatGPT"),
            "ChatGPT desktop metadata may appear here on Windows.",
        ),
        (
            "OpenAI local state",
            PathBuf::from(&home).join(".openai"),
            "OpenAI SDK or CLI-style local config and telemetry may appear here.",
        ),
        (
            "Claude local data",
            PathBuf::from(&home).join(".claude"),
            "Claude CLI/Desktop local metadata may be found here when installed.",
        ),
        (
            "Claude desktop",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("Claude"),
            "Claude Desktop app data may appear here on Windows.",
        ),
        (
            "Claude Code project data",
            PathBuf::from(&home).join(".claude").join("projects"),
            "Claude Code project transcripts or metadata may be organized here.",
        ),
        (
            "Gemini local data",
            PathBuf::from(&home).join(".gemini"),
            "Gemini CLI/Antigravity-related local metadata may be found here when installed.",
        ),
        (
            "Google AI Studio local data",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("Google")
                .join("AI Studio"),
            "Google AI Studio or related desktop/browser metadata may appear here.",
        ),
        (
            "Antigravity local data",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("Antigravity"),
            "Antigravity/Gemini agent local metadata may appear here when installed.",
        ),
        (
            "DeepSeek local data",
            PathBuf::from(&home).join(".deepseek"),
            "DeepSeek CLI, SDK, or local app metadata may appear here when configured.",
        ),
        (
            "Manus local data",
            PathBuf::from(&home).join(".manus"),
            "Manus AI desktop or browser-operator metadata may appear here when available.",
        ),
        (
            "Continue/Cursor style IDE data",
            PathBuf::from(&home).join(".continue"),
            "IDE agent logs may expose usage metadata when configured.",
        ),
        (
            "Cursor local data",
            PathBuf::from(&home).join(".cursor"),
            "Cursor or IDE assistant metadata may be found here when installed.",
        ),
        (
            "Cline local data",
            PathBuf::from(&home).join(".cline"),
            "Cline/Roo-style VS Code agents may keep local metadata here.",
        ),
        (
            "Roo Code local data",
            PathBuf::from(&home).join(".roo"),
            "Roo Code agent metadata may appear here when installed.",
        ),
        (
            "VS Code global storage",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("Code")
                .join("User")
                .join("globalStorage"),
            "Many AI coding extensions store local state under VS Code globalStorage.",
        ),
        (
            "VS Code logs",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("Code")
                .join("logs"),
            "VS Code extension logs may contain tool and model usage metadata.",
        ),
        (
            "JetBrains IDE logs",
            PathBuf::from(&home)
                .join("AppData")
                .join("Local")
                .join("JetBrains"),
            "JetBrains AI assistant or plugin logs may be under product-specific folders here.",
        ),
        (
            "Ollama local data",
            PathBuf::from(&home).join(".ollama"),
            "Local model usage and config for Ollama may appear here.",
        ),
        (
            "LM Studio local data",
            PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("LM Studio"),
            "LM Studio local model server/app metadata may appear here.",
        ),
    ];

    if cfg!(target_os = "macos") {
        candidates.extend([
            (
                "ChatGPT macOS app",
                PathBuf::from(&home)
                    .join("Library")
                    .join("Application Support")
                    .join("ChatGPT"),
                "ChatGPT desktop app support data may appear here on macOS.",
            ),
            (
                "Claude macOS app",
                PathBuf::from(&home)
                    .join("Library")
                    .join("Application Support")
                    .join("Claude"),
                "Claude Desktop app support data may appear here on macOS.",
            ),
            (
                "VS Code macOS global storage",
                PathBuf::from(&home)
                    .join("Library")
                    .join("Application Support")
                    .join("Code")
                    .join("User")
                    .join("globalStorage"),
                "AI coding extensions often store local state under VS Code globalStorage.",
            ),
        ]);
    }

    candidates
        .into_iter()
        .map(|(label, path, reason)| {
            let path_string = path.to_string_lossy().to_string();
            SuggestedLocalLogPath {
                label: label.to_string(),
                path: path_string.clone(),
                redacted_path: redact_path_for_error(&path_string),
                exists: path.exists(),
                reason: reason.to_string(),
            }
        })
        .collect()
}

fn fingerprint(kind: ObserveInputKind, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_str().as_bytes());
    hasher.update(b":");
    hasher.update(value.trim().as_bytes());
    format!("sha256:{}", hex::encode(&hasher.finalize()[..12]))
}

fn redacted_preview(kind: ObserveInputKind, value: &str) -> String {
    match kind {
        ObserveInputKind::LocalUsageLogPath => redact_path_for_error(value),
        _ => {
            let trimmed = value.trim();
            if trimmed.len() <= 8 {
                "********".to_string()
            } else {
                format!("{}...{}", &trimmed[..4], &trimmed[trimmed.len() - 4..])
            }
        }
    }
}

fn redact_path_for_error(value: &str) -> String {
    let mut redacted = value.trim().to_string();
    for key in ["USERPROFILE", "HOME"] {
        if let Ok(home) = std::env::var(key) {
            if !home.is_empty() {
                redacted = redacted.replace(&home, "<home>");
            }
        }
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_request_keeps_high_risk_paths_disabled() {
        let request = ObserveCredentialRequest::for_kind(ObserveInputKind::ProviderAdminWrite);
        assert!(!request.supported_now);
        assert_eq!(request.risk_level, "high");
        assert!(request.least_privilege_tip.contains("Avoid this"));
    }

    #[test]
    fn local_path_preview_redacts_home() {
        let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) else {
            return;
        };
        let path = format!("{home}\\AppData\\Roaming\\Antigravity");
        let redacted = redacted_preview(ObserveInputKind::LocalUsageLogPath, &path);
        assert!(redacted.contains("<home>"));
        assert!(!redacted.contains(&home));
    }

    #[test]
    fn suggested_paths_include_common_browser_and_agent_surfaces() {
        let labels: Vec<String> = suggested_local_log_paths()
            .into_iter()
            .map(|item| item.label)
            .collect();
        assert!(labels.iter().any(|label| label == "Codex sessions"));
        assert!(labels.iter().any(|label| label == "ChatGPT desktop"));
        assert!(labels
            .iter()
            .any(|label| label == "Claude Code project data"));
        assert!(labels.iter().any(|label| label == "Antigravity local data"));
        assert!(labels.iter().any(|label| label == "Manus local data"));
    }
}
