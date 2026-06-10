use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dek_control_plane_api::policy::*;
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant_id/policies",
            get(list_policies).post(create_policy),
        )
        .route(
            "/v1/tenants/:tenant_id/policies/:policy_id",
            get(get_policy).patch(patch_policy).delete(delete_policy),
        )
        .route(
            "/v1/tenants/:tenant_id/policies/:policy_id/publish",
            post(publish_policy),
        )
}

async fn list_policies(
    Path(_tenant_id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // TODO: implement with policy store
    (StatusCode::OK, Json(json!([])))
}

async fn create_policy(
    Path(_tenant_id): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<PolicyDraft>,
) -> impl IntoResponse {
    (StatusCode::CREATED, Json(json!(payload)))
}

async fn get_policy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn patch_policy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn delete_policy() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not implemented"})),
    )
}

async fn publish_policy(
    Path((tenant, _policy_id)): Path<(String, String)>,
    State(st): State<AppState>,
    Json(draft): Json<PolicyDraft>,
) -> impl IntoResponse {
    let build_number = st.build_number.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // 1. "Compile" draft into compiled artifacts
    let mut compiled = vec![];
    if let dek_control_plane_api::policy::PolicySource::RawText { language, text } = &draft.source {
        compiled.push(crate::bundle::CompiledArtifact {
            artifact_id: draft.name.clone(),
            adapter_id: language.clone(),
            artifact_type: format!("{}_text", language),
            bytes: text.as_bytes().to_vec(),
        });
    } else {
        compiled.push(crate::bundle::CompiledArtifact {
            artifact_id: draft.name.clone(),
            adapter_id: "cedar".into(),
            artifact_type: "cedar_text".into(),
            bytes: b"permit(principal,action,resource);".to_vec(),
        });
    }

    // 2. Snapshot registry
    let agents = st.registry_store.list_agents(&tenant).await.unwrap_or_default();
    let registry_snap = serde_json::json!({ "agents": agents });

    // 3. Build & Sign Bundle
    let built = crate::bundle::build_signed_bundle(
        &st.signer,
        &tenant,
        "default",
        "local",
        build_number,
        compiled,
        &registry_snap,
        &serde_json::json!({}),
        None,
    )
    .await
    .unwrap();

    // 4. Store raw payload and blobs
    st.policy_store
        .upsert_policy_raw(
            &tenant,
            "bundle:latest",
            &serde_json::to_value(&built.manifest).unwrap(),
        )
        .await
        .unwrap();
    for (path, bytes) in built.blobs {
        st.policy_store.put_blob(&tenant, &path, &bytes).await.unwrap();
    }

    (StatusCode::OK, Json(json!({
        "published": true,
        "bundle_id": built.manifest.bundle_id,
        "manifest": built.manifest
    })))
}
