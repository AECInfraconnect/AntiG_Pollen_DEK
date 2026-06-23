use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintDefinition {
    pub schema_version: String,
    pub definition_version: u64,
    pub released_at: String,
    pub min_engine_version: String,
    pub kind: DefinitionKind,
    pub base_version: Option<u64>,
    pub signatures: Vec<AgentSignatureV2>,
    pub removed_ids: Vec<String>,
    pub catalog_hash: String,
    #[serde(default)]
    pub model_classifier: Option<ModelClassifierDef>,
    #[serde(default)]
    pub web_ai_signatures: Vec<WebAiSignatureDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAiSignatureDef {
    pub domain: String,
    pub name: String,
    pub vendor: String,
    #[serde(default)] 
    pub capability_tags: Vec<String>,
    #[serde(default = "default_web_risk")] 
    pub risk_weight: f64,
}

fn default_web_risk() -> f64 { 0.4 }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelClassifierDef {
    #[serde(default)]
    pub vendors: Vec<VendorDef>,
    #[serde(default)]
    pub family_rules: Vec<FamilyRuleDef>,
    #[serde(default)]
    pub attribute_parsers: HashMap<String, AttributeParserDef>,
    #[serde(default)]
    pub risk_flags: Vec<RiskFlagDef>,
    #[serde(default)]
    pub popular_models: Vec<PopularModelDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorDef {
    pub ns: Vec<String>,
    pub vendor: String,
    pub license_class: Option<String>,
    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyRuleDef {
    pub id: String,
    pub pattern: String,
    pub family: String,
    pub vendor: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub risk_base: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeParserDef {
    String(String),
    Map(HashMap<String, String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlagDef {
    pub pattern: String,
    pub flag: String,
    pub risk_add: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularModelDef {
    pub match_pattern: String,
    #[serde(alias = "match")]
    pub match_: Option<String>, // the JSON has `match` instead of `match_pattern`, so we use alias and fallback
    pub display: String,
    pub vendor: Option<String>,
    pub family: String,
    pub license: Option<String>,
    pub arch: Option<String>,
    pub params_total_b: Option<f64>,
    pub params_active_b: Option<f64>,
    pub context: Option<u64>,
    #[serde(default)]
    pub modality: Vec<String>,
    pub popularity: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub risk_base: f64,
    #[serde(default)]
    pub flags: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelClass {
    pub raw_id: String,
    pub display: String,
    pub vendor: Option<String>,
    pub family: String,
    pub license: Option<String>,
    pub arch: Option<String>,
    pub params_total_b: Option<f64>,
    pub params_active_b: Option<f64>,
    pub context: Option<u64>,
    pub modality: Vec<String>,
    pub quant: Option<String>,
    pub variant: Vec<String>,
    pub capability_tags: Vec<String>,
    pub risk_score: f64,
    pub flags: Vec<String>,
    pub matched_tier: &'static str,
    pub needs_human: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassBase {
    pub display: String,
    pub family: String,
    pub vendor: Option<String>,
    pub license: Option<String>,
    pub arch: Option<String>,
    pub params_total_b: Option<f64>,
    pub params_active_b: Option<f64>,
    pub context: Option<u64>,
    pub modality: Vec<String>,
    pub quant: Option<String>,
    pub variant: Vec<String>,
    pub capability_tags: Vec<String>,
    pub risk_base: f64,
    pub flags: Vec<String>,
    pub matched_tier: &'static str,
}

impl ClassBase {
    pub fn unknown(vendor: Option<String>) -> Self {
        Self {
            display: "Unknown Model".into(),
            family: "unknown".into(),
            vendor,
            license: None,
            arch: None,
            params_total_b: None,
            params_active_b: None,
            context: None,
            modality: vec!["text".into()],
            quant: None,
            variant: vec![],
            capability_tags: vec![],
            risk_base: 0.4,
            flags: vec![],
            matched_tier: "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionKind {
    Full,
    Delta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSignatureV2 {
    pub id: String,
    pub display_name: String,
    pub agent_type: String,
    pub revision: u32,
    pub meta: SignatureMeta,

    pub process_names: Vec<String>,
    pub binary_hashes: Vec<String>,
    pub config_paths: HashMap<String, Vec<String>>,
    pub config_parsers: Vec<String>,
    pub ports: Vec<u16>,
    pub port_probe: Option<PortProbeSpec>,
    pub detection_logic: DetectionLogic,

    pub control_strategies: Vec<String>,
    pub risk_weight: f64,

    // ===== signal ใหม่ (แก้ node.exe ambiguity) =====
    #[serde(default)]
    pub cmd_patterns: Vec<String>,
    #[serde(default)]
    pub exe_path_patterns: Vec<String>,
    #[serde(default)]
    pub install_markers: Vec<InstallMarker>,
    #[serde(default)]
    pub cli_binaries: Vec<String>,
    #[serde(default)]
    pub package_markers: Vec<PackageMarker>,
    #[serde(default)]
    pub env_markers: Vec<String>,
    #[serde(default)]
    pub egress_hosts: Vec<String>,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub product: Option<String>,
    #[serde(default)]
    pub capability_tags: Vec<String>,
    #[serde(default)]
    pub signal_weights: Option<SignalWeights>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMarker {
    pub path: String,
    pub os: Option<String>,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMarker {
    pub ecosystem: String,
    pub name: String,
    pub global: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalWeights {
    pub process_name: f64,
    pub cmd_pattern: f64,
    pub exe_path: f64,
    pub install_marker: f64,
    pub cli_binary: f64,
    pub package: f64,
    pub config_path: f64,
    pub port: f64,
    pub egress: f64,
    pub binary_hash: f64,
}

impl Default for SignalWeights {
    fn default() -> Self {
        Self {
            process_name: 0.15,
            cmd_pattern: 0.45,
            exe_path: 0.40,
            install_marker: 0.55,
            cli_binary: 0.50,
            package: 0.45,
            config_path: 0.50,
            port: 0.25,
            egress: 0.30,
            binary_hash: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMeta {
    pub author: String,
    pub description: String,
    pub references: Vec<String>,
    pub added_in: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortProbeSpec {
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DetectionLogic {
    AnyOf,
    ProcessAndConfig,
    ProcessOrConfigWithPort,
    HashMatch,
}
