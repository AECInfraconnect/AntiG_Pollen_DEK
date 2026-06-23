// SPDX-License-Identifier: Apache-2.0
use sha2::{Digest, Sha256};

/// สร้าง "identity key" ที่เสถียรของ agent หนึ่งตัว (ใช้ merge ข้าม evidence/scan/run)
/// ลำดับความเชื่อถือ: signature_id > vendor+product > exe_path_hash > process_name+name
pub fn identity_key(
    signature_id: Option<&str>,
    vendor: Option<&str>,
    product: Option<&str>,
    exe_path_hash: Option<&str>,
    display_name: &str,
) -> String {
    let basis = if let Some(sig) = signature_id.filter(|s| !s.starts_with("claw_family_generic")) {
        format!("sig:{sig}")
    } else if let (Some(v), Some(p)) = (vendor, product) {
        format!("vp:{}|{}", v.to_lowercase(), p.to_lowercase())
    } else if let Some(h) = exe_path_hash {
        format!("exe:{h}")
    } else {
        format!("name:{}", display_name.to_lowercase())
    };
    let h = Sha256::digest(basis.as_bytes());
    format!("{:x}", h)[..24].to_string()   // 24 hex = พอแยก ไม่ยาว
}

/// candidate_id ที่ deterministic → re-scan แล้ว upsert ตัวเดิม (ไม่ซ้ำ)
pub fn deterministic_candidate_id(tenant: &str, identity_key: &str) -> String {
    format!("cand_{}", &Sha256::digest(format!("{tenant}:{identity_key}").as_bytes())
        .iter().take(8).map(|b| format!("{b:02x}")).collect::<String>())
}
