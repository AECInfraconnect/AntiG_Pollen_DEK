use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityMaturity {
    Stub,
    ObserveOnly,
    EnforceBeta,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCapability {
    pub capability_id: String,
    pub name: String,
    pub pep_type: String,
    pub maturity: CapabilityMaturity,
    pub supported_os: Vec<String>,
    pub limitations: Vec<String>,
}

impl RuntimeCapability {
    pub fn can_enforce(&self) -> bool {
        matches!(
            self.maturity,
            CapabilityMaturity::EnforceBeta | CapabilityMaturity::Production
        )
    }
}
