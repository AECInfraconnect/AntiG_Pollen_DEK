use std::path::Path;
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
