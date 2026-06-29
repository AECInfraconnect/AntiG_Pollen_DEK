// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

pub use dek_plugin_sdk::{
    PluginAbi, PluginAuthor, PluginCapabilities, PluginGovernance, PluginIdentity, PluginKind,
    PluginManifest, PluginRegistry, PluginSignature, PluginSource, RequestedCapability, WasmLimits,
};
use sha2::{Digest, Sha256};

pub const MANIFEST_SCHEMA_VERSION: &str = "pollek.plugin.v1";

#[derive(Debug, Clone)]
pub struct ManifestDraft {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
    pub wit_world: String,
    pub entry: String,
}

impl ManifestDraft {
    pub fn into_manifest(self) -> PluginManifest {
        PluginManifest {
            schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            id: self.id,
            name: self.name,
            version: self.version,
            kind: Some(self.kind),
            wit_world: Some(self.wit_world),
            abi: PluginAbi::Component,
            min_engine_version: Some("1.0.0".to_string()),
            max_engine_version: None,
            os: vec![
                "windows".to_string(),
                "linux".to_string(),
                "macos".to_string(),
            ],
            entry: Some(self.entry),
            capabilities: PluginCapabilities {
                host: vec!["logging".to_string(), "clock".to_string()],
                ..PluginCapabilities::default()
            },
            config_schema: None,
            author: None,
            homepage: None,
            license: Some("Apache-2.0".to_string()),
            signature: Some(PluginSignature {
                status: Some("missing".to_string()),
                ..PluginSignature::default()
            }),
            sbom: None,
            checksum: None,
            registry: Some(PluginRegistry {
                source: PluginSource::Sideload,
                update_channel: Some("local".to_string()),
                ..PluginRegistry::default()
            }),
            governance: Some(PluginGovernance {
                review_required: true,
                public_marketplace_allowed: false,
                trust_labels: vec!["developer_preview".to_string()],
                ..PluginGovernance::default()
            }),
            plugin_type: dek_plugin_sdk::PluginType::ControlPlane,
            runtime: "wasm".to_string(),
            entrypoint: "plugin.wasm".to_string(),
            permissions: Vec::new(),
            limits: WasmLimits::default(),
            signing: Default::default(),
        }
    }
}

pub fn parse_manifest(bytes: &[u8]) -> anyhow::Result<PluginManifest> {
    let manifest: PluginManifest = serde_json::from_slice(bytes)?;
    validate_manifest_basics(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest_basics(manifest: &PluginManifest) -> anyhow::Result<()> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported schema_version {}, expected {}",
            manifest.schema_version,
            MANIFEST_SCHEMA_VERSION
        );
    }
    if manifest.kind.is_none() {
        anyhow::bail!("plugin manifest requires kind");
    }
    if manifest.wit_world.as_deref().unwrap_or_default().is_empty() {
        anyhow::bail!("plugin manifest requires wit_world");
    }
    if manifest.entry_path().is_empty() {
        anyhow::bail!("plugin manifest requires entry");
    }
    Ok(())
}

pub fn sha256_checksum(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

pub fn capability_description(capability: &RequestedCapability) -> String {
    match capability {
        RequestedCapability::Host(name) if name == "logging" => {
            "Write local plugin diagnostic logs".to_string()
        }
        RequestedCapability::Host(name) if name == "clock" => {
            "Read the local clock for timestamps".to_string()
        }
        RequestedCapability::Host(name) => format!("Use host capability {name}"),
        RequestedCapability::HttpOut(host) => {
            format!("Send approved telemetry or requests to {host}")
        }
        RequestedCapability::Kv(scope) => format!("Read or write plugin state: {scope}"),
        RequestedCapability::Native(cap) => {
            format!("Use reviewed native OS capability provider {cap}")
        }
        RequestedCapability::DataScope(scope) => format!("Access Pollek data scope {scope}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_manifest_validates() -> anyhow::Result<()> {
        let manifest = ManifestDraft {
            id: "com.example.discovery".to_string(),
            name: "Example Discovery".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::DiscoverySignature,
            wit_world: "pollek:discovery/discovery-plugin@0.1.0".to_string(),
            entry: "plugin.wasm".to_string(),
        }
        .into_manifest();
        validate_manifest_basics(&manifest)
    }

    #[test]
    fn descriptions_are_human_readable() {
        let text = capability_description(&RequestedCapability::HttpOut(
            "splunk.example.com:443".to_string(),
        ));
        assert!(text.contains("splunk.example.com"));
    }
}
