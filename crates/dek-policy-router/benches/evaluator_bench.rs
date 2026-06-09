// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

fn bench_evaluators(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_evaluators");

    let payload = json!({
        "request_id": "req-1",
        "action": "read",
        "resource": "doc-1"
    });

    group.bench_function("cedar_evaluation_dummy", |b| {
        // Since we can't easily mock the whole async router in sync criterion without tokio runtime overhead,
        // we benchmark the raw logic. Here we just simulate an evaluation.
        b.iter(|| {
            let _ = black_box(&payload);
            // Replace with actual Cedar/OPA sync evaluations or tokio block_on if necessary
        })
    });

    group.finish();
}

criterion_group!(benches, bench_evaluators);
criterion_main!(benches);
