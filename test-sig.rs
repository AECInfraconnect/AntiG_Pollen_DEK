use ed25519_dalek::{Signer, SigningKey, Verifier};
use dek_control_plane_api::bundle::{BundleArtifact, PollenPolicyBundleManifest};
use sha2::Digest;
use base64::Engine;

fn main() {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).unwrap();
    let signing_key = SigningKey::from_bytes(&seed);
    
    let mut manifest = PollenPolicyBundleManifest {
        bundle_version: "v1".to_string(),
        bundle_id: "bundle-local-1".to_string(),
        tenant_id: "local".to_string(),
        workspace_id: "default".to_string(),
        environment_id: "local".to_string(),
        build_number: 1,
        created_at: "2026-06-09T16:08:17.169165500+00:00".to_string(),
        created_by: "local-admin".to_string(),
        registry_snapshot_sha256: "4e7d2773e89b75eaf683b4604e5c510a08e8f8c423e18d1420fab0f483b06501".to_string(),
        router_config_sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".to_string(),
        artifacts: vec![BundleArtifact {
            artifact_id: "e2e allow".to_string(),
            adapter_id: "cedar".to_string(),
            artifact_type: "cedar_text".to_string(),
            path: "artifacts/bacfa3fab8b34a87fbe68d2b949b8fb62bc093942a84fd9a08543f15a0b63ee2".to_string(),
            sha256: "bacfa3fab8b34a87fbe68d2b949b8fb62bc093942a84fd9a08543f15a0b63ee2".to_string(),
            size_bytes: 36,
        }],
        signatures: vec![],
        min_dek_version: "0.1.0".to_string(),
        rollback_from: None,
    };
    
    let signed_bytes = serde_json::to_vec(&manifest).unwrap();
    println!("LCP signed bytes len: {}", signed_bytes.len());
    println!("LCP bytes: {}", String::from_utf8_lossy(&signed_bytes));
    
    let sig = signing_key.sign(&signed_bytes);
    let sig_b64 = base64::prelude::BASE64_STANDARD.encode(sig.to_bytes());
    
    // Simulate what dek-bundle-sync does
    let manifest_val = serde_json::to_value(&manifest).unwrap();
    
    // In dek-bundle-sync
    let mut manifest_sync: PollenPolicyBundleManifest = serde_json::from_value(manifest_val).unwrap();
    manifest_sync.signatures.clear();
    let sync_bytes = serde_json::to_vec(&manifest_sync).unwrap();
    println!("SYNC signed bytes len: {}", sync_bytes.len());
    
    let sig_bytes = base64::prelude::BASE64_STANDARD.decode(&sig_b64).unwrap();
    let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().unwrap();
    let signature = ed25519_dalek::Signature::from_bytes(&sig_arr);
    
    let is_ok = signing_key.verifying_key().verify(&sync_bytes, &signature).is_ok();
    println!("Verification OK: {}", is_ok);
}
