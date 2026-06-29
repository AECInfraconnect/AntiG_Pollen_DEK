use anyhow::{Context, Result};
use dek_plugin_sdk::PluginManifest;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// Parses a PluginManifest from JSON bytes
pub fn parse_manifest(json_bytes: &[u8]) -> Result<PluginManifest> {
    serde_json::from_slice(json_bytes).context("Failed to parse PluginManifest JSON")
}

/// Verifies that the WASM binary matches the expected hash and is signed by the
/// provided public key. Remote Sigstore/TUF policy is intentionally external to
/// this local verifier, but invalid, missing, or test-only signature states are
/// fail-closed here unless a caller routes through an explicit developer preview
/// path before loading the plugin.
pub fn verify_plugin_signature(
    manifest: &PluginManifest,
    wasm_bytes: &[u8],
    public_key_bytes: &[u8; 32],
) -> Result<()> {
    ensure_signature_state_allows_load(manifest)?;

    // 1. Verify Hash
    let mut hasher = Sha256::new();
    hasher.update(wasm_bytes);
    let hash = format!("{:x}", hasher.finalize());

    let expected_hash = expected_sha256(manifest)?;

    if hash != expected_hash {
        anyhow::bail!(
            "WASM hash mismatch. Expected: {}, Actual: {}",
            expected_hash,
            hash
        );
    }

    // 2. Verify Signature
    // For this example, we assume `signing` has an `ed25519_signature` field (hex string).
    let sig_hex = manifest
        .signing
        .get("ed25519_signature")
        .context("Manifest is missing 'ed25519_signature' in signing block")?;

    let sig_bytes = hex::decode(sig_hex).context("Failed to decode signature hex")?;
    if sig_bytes.len() != 64 {
        anyhow::bail!("Invalid signature length");
    }

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from(sig_arr);

    let public_key =
        VerifyingKey::from_bytes(public_key_bytes).context("Invalid public key bytes")?;

    public_key
        .verify(wasm_bytes, &signature)
        .context("Cryptographic signature verification failed")?;

    Ok(())
}

fn ensure_signature_state_allows_load(manifest: &PluginManifest) -> Result<()> {
    let Some(signature) = &manifest.signature else {
        return Ok(());
    };
    match signature.status.as_deref().unwrap_or("unknown") {
        "valid" => Ok(()),
        state => anyhow::bail!("Plugin signature state is '{state}', refusing to load"),
    }
}

fn expected_sha256(manifest: &PluginManifest) -> Result<String> {
    if let Some(checksum) = &manifest.checksum {
        if let Some(rest) = checksum.strip_prefix("sha256:") {
            return Ok(rest.to_ascii_lowercase());
        }
        anyhow::bail!("Unsupported plugin checksum format; expected sha256:<hex>");
    }
    manifest
        .signing
        .get("sha256")
        .map(|value| value.to_ascii_lowercase())
        .context("Manifest is missing checksum or 'sha256' in signing block")
}

#[cfg(test)]
mod tests {
    use dek_plugin_sdk::{
        PluginAbi, PluginCapabilities, PluginManifest, PluginSignature, PluginType, WasmLimits,
    };
    use serde_json::json;

    use super::*;

    fn manifest(status: Option<&str>, checksum: Option<String>) -> PluginManifest {
        PluginManifest {
            schema_version: "pollek.plugin.v1".into(),
            id: "com.example.plugin".into(),
            name: "Example".into(),
            version: "1.0.0".into(),
            kind: None,
            wit_world: None,
            abi: PluginAbi::Component,
            min_engine_version: None,
            max_engine_version: None,
            os: vec![],
            entry: Some("plugin.wasm".into()),
            capabilities: PluginCapabilities::default(),
            config_schema: Some(json!({})),
            author: None,
            homepage: None,
            license: None,
            signature: status.map(|state| PluginSignature {
                status: Some(state.into()),
                ..PluginSignature::default()
            }),
            sbom: None,
            checksum,
            registry: None,
            governance: None,
            plugin_type: PluginType::Transform,
            runtime: "wasm".into(),
            entrypoint: "plugin.wasm".into(),
            permissions: vec![],
            limits: WasmLimits::default(),
            signing: Default::default(),
        }
    }

    #[test]
    fn invalid_signature_state_fails_closed() {
        let manifest = manifest(Some("invalid"), None);
        let err = ensure_signature_state_allows_load(&manifest).err();
        assert!(err
            .map(|error| error.to_string().contains("refusing to load"))
            .unwrap_or(false));
    }

    #[test]
    fn checksum_uses_schema_sha256_prefix() -> anyhow::Result<()> {
        let manifest = manifest(
            Some("valid"),
            Some("sha256:AA00000000000000000000000000000000000000000000000000000000000000".into()),
        );
        assert!(expected_sha256(&manifest)?.starts_with("aa"));
        Ok(())
    }
}
