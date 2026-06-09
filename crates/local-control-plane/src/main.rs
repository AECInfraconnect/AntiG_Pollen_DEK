use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};
use dek_control_plane_api::identity::ControlPlaneIdentity;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

mod policy;
mod registry;
mod store;

#[derive(Clone)]
pub struct AppState {
    pub identity: ControlPlaneIdentity,
    pub registry_store: Arc<dyn store::RegistryStore>,
}

pub async fn local_tenant_guard(
    Path(tenant_id): Path<String>,
    State(state): State<AppState>,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if state.identity.tenant_id == "local" && tenant_id != "local" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({"error": "Local Admin Dashboard only supports tenant_id=local"}),
            ),
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // SQLite local store
    let db_url = "sqlite::memory:"; // Use memory for initial testing, later change to file
    let registry_store = Arc::new(store::SqliteStore::new(db_url).await?);

    let state = AppState {
        identity: ControlPlaneIdentity::local_default(),
        registry_store,
    };

    let static_dir = std::env::var("DEK_DASHBOARD_DIR")
        .unwrap_or_else(|_| "../../apps/local-admin-dashboard/dist".to_string());

    let app = Router::new()
        .merge(registry::router())
        .merge(policy::router())
        .fallback_service(
            ServeDir::new(&static_dir)
                .not_found_service(ServeFile::new(format!("{}/index.html", static_dir))),
        )
        .with_state(state.clone());

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    info!("Local Control Plane listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
