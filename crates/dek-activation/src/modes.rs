use dek_config::ActivationMode;
use std::path::Path;
use tracing::info;
use crate::ActivationError;

pub fn handle_activation_mode(staged_path: &Path, mode: ActivationMode) -> Result<(), ActivationError> {
    let active_path = dek_config::paths::get_active_bundle_path();
    let shadow_path = dek_config::paths::get_data_dir().join("shadow_bundle.json");

    match mode {
        ActivationMode::Full => {
            // Replaced by ArcSwap in memory, but we persist the active bundle to disk
            if let Err(e) = std::fs::rename(staged_path, &active_path) {
                return Err(ActivationError::SnapshotSwapFailed(format!("Failed to move bundle to active path: {}", e)));
            }
            info!("Bundle activated to Full mode on disk.");
        }
        ActivationMode::ObserveOnly | ActivationMode::Shadow | ActivationMode::Canary => {
            // Store as shadow bundle
            if let Err(e) = std::fs::rename(staged_path, &shadow_path) {
                return Err(ActivationError::SnapshotSwapFailed(format!("Failed to move bundle to shadow path: {}", e)));
            }
            info!("Bundle activated to {:?} mode at shadow_bundle.json", mode);
        }
    }
    Ok(())
}
