use clap::Parser;
use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::info;
use tracing_subscriber;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "http://127.0.0.1:8443")]
    target: String,

    #[arg(short, long, default_value_t = 1000)]
    concurrency: usize,

    #[arg(short, long, default_value_t = 10000)]
    requests: usize,
    
    #[arg(short, long, default_value = "tenant-1")]
    tenant_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!("Starting Soak Test Generator");
    info!("Target: {}", args.target);
    info!("Concurrency: {}", args.concurrency);
    info!("Total Requests: {}", args.requests);

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .pool_max_idle_per_host(args.concurrency)
        .build()?;

    let start_time = Instant::now();

    let mut set = tokio::task::JoinSet::new();

    // Create a stream of requests to make
    for i in 0..args.requests {
        let client = client.clone();
        let target = args.target.clone();
        let tenant_id = args.tenant_id.clone();
        
        set.spawn(async move {
            let start = Instant::now();
            let payload = json!({
                "events": [{
                    "Decision": {
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "device_id": format!("device-{}", i % 10),
                        "action": "tools/call",
                        "resource": "mcp_tool",
                        "decision": "Permit",
                        "reason": "Matched policy",
                        "context": {}
                    }
                }]
            });
            
            let url = format!("{}/v1/tenants/{}/telemetry/events", target, tenant_id);
            let res = client.post(&url).json(&payload).send().await;
            
            let elapsed = start.elapsed();
            (res, elapsed)
        });
    }

    // Run concurrently
    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(data) = res {
            results.push(data);
        }
    }

    let total_time = start_time.elapsed();
    let mut successes = 0;
    let mut failures = 0;
    let mut max_latency = Duration::from_millis(0);
    let mut min_latency = Duration::from_secs(100);
    let mut total_latency = Duration::from_millis(0);

    for (res, latency) in results {
        if res.is_ok() && res.unwrap().status().is_success() {
            successes += 1;
        } else {
            failures += 1;
        }
        
        if latency > max_latency { max_latency = latency; }
        if latency < min_latency { min_latency = latency; }
        total_latency += latency;
    }

    let avg_latency = total_latency / args.requests as u32;
    let rps = args.requests as f64 / total_time.as_secs_f64();

    info!("--- Soak Test Results ---");
    info!("Total Time: {:?}", total_time);
    info!("Requests/sec: {:.2}", rps);
    info!("Successes: {}", successes);
    info!("Failures: {}", failures);
    info!("Avg Latency: {:?}", avg_latency);
    info!("Max Latency: {:?}", max_latency);
    info!("Min Latency: {:?}", min_latency);

    Ok(())
}
