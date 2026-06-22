use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSignature {
    pub id: String,
    pub display_name: String,
    pub agent_type: String,
    pub process_names: Vec<String>,
    pub config_paths: Option<std::collections::HashMap<String, Vec<String>>>,
    pub forensic_artifacts: Option<std::collections::HashMap<String, Vec<String>>>,
    pub config_parsers: Option<Vec<String>>,
    pub ports: Option<Vec<u16>>,
    pub control_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCatalog {
    pub schema_version: String,
    pub catalog_version: String,
    pub signatures: Vec<AgentSignature>,
}

pub fn load_default_catalog() -> SourceCatalog {
    const EMBEDDED: &str = include_str!("../data/agent_signatures.v2.json");
    serde_json::from_str(EMBEDDED).unwrap_or_else(|_| SourceCatalog {
        schema_version: "pollen.agent_signature_catalog.v2".into(),
        catalog_version: "fallback".into(),
        signatures: vec![],
    })
}
