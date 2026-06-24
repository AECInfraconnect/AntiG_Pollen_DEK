// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::state::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/chaos/outage", post(toggle_outage))
        .route("/admin/chaos/latency", post(set_latency))
}

#[derive(serde::Deserialize)]
pub struct OutageReq {
    pub enabled: bool,
}

async fn toggle_outage(
    State(state): State<AppState>,
    Json(req): Json<OutageReq>,
) -> impl IntoResponse {
    {
        let mut cfg = state.chaos_config.lock().unwrap(); //
        cfg.outage_enabled = req.enabled;
    }
    Json(serde_json::json!({"status": "outage_updated", "enabled": req.enabled}))
}

#[derive(serde::Deserialize)]
pub struct LatencyReq {
    pub delay_ms: u64,
}

async fn set_latency(
    State(state): State<AppState>,
    Json(req): Json<LatencyReq>,
) -> impl IntoResponse {
    {
        let mut cfg = state.chaos_config.lock().unwrap(); //
        cfg.global_latency_ms = req.delay_ms as i64;
    }
    Json(serde_json::json!({"status": "latency_updated", "delay_ms": req.delay_ms}))
}

pub async fn chaos_middleware(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (outage, latency) = {
        let cfg = state.chaos_config.lock().unwrap(); //
        (cfg.outage_enabled, cfg.global_latency_ms)
    };

    if outage {
        // Only affect /v1 routes to allow admin routes to still function to toggle it off
        if req.uri().path().starts_with("/v1/") {
            return (StatusCode::SERVICE_UNAVAILABLE, "Simulated Cloud Outage").into_response();
        }
    }

    if latency > 0 && req.uri().path().starts_with("/v1/") {
        tokio::time::sleep(std::time::Duration::from_millis(latency as u64)).await;
    }

    next.run(req).await
}
