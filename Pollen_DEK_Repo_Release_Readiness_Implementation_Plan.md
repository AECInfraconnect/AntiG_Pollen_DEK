# Pollen DEK Repository Release Readiness & Full-System Implementation Plan

**Target repo:** `AECInfraconnect/AntiG_Pollen_DEK`  
**Analysis date:** 2026-06-18  
**Target release:** `v1.0.0-beta.6` → `v1.0.0-beta.7` → `v1.0.0`  
**Goal:** Make DEK, Local Control Plane, Local Admin Dashboard, Mock-Cloud, Contract Hub, policy runtime, telemetry, and OS enforcement modules buildable, testable, releaseable, and safe to connect to a separate Pollen Cloud repo without contract drift.

> This implementation plan is written for an AI coding agent or engineering team. It is intentionally prescriptive: apply items in order, do not skip release blockers, and keep CI/docs updated as code evolves.

---

## 0. Executive Summary

The repository is substantially more mature than the earlier prototype state. It now includes:

- Apache-2.0 open-source positioning for DEK with Pollen Cloud commercial separation.
- Rust workspace with multiple DEK crates and a generated `pollen-contract` crate under `contracts/generated/rust/pollen-contract`.
- Contract Hub staging under `contracts/` with TypeSpec, OpenAPI generation, AsyncAPI copy, JSON Schema validation, capability catalog, and CI.
- Release supply-chain workflow with auditable Rust builds, SBOM, SHA256 checksums, and cosign keyless signing.
- Local Control Plane and Mock-Cloud that both aim to implement the same DEK↔Cloud contract.
- Wasmtime plugin pooling work, secure telemetry spool work, eBPF DNS/LRU work, and platform modules for WFP/NEFilter.

However, the repo is **not yet release-ready**. The most urgent blockers are not architectural; they are consistency and build/release correctness issues:

1. **Workspace/CI mismatch:** root `Cargo.toml` excludes `crates/dek-ebpf-prog` and `crates/dek-ebpfd`, while CI still uses `cargo ... --workspace --exclude dek-ebpf-prog --exclude dek-ebpfd`. Because excluded crates are not workspace members, this can fail or hide intended checks depending on Cargo behavior and command context.
2. **`dek-core` likely compile blocker:** `supervisor.rs` references `jwks_tx` in the trust-bundle poller path, but the inspected file section does not show any declaration. Fix before any release candidate.
3. **Local Mode bundle endpoint mismatch:** `BundleSyncAgent::run_pipeline_local()` fetches `/bundles/manifest`, but `local-control-plane` exposes `/bundles/latest`, not `/bundles/manifest`.
4. **Contract drift already exists:** TypeSpec says `/bundles/latest` returns `BundleFetchResponse { schema_version, status, envelope }`, while Local Control Plane returns the raw envelope; Mock-Cloud discovery response shape differs from TypeSpec; telemetry batch response shape differs from TypeSpec.
5. **WFP and NEFilter are stubs that return success:** They advertise capability in `contracts/catalog/capabilities.yaml`, but current implementations only log and return `Ok(())`; this creates a dangerous false sense of enforcement.
6. **eBPF has fail-open paths and unfinished DNS cache lifecycle:** `dek_connect4` defaults to allow on internal errors, DNS cache LRU map exists, but userspace update/janitor logic is mostly TODO.
7. **CI is incomplete:** Dashboard tests are not wired into CI; contract CI does not call `lint:semantic`; release packaging likely misses key artifacts and may use incorrect eBPF object path.
8. **Clippy strict lint risk:** Workspace denies `clippy::panic`, but several production paths still use explicit panic or `unwrap_or_else(|| panic!(...))`.

The plan below converts these into ordered phases with exact code examples.

---

## 1. Source-of-Truth Observations from Current Repo

### 1.1 README / Architecture

Current README positions DEK as a local PEP/PDP that can run fully locally or connect to commercial Pollen Cloud, sharing schema/bundle/telemetry contracts across both modes. This is the correct open-core strategy.

The architecture file describes `dek-core`, `dek-policy-syncer`, `dek-bundle-sync`, `dek-activation`, `dek-secure-spool`, `dek-policy-router`, `dek-policy-runtime`, PDP adapters, OS modules, Local Control Plane, Local Admin Dashboard, and Mock-Cloud. It also declares fail-closed behavior: no bundle, stale bundle, PDP down, SVID failure, or kernel rule apply failure should result in deny/fail-closed behavior.

### 1.2 Root Cargo workspace

Current root workspace includes:

```toml
[workspace]
members = [
    "crates/*",
    "plugins/*",
    "contracts/generated/rust/pollen-contract"
]
exclude = [
    "crates/dek-ebpf-prog",
    "crates/dek-ebpfd"
]
resolver = "2"
```

This means the eBPF daemon/prog are not workspace members despite CI trying to exclude them from workspace commands. Fix by either:

- making `dek-ebpfd` a workspace member and only excluding `dek-ebpf-prog`, or
- keeping both excluded and never mentioning them in `cargo --workspace --exclude ...` commands.

Recommended: **make `dek-ebpfd` a workspace member**. It is a userspace Rust daemon and should pass normal `cargo check/clippy/test` on Linux. Keep only `dek-ebpf-prog` excluded because it requires nightly/BPF target.

### 1.3 Contract Hub staging

Current Contract Hub is good but incomplete:

- `contracts/spec/main.tsp` imports REST APIs and declares `Pollen DEK Control-Plane API` version `1.0.0`.
- `common.tsp` defines `X-Pollen-Contract-Version`, `X-Pollen-Device-Id`, `X-Pollen-Tenant-Id`, and `PollenError`.
- `contract-discovery.tsp` expects:

```json
{
  "schema_version": "contract-discovery.v1",
  "supported": ["1.0"],
  "preferred": "1.0",
  "minimum_dek_version": "1.0.0-beta.5",
  "sunset": {},
  "capabilities": []
}
```

- `bundles.tsp` defines `/v1/tenants/{tenant_id}/devices/{device_id}/bundles/latest` as POST and expects a `BundleFetchResponse` wrapper.
- `telemetry.tsp` defines `/v1/telemetry/batches` with `TelemetryIngestResponse { accepted, rejected, retry_after_seconds? }`.
- `bundle-envelope.v1.schema.json` correctly formalizes the Local Mode envelope shape with `schema_version`, `manifest`, `signatures`.

But implementations do not yet consistently match this contract.

---

## 2. Release Readiness Levels

Use these levels to avoid falsely calling the repo release-ready.

| Level | Name | Meaning | Allowed release? |
|---|---|---|---|
| R0 | Compiles | Main workspace builds on Linux/macOS/Windows; eBPF builds on Linux; dashboard builds | No |
| R1 | Testable | Unit/integration/contract tests pass; dashboard tests pass | Internal beta only |
| R2 | Contract-correct | LCP + Mock-Cloud conform to generated OpenAPI/JSON Schema/AsyncAPI | Beta public |
| R3 | Fail-closed correct | No advertised enforcement capability is a no-op; unknown enforcement = disabled capability or hard error | Beta public |
| R4 | Releaseable | SBOM, signatures, installers/packages, docs, upgrade/rollback, smoke tests pass | Public release |
| R5 | Cloud-ready | Separate Pollen Cloud repo can consume contract artifacts and pass provider conformance tests | Cloud alpha/beta |

Current repo is approximately **R0-minus / R1-partial**. It has many parts, but likely has CI/build blockers and contract drift.

---

## 3. Phase 0 — Immediate CI Compile Blockers

### 3.1 Fix workspace membership and CI cargo commands

#### Problem

Root workspace excludes both `dek-ebpf-prog` and `dek-ebpfd`, but `ci.yml` runs:

```yaml
cargo clippy --workspace --exclude dek-ebpf-prog --exclude dek-ebpfd --exclude pii-redactor-plugin --all-targets --all-features -- -D warnings
```

This is fragile because `dek-ebpfd` is not a workspace member. Also `--all-features` on every OS can activate feature combinations that are not meaningful across OS modules.

#### Recommended root `Cargo.toml`

```toml
[workspace]
members = [
    "crates/*",
    "plugins/*",
    "contracts/generated/rust/pollen-contract"
]
exclude = [
    "crates/dek-ebpf-prog"
]
resolver = "2"

[workspace.package]
version = "1.0.0-beta.6"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/AECInfraconnect/AntiG_Pollen_DEK"
rust-version = "1.85"
```

#### Recommended CI cargo commands

Replace broad `--all-features` with explicit profiles:

```yaml
- name: Check Formatting
  run: cargo fmt --all -- --check

- name: Check default workspace
  run: cargo check --workspace --exclude dek-ebpf-prog --exclude pii-redactor-plugin --locked

- name: Clippy default workspace
  run: cargo clippy --workspace --exclude dek-ebpf-prog --exclude pii-redactor-plugin --all-targets --locked -- -D warnings

- name: Test default workspace
  run: cargo test --workspace --exclude dek-ebpf-prog --exclude pii-redactor-plugin --locked

- name: Build release default workspace
  run: cargo build --workspace --exclude dek-ebpf-prog --exclude pii-redactor-plugin --release --locked
```

Add OS-specific feature jobs only where meaningful:

```yaml
os_specific:
  strategy:
    matrix:
      include:
        - os: ubuntu-latest
          command: cargo check -p dek-core --features linux-ebpf --locked
        - os: windows-latest
          command: cargo check -p dek-core --locked
        - os: macos-latest
          command: cargo check -p dek-core --locked
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: ${{ matrix.command }}
```

> Note: Add features only if they exist. If `linux-ebpf` does not exist, do not invent it; instead rely on `target.'cfg(...)'.dependencies` and platform-specific jobs.

---

### 3.2 Fix `jwks_tx` compile blocker in `dek-core::supervisor`

#### Problem

`supervisor.rs` calls:

```rust
dek_spire_node::spawn_trust_bundle_poller(
    tb_client,
    self.cloud_url.clone(),
    self.bootstrap.mtls.root_ca_path.clone(),
    jwks_tx,
    roots_changed_tx,
    self.cancel.clone(),
);
```

The inspected file does not show `jwks_tx` being declared before use. If this is not declared elsewhere via macro or conditional compilation, `dek-core` will not compile.

#### Implementation option A — create a real JWKS watch channel

Add before `roots_changed_tx`:

```rust
let (jwks_tx, _jwks_rx) = tokio::sync::watch::channel::<serde_json::Value>(serde_json::json!({
    "keys": []
}));
let (roots_changed_tx, mut roots_changed_rx) = tokio::sync::watch::channel(0u64);
```

If no consumer exists yet, make it explicit:

```rust
// TODO(pollen-cloud): wire _jwks_rx into JWT-SVID validator once JWT-SVID protected endpoints are active.
```

#### Implementation option B — change `spawn_trust_bundle_poller` API

If JWKS is not used, remove the `jwks_tx` parameter from `dek-spire-node::spawn_trust_bundle_poller` and all call sites.

#### Acceptance criteria

```bash
cargo check -p dek-core --locked
cargo test -p dek-core --locked
```

---

### 3.3 Remove production panic paths under strict clippy

Workspace lints deny `clippy::panic`, `unwrap_used`, `expect_used`, `todo`, and `unimplemented`.

Current risk examples:

- `dek-secure-spool::crypto::RecordAad::to_bytes()` uses `unwrap_or_else(|e| panic!(...))`.
- `dek-wasm-host::pool::WorkerLease::worker_mut()` and `take()` use `unwrap_or_else(|| panic!(...))`.
- `local-control-plane::bundle.rs` globally allows `clippy::panic`; keep test helpers allowed, but avoid production `panic!` in signing/serialization paths.

#### Replace `RecordAad::to_bytes()`

```rust
impl RecordAad {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}
```

Then update encryption/decryption:

```rust
let aad_bytes = aad.to_bytes().map_err(|_| CryptoError::Encrypt)?;
```

```rust
let aad_bytes = record.aad.to_bytes().map_err(|_| CryptoError::Decrypt)?;
```

#### Replace `WorkerLease` panic with Result

```rust
impl WorkerLease {
    pub fn worker_mut(&mut self) -> anyhow::Result<&mut PluginWorker> {
        self.worker
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("worker lease already released"))
    }

    fn take(mut self) -> anyhow::Result<PluginWorker> {
        self.worker
            .take()
            .ok_or_else(|| anyhow::anyhow!("worker lease already released"))
    }
}
```

Update release:

```rust
pub async fn release(&self, lease: WorkerLease) -> Result<()> {
    let mut worker = lease.take()?;
    // ... existing logic
    Ok(())
}
```

---

## 4. Phase 1 — Contract Hub Must Become Runtime Truth

### 4.1 Implement `/.well-known/pollen-contract` in Local Control Plane

#### Problem

TypeSpec defines `/.well-known/pollen-contract`, Mock-Cloud exposes a minimal endpoint, but Local Control Plane app currently does not route it.

#### Required response shape

```json
{
  "schema_version": "contract-discovery.v1",
  "supported": ["1.0"],
  "preferred": "1.0",
  "minimum_dek_version": "1.0.0-beta.6",
  "sunset": {
    "0.9": "2026-10-01T00:00:00Z"
  },
  "capabilities": [
    "contract.discovery.v1",
    "bundle.signed-envelope.v1",
    "telemetry.batch.v1",
    "policy.opa-wasm.v1",
    "policy.cedar.v1",
    "policy.openfga.v1"
  ]
}
```

Do not advertise OS-specific L4 capabilities unless they are real on the running OS.

#### Code: `crates/local-control-plane/src/discovery.rs`

```rust
use axum::{Json, Router, routing::get};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/.well-known/pollen-contract", get(get_discovery))
}

async fn get_discovery() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "schema_version": "contract-discovery.v1",
        "supported": ["1.0"],
        "preferred": "1.0",
        "minimum_dek_version": "1.0.0-beta.6",
        "sunset": { "0.9": "2026-10-01T00:00:00Z" },
        "capabilities": [
            "contract.discovery.v1",
            "bundle.signed-envelope.v1",
            "telemetry.batch.v1",
            "policy.opa-wasm.v1",
            "policy.cedar.v1",
            "policy.openfga.v1"
        ]
    }))
}
```

#### Wire in `app.rs`

```rust
use crate::{auth, bundle, connectors, discovery, policy, push, registry, state::AppState, telemetry};

pub fn create_app(state: AppState, static_dir: &str) -> Router {
    let public_routes = Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .merge(discovery::router());

    let api_routes = Router::new()
        .merge(registry::router())
        .merge(policy::router())
        .merge(telemetry::router())
        .merge(bundle::router())
        .merge(connectors::router())
        .route("/v1/push", axum::routing::get(push::sse_handler))
        .layer(axum::middleware::from_fn_with_state(state.clone(), local_tenant_guard))
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth::require_token));

    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .fallback_service(
            ServeDir::new(static_dir)
                .not_found_service(ServeFile::new(format!("{}/index.html", static_dir))),
        )
        .with_state(state)
}
```

Discovery should be public because DEK needs to know how to talk before authenticated calls; do not include secrets in this response.

---

### 4.2 Fix Bundle endpoint mismatch

#### Current mismatch

- TypeSpec: `POST /v1/tenants/{tenant_id}/devices/{device_id}/bundles/latest` returns wrapper.
- Local BundleSyncAgent local mode: `GET /v1/tenants/{tenant}/devices/{device}/bundles/manifest`.
- Local Control Plane: `POST /v1/tenants/:tenant/devices/:device/bundles/latest` returns raw envelope.

#### Required compatibility fix

For beta.6, support both endpoints:

1. `/bundles/latest` — contract-first wrapper.
2. `/bundles/manifest` — legacy alias returning raw envelope until DEK syncer is migrated.

#### Code: Local Control Plane `bundle.rs`

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/tenants/:tenant/devices/:device/bundles/latest",
            axum::routing::post(get_latest_bundle),
        )
        .route(
            "/v1/tenants/:tenant/devices/:device/bundles/manifest",
            axum::routing::get(get_manifest_legacy),
        )
        .route(
            "/v1/tenants/:tenant/devices/:device/bundles/artifacts/:sha",
            axum::routing::get(get_artifact),
        )
        // existing routes...
}

async fn get_latest_bundle(
    Path((tenant, _device)): Path<(String, String)>,
    State(st): State<AppState>,
    body: Option<Json<serde_json::Value>>,
) -> ApiResult<Json<serde_json::Value>> {
    let _ = body;
    match st.policy_store.get_policy_raw(&tenant, "bundle:latest").await {
        Ok(Some(envelope)) => Ok(Json(serde_json::json!({
            "schema_version": "bundle-fetch-response.v1",
            "status": "bundle_ready",
            "generation": st.build_number.load(std::sync::atomic::Ordering::SeqCst),
            "envelope": envelope
        }))),
        Ok(None) => Ok(Json(serde_json::json!({
            "schema_version": "bundle-fetch-response.v1",
            "status": "not_modified"
        }))),
        Err(e) => Err(ApiError::Internal(e)),
    }
}

async fn get_manifest_legacy(
    Path((tenant, _device)): Path<(String, String)>,
    State(st): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    match st.policy_store.get_policy_raw(&tenant, "bundle:latest").await {
        Ok(Some(envelope)) => Ok(Json(envelope)),
        Ok(None) => Err(ApiError::NotFound("bundle".into())),
        Err(e) => Err(ApiError::Internal(e)),
    }
}
```

#### Next step after beta.6

Change `BundleSyncAgent::run_pipeline_local()` to call `/bundles/latest`, parse `envelope`, then remove `/bundles/manifest` alias after one compatibility window.

---

### 4.3 Fix `bundle-envelope.v1` implementation

Schema requires `schema_version`, `manifest`, and `signatures`. Current Local Control Plane envelope builder emits only `manifest` and `signatures`.

#### Fix

```rust
let envelope = serde_json::json!({
    "schema_version": "bundle-envelope.v1",
    "manifest": manifest,
    "signatures": [{
        "signature_id": format!("sig-{}", bundle_version),
        "signature_type": "ed25519",
        "payload": sig_b64,
        "public_key_fingerprint": _signer.key_id.clone(),
    }]
});
```

#### Update DEK local parser

```rust
if manifest_val.get("schema_version").and_then(|v| v.as_str()) != Some("bundle-envelope.v1") {
    anyhow::bail!("unsupported bundle envelope schema_version");
}
```

---

### 4.4 Fix Mock-Cloud discovery response shape

Current Mock-Cloud returns:

```json
{
  "contract_version": "1.0",
  "capabilities": ["telemetry-batches", "sse-hot-reload"]
}
```

Replace with same response shape as TypeSpec.

```rust
.route(
    "/.well-known/pollen-contract",
    get(|| async {
        Json(serde_json::json!({
            "schema_version": "contract-discovery.v1",
            "supported": ["1.0"],
            "preferred": "1.0",
            "minimum_dek_version": "1.0.0-beta.6",
            "sunset": { "0.9": "2026-10-01T00:00:00Z" },
            "capabilities": [
                "contract.discovery.v1",
                "bundle.signed-envelope.v1",
                "sse.bundle-ready.v1",
                "telemetry.batch.v1",
                "policy.opa-wasm.v1",
                "policy.cedar.v1",
                "policy.openfga.v1",
                "os.l4.ebpf.v1"
            ]
        }))
    }),
)
```

Do not include `os.l4.wfp.v1` or `os.l4.nefilter.v1` unless running on Windows/macOS with a real implementation.

---

### 4.5 Fix telemetry batch response shape

TypeSpec requires:

```json
{
  "schema_version": "telemetry-ingest-response.v1",
  "accepted": 10,
  "rejected": 0,
  "retry_after_seconds": 30
}
```

Mock-Cloud currently returns `{ "status": "ingested", "kind": "batches", "count": n }`.

#### Fix `ingest_batches`

```rust
async fn ingest_batches(
    State(s): State<AppState>,
    Json(p): Json<TelemetryBatchRequest>,
) -> impl IntoResponse {
    if p.schema_version != "telemetry-batch.v1" {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": "UNSUPPORTED_SCHEMA_VERSION",
                "message": "expected telemetry-batch.v1"
            })),
        );
    }

    let mut logs = s.telemetry_events.lock().unwrap();
    let mut accepted = 0;
    for event in p.events {
        logs.push_front(event);
        if logs.len() > 2000 { logs.pop_back(); }
        accepted += 1;
    }
    drop(logs);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": "telemetry-ingest-response.v1",
            "accepted": accepted,
            "rejected": 0
        })),
    )
}
```

Apply the same response in Local Control Plane.

---

## 5. Phase 2 — CI Must Prove No Drift

### 5.1 Fix `contract-ci.yml`

#### Current gaps

- CI validates build/lint/openapi/asyncapi/schema/generated drift.
- It does **not** run `npm run lint:semantic` despite `package.json` defining it.
- The oasdiff gate writes `/tmp/base.yaml` and then uses `hashFiles('/tmp/base.yaml')`; GitHub `hashFiles()` is workspace-oriented and may not work as intended for `/tmp`.
- Docs-must-change is good; keep it.

#### Recommended workflow

```yaml
name: contract-ci

on:
  pull_request:
    paths:
      - 'contracts/**'
      - '.spectral.yaml'
      - '.github/workflows/contract-ci.yml'
  push:
    branches: [main]
    paths:
      - 'contracts/**'
      - '.spectral.yaml'

permissions:
  contents: read

jobs:
  validate-build-and-drift:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: contracts
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: npm
          cache-dependency-path: contracts/package-lock.json
      - run: npm ci
      - run: npm run build
      - run: npm run lint:openapi
      - run: npm run lint:asyncapi
      - run: npm run validate:schemas
      - run: npm run lint:semantic
      - run: npm run check:generated

  breaking-change-gate:
    runs-on: ubuntu-latest
    needs: validate-build-and-drift
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install oasdiff
        run: |
          curl -L https://github.com/oasdiff/oasdiff/releases/latest/download/oasdiff_linux_amd64.tar.gz -o /tmp/oasdiff.tar.gz
          tar -xzf /tmp/oasdiff.tar.gz -C /tmp
          sudo mv /tmp/oasdiff /usr/local/bin/oasdiff
      - name: Generate current spec
        working-directory: contracts
        run: npm ci && npm run build
      - name: Export base spec
        run: |
          set +e
          git show origin/${{ github.base_ref }}:contracts/generated/openapi/pollen.v1.yaml > /tmp/base.yaml
          echo "status=$?" >> $GITHUB_OUTPUT
        id: base
      - name: Breaking change check
        if: steps.base.outputs.status == '0'
        run: |
          oasdiff breaking /tmp/base.yaml contracts/generated/openapi/pollen.v1.yaml --fail-on ERR
```

### 5.2 Add provider conformance tests for LCP and Mock-Cloud

Create a shared test helper crate or test module:

```text
crates/contract-conformance/
├── Cargo.toml
└── src/lib.rs
```

#### Example conformance helper

```rust
use jsonschema::validator_for;
use serde_json::Value;

pub fn validate_schema(schema_str: &str, payload: &Value) -> anyhow::Result<()> {
    let schema: Value = serde_json::from_str(schema_str)?;
    let validator = validator_for(&schema)?;
    validator.validate(payload)?;
    Ok(())
}

pub fn assert_contract_discovery(v: &Value) -> anyhow::Result<()> {
    anyhow::ensure!(v["schema_version"] == "contract-discovery.v1");
    anyhow::ensure!(v["supported"].as_array().is_some());
    anyhow::ensure!(v["preferred"].as_str().is_some());
    anyhow::ensure!(v["capabilities"].as_array().is_some());
    Ok(())
}
```

#### Required conformance cases

For both `local-control-plane` and `mock-cloud`:

```text
GET  /.well-known/pollen-contract
POST /v1/telemetry/batches
POST /v1/tenants/local/devices/device-001/bundles/latest
GET  /v1/tenants/local/devices/device-001/bundles/manifest   # legacy only
GET  /v1/tenants/local/devices/device-001/bundles/artifacts/{sha}
GET  /v1/tenants/local/devices/device-001/trusted-keys
GET  /v1/tenants/local/devices/device-001/events             # SSE smoke
```

### 5.3 Add Local Admin Dashboard CI

Current dashboard has scripts for build, typecheck, lint, vitest, and Playwright, but no workflow is present.

Create `.github/workflows/dashboard-ci.yml`:

```yaml
name: dashboard-ci

on:
  pull_request:
    paths:
      - 'apps/local-admin-dashboard/**'
      - 'contracts/generated/typescript/**'
      - '.github/workflows/dashboard-ci.yml'
  push:
    branches: [main]
    paths:
      - 'apps/local-admin-dashboard/**'
      - 'contracts/generated/typescript/**'

permissions:
  contents: read

jobs:
  dashboard:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: apps/local-admin-dashboard
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: npm
          cache-dependency-path: apps/local-admin-dashboard/package-lock.json
      - run: npm ci
      - run: npm run typecheck
      - run: npm run lint
      - run: npm run test
      - run: npm run build
```

If dashboard has no `package-lock.json`, generate and commit it.

### 5.4 Add release dry-run workflow

Release workflow should not be tested only on tags. Add a PR-triggered dry-run that builds packages but does not publish.

```yaml
name: release-dry-run

on:
  pull_request:
    paths:
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'crates/**'
      - 'plugins/**'
      - 'packaging/**'
      - '.github/workflows/release.yml'
      - '.github/workflows/release-dry-run.yml'

permissions:
  contents: read
  id-token: write

jobs:
  dry_run:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --workspace --exclude dek-ebpf-prog --exclude pii-redactor-plugin --release --target ${{ matrix.target }} --locked
```

---

## 6. Phase 3 — OS L4 Enforcement Reality Check

### 6.1 Capability registry must not advertise stubs

Current `capabilities.yaml` includes:

```yaml
- id: os.l4.wfp.v1
- id: os.l4.nefilter.v1
```

But `dek-windows-wfp` and `dek-macos-nefilter` are stubs that only log.

#### Rule

A capability means “safe to rely on in enforcement.” Therefore:

- If WFP/NEFilter are stubs, rename capability to `os.l4.wfp.observe-only.v0` / `os.l4.nefilter.observe-only.v0`, or remove from default discovery.
- Do not compile bundles that require `os.l4.wfp.v1` or `os.l4.nefilter.v1` unless the module passes platform integration tests.

#### Code guard

```rust
fn advertised_os_capabilities() -> Vec<&'static str> {
    let mut caps = vec![];
    #[cfg(target_os = "linux")]
    caps.push("os.l4.ebpf.v1");

    #[cfg(all(windows, feature = "real-wfp"))]
    caps.push("os.l4.wfp.v1");

    #[cfg(all(target_os = "macos", feature = "real-nefilter"))]
    caps.push("os.l4.nefilter.v1");

    caps
}
```

### 6.2 Make WFP/NEFilter stubs fail-safe

Current stubs return `Ok(())` when rules are “applied.” That is unsafe because `network_loop` will store LKG and believe enforcement succeeded.

#### Replace stub `apply_rules`

```rust
fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
    anyhow::bail!(
        "WFP enforcement is not implemented; refusing to claim enforcement for policy {}",
        rules.policy_id
    )
}
```

For observe-only development, add explicit config:

```rust
if std::env::var("DEK_ALLOW_OBSERVE_ONLY_OS_ENFORCER").ok().as_deref() == Some("1") {
    tracing::warn!("observe-only WFP stub enabled; no kernel enforcement is active");
    return Ok(());
}
anyhow::bail!("WFP enforcement unavailable")
```

Do the same for NEFilter.

### 6.3 eBPF fail-closed path

Current eBPF program defaults to allow on error:

```rust
pub fn dek_connect4(ctx: SockAddrContext) -> i32 {
    try_dek_connect4(&ctx).unwrap_or(1) // default ALLOW on error
}
```

This contradicts fail-closed semantics for protected workloads.

#### Required policy mode map

Add a small config map:

```rust
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DekRuntimeMode {
    pub fail_closed: u8,
    pub observe_only: u8,
}

#[map]
static RUNTIME_MODE: Array<DekRuntimeMode> = Array::with_max_entries(1, 0);
```

Then:

```rust
#[cgroup_sock_addr(connect4)]
pub fn dek_connect4(ctx: SockAddrContext) -> i32 {
    match try_dek_connect4(&ctx) {
        Ok(v) => v,
        Err(_) => runtime_default_verdict(),
    }
}

#[inline(always)]
fn runtime_default_verdict() -> i32 {
    if let Some(mode) = unsafe { RUNTIME_MODE.get(0) } {
        if mode.observe_only != 0 { return 1; }
        if mode.fail_closed != 0 { return 0; }
    }
    0 // safe default: deny
}
```

### 6.4 Complete DNS LRU cache lifecycle

The eBPF program defines `DNS_IP_CACHE_V4` as `LruHashMap`, which is correct. But userspace update is TODO and the janitor is commented.

#### Required eBPFD update flow

When DNS observation includes A records:

```rust
for rec in &obs.answers {
    if let std::net::IpAddr::V4(ipv4) = rec.ip {
        let expires_at_ns = now_monotonic_ns()? + (rec.ttl_secs as u64) * 1_000_000_000;
        dns_cache::upsert_dns_cache_v4(&mut bpf, ipv4, expires_at_ns, obs.cgroup_id)?;
    }
}
```

#### Add map pressure metrics

```rust
metrics::gauge!("dek_ebpf_dns_cache_entries").set(current_entries as f64);
metrics::counter!("dek_ebpf_dns_cache_evictions_estimated_total").increment(evicted_delta);
```

### 6.5 eBPF build should fail in release if bytecode is missing

`dek-ebpfd/build.rs` currently creates an empty `dek-ebpf-prog` placeholder and warns if aya build fails. This is acceptable for local app-layer-only development, not for release.

Add an environment gate:

```rust
fn main() {
    let release_required = std::env::var("DEK_REQUIRE_EBPF_BYTECODE").ok().as_deref() == Some("1");

    // existing placeholder creation...

    if target_os == "linux" {
        if let Err(e) = aya_build::build_ebpf(std::iter::empty::<cargo_metadata::Package>()) {
            if release_required {
                panic!("eBPF build failed and DEK_REQUIRE_EBPF_BYTECODE=1: {e}");
            } else {
                println!("cargo:warning=Failed to build eBPF programs: {e}");
            }
        }
    }
}
```

In release workflow:

```yaml
env:
  DEK_REQUIRE_EBPF_BYTECODE: "1"
```

---

## 7. Phase 4 — Policy Runtime and Plugin Host

### 7.1 Keep Wasmtime pooling, but remove state leakage risk

Current `dek-wasm-host` has a `PluginWorkerPool`, `prewarm()`, generation checks, `max_worker_uses`, reset-before-reuse, and semaphores. That is the correct direction.

Complete these checks:

- Every plugin must export `pollen_plugin_reset` or declare `reusable=false` in manifest.
- If reset fails, discard worker.
- If plugin has mutable global state and no reset, do not pool instances; pool compiled modules only.
- Record metrics:

```text
dek_wasm_pool_hit_total{plugin}
dek_wasm_pool_miss_total{plugin}
dek_wasm_worker_recycled_total{plugin,reason}
dek_wasm_invoke_latency_ms{plugin}
dek_wasm_acquire_timeout_total{plugin}
```

Wasmtime recommends pooling allocator, copy-on-write heap images, and `InstancePre` to remove import resolution/type-checking from the instantiation critical path. Use these as defaults for trusted built-in plugins and configurable for third-party plugins.

### 7.2 Add plugin ABI conformance tests

For every WASM plugin:

```text
- exports alloc/dealloc
- exports pollen_plugin_decide or configured entrypoint
- exports pollen_plugin_reset if reusable=true
- max memory within policy
- fuel/epoch timeout enforced
- malformed input returns structured error, not trap
```

Example test skeleton:

```rust
#[tokio::test]
async fn pii_plugin_exports_required_abi() -> anyhow::Result<()> {
    let host = WasmPluginHost::new(WasmHostConfig::test())?;
    let plugin = host.load_plugin("plugins/pii-redactor-plugin/target/wasm32-wasip1/release/pii_redactor_plugin.wasm").await?;
    plugin.assert_export("pollen_plugin_decide")?;
    plugin.assert_export("pollen_plugin_reset")?;
    Ok(())
}
```

---

## 8. Phase 5 — Secure Telemetry Spooling Integration

### 8.1 Current status

`dek-secure-spool` already has:

- AES-256-GCM record encryption.
- AAD with tenant/device/segment/seq/key_id/alg.
- Encrypted frame format with magic, version, length, frame, CRC32C.
- OS-specific key store module structure.

This is a good base.

### 8.2 Required release fixes

1. Remove panic path in AAD serialization.
2. Implement and test Windows DPAPI and macOS Keychain modules; Linux fallback is acceptable for beta only if documented as fallback.
3. Add replay-resistant batch IDs and delete/mark-sent semantics.
4. Add spool quota and retention:

```text
max_total_bytes = 256 MB default
max_segment_bytes = 8 MB default
max_retention_days = 7 default
on_quota_exceeded = drop_low_priority_then_oldest
```

5. Make `CloudTelemetrySink` use `dek-secure-spool` by default, not only standalone tests.
6. Add migration test from old plaintext telemetry DB/queue to encrypted spool.

### 8.3 Required tests

```bash
cargo test -p dek-secure-spool --locked
cargo test -p dek-telemetry --locked secure_spool
```

Test cases:

```text
- encrypt/decrypt round trip
- AAD tamper fails
- ciphertext tamper fails
- wrong key fails
- truncated frame returns error but preserves following valid segment if possible
- quota eviction drops low-priority first
- old plaintext spool migration encrypts then securely removes plaintext
```

---

## 9. Phase 6 — Local Control Plane and Dashboard Readiness

### 9.1 LCP must be the reference implementation of the contract

Because Pollen Cloud is in another repo, `local-control-plane` should be the **reference provider** for the contract in this repo.

Required:

```text
LCP endpoint exists in TypeSpec/OpenAPI
LCP handler returns generated contract shape
LCP tests validate response body against JSON Schema/OpenAPI examples
Mock-Cloud mirrors LCP contract behavior unless it intentionally tests chaos/failure
```

### 9.2 Dashboard must consume generated TypeScript contract

Current dashboard package does not show `@pollen/contract` / generated type usage. Add an API client layer.

#### Example `apps/local-admin-dashboard/src/api/client.ts`

```ts
import createClient from "openapi-fetch";
import type { paths } from "../../../contracts/generated/typescript/api";

export const api = createClient<paths>({
  baseUrl: import.meta.env.VITE_API_BASE_URL ?? "",
  headers: {
    "X-Pollen-Contract-Version": "1.0",
  },
});
```

#### Example use

```ts
export async function getContractDiscovery() {
  const { data, error } = await api.GET("/.well-known/pollen-contract");
  if (error) throw error;
  return data;
}
```

### 9.3 Dashboard e2e smoke

Add Playwright test:

```ts
import { test, expect } from "@playwright/test";

test("dashboard loads and shows contract status", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText(/Pollen/i)).toBeVisible();
  await expect(page.getByText(/Contract/i)).toBeVisible();
});
```

CI can run Playwright after LCP starts:

```yaml
- name: Build LCP
  run: cargo build -p local-control-plane --release --locked
- name: Start LCP
  run: |
    ./target/release/local-control-plane > /tmp/lcp.log 2>&1 &
    echo $! > /tmp/lcp.pid
    sleep 3
- name: Dashboard e2e
  working-directory: apps/local-admin-dashboard
  run: npm run e2e
```

---

## 10. Phase 7 — Mock-Cloud Must Not Become Drift Source

### 10.1 Contract-first Mock-Cloud

Mock-Cloud should be verified against the same generated OpenAPI/JSON Schema as LCP. It can add chaos/admin/dev endpoints, but all DEK-facing endpoints must conform.

Required labels:

```text
contracted endpoint: must conform to OpenAPI
legacy endpoint: allowed only with deprecation date
chaos/admin endpoint: outside DEK contract; must be under /mock/admin or /admin
```

### 10.2 Remove static seed from production-like tests

Mock-Cloud uses a static Ed25519 seed. This is acceptable for deterministic tests, but never for anything called “release environment.”

Add explicit guard:

```rust
if std::env::var("POLLEN_MOCK_ALLOW_STATIC_SEED").ok().as_deref() != Some("1") {
    tracing::warn!("Mock-Cloud static signing key enabled; use only for tests");
}
```

Better:

```rust
let signing_seed = match std::env::var("POLLEN_MOCK_SIGNING_SEED_B64") {
    Ok(seed) => decode_seed(seed)?,
    Err(_) => BUNDLE_SEED.to_vec(), // test default only
};
```

### 10.3 Mock-Cloud CI scenarios

Add acceptance scenarios:

```text
- enroll device
- publish policy
- fetch signed bundle
- verify bundle signature
- hot reload via SSE bundle_ready
- telemetry batch ingest
- key rotation
- revoked key rejects bundle
- stale bundle enters GracePeriod then StrictDeny
- cloud outage uses LKG then StrictDeny
- tampered artifact hash rejected
- rollback generation rejected
```

---

## 11. Phase 8 — Release Workflow Fixes

### 11.1 eBPF artifact path

Release workflow currently extracts:

```bash
cp crates/dek-ebpf-prog/target/bpfel-unknown-none/release/dek-ebpf-prog dek-ebpf-prog.bpf.o
```

When using Cargo from the workspace root, target output is likely:

```bash
target/bpfel-unknown-none/release/dek-ebpf-prog
```

Fix robustly:

```bash
set -euo pipefail
CANDIDATES=(
  "target/bpfel-unknown-none/release/dek-ebpf-prog"
  "crates/dek-ebpf-prog/target/bpfel-unknown-none/release/dek-ebpf-prog"
)
for c in "${CANDIDATES[@]}"; do
  if [ -f "$c" ]; then
    cp "$c" dek-ebpf-prog.bpf.o
    exit 0
  fi
done
echo "eBPF object not found"
find . -path '*bpfel-unknown-none*' -type f -maxdepth 6
exit 1
```

### 11.2 Package complete runtime artifacts

Current staging copies only:

```text
dek-core
dek-mcp-proxy
dek-ext-authz
```

But README quickstart references `local-control-plane` and `dek-cli`, and packaging metadata references `dek-updater`.

Define release profiles:

#### DEK runtime package

```text
dek-core
dek-cli or dekctl
dek-mcp-proxy
dek-ext-authz
dek-updater if exists
policy adapter artifacts if dynamic
default config templates
LICENSE / NOTICE
README.quickstart
```

#### Local developer package

```text
local-control-plane
local-admin-dashboard static dist
mock-cloud optional
sample policies
sample contracts
```

#### Linux kernel enforcement package

```text
dek-ebpfd
dek-ebpf-prog.bpf.o
systemd unit
udev/cgroup setup script
```

Do not claim a binary in README if it is not released.

### 11.3 Build Apple Silicon and Linux ARM64

Add targets:

```yaml
- os: macos-latest
  target: aarch64-apple-darwin
  artifact_name: pollen-dek-macos-arm64
- os: ubuntu-latest
  target: aarch64-unknown-linux-gnu
  artifact_name: pollen-dek-linux-arm64
```

Use `cross` or install cross toolchains for Linux ARM64.

### 11.4 Windows signing robustness

Do not hardcode Windows SDK version path. Resolve `signtool.exe`:

```powershell
$signtool = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter signtool.exe |
  Where-Object { $_.FullName -like "*\x64\signtool.exe" } |
  Sort-Object FullName -Descending |
  Select-Object -First 1
if (-not $signtool) { throw "signtool.exe not found" }
& $signtool.FullName sign /f cert.pfx /p $env:WINDOWS_PFX_PASSWORD /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 staging\*.exe
```

### 11.5 Cosign and artifact attestations

Current cosign keyless signing is good. Add GitHub artifact attestations if desired:

```yaml
permissions:
  contents: write
  id-token: write
  attestations: write

- name: Generate artifact attestation
  uses: actions/attest-build-provenance@v2
  with:
    subject-path: release_artifacts/*
```

Keep cosign because users can verify detached signatures outside GitHub.

---

## 12. Phase 9 — Security Hardening Before Public Beta

### 12.1 Remove sensitive debug logs

Current logs include debug messages printing pinned public keys. Public keys are not secret, but the debug labels should not be in normal startup logs.

Replace:

```rust
tracing::info!("DEBUG BOOTSTRAP: Loaded from {}, key is: {}", bootstrap_path, bootstrap.pinned_bundle_public_key);
```

with:

```rust
tracing::debug!(
    bootstrap_path = %bootstrap_path,
    pinned_key_fingerprint = %fingerprint(&bootstrap.pinned_bundle_public_key),
    "bootstrap loaded"
);
```

### 12.2 Auth disabled guard

Local Control Plane supports `DEK_LCP_AUTH_DISABLE=1`. This is acceptable for tests but should refuse non-loopback bind.

```rust
if cfg.auth_disabled && !cfg.bind_addr.ip().is_loopback() {
    anyhow::bail!("DEK_LCP_AUTH_DISABLE=1 is only allowed on loopback addresses");
}
```

### 12.3 Strict contract version handling

DEK should send:

```http
X-Pollen-Contract-Version: 1.0
X-Pollen-Device-Id: <device>
X-Pollen-Tenant-Id: <tenant>
```

On unsupported response:

```text
- if active LKG exists: enter GracePeriod and emit contract.version.unsupported
- if no LKG: StrictDeny
- never silently ignore contract mismatch
```

### 12.4 Signed bundle canonicalization

Local mode signs `serde_json::to_vec(&manifest)`, while Cloud/TUF code uses JCS for signed metadata in some paths. Standardize:

```text
All signed JSON payloads must use RFC 8785 / JCS canonicalization before Ed25519 signing.
```

Implementation:

```rust
let signed_bytes = serde_jcs::to_vec(&manifest)?;
let sig_b64 = signer.sign_b64(&signed_bytes);
```

And verify using the same canonicalization.

---

## 13. Phase 10 — Pollen Cloud Separate Repo Integration

### 13.1 Contract artifact publishing

Until `pollen-contracts` is split into a standalone repo, publish contract artifacts from DEK repo release:

```text
contracts/generated/openapi/pollen.v1.yaml
contracts/generated/asyncapi/pollen-sse.v1.yaml
contracts/schemas/*.schema.json
contracts/catalog/*.yaml
contracts/generated/typescript/api.d.ts
contracts/generated/rust/pollen-contract crate tarball or git tag
SHA256SUMS
cosign signatures
```

### 13.2 Cloud repo consumption rule

In Pollen Cloud repo:

```text
Do not copy schemas manually.
Do not hand-write DTOs that already exist in pollen-contract.
Do not add DEK-facing endpoint outside OpenAPI.
Do not emit SSE event outside AsyncAPI.
```

Cloud repo should use either:

```toml
pollen-contract = { git = "https://github.com/AECInfraconnect/AntiG_Pollen_DEK", tag = "v1.0.0-beta.6", package = "pollen-contract" }
```

or consume a split `pollen-contracts` repo later.

### 13.3 Cloud provider conformance CI

Cloud repo must download the exact contract artifact and run provider tests:

```yaml
name: cloud-contract-conformance

on: [pull_request]

jobs:
  conformance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Download contract artifact
        run: |
          curl -L -o pollen.v1.yaml https://github.com/AECInfraconnect/AntiG_Pollen_DEK/releases/download/v1.0.0-beta.6/pollen.v1.yaml
      - name: Start Cloud API test server
        run: cargo run -p pollen-cloud-api -- --test-mode &
      - name: Run provider conformance
        run: cargo test -p pollen-cloud-contract-tests --locked
```

### 13.4 Compatibility matrix must be updated by CI

Any contract PR must update:

```text
contracts/CHANGELOG.md
contracts/COMPATIBILITY.md
contracts/docs/CLOUD_CONSUMER_GUIDE.md
contracts/docs/MOCK_CLOUD_CONFORMANCE.md
```

The existing `docs-must-change` job should remain and be extended to block Cloud-impacting changes without Cloud guide updates.

---

## 14. Recommended GitHub Actions Layout

Final workflow set:

```text
.github/workflows/
├── ci.yml                    # Rust workspace default + OS matrix
├── ebpf-ci.yml               # Linux nightly BPF build + verifier smoke
├── contract-ci.yml           # TypeSpec/OpenAPI/AsyncAPI/schema/oasdiff/semantic/docs gate
├── dashboard-ci.yml          # Local Admin Dashboard TS/lint/test/build
├── conformance-ci.yml        # LCP + Mock-Cloud provider conformance
├── security-ci.yml           # cargo audit/deny, cargo machete, zizmor/actionlint, secret scan
├── release-dry-run.yml       # package but do not publish
└── release.yml               # signed release publish
```

### 14.1 Security CI example

```yaml
name: security-ci

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read
  security-events: write

jobs:
  rust_security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: cargo-bins/cargo-binstall@main
      - run: cargo binstall -y cargo-audit cargo-deny cargo-machete
      - run: cargo audit
      - run: cargo deny check advisories licenses bans sources
      - run: cargo machete

  workflow_lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pipx install zizmor
      - run: zizmor .github/workflows
      - uses: raven-actions/actionlint@v2
```

---

## 15. Release Acceptance Criteria

### 15.1 `v1.0.0-beta.6` minimum

```text
[ ] cargo fmt passes
[ ] cargo check workspace passes on ubuntu/windows/macos
[ ] cargo clippy workspace passes on ubuntu/windows/macos
[ ] cargo test workspace passes on ubuntu/windows/macos
[ ] eBPF program builds on ubuntu nightly
[ ] dashboard typecheck/lint/test/build passes
[ ] contracts build/lint/schema/semantic/generated-drift passes
[ ] LCP and Mock-Cloud expose /.well-known/pollen-contract with contract shape
[ ] LCP supports /bundles/latest and legacy /bundles/manifest
[ ] bundle envelope includes schema_version
[ ] telemetry batch returns contract response shape
[ ] WFP/NEFilter stubs do not advertise real enforcement capability
[ ] eBPF fail-open path is controlled by explicit observe-only/fail-closed runtime mode
[ ] release-dry-run packages artifacts on all target OSes
```

### 15.2 `v1.0.0-beta.7` minimum

```text
[ ] secure telemetry spool integrated into CloudTelemetrySink
[ ] encrypted spool migration from plaintext path tested
[ ] eBPF DNS cache userspace update + janitor implemented
[ ] WFP user-mode static filters implemented on Windows or capability remains disabled
[ ] NEFilter client has real IPC/System Extension integration or capability remains disabled
[ ] Wasmtime plugin ABI conformance tests pass
[ ] Contract conformance tests validate LCP and Mock-Cloud responses
[ ] Release workflow publishes SBOM, checksums, cosign signatures, and artifact attestations
```

### 15.3 `v1.0.0` minimum

```text
[ ] Pollen Cloud separate repo consumes the same contract artifacts
[ ] Cloud provider conformance passes against current contract
[ ] DEK N-1 can connect to Cloud latest under compatibility matrix
[ ] LKG/GracePeriod/StrictDeny scenarios covered in acceptance tests
[ ] OS enforcement capability registry is truthful per OS
[ ] Installer/service/startup flows work on Linux/Windows/macOS
[ ] User docs are correct for downloaded artifacts
[ ] Upgrade/rollback path verified
```

---

## 16. AI Agent Task Breakdown

Give these tasks to an AI coding agent in order.

### Task A — Build/CI rescue

```text
Fix root Cargo workspace membership and CI commands so default Rust CI passes on Linux, Windows, and macOS. Do not change product behavior. Make dek-ebpfd a workspace member or remove invalid --exclude references. Ensure cargo fmt/check/clippy/test/build pass. Remove production panic paths caught by clippy. Produce a short summary of exact commands that pass locally/CI.
```

### Task B — Contract conformance rescue

```text
Implement /.well-known/pollen-contract in Local Control Plane and make Mock-Cloud response match TypeSpec. Fix bundle endpoints so /bundles/latest returns BundleFetchResponse and /bundles/manifest remains a legacy alias. Add schema_version to bundle envelope. Fix telemetry batch response shape. Add provider conformance tests for LCP and Mock-Cloud.
```

### Task C — OS capability honesty

```text
Change capability advertisement so os.l4.wfp.v1 and os.l4.nefilter.v1 are not advertised unless real enforcement is enabled. Make WFP/NEFilter stubs return error by default instead of Ok. Add observe-only environment override for dev only. Add tests that stubs do not create false LKG.
```

### Task D — eBPF release hardening

```text
Make eBPF connect4 default deny under fail-closed mode. Add runtime mode map. Implement DNS LRU userspace update and janitor metrics. Make release build fail if eBPF bytecode is missing when DEK_REQUIRE_EBPF_BYTECODE=1. Fix release workflow eBPF artifact path.
```

### Task E — Dashboard CI and typed API

```text
Wire Local Admin Dashboard to generated TypeScript OpenAPI types. Add dashboard-ci.yml for typecheck/lint/test/build. Add a simple Playwright smoke test that runs against Local Control Plane.
```

### Task F — Release workflow finalization

```text
Create release-dry-run.yml. Fix release.yml packaging to include only existing binaries and all required runtime artifacts. Add macOS arm64 target, optional Linux arm64 target, robust Windows signtool discovery, and artifact attestations. Ensure README download instructions match actual release assets.
```

---

## 17. References

- GitHub repo: https://github.com/AECInfraconnect/AntiG_Pollen_DEK
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- GitHub Actions matrix jobs: https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/run-job-variations
- GitHub Actions secure use: https://docs.github.com/en/actions/reference/security/secure-use
- Wasmtime fast instantiation: https://docs.wasmtime.dev/examples-fast-instantiation.html
- Windows WFP ALE: https://learn.microsoft.com/en-us/windows/win32/fwp/application-layer-enforcement--ale-
- Linux eBPF LRU hash maps: https://docs.kernel.org/bpf/map_hash.html

---

## 18. Final Recommendation

Proceed with a **release-rescue branch** before adding new features. The repo has the right architecture and many correct components, but public release should wait until CI is truthful and contract drift is eliminated.

Recommended branch name:

```text
release/v1.0.0-beta.6-readiness
```

Recommended PR title:

```text
Release readiness: contract conformance, CI rescue, and OS capability hardening
```

Recommended merge condition:

```text
All workflows green:
- ci
- ebpf-ci
- contract-ci
- dashboard-ci
- conformance-ci
- security-ci
- release-dry-run
```
