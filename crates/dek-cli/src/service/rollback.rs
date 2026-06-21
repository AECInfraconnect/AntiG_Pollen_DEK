use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};

pub fn execute_rollback() -> Result<()> {
    info!("Starting emergency rollback via dekctl...");

    let config_dir = dek_config::paths::get_config_dir();
    let marker_path = config_dir.join("update_pending.json");

    if !marker_path.exists() {
        warn!("No update_pending.json found. Nothing to rollback.");
        return Ok(());
    }

    let marker_data = fs::read_to_string(&marker_path)
        .context("Failed to read update_pending.json")?;
    
    let marker: serde_json::Value = serde_json::from_str(&marker_data)
        .context("Failed to parse update_pending.json")?;

    let backup_path_str = marker.get("backup_path")
        .and_then(|v| v.as_str())
        .context("Missing backup_path in marker")?;
    let backup_path = PathBuf::from(backup_path_str);

    if !backup_path.exists() {
        error!("Backup file {:?} does not exist! Cannot rollback.", backup_path);
        return Err(anyhow::anyhow!("Backup file missing"));
    }

    // Determine target binary path (can be from marker or derive from backup_path)
    let target_path = if let Some(target) = marker.get("target_path").and_then(|v| v.as_str()) {
        PathBuf::from(target)
    } else {
        // Fallback derive using OS extension
        let mut t = backup_path.clone();
        t.set_extension(std::env::consts::EXE_EXTENSION);
        t
    };

    info!("Restoring backup from {:?} to {:?}", backup_path, target_path);

    // Perform atomic replace (we are in dekctl, and the service is stopped, so file is not locked)
    fs::copy(&backup_path, &target_path)
        .context("Failed to copy backup over target")?;

    info!("Rollback binary restored. Cleaning up markers...");

    let _ = fs::remove_file(&backup_path);
    let _ = fs::remove_file(&marker_path);

    info!("Restarting Pollen DEK service...");
    restart_service()?;

    Ok(())
}

fn restart_service() -> Result<()> {
    #[cfg(unix)]
    {
        // Absolute path to prevent PATH hijacking
        std::process::Command::new("/usr/bin/systemctl")
            .args(&["restart", "pollen-dek"])
            .status()
            .context("Failed to restart service via systemctl")?;
    }
    #[cfg(windows)]
    {
        // Absolute path for Windows PowerShell
        let powershell_path = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";
        let script = "Restart-Service -Name PollenDEK -Force";
        std::process::Command::new(powershell_path)
            .args(&["-Command", script])
            .status()
            .context("Failed to restart service via PowerShell")?;
    }
    Ok(())
}
