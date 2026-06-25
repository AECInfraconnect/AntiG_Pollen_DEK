use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub seq: u64,
    pub timestamp: String,
    pub payload_json: String,
    pub prev_hash: String,  // hash ของ entry ก่อนหน้า
    pub entry_hash: String, // hash(prev_hash + payload)
}

impl AuditEntry {
    pub fn new(seq: u64, timestamp: String, payload_json: String, prev_hash: &str) -> Self {
        let mut h = Sha256::new();
        h.update(prev_hash.as_bytes());
        h.update(payload_json.as_bytes());
        let entry_hash = h
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Self {
            seq,
            timestamp,
            payload_json,
            prev_hash: prev_hash.to_string(),
            entry_hash,
        }
    }
}

/// ตรวจ chain ทั้งสาย — ถ้ามีใครแก้ entry กลางทาง entry_hash จะไม่ต่อกัน
pub fn verify_chain(entries: &[AuditEntry]) -> Result<(), u64> {
    let mut prev = "GENESIS".to_string();
    for e in entries {
        if e.prev_hash != prev {
            return Err(e.seq);
        }
        let recomputed = AuditEntry::new(e.seq, e.timestamp.clone(), e.payload_json.clone(), &prev);
        if recomputed.entry_hash != e.entry_hash {
            return Err(e.seq); // คืน seq ที่ถูกแก้
        }
        prev = e.entry_hash.clone();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_tampering() {
        let e0 = AuditEntry::new(0, "t0".into(), "decision:allow".into(), "GENESIS");
        let mut e1 = AuditEntry::new(1, "t1".into(), "decision:deny".into(), &e0.entry_hash);
        let chain_ok = vec![e0.clone(), e1.clone()];
        assert!(verify_chain(&chain_ok).is_ok());

        // แก้ payload หลังบันทึก → chain พัง
        e1.payload_json = "decision:allow".into();
        assert_eq!(verify_chain(&[e0, e1]), Err(1));
    }
}
