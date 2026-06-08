use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/dek/v1/updates/:channel/metadata", get(get_update_metadata))
        .route("/api/dek/v1/updates/artifacts/:artifact_id", get(get_update_artifact))
}

async fn get_update_metadata(
    Path(channel): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let published_at = now.to_rfc3339();
    let expires_at = (now + chrono::Duration::days(30)).to_rfc3339();

    let metadata = json!({
        "schema_version": "pollen.update.v1",
        "channel": channel,
        "version": "1.4.2",
        "published_at": published_at,
        "expires_at": expires_at,
        "min_supported_version": "1.0.0",
        "rollout": {
            "percentage": 100,
            "tenant_allowlist": [],
            "device_allowlist": []
        },
        "artifacts": [
            {
                "os": "windows",
                "arch": "x86_64",
                "format": "msi",
                "url": "https://127.0.0.1:43891/api/dek/v1/updates/artifacts/win-msi-142",
                "sha256": "dummy_hash_windows",
                "size_bytes": 12345678,
                "signature": "dummy_signature_windows"
            },
            {
                "os": "linux",
                "arch": "x86_64",
                "format": "deb",
                "url": "https://127.0.0.1:43891/api/dek/v1/updates/artifacts/linux-deb-142",
                "sha256": "dummy_hash_linux",
                "size_bytes": 12345678,
                "signature": "dummy_signature_linux"
            }
        ],
        "security": {
            "signing_key_id": "release-key-2026-01",
            "threshold": 1,
            "signatures": ["dummy_meta_signature"]
        }
    });

    (StatusCode::OK, Json(metadata))
}

async fn get_update_artifact(
    Path(_artifact_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // In a real mock, we would stream the artifact binary.
    // For tests, we just return dummy content.
    (StatusCode::OK, "DUMMY_ARTIFACT_DATA")
}
