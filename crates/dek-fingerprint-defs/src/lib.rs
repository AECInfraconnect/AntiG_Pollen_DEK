pub mod loader;
pub mod merge;
pub mod model;
pub mod model_classifier;
pub mod store;
pub mod verify;

use model::*;

pub fn embedded_baseline() -> FingerprintDefinition {
    const BASELINE: &str = include_str!("../data/baseline.v3.json");
    serde_json::from_str(BASELINE).unwrap_or_else(|_| FingerprintDefinition {
        schema_version: "pollen.def.v3".into(),
        definition_version: 0,
        released_at: "1970-01-01T00:00:00Z".into(),
        min_engine_version: "0.0.0".into(),
        kind: DefinitionKind::Full,
        base_version: None,
        signatures: vec![],
        removed_ids: vec![],
        catalog_hash: String::new(),
        model_classifier: None,
        web_ai_signatures: vec![],
        installed_app_signatures: vec![],
    })
}

fn get_data_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("DEK_STATE_DIR") {
        return std::path::PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("DEK_DATA_DIR") {
        return std::path::PathBuf::from(dir);
    }
    #[cfg(target_os = "windows")]
    {
        let program_data =
            std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string());
        std::path::PathBuf::from(program_data)
            .join("PollenDEK")
            .join("state")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::path::PathBuf::from("/var/lib/pollen-dek")
    }
}

pub fn load_latest_baseline() -> FingerprintDefinition {
    let baseline = embedded_baseline();

    // Attempt to load dynamically from bundle
    let bundle_dir = get_data_dir().join("bundles").join("latest");

    let baseline_path = bundle_dir.join("baseline.v3.json");
    if baseline_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&baseline_path) {
            if let Ok(parsed) = serde_json::from_str::<FingerprintDefinition>(&content) {
                tracing::info!("Loaded dynamic baseline.v3.json from bundle");
                return parsed;
            } else {
                tracing::warn!(
                    "Failed to parse dynamic baseline.v3.json, falling back to embedded"
                );
            }
        }
    }

    baseline
}
