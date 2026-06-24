// SPDX-License-Identifier: Apache-2.0
//! signing.rs โ€” local control-plane ed25519 signing key (L2.1).
//!
//! The Local control plane signs policy bundles with its own key so the DEK can
//! verify them with the EXACT same chain-of-trust path it uses for Cloud
//! bundles (invariant I3: bundles are always signed; DEK verify is identical).
//! Cutover Local->Cloud changes only the trust store (which key the DEK trusts).
//!
//! Key is generated on first run and persisted 0600 in the local data dir.

use anyhow::{Context, Result};
use ed25519_dalek::{Signer, SigningKey};
use std::path::{Path, PathBuf};

pub struct LocalSigner {
    key: SigningKey,
    pub key_id: String,
}

impl LocalSigner {
    /// Load the signing key from `dir/local_signing.key`, generating + persisting
    /// it (0600) if absent.
    pub fn load_or_create(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir).context("create data dir")?;
        let path = dir.join("local_signing.key");
        let key = if path.exists() {
            let bytes = std::fs::read(&path).context("read signing key")?;
            let arr: [u8; 32] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("signing key file must be 32 bytes"))?;
            SigningKey::from_bytes(&arr)
        } else {
            let mut seed = [0u8; 32];
            getrandom::getrandom(&mut seed).map_err(|e| anyhow::anyhow!("rng error: {}", e))?;
            let key = SigningKey::from_bytes(&seed);
            write_private(&path, &seed)?;
            key
        };
        let key_id = fingerprint(&key);
        Ok(Self { key, key_id })
    }

    /// Base64 public key โ€” the DEK's local trust store is seeded with this.
    pub fn public_key_b64(&self) -> String {
        use base64::Engine;
        base64::prelude::BASE64_STANDARD.encode(self.key.verifying_key().as_bytes())
    }

    /// Sign canonical bytes, returning base64 signature.
    pub fn sign_b64(&self, bytes: &[u8]) -> String {
        use base64::Engine;
        base64::prelude::BASE64_STANDARD.encode(self.key.sign(bytes).to_bytes())
    }
}

fn fingerprint(key: &SigningKey) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(key.verifying_key().as_bytes());
    format!("local-{}", hex::encode(&digest[..8]))
}

#[cfg(unix)]
fn write_private(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .context("create key file 0600")?;
    f.write_all(bytes)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    // Windows: rely on user-profile ACLs; data dir is per-user.
    std::fs::write(path, bytes).context("write key file")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn persist_and_reload_same_key() {
        let dir = std::env::temp_dir().join(format!("lcp-sign-{}", std::process::id()));
        let s1 = LocalSigner::load_or_create(&dir).unwrap(); //
        let s2 = LocalSigner::load_or_create(&dir).unwrap(); //
        assert_eq!(s1.public_key_b64(), s2.public_key_b64(), "key must persist");
        assert_eq!(s1.key_id, s2.key_id);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn signature_is_deterministic_per_key() {
        let dir = std::env::temp_dir().join(format!("lcp-sign2-{}", std::process::id()));
        let s = LocalSigner::load_or_create(&dir).unwrap(); //
        let a = s.sign_b64(b"hello");
        let b = s.sign_b64(b"hello");
        assert_eq!(a, b, "ed25519 is deterministic");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
