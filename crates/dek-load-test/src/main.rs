// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use anyhow::Result;
use clap::Parser;
use hdrhistogram::Histogram;
use reqwest::{Client, Method};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL to test
    #[arg(short, long, default_value = "http://127.0.0.1:43890/v1/decision/check")]
    target: String,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 20)]
    concurrency: usize,

    /// Duration of the test in seconds
    #[arg(short, long, default_value_t = 10)]
    duration: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    println!("Starting load test...");
    println!("Target: {}", args.target);
    println!("Concurrency: {}", args.concurrency);
    println!("Duration: {}s", args.duration);

    let client = Client::builder()
        .pool_max_idle_per_host(args.concurrency)
        .build()?;

    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    // Channel for collecting latencies
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<u64>();

    let start_time = Instant::now();
    let duration = Duration::from_secs(args.duration);

    let client = client.clone();
    let target = args.target.clone();
    let concurrency = args.concurrency;

    let success_count_ref = Arc::clone(&success_count);
    let error_count_ref = Arc::clone(&error_count);

    // Payload to send
    let payload = serde_json::json!({
        "request_id": "test-req-123",
        "tenant_id": "tenant-A",
        "device_id": "dev-001",
        "principal": {
            "id": "user-1",
            "roles": ["employee"]
        },
        "action": "read",
        "resource": {
            "kind": "document",
            "id": "doc-1"
        },
        "context": {},
        "input_hash": "hash"
    });

    // Spawn workers
    let tasks: Vec<_> = (0..concurrency)
        .map(|_| {
            let client = client.clone();
            let target = target.clone();
            let tx = tx.clone();
            let success_count = Arc::clone(&success_count_ref);
            let error_count = Arc::clone(&error_count_ref);
            let payload = payload.clone();

            tokio::spawn(async move {
                while start_time.elapsed() < duration {
                    let req_start = Instant::now();
                    let req = client.request(Method::POST, &target).json(&payload);
                    
                    match req.send().await {
                        Ok(resp) if resp.status().is_success() => {
                            let latency = req_start.elapsed().as_micros() as u64;
                            success_count.fetch_add(1, Ordering::Relaxed);
                            let _ = tx.send(latency);
                        }
                        Ok(resp) => {
                            tracing::warn!("Request failed with status: {}", resp.status());
                            error_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => {
                            tracing::warn!("Request error: {}", e);
                            error_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            })
        })
        .collect();

    // Drop the original sender so the receiver will close when tasks finish
    drop(tx);

    let mut hist = Histogram::<u64>::new(3).unwrap();
    
    // Collect all latencies
    while let Some(latency) = rx.recv().await {
        hist.record(latency).unwrap_or_else(|e| {
            tracing::warn!("Failed to record latency: {}", e);
        });
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let successes = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    let total = successes + errors;

    println!("\n--- Load Test Results ---");
    println!("Total requests: {}", total);
    println!("Successful:     {}", successes);
    println!("Errors:         {}", errors);
    if total > 0 {
        println!("Success rate:   {:.2}%", (successes as f64 / total as f64) * 100.0);
        println!("Req/sec:        {:.2}", total as f64 / elapsed);
    }

    if successes > 0 {
        println!("\n--- Latency Distribution ---");
        println!("P50:  {:.2} ms", hist.value_at_quantile(0.5) as f64 / 1000.0);
        println!("P90:  {:.2} ms", hist.value_at_quantile(0.9) as f64 / 1000.0);
        println!("P95:  {:.2} ms", hist.value_at_quantile(0.95) as f64 / 1000.0);
        println!("P99:  {:.2} ms", hist.value_at_quantile(0.99) as f64 / 1000.0);
        println!("Max:  {:.2} ms", hist.max() as f64 / 1000.0);
        println!("Mean: {:.2} ms", hist.mean() / 1000.0);
    }

    Ok(())
}

