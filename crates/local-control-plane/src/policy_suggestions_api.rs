use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::{
    error::{ApiError, ApiResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant/policy-suggestions",
            get(list_suggestions),
        )
        .route(
            "/v1/tenants/:tenant/policy-suggestions/generate",
            post(generate),
        )
        .route(
            "/v1/tenants/:tenant/policy-suggestions/:suggestion_id/create-draft",
            post(create_draft),
        )
        .route(
            "/v1/tenants/:tenant/policy-suggestions/:suggestion_id/simulate",
            post(simulate),
        )
        .route(
            "/v1/tenants/:tenant/policy-suggestions/:suggestion_id/dismiss",
            post(dismiss),
        )
        .route(
            "/v1/tenants/:tenant/policy-suggestions/:suggestion_id/approve",
            post(approve),
        )
}

async fn list_suggestions(
    Path(tenant): Path<String>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let items = st
        .registry_store
        .list_raw(&tenant, "policy_suggestion")
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(serde_json::json!({
        "schema_version": "policy-suggestions-list.v1",
        "items": items // 5.1 Fix list response format
    })))
}

async fn generate(
    Path(tenant): Path<String>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let raw_candidates = st
        .registry_store
        .list_raw(&tenant, "discovery_candidate")
        .await
        .map_err(ApiError::Internal)?;

    let mut candidates = vec![];
    for raw in raw_candidates {
        if let Ok(c) = serde_json::from_value(raw) {
            candidates.push(c);
        }
    }

    let raw_events = st
        .registry_store
        .list_raw(&tenant, "obs")
        .await
        .unwrap_or_default();

    let mut events = vec![];
    for raw in raw_events {
        if let Ok(e) = serde_json::from_value(raw) {
            events.push(e);
        }
    }

    let suggestions =
        dek_policy_suggester::api::generate_suggestions(&tenant, &candidates, &events)
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("failed generation")))?;

    for s in &suggestions {
        if let Ok(v) = serde_json::to_value(s) {
            let _ = st
                .registry_store
                .upsert_raw(&tenant, "policy_suggestion", &s.suggestion_id, &v)
                .await;
        }
    }
    Ok(Json(serde_json::json!({
        "schema_version": "generate-suggestions-response.v1",
        "generated_count": suggestions.len()
    })))
}

async fn create_draft(
    Path((tenant, suggestion_id)): Path<(String, String)>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    let raw = st
        .registry_store
        .get_raw(&tenant, "policy_suggestion", &suggestion_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound(suggestion_id.clone()))?;

    let suggestion: dek_policy_suggester::model::PolicySuggestion =
        serde_json::from_value(raw).map_err(|_| ApiError::Internal(anyhow::anyhow!("bad data")))?;

    // Create a policy draft from the suggestion
    let draft_id = format!("draft-{}", uuid::Uuid::new_v4());
    
    // For simplicity, take the first artifact if any
    let source = if let Some(artifact) = suggestion.artifacts.first() {
        dek_control_plane_api::policy::PolicySource::RawText {
            language: format!("{:?}", artifact.language).to_lowercase(),
            text: artifact.content.clone(),
        }
    } else {
        return Err(ApiError::Internal(anyhow::anyhow!("No artifacts to create draft from")));
    };

    let draft = dek_control_plane_api::policy::PolicyDraft {
        meta: dek_control_plane_api::registry::ObjectMeta {
            schema_version: "v1".into(),
            tenant_id: tenant.clone(),
            workspace_id: "default".into(),
            environment_id: "default".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            created_by: "system".into(),
            updated_by: "system".into(),
            source: dek_control_plane_api::registry::RegistrationSource::Discovery,
            status: dek_control_plane_api::registry::RegistryStatus::Draft,
            tags: vec![],
        },
        policy_id: draft_id.clone(),
        name: format!("Draft from {}", suggestion.title),
        description: Some(suggestion.summary.clone()),
        policy_type: dek_control_plane_api::policy::PolicyType::Cedar,
        targets: dek_control_plane_api::policy::PolicyTargets {
            agent_ids: suggestion.target_agent_id.map(|id| vec![id]).unwrap_or_default(),
            tool_ids: suggestion.target_tool_id.map(|id| vec![id]).unwrap_or_default(),
            resource_ids: suggestion.target_resource_id.map(|id| vec![id]).unwrap_or_default(),
            entity_ids: vec![],
            route_ids: vec![],
        },
        source,
        compile_options: dek_control_plane_api::policy::PolicyCompileOptions {
            optimization_level: None,
            fail_on_warnings: Some(true),
        },
    };

    st.policy_store
        .upsert_policy(draft)
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(serde_json::json!({
        "draft_id": draft_id
    })))
}

async fn simulate(
    Path((_tenant, _suggestion_id)): Path<(String, String)>,
    State(_st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // Mock simulation
    Ok(Json(serde_json::json!({
        "blocked_events": 5,
        "allowed_events": 95,
        "impact": "Low"
    })))
}

async fn dismiss(
    Path((tenant, suggestion_id)): Path<(String, String)>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    update_suggestion_status(&tenant, &suggestion_id, &st, dek_policy_suggester::model::SuggestionStatus::Dismissed).await
}

async fn approve(
    Path((tenant, suggestion_id)): Path<(String, String)>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    update_suggestion_status(&tenant, &suggestion_id, &st, dek_policy_suggester::model::SuggestionStatus::Approved).await
}

async fn update_suggestion_status(
    tenant: &str,
    suggestion_id: &str,
    st: &AppState,
    status: dek_policy_suggester::model::SuggestionStatus,
) -> ApiResult<Json<serde_json::Value>> {
    let raw = st
        .registry_store
        .get_raw(tenant, "policy_suggestion", suggestion_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound(suggestion_id.to_string()))?;

    let mut suggestion: dek_policy_suggester::model::PolicySuggestion =
        serde_json::from_value(raw).map_err(|_| ApiError::Internal(anyhow::anyhow!("bad data")))?;

    suggestion.status = status;

    if let Ok(v) = serde_json::to_value(&suggestion) {
        st.registry_store
            .upsert_raw(tenant, "policy_suggestion", suggestion_id, &v)
            .await
            .map_err(ApiError::Internal)?;
    }

    Ok(Json(serde_json::json!({"success": true})))
}
