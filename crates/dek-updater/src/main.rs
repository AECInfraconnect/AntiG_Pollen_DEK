use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dek-updater", about = "Pollen DEK Auto-Updater")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for updates
    Check {
        #[arg(long, default_value = "stable")]
        channel: String,
    },
    /// Verify an executable against TUF metadata
    Verify {
        target_exe: PathBuf,
        metadata_path: PathBuf,
    },
    /// Perform the update by replacing the current executable
    Update {
        target_exe: PathBuf,
        new_exe: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { channel } => {
            println!("Checking for updates on channel: {}", channel);
            // In a real implementation, this would fetch from the TUF repository.
            println!("Mock: No updates available on channel '{}'.", channel);
        }
        Commands::Verify {
            target_exe,
            metadata_path,
        } => {
            verify_tuf_signature(&target_exe, &metadata_path)?;
            println!("TUF Verification Successful.");
        }
        Commands::Update {
            target_exe,
            new_exe,
            dry_run,
        } => {
            if !new_exe.exists() {
                anyhow::bail!("New executable not found: {:?}", new_exe);
            }
            if dry_run {
                println!(
                    "DRY RUN: Would replace {:?} with {:?}",
                    target_exe, new_exe
                );
                return Ok(());
            }

            let backup_exe = target_exe.with_extension("exe.bak");
            if backup_exe.exists() {
                let _ = fs::remove_file(&backup_exe);
            }
            if target_exe.exists() {
                fs::rename(&target_exe, &backup_exe)
                    .context("Failed to rename active executable")?;
            }
            if let Err(e) = fs::rename(&new_exe, &target_exe) {
                eprintln!("Failed to move new executable into place: {e}");
                if backup_exe.exists() {
                    let _ = fs::rename(&backup_exe, &target_exe);
                }
                anyhow::bail!("Update failed and rolled back");
            }
            println!("Update successful. Please restart the service.");
        }
    }
    Ok(())
}

fn verify_tuf_signature(new_exe: &PathBuf, metadata_path: &PathBuf) -> Result<()> {
    let content = fs::read_to_string(metadata_path).context("Failed to read TUF metadata")?;
    let metadata: serde_json::Value = serde_json::from_str(&content)?;

    // Handle full TUF format { "signed": { "targets": ... } } or raw payload
    let signed = if metadata.get("signed").is_some() {
        &metadata["signed"]
    } else {
        &metadata
    };

    // 1. Check expiration
    if let Some(expires_str) = signed["expires"].as_str() {
        if let Ok(expires_dt) = expires_str.parse::<DateTime<Utc>>() {
            if Utc::now() > expires_dt {
                anyhow::bail!("TUF metadata has expired on {}", expires_dt);
            }
        }
    }

    let target_name = new_exe
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let target_info = signed["targets"][&target_name]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Target {} not found in TUF metadata", target_name))?;

    let expected_hash = target_info["hashes"]["sha256"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing sha256 hash in metadata for target"))?;
    let expected_size = target_info["length"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("Missing length in metadata for target"))?;

    // 2. Hash and Size check
    let mut file = fs::File::open(new_exe)?;
    let meta = file.metadata()?;
    if meta.len() != expected_size {
        anyhow::bail!("File size mismatch: expected {}, got {}", expected_size, meta.len());
    }

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash != expected_hash {
        anyhow::bail!("Hash mismatch: expected {}, got {}", expected_hash, actual_hash);
    }

    // 3. Platform checking (mock constraint)
    if let Some(platform) = target_info.get("custom").and_then(|c| c.get("platform")).and_then(|p| p.as_str()) {
        let current_platform = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" };
        if !platform.contains(current_platform) {
            anyhow::bail!("Platform mismatch: target is for {}, but we are on {}", platform, current_platform);
        }
    }

    Ok(())
}
