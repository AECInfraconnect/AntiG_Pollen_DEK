use dek_fingerprint_defs::model::AgentSignatureV2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBinding {
    pub binding_id: String,
    pub agent_instance_id: String,
    pub signature_id: String,
    pub tenant_id: String,
    pub device_id: String,

    pub capabilities: crate::capability::CapabilityDescriptor,
    pub control: Vec<crate::control::ControlBindingSpec>,
    pub enforcement: crate::enforce::EnforcementHooks,
    pub telemetry: crate::telemetry::TelemetrySpec,

    pub owner: Option<String>,
    pub purpose: Option<String>,
    pub scope: Vec<String>,

    pub lifecycle: BindingLifecycle,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BindingLifecycle {
    Discovered,
    Provisioned,
    Enforced,
    Suspended,
    Deprovisioned,
}

impl AgentBinding {
    pub fn from_discovery(
        sig: &AgentSignatureV2,
        candidate_id: &str,
        tenant: &str,
        device: &str,
        discovered_surfaces: Vec<crate::capability::Surface>,
    ) -> Self {
        let capabilities = crate::capability::capabilities_from_discovery(sig, discovered_surfaces);
        let control = crate::control::plan_control_binding(
            &capabilities,
            &[
                "mcp-stdio".to_string(),
                "mcp-http".to_string(),
                "linux-ebpf".to_string(),
                "windows-wfp".to_string(),
                "macos-nefilter".to_string(),
            ],
            crate::control::ControlLevel::Enforce,
        );
        let enforcement = crate::enforce::derive_enforcement(&capabilities);
        let telemetry = crate::telemetry::derive_telemetry(&capabilities);
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            binding_id: uuid::Uuid::new_v4().to_string(),
            agent_instance_id: candidate_id.into(),
            signature_id: sig.id.clone(),
            tenant_id: tenant.into(),
            device_id: device.into(),
            capabilities,
            control,
            enforcement,
            telemetry,
            owner: None,
            purpose: None,
            scope: vec![],
            lifecycle: BindingLifecycle::Discovered,
            first_seen: now.clone(),
            last_seen: now,
        }
    }

    pub fn reevaluate(&mut self, observed_tools: &[String]) -> Vec<String> {
        let known: std::collections::HashSet<_> = self
            .capabilities
            .tool_capabilities
            .iter()
            .map(|t| &t.tool_name)
            .collect();
        observed_tools
            .iter()
            .filter(|t| !known.contains(*t))
            .map(|t| format!("capability_drift:new_tool:{t}"))
            .collect()
    }

    pub fn transition_to(&mut self, next_state: BindingLifecycle) -> Result<(), String> {
        use BindingLifecycle::*;
        match (&self.lifecycle, &next_state) {
            (Discovered, Provisioned) => self.lifecycle = next_state,
            (Provisioned, Enforced) => self.lifecycle = next_state,
            (Enforced, Suspended) => self.lifecycle = next_state,
            (Suspended, Enforced) => self.lifecycle = next_state,
            (Discovered, Suspended) => self.lifecycle = next_state,
            (_, Deprovisioned) => self.lifecycle = next_state,
            (curr, next) => {
                return Err(format!(
                    "Invalid lifecycle transition from {:?} to {:?}",
                    curr, next
                ));
            }
        }
        self.last_seen = chrono::Utc::now().to_rfc3339();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dek_fingerprint_defs::model::{DetectionLogic, SignatureMeta};

    fn test_signature(id: &str) -> AgentSignatureV2 {
        AgentSignatureV2 {
            id: id.into(),
            display_name: id.into(),
            agent_type: "cli_agent".into(),
            revision: 1,
            meta: SignatureMeta {
                author: "t".into(),
                description: "".into(),
                references: vec![],
                added_in: "1".into(),
                tags: vec![],
            },
            process_names: vec![],
            binary_hashes: vec![],
            config_paths: Default::default(),
            config_parsers: vec![],
            ports: vec![],
            port_probe: None,
            detection_logic: DetectionLogic::AnyOf,
            control_strategies: vec!["mcp_stdio_wrapper".into()],
            risk_weight: 0.5,
            cmd_patterns: vec![],
            exe_path_patterns: vec![],
            install_markers: vec![],
            cli_binaries: vec![],
            package_markers: vec![],
            env_markers: vec![],
            egress_hosts: vec![],
            vendor: None,
            product: None,
            capability_tags: vec![],
            signal_weights: None,
        }
    }

    #[test]
    fn binding_wires_all_layers() {
        let sig = test_signature("ollama");
        let b = AgentBinding::from_discovery(&sig, "cand-1", "local", "dev-1", vec![]);
        assert_eq!(b.lifecycle, BindingLifecycle::Discovered);
        assert_eq!(b.signature_id, "ollama");
        assert!(!b.telemetry.otel_attributes.is_empty());
    }

    #[test]
    fn reevaluate_detects_drift() {
        let sig = test_signature("cursor");
        let b = AgentBinding::from_discovery(&sig, "c", "t", "d", vec![]);
        let drift = {
            let mut bb = b.clone();
            bb.reevaluate(&["unknown_tool".into()])
        };
        assert!(drift.iter().any(|d| d.contains("capability_drift")));
    }
}
