use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dek_agent_observer::model::AgentObservationEvent;
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/observations", post(ingest_observation))
        .route("/v1/tenants/:tenant/observations/costs", get(cost_summary))
}

async fn ingest_observation(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
    Json(event): Json<AgentObservationEvent>,
) -> impl IntoResponse {
    let mut ev = event;
    ev.tenant_id = tenant.clone();

    // 1. Correlate Shadow Candidates
    dek_agent_observer::correlate::correlate_shadow_candidate(&mut ev);

    // 2. Insert to DB
    if let Err(e) = state
        .observability_store
        .insert_observation_event(&ev)
        .await
    {
        tracing::error!("Failed to insert observation: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        );
    }

    // 3. Calculate Cost Ledger Entry
    // We mock a price catalog for now
    let mut catalog_providers = std::collections::HashMap::new();
    let mut openai_models = std::collections::HashMap::new();
    openai_models.insert(
        "gpt-4o".into(),
        dek_agent_observer::cost::ModelPrice {
            input_per_1m: 5.0,
            output_per_1m: 15.0,
        },
    );
    catalog_providers.insert("openai".into(), openai_models);

    let catalog = dek_agent_observer::cost::PriceCatalog {
        catalog_version: "2026-06".into(),
        currency: "USD".into(),
        providers: catalog_providers,
    };

    // Extract provider heuristic (e.g. from token_usage model or payload)
    // For now we assume openai if there is a model.
    if let Some(cost_entry) = dek_agent_observer::cost::calculate_cost(&ev, "openai", &catalog) {
        if let Err(e) = state
            .observability_store
            .insert_cost_ledger(&cost_entry)
            .await
        {
            tracing::error!("Failed to insert cost ledger: {}", e);
        }
    }

    // 4. Generate Policy Suggestions
    // We mock passing all events by just passing this one event
    if let Ok(suggestions) =
        dek_policy_suggester::api::generate_suggestions(&tenant, &[], &[ev.clone()])
    {
        for sug in suggestions {
            if let Err(e) = state
                .observability_store
                .upsert_policy_suggestion(&sug)
                .await
            {
                tracing::error!("Failed to upsert suggestion: {}", e);
            }
        }
    }

    (StatusCode::CREATED, Json(json!({"status": "ingested"})))
}

async fn cost_summary(
    State(state): State<AppState>,
    Path(tenant): Path<String>,
) -> impl IntoResponse {
    let ledger_entries = match state.observability_store.list_cost_ledger().await {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!("Failed to fetch cost ledger: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    let mut total_cost = 0.0;
    let mut total_tokens = 0;
    let mut provider_costs = std::collections::HashMap::new();

    for entry in ledger_entries {
        // Simple mock to filter by tenant if we had tenant_id in cost_ledger
        // For now we aggregate all since local-control-plane is mostly single-tenant in demo.
        total_cost += entry.total_cost;
        total_tokens += entry.total_tokens;
        *provider_costs.entry(entry.provider).or_insert(0.0) += entry.total_cost;
    }

    let result = json!({
        "schema_version": "cost-summary.v1",
        "tenant_id": tenant,
        "period": "current_month",
        "total_estimated_cost_usd": total_cost,
        "total_tokens": total_tokens,
        "provider_breakdown": provider_costs,
        "agent_breakdown": {
            "marketing_agent": total_cost * 0.7,
            "support_agent": total_cost * 0.3
        }
    });

    (StatusCode::OK, Json(result))
}
