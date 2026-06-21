// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![allow(clippy::unwrap_used, clippy::expect_used)]
use criterion::{criterion_group, criterion_main, Criterion};
use dek_cedar::CedarAdapter;
use dek_policy_runtime::PolicyRuntime;
use serde_json::json;
use tokio::runtime::Runtime;

fn bench_authorize(c: &mut Criterion) {
    let policy_src = r#"
permit(
    principal,
    action == Action::"read",
    resource
);
"#;
    let adapter = CedarAdapter::new(policy_src).expect("Failed to create Cedar adapter");
    
    // We will benchmark the `evaluate` method
    let input = json!({
        "principal": "User::\"alice\"",
        "action": "Action::\"read\"",
        "resource": "Resource::\"doc\"",
        "context": {},
        "entities": []
    });

    // Populate cache with one request
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        adapter.evaluate(input.clone()).await.unwrap();
    });

    c.bench_function("cedar_authorize", |b| {
        b.to_async(&rt).iter(|| async {
            let res = adapter.evaluate(input.clone()).await.unwrap();
            assert!(res.allow);
        })
    });
}

criterion_group!(benches, bench_authorize);
criterion_main!(benches);

