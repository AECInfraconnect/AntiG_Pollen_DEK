use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("key not found")]
    NotFound,
    #[error("os key store error: {0}")]
    Os(String),
    #[error("invalid key material")]
    Invalid,
}

pub trait OsKeyStore: Send + Sync {
    fn load_or_create_master_key(&self) -> Result<[u8; 32], KeyStoreError>;
    fn rotate_master_key(&self) -> Result<[u8; 32], KeyStoreError>;
}

pub struct SpoolKeyManager<S: OsKeyStore> {
    store: S,
}

impl<S: OsKeyStore> SpoolKeyManager<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn active_aead_key(&self) -> Result<crate::crypto::AeadKey, KeyStoreError> {
        let master = self.store.load_or_create_master_key()?;
        Ok(crate::crypto::AeadKey::new("local-master-v1", master))
    }
}
