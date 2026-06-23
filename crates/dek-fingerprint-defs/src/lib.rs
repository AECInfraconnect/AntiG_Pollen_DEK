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
    })
}
