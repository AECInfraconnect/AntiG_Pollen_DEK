// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::identity_op)]
use dek_policy_runtime::{PolicyRuntime, WasmProfile, WasmtimePolicyRuntime};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn write_wat_fixture(name: &str, content: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("{}.wat", name));
    fs::write(&path, content).unwrap();
    path
}

#[tokio::test]
#[ignore = "Wasmtime fuel traps may cause STATUS_STACK_BUFFER_OVERRUN on Windows debug builds due to unwinding limitations"]
async fn test_wasm_fuel_exhaustion() {
    let wat = r#"
    (module
      (memory (export "memory") 1)
      (func $start (export "_start")
        (loop $my_loop
          br $my_loop
        )
      )
    )
    "#;
    let path = write_wat_fixture("infinite_loop", wat);

    // Profile with very low fuel
    let profile = WasmProfile {
        max_memory_bytes: 10 * 1024 * 1024,
        max_fuel: 100, // Exhaust quickly
    };

    let runtime = WasmtimePolicyRuntime::new(path.to_str().unwrap(), Some(profile)).unwrap();
    let decision = runtime.evaluate(json!({})).await.unwrap();

    // The execution should fail gracefully, and the host shouldn't hang.
    assert!(!decision.allow);
    assert!(decision.reason.contains("WASM execution failed"));
    assert!(decision.reason.contains("fuel"));
}

#[tokio::test]
#[ignore = "Wasmtime memory traps may cause STATUS_STACK_BUFFER_OVERRUN on Windows debug builds due to unwinding limitations"]
async fn test_wasm_memory_exhaustion() {
    let wat = r#"
    (module
      (memory (export "memory") 1)
      (func $start (export "_start")
        (loop $my_loop
          ;; grow memory by 100 pages (6.4MB) in each loop iteration
          (memory.grow (i32.const 100))
          drop
          br $my_loop
        )
      )
    )
    "#;
    let path = write_wat_fixture("memory_leak", wat);

    // Profile with low memory and moderate fuel
    let profile = WasmProfile {
        max_memory_bytes: 1 * 1024 * 1024, // 1 MB limit (less than 100 pages)
        max_fuel: 10_000,
    };

    let runtime = WasmtimePolicyRuntime::new(path.to_str().unwrap(), Some(profile)).unwrap();
    let decision = runtime.evaluate(json!({})).await.unwrap();

    // The execution should fail due to OOM
    assert!(!decision.allow);
    assert!(decision.reason.contains("WASM execution failed"));
    // Reason could mention out of bounds memory access or out of memory.
}
