use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dek_control_plane_api::policy::*;
use dek_control_plane_api::policy::*;
use serde_json::json;

fn not_found() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_FOUND, Json(json!({"error": "not found"})))
}

fn internal_error(e: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))
}

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
        .route(
            "/v1/tenants/:tenant_id/policies/:policy_id/validate",
            post(validate_policy),
        )
        .route(
            "/v1/tenants/:tenant_id/policies/:policy_id/simulate",
            post(simulate_policy),
        )
}

async fn list_policies(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.policy_store.list_policies(&tenant_id).await {
        Ok(items) => (StatusCode::OK, Json(json!(items))),
        Err(e) => internal_error(e),
    }
}

async fn create_policy(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
    Json(mut payload): Json<PolicyDraft>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.policy_store.upsert_policy(payload.clone()).await {
        Ok(item) => (StatusCode::CREATED, Json(json!(item))),
        Err(e) => internal_error(e),
    }
}

async fn get_policy(
    Path((tenant_id, policy_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.policy_store.get_policy(&tenant_id, &policy_id).await {
        Ok(Some(item)) => (StatusCode::OK, Json(json!(item))),
        Ok(None) => not_found(),
        Err(e) => internal_error(e),
    }
}

async fn patch_policy(
    Path((tenant_id, _policy_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(mut payload): Json<PolicyDraft>,
) -> impl IntoResponse {
    payload.meta.tenant_id = tenant_id;
    match state.policy_store.upsert_policy(payload.clone()).await {
        Ok(item) => (StatusCode::OK, Json(json!(item))),
        Err(e) => internal_error(e),
    }
}

async fn delete_policy(
    Path((tenant_id, policy_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.policy_store.delete_policy(&tenant_id, &policy_id).await {
        Ok(true) => (StatusCode::NO_CONTENT, Json(json!({}))),
        Ok(false) => not_found(),
        Err(e) => internal_error(e),
    }
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

    // 5. Broadcast to SSE for hot-reload
    let _ = st.bundle_tx.send(built.manifest.bundle_id.clone());

    (StatusCode::OK, Json(json!({
        "published": true,
        "bundle_id": built.manifest.bundle_id,
        "manifest": built.manifest
    })))
}

async fn validate_policy(
    Path((tenant_id, policy_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = (tenant_id, policy_id, state);
    // Mock validate implementation
    (
        StatusCode::OK,
        Json(json!({ "is_valid": true, "errors": [] })),
    )
}

async fn simulate_policy(
    Path((tenant_id, policy_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> impl IntoResponse {
    let _ = (tenant_id, policy_id, state, input);
    // Mock simulate implementation
    (
        StatusCode::OK,
        Json(json!({
            "allowed": true,
            "evaluation_time_ms": 2,
            "log_output": ["mock simulate"]
        })),
    )
}

