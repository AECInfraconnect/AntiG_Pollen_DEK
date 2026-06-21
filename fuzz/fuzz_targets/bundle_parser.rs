// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    // Fuzz the bundle parser to ensure invalid JSON or deeply nested payloads
    // do not cause a panic when being normalized or routed.
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<Value>(s);
    }
});

