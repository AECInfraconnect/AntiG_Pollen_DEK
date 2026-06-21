use anyhow::{Context, Result};
use std::fs;
use tracing::info;

pub fn run() -> Result<()> {
    let bootstrap_path = dek_config::paths::get_bootstrap_path();
    let config_dir = dek_config::paths::get_config_dir();
    let certs_dir = config_dir.join("certs");

    println!("WARNING: This will permanently remove local identity and config.");
    
    // In a real CLI you would prompt for confirmation here.
    // For now we'll just execute.
    
    if bootstrap_path.exists() {
        fs::remove_file(&bootstrap_path).context("remove bootstrap.json")?;
        info!("Removed bootstrap.json");
    }

    if certs_dir.exists() {
        fs::remove_dir_all(&certs_dir).context("remove certs dir")?;
        info!("Removed certs directory");
    }

    // Attempt to remove from OS keystore
    let ks = dek_keystore::get_keystore();
    let _ = ks.delete_key("mtls_client_key");
    let _ = ks.delete_key("pinned_bundle_public_key");

    println!("✓ Device unenrolled locally.");
    Ok(())
}
