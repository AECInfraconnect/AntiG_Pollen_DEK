//! # DEK Core Supervisor
//!
//! Pollen DEK Core Supervisor manages device lifecycle including:
//! - Bootstrapping and mTLS config fetch from Pollen Cloud
//! - Periodic bundle synchronization
//! - Local IPC endpoint for health checks and commands
//! - Telemetry emission and Prometheus metrics push (OTLP/Pushgateway)

use anyhow::{Context, Result};
use dek_config::BootstrapConfig;
use dek_telemetry::CloudTelemetrySink;
use metrics::gauge;
use metrics_exporter_prometheus::PrometheusBuilder;
use serde_json::json;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

mod service_integration;
mod ebpf;
mod keystore_migration;
mod updater;
mod probation;
mod ipc_client;
mod supervisor;
mod ipc_server;
mod bundle_loop;
mod metrics_push;

use supervisor::Supervisor;

fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

async fn load_bootstrap(bootstrap_path: &str) -> Result<BootstrapConfig> {
    let bootstrap = BootstrapConfig::load_or_default(bootstrap_path)?;
    info!(
        "Loaded Bootstrap Config for device: {}",
        bootstrap.device_id
    );
    Ok(bootstrap)
}

fn main() -> Result<()> {
    service_integration::run_as_service_if_needed(core_main())
}

async fn core_main() -> Result<()> {
    dek_config::logging::init_logging("dek-core").unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
    });
    info!("Starting Pollen DEK Core Supervisor...");

    let mut supervisor = Supervisor::new();

    // A/B Update Probation Check
    let config_dir = dek_config::paths::get_config_dir();
    let pending_update = probation::detect(&config_dir);
    if pending_update.is_some() {
        info!("A/B update marker detected — probation will run once services are up");
        
        #[cfg(target_os = "linux")]
        {
            let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Watchdog]);
        }
    }

    // Load Layer 2 eBPF Guardrails (Linux only)
    if let Err(e) = ebpf::load_and_attach() {
        tracing::error!("Failed to initialize eBPF Layer 2 guardrails: {}", e);
    }

    let pollen_cloud_url = get_env_var("POLLEN_CLOUD_URL", "https://127.0.0.1:43891");
    let ipc_listen_addr = get_env_var("DEK_IPC_ADDR", "127.0.0.1:43889");
    let bootstrap_path = get_env_var("DEK_BOOTSTRAP_PATH", &dek_config::paths::get_bootstrap_path().to_string_lossy());
    let bundle_sync_interval = get_env_var("DEK_BUNDLE_SYNC_INTERVAL", "10")
        .parse::<u64>()
        .unwrap_or(10);

    if !pollen_cloud_url.starts_with("https://") {
        error!(
            "Fatal Error: POLLEN_CLOUD_URL must start with https:// to prevent downgrade attacks."
        );
        std::process::exit(1);
    }

    let pollen_telemetry_url = format!("{}/telemetry", pollen_cloud_url);
    let metrics_push_url = format!("{}/metrics", pollen_cloud_url);

    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");
    info!("Prometheus metrics recorder installed (Push Model enabled)");

    gauge!("dek_core_start_timestamp_seconds").set(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64(),
    );

    let bootstrap = load_bootstrap(&bootstrap_path).await?;

    let mut client_key_override: Option<Vec<u8>> = None;
    let mut pinned_key_override: Option<String> = None;
    if keystore_migration::run_migration(&bootstrap, &pollen_cloud_url).await {
        let keystore = dek_keystore::get_keystore();
        if let Ok(key_data) = keystore.load_key("mtls_client_key") {
            client_key_override = Some(key_data);
        }
        if let Ok(bundle_pk_data) = keystore.load_key("pinned_bundle_public_key") {
            if let Ok(pk_str) = String::from_utf8(bundle_pk_data) {
                pinned_key_override = Some(pk_str);
            }
        }
    }

    let actual_pinned_key = pinned_key_override.unwrap_or_else(|| bootstrap.pinned_bundle_public_key.clone());

    // Create BundleSyncAgent using Bootstrap mTLS
    let bundle_agent = Arc::new(dek_bundle_sync::BundleSyncAgent::new(
        &pollen_cloud_url,
        &bootstrap.device_id,
        &bootstrap.mtls,
        &actual_pinned_key,
        client_key_override.as_deref(),
    )?);

    let telemetry_sink = Arc::new(CloudTelemetrySink::new(
        &pollen_telemetry_url,
        &bootstrap.mtls,
        client_key_override.as_deref(),
    )?);

    let metrics_client = Arc::new(RwLock::new(
        bootstrap
            .mtls
            .build_client(client_key_override.as_deref())
            .context("Failed to build metrics MTLS client")?,
    ));

    let start_time = Instant::now();

    let ipc_handle = ipc_server::spawn_ipc_server_task(
        supervisor.cancel_token.clone(),
        ipc_listen_addr.clone(),
        telemetry_sink.clone(),
        bundle_agent.clone(),
        metrics_client.clone(),
        start_time,
    )
    .await?;

    supervisor.join_set.spawn(async move {
        let _ = ipc_handle.await;
    });

    // Signal readiness to OS Service Managers BEFORE blocking on cloud sync
    service_integration::notify_ready();

    if let Some(marker) = pending_update {
        let config_dir = config_dir.clone();
        let cloud = pollen_cloud_url.clone();
        let bootstrap = bootstrap.clone();
        let bundle_path = dek_config::paths::get_active_bundle_path();
        let ipc_addr = ipc_listen_addr.clone();

        supervisor.join_set.spawn(async move {
            let probe = move || {
                let addr = ipc_addr.clone();
                async move { ipc_client::ipc_health_ok(&addr).await }
            };
            probation::finalize(
                config_dir,
                cloud,
                bootstrap,
                bundle_path,
                probation::ProbationSettings::default(),
                marker,
                probe,
            )
            .await;
        });
    }

    #[cfg(target_os = "linux")]
    {
        supervisor.join_set.spawn(async move {
            if let Ok(timeout) = sd_notify::watchdog_enabled(false, &mut std::env::vars()) {
                if timeout > std::time::Duration::ZERO {
                    let interval = timeout / 2;
                    info!("Systemd Watchdog enabled. Pinging every {:?}", interval);
                    loop {
                        let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Watchdog]);
                        tokio::time::sleep(interval).await;
                    }
                }
            }
        });
    }

    // Spawn the cloud sync and background tasks into a separate tokio task
    // so that the IPC Server remains healthy and responsive immediately!
    let sync_bundle_agent = bundle_agent.clone();
    let sync_telemetry_sink = telemetry_sink.clone();
    let sync_metrics_client = metrics_client.clone();
    let sync_cancel_token = supervisor.cancel_token.clone();
    
    supervisor.join_set.spawn(async move {
        // Initial startup sync using the unified pipeline (blocks up to 2 minutes on retries)
        let config = match bundle_loop::run_sync_pipeline_with_retry(&sync_bundle_agent).await {
            Ok(c) => c,
            Err(e) => {
                error!("Initial cloud sync failed completely: {}. Background tasks will not start.", e);
                return;
            }
        };

        let _ = sync_telemetry_sink
            .emit_async(json!({
                "event_type": "pollen.dek.startup",
                "device_id": config.device_id,
                "status": "online"
            }))
            .await;

        let sync_handle = bundle_loop::spawn_bundle_sync_task(
            sync_cancel_token.clone(),
            sync_bundle_agent,
            bundle_sync_interval,
            sync_metrics_client.clone(),
            actual_pinned_key.clone(),
        );

        let metrics_handle = metrics_push::spawn_metrics_push_task(
            sync_cancel_token.clone(),
            sync_metrics_client,
            metrics_push_url,
            prometheus_handle,
        );
        
        // Wait for them to finish
        let _ = tokio::join!(sync_handle, metrics_handle);
    });

    // Wait for shutdown signal
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?;
        tokio::select! {
            _ = sigterm.recv() => info!("Received SIGTERM"),
            _ = sigint.recv() => info!("Received SIGINT"),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
        info!("Received SIGINT");
    }

    info!("Initiating graceful shutdown...");
    supervisor.shutdown();
    supervisor.wait_for_shutdown().await;

    Ok(())
}
