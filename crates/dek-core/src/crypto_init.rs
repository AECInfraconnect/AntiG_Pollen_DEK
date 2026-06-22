// SPDX-License-Identifier: Apache-2.0

/// ต้องเรียกเป็นบรรทัดแรกใน main() ของทุก binary ที่ทำ TLS.
/// idempotent-safe: ถ้าถูก install แล้วจะ log debug แทน panic.
pub fn install_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| {
                // race: thread อื่น install ก่อน — ปลอดภัย ไม่ใช่ error
                tracing::debug!("crypto provider already installed by another path");
            })
            .ok();
    }
}
