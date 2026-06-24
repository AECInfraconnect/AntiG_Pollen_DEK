# Pollek Local Enforcement Kit Recorder Integration Wiring

This guide outlines the steps required to seamlessly integrate the shared `dek-metrics` crate into the workspace and wire it to the execution paths of `dek-mcp-proxy` and `dek-ext-authz` during merge.

## 1. Workspace Configuration

Add `dek-metrics` to the workspace `members` array in the root `Cargo.toml`.

```toml
# Cargo.toml (root)
[workspace]
members = [
    # ...
    "crates/dek-metrics",
    # ...
]
```

## 2. Dependency Wiring

Add the `dek-metrics` local dependency to both proxy crates:

```toml
# crates/dek-mcp-proxy/Cargo.toml
# crates/dek-ext-authz/Cargo.toml

[dependencies]
dek-metrics = { path = "../dek-metrics" }
```

Ensure that the target crates enable the necessary tokio features (`time`, `macros`, `sync`), as `dek-metrics` leverages them for background pushing and synchronization:

```toml
tokio = { version = "1.0", features = ["full"] } # Or ["rt", "rt-multi-thread", "time", "macros", "sync"]
```

## 3. Proxy Integration (`dek-mcp-proxy`)

Replace the raw Prometheus recorder bootstrapping in `crates/dek-mcp-proxy/src/main.rs`.

**Before**:

```rust
metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder().unwrap();
```

**After**:

```rust
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use std::time::Duration;

// 1. Install Recorder
let handle = dek_metrics::install_recorder("dek-mcp-proxy").unwrap();

// 2. Setup Client & Notifier for pushing
let client = Arc::new(RwLock::new(reqwest::Client::new())); // Can be updated on mTLS renew
let shutdown = Arc::new(Notify::new());

// 3. Spawn Push Task
dek_metrics::spawn_push(
    handle,
    "https://telemetry.Pollek-cloud.internal/v1/metrics".into(), // Use real push URL from config
    client,
    shutdown.clone(),
    Duration::from_secs(60),
);

// Note: Gracefully notify shutdown before process exit
// shutdown.notify_waiters();
```

Verify that the `BootstrapConfig::load()` API behavior aligns with your environment. Ensure you pass the correct `push_url` depending on the bootstrap parameters instead of the hardcoded internal address.

## 4. Ext-Authz Integration (`dek-ext-authz`)

Similarly, integrate into `crates/dek-ext-authz/src/main.rs` prior to invoking the `Server::serve` gRPC call.

**Before**:

```rust
metrics_exporter_prometheus::PrometheusBuilder::new().install_recorder().unwrap();
```

**After**:

```rust
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use std::time::Duration;

let handle = dek_metrics::install_recorder("dek-ext-authz").unwrap();

let client = Arc::new(RwLock::new(reqwest::Client::new()));
let shutdown = Arc::new(Notify::new());

dek_metrics::spawn_push(
    handle,
    "https://telemetry.Pollek-cloud.internal/v1/metrics".into(),
    client,
    shutdown.clone(),
    Duration::from_secs(60),
);

// ...
```

## 5. Central Supervisor Integration (`dek-core`)

By moving to `dek-metrics`, `dek-core` can share the exact same logic, eliminating the bespoke code in `metrics_push.rs`. The `Arc<RwLock<reqwest::Client>>` passed to `spawn_push` is specifically designed so `dek-core`'s SVID renewal loop can write-lock the client to safely inject a newly minted mTLS identity without restarting the task.

## Future Merge Considerations

- **Target Architectures**: If you wish to natively build for macOS and ARM Linux in the CI release artifacts, modify the `release.yml` matrix by adding `x86_64-apple-darwin` and `aarch64-linux` to the `target` list.
- **SLSA Provenance**: The pipeline currently relies on generic OIDC keyless signatures (`cosign keyless`). To obtain strict SLSA Level 3 guarantees mapping source commit to final artifact, evaluate integrating `slsa-github-generator`.
