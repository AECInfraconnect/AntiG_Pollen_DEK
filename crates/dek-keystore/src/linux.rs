use crate::Keystore;
use anyhow::{Context, Result};
use linux_keyutils::{Key, KeyRing, KeyRingIdentifier};

pub struct KernelKeystore {}

impl KernelKeystore {
    pub fn new() -> Self {
        Self {}
    }
}

impl Keystore for KernelKeystore {
    fn store_key(&self, alias: &str, data: &[u8]) -> Result<()> {
        tracing::info!("Writing {} to Linux User Keyring", alias);
        let _key = Key::add(
            alias,
            data,
            KeyRingIdentifier::User,
            None,
        ).context("Failed to add key to user keyring")?;
        Ok(())
    }

    fn load_key(&self, alias: &str) -> Result<Vec<u8>> {
        tracing::info!("Reading {} from Linux User Keyring", alias);
        let keyring = KeyRing::from(KeyRingIdentifier::User);
        let key = keyring.search::<Key>(alias).context("Failed to search for key")?;
        let mut data = vec![0u8; 2048]; // Max reasonable size for a key
        let len = key.read(&mut data).context("Failed to read key data")?;
        data.truncate(len);
        Ok(data)
    }

    fn delete_key(&self, alias: &str) -> Result<()> {
        let keyring = KeyRing::from(KeyRingIdentifier::User);
        if let Ok(key) = keyring.search::<Key>(alias) {
            let _ = key.invalidate();
        }
        Ok(())
    }
}
