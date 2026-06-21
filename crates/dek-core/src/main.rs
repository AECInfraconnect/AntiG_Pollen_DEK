#![warn(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod api;
mod bundle_loop;
mod ebpf;
mod ipc_client;
mod ipc_server;
mod keystore_migration;
mod metrics_push;
mod probation;
mod reload_coordinator;
mod service_integration;
mod supervisor;
mod svid_renewal;
mod updater;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    service_integration::run_as_service_if_needed(async { run().await })?;
    #[cfg(not(windows))]
    run().await?;
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    supervisor::Supervisor::bootstrap().await?.run().await
}
