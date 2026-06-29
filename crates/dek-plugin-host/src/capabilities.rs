use std::collections::BTreeSet;

use dek_plugin_sdk::{PluginManifest, RequestedCapability};

#[derive(Debug, Clone, Default)]
pub struct ConsentSet {
    allowed: BTreeSet<RequestedCapability>,
}

impl ConsentSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn allow(mut self, capability: RequestedCapability) -> Self {
        self.allowed.insert(capability);
        self
    }

    pub fn allows(&self, capability: &RequestedCapability) -> bool {
        self.allowed.contains(capability)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    granted: BTreeSet<RequestedCapability>,
}

impl CapabilitySet {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, capability: RequestedCapability) {
        self.granted.insert(capability);
    }

    pub fn has(&self, capability: &RequestedCapability) -> bool {
        self.granted.contains(capability)
    }

    pub fn granted(&self) -> impl Iterator<Item = &RequestedCapability> {
        self.granted.iter()
    }

    pub fn can_http_out(&self, host: &str) -> bool {
        self.has(&RequestedCapability::HttpOut(host.to_string()))
    }

    pub fn can_native(&self, cap_id: &str) -> bool {
        self.has(&RequestedCapability::Native(cap_id.to_string()))
    }
}

pub fn grant_capabilities(manifest: &PluginManifest, user_consent: &ConsentSet) -> CapabilitySet {
    let mut caps = CapabilitySet::none();
    for requested in manifest.requested_capabilities() {
        if requested.is_basic() || user_consent.allows(&requested) {
            caps.grant(requested);
        }
    }
    caps
}

#[cfg(test)]
mod tests {
    use dek_plugin_sdk::{
        PluginAbi, PluginCapabilities, PluginManifest, PluginType, RequestedCapability, WasmLimits,
    };

    use super::*;

    fn manifest() -> PluginManifest {
        PluginManifest {
            schema_version: "pollek.plugin.v1".into(),
            id: "com.example.exporter".into(),
            name: "Example Exporter".into(),
            version: "1.0.0".into(),
            kind: None,
            wit_world: Some("pollek:telemetry/telemetry-exporter@0.1.0".into()),
            abi: PluginAbi::Component,
            min_engine_version: Some("1.0.0".into()),
            max_engine_version: None,
            os: vec![],
            entry: Some("plugin.wasm".into()),
            capabilities: PluginCapabilities {
                host: vec!["logging".into(), "clock".into()],
                http_out: vec!["splunk.example.com:443".into()],
                kv: vec!["read".into(), "write".into()],
                native: vec!["wfp".into()],
                data_scope: vec!["telemetry:read".into(), "candidates:write".into()],
            },
            config_schema: None,
            author: None,
            homepage: None,
            license: None,
            signature: None,
            sbom: None,
            checksum: None,
            registry: None,
            governance: None,
            plugin_type: PluginType::TelemetrySink,
            runtime: "wasm".into(),
            entrypoint: "plugin.wasm".into(),
            permissions: vec![],
            limits: WasmLimits::default(),
            signing: Default::default(),
        }
    }

    #[test]
    fn grants_basic_host_capabilities_without_consent() {
        let caps = grant_capabilities(&manifest(), &ConsentSet::empty());

        assert!(caps.has(&RequestedCapability::Host("logging".into())));
        assert!(caps.has(&RequestedCapability::Host("clock".into())));
    }

    #[test]
    fn denies_sensitive_capabilities_without_consent() {
        let caps = grant_capabilities(&manifest(), &ConsentSet::empty());

        assert!(!caps.can_http_out("splunk.example.com:443"));
        assert!(!caps.can_native("wfp"));
        assert!(!caps.has(&RequestedCapability::DataScope("telemetry:read".into())));
    }

    #[test]
    fn grants_sensitive_capabilities_with_matching_consent() {
        let consent = ConsentSet::empty()
            .allow(RequestedCapability::HttpOut(
                "splunk.example.com:443".into(),
            ))
            .allow(RequestedCapability::Native("wfp".into()));
        let caps = grant_capabilities(&manifest(), &consent);

        assert!(caps.can_http_out("splunk.example.com:443"));
        assert!(caps.can_native("wfp"));
        assert!(!caps.has(&RequestedCapability::DataScope("telemetry:read".into())));
    }
}
