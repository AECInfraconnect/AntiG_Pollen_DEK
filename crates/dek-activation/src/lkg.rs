// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use tracing::{info, warn};

pub fn update_lkg() {
    let active_path = dek_config::paths::get_active_bundle_path();
    let lkg_path = dek_config::paths::get_data_dir().join("active_bundle_lkg.json");

    if active_path.exists() {
        if let Err(e) = std::fs::copy(&active_path, &lkg_path) {
            warn!("Failed to backup active bundle to LKG: {}", e);
        } else {
            info!("Updated Last Known Good (LKG) backup.");
        }
    }
}

pub fn rollback_lkg() -> Result<(), std::io::Error> {
    let active_path = dek_config::paths::get_active_bundle_path();
    let lkg_path = dek_config::paths::get_data_dir().join("active_bundle_lkg.json");

    if lkg_path.exists() {
        tracing::warn!("Rolling back to Last Known Good (LKG) bundle!");
        // Atomic rename from LKG to active
        std::fs::rename(&lkg_path, &active_path)?;
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "LKG not found",
        ))
    }
}
