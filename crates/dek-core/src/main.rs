#![warn(clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod service_integration;
mod ebpf;
mod keystore_migration;
mod updater;
mod ipc_server;
mod bundle_loop;
mod ipc_client;
mod probation;
mod supervisor;
mod metrics_push;
mod svid_renewal;

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
