use anyhow::{Context, Result};
use dek_plugin_sdk::PluginManifest;
use sha2::{Digest, Sha256};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};

/// Parses a PluginManifest from JSON bytes
pub fn parse_manifest(json_bytes: &[u8]) -> Result<PluginManifest> {
    serde_json::from_slice(json_bytes).context("Failed to parse PluginManifest JSON")
}

/// Verifies that the WASM binary matches the expected hash and is signed by the provided public key.
/// In a real implementation, this would integrate with TUF/Sigstore.
pub fn verify_plugin_signature(
    manifest: &PluginManifest,
    wasm_bytes: &[u8],
    public_key_bytes: &[u8; 32],
) -> Result<()> {
    // 1. Verify Hash
    let mut hasher = Sha256::new();
    hasher.update(wasm_bytes);
    let hash = format!("{:x}", hasher.finalize());

    let expected_hash = manifest
        .signing
        .get("sha256")
        .context("Manifest is missing 'sha256' in signing block")?;

    if &hash != expected_hash {
        anyhow::bail!("WASM hash mismatch. Expected: {}, Actual: {}", expected_hash, hash);
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

    let public_key = VerifyingKey::from_bytes(public_key_bytes).context("Invalid public key bytes")?;

    public_key.verify(wasm_bytes, &signature).context("Cryptographic signature verification failed")?;

    Ok(())
}
