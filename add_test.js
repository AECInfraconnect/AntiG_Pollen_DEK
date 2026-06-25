const fs = require("fs");
const file = "crates/dek-secure-spool/src/lib.rs";
let content = fs.readFileSync(file, "utf8");

content += `
    #[tokio::test]
    async fn test_secure_spool_tamper_quarantine() {
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
        
        // Simulate tamper by modifying the file directly
        if let Ok(mut entries) = std::fs::read_dir(&dir) {
            if let Some(Ok(entry)) = entries.next() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("pds") {
                    // write some garbage to invalidate the block/chain
                    std::fs::write(entry.path(), b"GARBAGE").unwrap();
                }
            }
        }

        let err = spool.replay().await;
        assert!(matches!(err, Err(SpoolError::Tampered)));

        // check if quarantine file exists
        let mut quarantined = false;
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("quarantine") {
                    quarantined = true;
                    break;
                }
            }
        }
        assert!(quarantined);

        let _ = fs::remove_dir_all(dir);
    }
`;

fs.writeFileSync(file, content);
console.log("Updated dek-secure-spool tests");

