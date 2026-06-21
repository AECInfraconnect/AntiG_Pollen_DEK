#![no_main]

use dek_mcp_normalizer::{http::HttpTransportAdapter, TransportAdapter};
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    // We only care about well-formed JSON to test the normalizer logic
    if let Ok(json_val) = serde_json::from_slice::<Value>(data) {
        let adapter = HttpTransportAdapter;
        
        // Fuzz normalize_request
        let _ = adapter.normalize_request(
            json_val.clone(),
            "fuzz_tenant",
            "fuzz_device",
            Some("spiffe://fuzz"),
            Some("fuzz_user"),
        );

        // Fuzz normalize_response
        let _ = adapter.normalize_response(
            json_val,
            "fuzz_tenant",
            "fuzz_device",
            Some("spiffe://fuzz"),
            Some("fuzz_user"),
        );
    }
});
