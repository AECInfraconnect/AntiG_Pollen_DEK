use anyhow::Result;
use dek_domain_schema::{PdpKind, PepBinding};

pub struct PolicyEngine {
    pub current_bindings: Vec<PepBinding>,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            current_bindings: Vec::new(),
        }
    }

    pub async fn push_to_runtime(&mut self, pep: PepBinding, _pdp_kind: PdpKind) -> Result<()> {
        // Push the policy down to the enforcement API or local control plane
        self.current_bindings.push(pep);
        Ok(())
    }

    pub fn get_active_policies(&self) -> &[PepBinding] {
        &self.current_bindings
    }
}
