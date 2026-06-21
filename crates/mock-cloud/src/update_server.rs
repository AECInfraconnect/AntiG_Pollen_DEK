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
    let expires_at = (now + chrono::Duration::days(30)).to_rfc3339();

    let metadata = json!({
        "signatures": [],
        "signed": {
            "_type": "targets",
            "version": 1,
            "expires": expires_at,
            "targets": {
                "dek-core-windows-amd64.exe": {
                    "hashes": {
                        "sha256": "dummy_hash_windows"
                    },
                    "length": 12345678,
                    "custom": {
                        "platform": "windows"
                    }
                },
                "dek-core-linux-amd64": {
                    "hashes": {
                        "sha256": "dummy_hash_linux"
                    },
                    "length": 12345678,
                    "custom": {
                        "platform": "linux"
                    }
                },
                "dek-core-macos-amd64": {
                    "hashes": {
                        "sha256": "dummy_hash_macos"
                    },
                    "length": 12345678,
                    "custom": {
                        "platform": "macos"
                    }
                }
            }
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
