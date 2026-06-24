// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::ActivationError;
use dek_config::ActivationMode;
use std::path::Path;
use tracing::info;

pub fn handle_activation_mode(
    staged_path: &Path,
    mode: ActivationMode,
) -> Result<(), ActivationError> {
    let active_path = dek_config::paths::get_active_bundle_path();
    let shadow_path = dek_config::paths::get_data_dir().join("shadow_bundle.json");

    match mode {
        ActivationMode::Full => {
            // Replaced by ArcSwap in memory, but we persist the active bundle to disk
            if staged_path != active_path {
                if let Err(e) = std::fs::rename(staged_path, &active_path) {
                    tracing::warn!("Rename failed, falling back to copy for atomic swap: {}", e);
                    if let Err(copy_err) = std::fs::copy(staged_path, &active_path) {
                        return Err(ActivationError::SnapshotSwapFailed(format!(
                            "Failed to move bundle to active path: {}",
                            copy_err
                        )));
                    }
                    // It's okay if delete fails since we successfully copied.
                    let _ = std::fs::remove_file(staged_path);
                }
            }
            info!("Bundle activated to Full mode on disk.");
        }
        ActivationMode::ObserveOnly | ActivationMode::Shadow | ActivationMode::Canary => {
            // Store as shadow bundle
            if staged_path != shadow_path {
                if let Err(e) = std::fs::rename(staged_path, &shadow_path) {
                    tracing::warn!("Rename failed, falling back to copy for shadow swap: {}", e);
                    if let Err(copy_err) = std::fs::copy(staged_path, &shadow_path) {
                        return Err(ActivationError::SnapshotSwapFailed(format!(
                            "Failed to move bundle to shadow path: {}",
                            copy_err
                        )));
                    }
                    let _ = std::fs::remove_file(staged_path);
                }
            }
            info!("Bundle activated to {:?} mode at shadow_bundle.json", mode);
        }
    }
    Ok(())
}
