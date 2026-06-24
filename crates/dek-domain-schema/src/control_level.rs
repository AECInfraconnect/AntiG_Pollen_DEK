use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevel {
    Observe,
    Warn,
    Approval,
    Enforce,
}

impl ControlLevel {
    pub fn may_block(self) -> bool {
        matches!(self, Self::Approval | Self::Enforce)
    }
}

impl std::fmt::Display for ControlLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observe => write!(f, "observe"),
            Self::Warn => write!(f, "warn"),
            Self::Approval => write!(f, "approval"),
            Self::Enforce => write!(f, "enforce"),
        }
    }
}
