const fs = require("fs");
const file = "crates/dek-secure-spool/src/lib.rs";
let content = fs.readFileSync(file, "utf8");

const cleanTestsStart = content.indexOf("#[cfg(test)]\n#[allow(clippy::unwrap_used)]\nmod tests {");
if (cleanTestsStart !== -1) {
    content = content.substring(0, cleanTestsStart);
}

content += `#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    struct DummyKeyStore;
    impl crate::key_manager::OsKeyStore for DummyKeyStore {
        fn load_or_create_master_key(&self) -> Result<[u8; 32], crate::key_manager::KeyStoreError> {
            Ok([0u8; 32])
        }
        fn rotate_master_key(&self) -> Result<[u8; 32], crate::key_manager::KeyStoreError> {
            Ok([0u8; 32])
        }
    }

    #[tokio::test]
    async fn test_spool_enqueue_and_replay() {
        let dir = std::env::temp_dir().join(format!("test_spool_{}", Uuid::new_v4()));
        let km = key_manager::SpoolKeyManager::new(DummyKeyStore);
        let spool = Spool::new(
            dir.clone(),
            1024 * 1024,
            Some(km),
            "test".to_string(),
            "test".to_string(),
        );

        spool.enqueue(b"event1".to_vec()).await.unwrap();
        spool.enqueue(b"event2".to_vec()).await.unwrap();

        let replays = spool.replay().await.unwrap();
        assert_eq!(replays.len(), 2);
        assert_eq!(replays[0].payload_json, "event1");
        assert_eq!(replays[1].payload_json, "event2");

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn test_spool_full() {
        let dir = std::env::temp_dir().join(format!("test_spool_{}", Uuid::new_v4()));
        let km = key_manager::SpoolKeyManager::new(DummyKeyStore);
        let spool = Spool::new(
            dir.clone(),
            10,
            Some(km),
            "test".to_string(),
            "test".to_string(),
        );

        let _ = spool.enqueue(b"event1".to_vec()).await;
        let err = spool
            .enqueue(b"very long event string to fill up spool size quickly".to_vec())
            .await;
        assert!(err.is_err());

        let _ = fs::remove_dir_all(dir);
    }
}
`;

fs.writeFileSync(file, content);
console.log("Cleaned up dek-secure-spool tests");

