# Pollen DEK Repo Deep Research & Full-System Implementation Plan

**Repository:** `AECInfraconnect/AntiG_Pollen_DEK`  
**Analysis date:** 2026-06-18  
**Current observed workspace version:** `1.0.0-beta.6`  
**Purpose:** เอกสารนี้เป็นแผนปรับปรุงและพัฒนาระบบ Pollen DEK ให้สมบูรณ์พร้อมใช้งานจริงทุกฟังก์ชัน โดยอ้างอิง repo ล่าสุดที่ตรวจผ่าน GitHub connector, แนวทางออกแบบเดิมของ Pollen DEK, และ best practices จาก ecosystem ที่เกี่ยวข้อง เช่น Rust, Wasmtime, OPA, Cedar, OpenFGA, eBPF, Windows WFP, macOS NetworkExtension, OpenAPI/AsyncAPI และ supply-chain security

> เอกสารนี้ออกแบบให้ AI Agent / ทีม Dev ใช้เป็น implementation guide ได้ทันที  
> หลักการทำงาน: **แก้ CI ให้เชื่อถือได้ → ทำ contract ให้ไม่ drift → ทำ runtime path ให้ enforce จริง → ทำ release path ให้ verify ได้ → ทำ docs/tests ให้คงสภาพระหว่างพัฒนา**

---

## 0. Executive Summary

Pollen DEK repo ล่าสุดมีความก้าวหน้ามากเมื่อเทียบกับ prototype ก่อนหน้า:

- Workspace ขยับเป็น `1.0.0-beta.6`
- มี Contract Hub staging ใน `contracts/`
- มี OpenAPI/TypeSpec/AsyncAPI/JSON Schema pipeline
- มี Local Control Plane + Dashboard + Mock Cloud
- มี signed bundle envelope และ signed release workflow
- มี `dek-secure-spool`
- มี Wasmtime plugin host พร้อม worker pool
- มี Linux eBPF runtime mode + DNS LRU map
- มี CI matrix และ release supply-chain workflow
- มี Local discovery endpoint `/.well-known/pollen-contract`

แต่ยังไม่ควรถือว่า production-ready เพราะยังมี release blockers และ correctness gaps ที่กระทบการใช้งานจริง:

1. **Contract drift ยังมีอยู่**
   - TypeSpec กำหนด response format หนึ่ง แต่ LCP/Mock-Cloud บาง endpoint ตอบอีกแบบ
   - `bundle-envelope.v1.schema.json` กำหนด `schema_version = "bundle-envelope.v1"` แต่ LCP สร้าง envelope เป็น `"bundle.signed-envelope.v1"`
   - telemetry ingest response ใน TypeSpec ไม่ตรงกับ LCP/Mock-Cloud implementation

2. **OS L4 enforcement ยังไม่ครบ**
   - Linux eBPF ใกล้สุด แต่ยังมี code ที่น่าจะ compile fail ใน `dns_cache.rs` และ runtime mode/fail-closed ต้อง test จริง
   - Windows WFP และ macOS NEFilter ยังเป็น stub ที่ `start()` bail
   - capability discovery ไม่ควร advertise OS L4 capability จนกว่า module enforce ได้จริง

3. **CI ยังไม่พอสำหรับ release จริง**
   - CI หลักมี Rust matrix, eBPF build, WASM build แต่ยังไม่เห็น Dashboard CI/E2E ใน workflow หลัก
   - Contract CI มี `lint:semantic` ใน `package.json` แต่ workflow ไม่เรียก
   - ยังไม่มี provider conformance test ที่ validate LCP/Mock-Cloud response against OpenAPI/JSON Schema ทุก endpoint
   - release workflow มี supply-chain ดีขึ้น แต่ eBPF artifact path, cross-target packaging, และ OS-specific signing/notarization ยังต้อง harden

4. **Telemetry secure spool ยังเป็น crate แยก**
   - มี AES-GCM record-level encryption และ segment frame แล้ว
   - แต่ต้องตรวจว่า `dek-telemetry::CloudTelemetrySink` ใช้ secure spool จริงทุก path หรือยัง
   - ต้องมี replay, retention, quota, key rotation, redaction และ migration test

5. **Dashboard ยังไม่ consume generated contract**
   - `apps/local-admin-dashboard/package.json` ยังไม่มี `openapi-fetch` หรือ dependency ที่ bind กับ `contracts/generated/typescript/api.d.ts`
   - เสี่ยง drift ระหว่าง UI กับ LCP/Cloud API

---

## 1. Current Repository Snapshot

### 1.1 Workspace

Current root workspace observed:

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
rust-version = "1.85"
```

Implication:

- `dek-ebpfd` is now inside workspace, while `dek-ebpf-prog` remains excluded because it targets `bpfel-unknown-none`
- CI should no longer exclude `dek-ebpfd` from default workspace unless dependency resolution breaks on non-Linux
- ทุก crate ควร inherit workspace version หรืออย่างน้อย version alignment ต้องถูกตรวจใน CI

### 1.2 Architecture State

จาก architecture ของ repo ระบบวางตัวเป็น Rust PEP + local PDP มี dual-mode ระหว่าง Local Control Plane และ Pollen Cloud โดยใช้ shared contract เดียวกัน:

```text
Local Admin Dashboard / Local Control Plane
        ↕ same contract
Pollen DEK Runtime
        ↕ same contract
Pollen Cloud / Mock Cloud
```

Key components:

- `dek-core`: supervisor, sidecar API, hot reload, SVID/mTLS, OS enforcement loop
- `dek-policy-syncer`: bundle sync lifecycle + enforcement state
- `dek-bundle-sync`: TUF-lite / signed envelope / artifact fetch
- `dek-policy-router`: route matching, engine selection, circuit breaker, dry-run
- PDP adapters: Cedar, OPA/WASM, OpenFGA
- `dek-wasm-host`: Wasmtime plugin host and worker pool
- `dek-secure-spool`: encrypted telemetry spool
- `dek-ebpfd` + `dek-ebpf-prog`: Linux L4 enforcement / DNS observe
- `dek-windows-wfp`: currently stub, planned Windows WFP enforcement
- `dek-macos-nefilter`: currently stub, planned NetworkExtension enforcement
- `local-control-plane`: Axum + SQLite + signing + dashboard API
- `mock-cloud`: reference Cloud implementation for offline test
- `contracts`: Contract Hub staging

---

## 2. Deep Research Summary

### 2.1 Contract-first API design

OpenAPI 3.1 is a language-agnostic standard interface description for HTTP APIs and enables humans/tools to discover API capabilities without source code. It also aligns its schema model with JSON Schema 2020-12. This fits Pollen because DEK, LCP, Dashboard, Mock-Cloud and Cloud must share one contract and must not drift.

Recommended policy:

```text
contracts/spec/*.tsp      = authoring source
contracts/generated/*     = generated artifacts
contracts/schemas/*.json  = canonical runtime validation models
contracts/catalog/*.yaml  = canonical enums/capabilities/error codes
```

### 2.2 Async/event contract

AsyncAPI is appropriate for SSE/event-driven push flows such as:

- `BundleReady`
- `PolicyActivationChanged`
- `DeviceSuspended`
- `ContractDeprecated`
- `TelemetryBackpressure`

Use AsyncAPI for event shape and JSON Schema `$ref` for payload to prevent REST/SSE schema duplication.

### 2.3 OPA-style bundle lifecycle

OPA's bundle model supports hot policy updates, persistence, status reporting, ETag/If-None-Match, and signed bundles. Pollen should borrow these concepts even though its bundle format is custom:

- signed bundle envelope
- persistent LKG bundle
- status/audit update on activation
- no mutation of bundle-loaded policy at runtime
- bundle fetch supports 304 not modified
- every activation produces status + decision-log traceability

### 2.4 Cedar validation

Cedar best practice is to validate policies against a schema before using them for runtime authorization. For Pollen this means:

- PPI → Cedar compile must validate schema
- Local Dashboard structured form must generate schema-valid Cedar
- Cloud NL Policy Editor must output PPI first, not raw Cedar directly
- every Cedar artifact in bundle must include `schema_hash` and `validator_version`

### 2.5 OpenFGA model lifecycle

OpenFGA works with relationship tuples and authorization models. For Pollen:

- OpenFGA is suitable for ReBAC policies, e.g. `user owns resource`, `agent member_of workspace`, `tool allowed_by relation`
- tuple data must not be pushed blindly to DEK kernel layer
- use OpenFGA adapter for user-mode PDP only
- bundle artifact should include `model.fga` and optional tuple snapshot/delta only where safe

### 2.6 Wasmtime performance

Wasmtime official performance guidance says startup has compile + instantiate cost. Recommended improvements:

- precompile or cache modules
- use `InstancePre`
- enable pooling allocator
- enable copy-on-write memory init
- bound concurrency with semaphore
- rotate worker instances to avoid mutable state leakage

Repo already has `dek-wasm-host/src/pool.rs`, so the next phase is benchmark + hardening, not redesign.

### 2.7 Linux eBPF LRU maps

Linux kernel docs say `BPF_MAP_TYPE_LRU_HASH` evicts least recently used entries when capacity is reached. This is correct for DNS/IP cache and connection decision cache, but not for authoritative policy maps.

Recommended split:

```text
DNS/IP cache:          LruHashMap + TTL
Connection cache:      LruHashMap + TTL
Policy LPM map:        LpmTrie, no LRU
Cgroup policy map:     HashMap, no LRU
Metrics:               PerCpuArray
Events:                RingBuf
```

### 2.8 Windows WFP ALE

WFP ALE layers are appropriate for stateful L4 enforcement because they authorize inbound/outbound connection creation and allow filtering by application identity and user identity. For Pollen:

- v1 should use user-mode WFP filters at ALE_AUTH_CONNECT_V4/V6 and ALE_AUTH_RECV_ACCEPT_V4/V6
- kernel callout driver should be advanced mode only
- user-mode static allow/deny is enough for beta if policies are simple exact/CIDR/port/process rules

### 2.9 macOS NetworkExtension

Apple NetworkExtension Content Filter is the supported direction for macOS network filtering. For Pollen:

- implement System Extension + `NEFilterDataProvider`
- metadata-first flow: process/app identity + hostname/IP/port + flow direction
- no TLS MITM for beta
- Rust daemon communicates to Swift extension over XPC or local socket

### 2.10 Supply-chain security

The release workflow already uses cosign keyless signing and GitHub OIDC. This should be extended with:

- GitHub artifact attestations
- SBOM per target
- checksum + signature verification test before release
- pinned action versions
- least-privilege `permissions` per job
- release dry-run on pull_request/manual workflow

---

## 3. Critical Findings from Latest Repo

### Finding F1 — Contract Hub exists but is not fully enforced

`contracts/package.json` has:

```json
"lint:semantic": "tsx scripts/semantic-contract-lint.ts",
"test": "npm run build && npm run lint:openapi && npm run lint:asyncapi && npm run validate:schemas && npm run lint:semantic"
```

But `.github/workflows/contract-ci.yml` calls only:

```yaml
npm run build
npm run lint:openapi
npm run lint:asyncapi
npm run validate:schemas
npm run check:generated
```

**Gap:** semantic lint is defined but not enforced in CI.

**Required fix:** Replace those steps with:

```yaml
- run: npm ci
- run: npm test
- run: npm run check:generated
```

or explicitly add:

```yaml
- run: npm run lint:semantic
```

---

### Finding F2 — Bundle envelope schema_version mismatch

Schema says:

```json
"schema_version": {
  "type": "string",
  "const": "bundle-envelope.v1"
}
```

LCP creates:

```rust
"schema_version": "bundle.signed-envelope.v1"
```

**Impact:** A JSON Schema validator should reject the LCP-generated bundle envelope. This is a high-priority contract bug.

**Recommended decision:** Use one canonical value:

```text
bundle-envelope.v1
```

because that is already schema title/path style.

**Patch:**

```rust
// crates/local-control-plane/src/bundle.rs
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

Add test:

```rust
#[test]
fn local_signed_bundle_matches_bundle_envelope_schema() {
    let schema: serde_json::Value = serde_json::from_str(
        include_str!("../../../contracts/schemas/bundle-envelope.v1.schema.json")
    ).unwrap();

    let validator = jsonschema::validator_for(&schema).unwrap();
    let envelope = build_test_signed_bundle_envelope();
    assert!(validator.validate(&envelope).is_ok());
}
```

---

### Finding F3 — Bundle latest response does not match TypeSpec

TypeSpec says `/bundles/latest` returns:

```typespec
model BundleFetchResponse {
  schema_version: "bundle-fetch-response.v1";
  status: "not_modified" | "bundle_ready";
  generation?: int32;
  envelope?: unknown;
}
```

LCP currently returns:

```rust
Ok(Json(serde_json::json!({"envelope": val})))
```

**Impact:** Consumer generated from OpenAPI will expect `schema_version` and `status`; LCP/Cloud implementation will not conform.

**Patch:**

```rust
async fn get_latest_bundle(
    Path((tenant, _device)): Path<(String, String)>,
    State(st): State<AppState>,
    body: Option<Json<serde_json::Value>>,
) -> ApiResult<Json<serde_json::Value>> {
    let current_generation = body
        .as_ref()
        .and_then(|b| b.get("current_generation"))
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    match st.policy_store.get_policy_raw(&tenant, "bundle:latest").await {
        Ok(Some(val)) => {
            let generation = st.build_number.load(std::sync::atomic::Ordering::SeqCst) as i64;
            if current_generation == generation {
                return Ok(Json(serde_json::json!({
                    "schema_version": "bundle-fetch-response.v1",
                    "status": "not_modified",
                    "generation": generation
                })));
            }

            Ok(Json(serde_json::json!({
                "schema_version": "bundle-fetch-response.v1",
                "status": "bundle_ready",
                "generation": generation,
                "envelope": val
            })))
        }
        Ok(None) => Err(ApiError::NotFound("bundle".into())),
        Err(e) => Err(ApiError::Internal(e)),
    }
}
```

Keep `/bundles/manifest` as legacy endpoint only until DEK uses `/bundles/latest`.

---

### Finding F4 — Telemetry response does not match TypeSpec

TypeSpec expects:

```typespec
model TelemetryIngestResponse {
  schema_version: "telemetry-ingest-response.v1";
  accepted: int32;
  rejected: int32;
  retry_after_seconds?: int32;
}
```

LCP returns:

```json
{ "status": "ok", "processed": stored }
```

Mock-Cloud returns:

```json
{ "status": "ingested", "kind": "batches", "count": n }
```

**Patch for LCP:**

```rust
fn telemetry_response(accepted: usize, rejected: usize) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "schema_version": "telemetry-ingest-response.v1",
        "accepted": accepted as i32,
        "rejected": rejected as i32
    }))
}
```

Apply to:

- `crates/local-control-plane/src/telemetry.rs`
- `crates/mock-cloud/src/telemetry.rs`

Acceptance:

```bash
cd contracts && npm test
cargo test -p local-control-plane telemetry_contract
cargo test -p mock-cloud telemetry_contract
```

---

### Finding F5 — eBPF DNS cache code likely has compile issue

`crates/dek-ebpfd/src/dns_cache.rs` imports:

```rust
use aya::maps::{LruHashMap, MapData};
```

but later uses:

```rust
pub fn estimate_map_entries_v4(bpf: &mut Ebpf, sample_limit: usize) -> Result<usize> {
    let map: HashMap<_, DekIp4Key, DekDnsCacheValue> = HashMap::try_from(...)
}
```

`Ebpf` and `HashMap` are not imported in the shown file.

**Patch:**

```rust
#[cfg(target_os = "linux")]
use aya::{
    maps::{HashMap, LruHashMap, MapData},
    Ebpf,
};
```

Alternative: change the function to use `LruHashMap` consistently:

```rust
#[cfg(target_os = "linux")]
pub fn estimate_map_entries_v4(sample_limit: usize) -> Result<usize> {
    let pin_path = format!("{}/DNS_IP_CACHE_V4", crate::linux::BPFFS_PATH);
    let map_data = MapData::from_pin(&pin_path).context("load pinned DNS_IP_CACHE_V4")?;
    let map: LruHashMap<_, DekIp4Key, DekDnsCacheValue> = LruHashMap::try_from(map_data)?;

    let mut count = 0usize;
    for entry in map.iter() {
        let _ = entry?;
        count += 1;
        if count >= sample_limit {
            break;
        }
    }
    Ok(count)
}
```

---

### Finding F6 — `BPFFS_PATH` visibility risk

`dns_cache.rs` references:

```rust
crate::linux::BPFFS_PATH
```

But `BPFFS_PATH` in `lib.rs` must be visible to sibling module. Ensure it is:

```rust
pub(crate) const BPFFS_PATH: &str = "/sys/fs/bpf/pollen-dek";
```

Acceptance:

```bash
cargo check --manifest-path crates/dek-ebpfd/Cargo.toml
```

---

### Finding F7 — eBPF object build path is fragile

`build.rs` creates dummy file then uses `aya_build::build_ebpf`, then tries to copy from:

```rust
../../target/bpfel-unknown-none/release/dek-ebpf-prog
```

This can be fragile because target dir may differ under workspace, CI, or cross-target build.

**Recommended fix:** Add env override and hard fail in production profile.

```rust
let artifact = std::env::var_os("DEK_EBPF_OBJECT")
    .map(PathBuf::from)
    .unwrap_or_else(|| {
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("../../target/bpfel-unknown-none/release/dek-ebpf-prog")
    });

if artifact.exists() {
    std::fs::copy(&artifact, &dest)?;
} else if std::env::var("PROFILE").as_deref() == Ok("release") {
    panic!("missing eBPF artifact; set DEK_EBPF_OBJECT or build dek-ebpf-prog first");
}
```

CI release should build eBPF object first and pass path:

```yaml
env:
  DEK_EBPF_OBJECT: ${{ github.workspace }}/target/bpfel-unknown-none/release/dek-ebpf-prog
```

---

### Finding F8 — Windows WFP and macOS NEFilter are explicit stubs

Current WFP:

```rust
fn start(&mut self) -> Result<()> {
    anyhow::bail!("Windows Filtering Platform (WFP) integration is not compiled. The current build is a stub.");
}
```

Current macOS:

```rust
fn start(&mut self) -> Result<()> {
    anyhow::bail!("macOS Network Extension integration is not compiled. The current build is a stub.")
}
```

**Correct behavior for beta:**

- Do not advertise `os.l4.wfp.v1` or `os.l4.nefilter.v1` capability at runtime unless real backend starts.
- If a policy requires WFP/NEFilter and backend is stub, DEK must strict-deny or block local network plane depending on user mode.
- Tests must not hide stub failure by ignoring `start()` errors. The latest test changed `manager.start().unwrap()` to `let _ = manager.start()`, which makes CI green but masks the incomplete module.

**Patch strategy:**

1. Split capability into:
   - `os.l4.wfp.stub`
   - `os.l4.wfp.v1`
   - `os.l4.nefilter.stub`
   - `os.l4.nefilter.v1`

2. Runtime discovery:

```rust
pub fn runtime_os_capabilities() -> Vec<&'static str> {
    let mut caps = vec![];

    #[cfg(target_os = "linux")]
    if dek_ebpfd::probe_available() {
        caps.push("os.l4.ebpf.v1");
    }

    #[cfg(windows)]
    if dek_windows_wfp::probe_available() {
        caps.push("os.l4.wfp.v1");
    } else {
        caps.push("os.l4.wfp.stub");
    }

    #[cfg(target_os = "macos")]
    if dek_macos_nefilter::probe_available() {
        caps.push("os.l4.nefilter.v1");
    } else {
        caps.push("os.l4.nefilter.stub");
    }

    caps
}
```

3. Policy activation rejects required unsupported OS modules:

```rust
if bundle.compatibility.required_os_modules.windows.contains("os.l4.wfp.v1")
   && !caps.contains(&"os.l4.wfp.v1")
{
    return Err(ActivationError::MissingCapability("os.l4.wfp.v1".into()));
}
```

---

### Finding F9 — Dashboard is not contract-generated

`apps/local-admin-dashboard/package.json` has React/Vite/TS scripts but no contract client dependency such as `openapi-fetch`, and no script to sync `contracts/generated/typescript/api.d.ts`.

**Patch:**

```json
{
  "dependencies": {
    "openapi-fetch": "^0.15.0"
  },
  "scripts": {
    "contract:check": "test -f ../../contracts/generated/typescript/api.d.ts",
    "build": "npm run contract:check && tsc && vite build"
  }
}
```

`src/lib/api.ts`:

```ts
import createClient from "openapi-fetch";
import type { paths } from "../../../contracts/generated/typescript/api";

const baseUrl = import.meta.env.VITE_LCP_URL ?? "";

export const api = createClient<paths>({
  baseUrl,
  headers: () => {
    const token = localStorage.getItem("pollen_lcp_token");
    return token ? { Authorization: `Bearer ${token}` } : {};
  },
});
```

Example usage:

```ts
export async function fetchContractDiscovery() {
  const { data, error } = await api.GET("/.well-known/pollen-contract", {
    headers: { "X-Pollen-Contract-Version": "1.0" },
  });
  if (error) throw error;
  return data;
}
```

---

### Finding F10 — CI misses Dashboard and full provider conformance

Current CI has:

- Rust build/test matrix
- eBPF build
- WASM plugin build

Missing:

- Dashboard `npm ci`, lint, typecheck, test, build
- Mock-Cloud OpenAPI conformance
- LCP OpenAPI conformance
- Contract semantic lint
- release preflight as reusable workflow
- cargo deny/audit
- cargo nextest optional

---

## 4. Target Architecture for Production-Ready DEK

### 4.1 Production readiness principle

```text
A build is releasable only if:
1. Contract generated artifacts are in sync
2. Local Control Plane conforms to OpenAPI/JSON Schema
3. Mock-Cloud conforms to OpenAPI/JSON Schema
4. DEK can enroll, fetch signed bundle, activate, enforce, emit telemetry
5. LKG + GracePeriod + StrictDeny are tested
6. OS L4 capability is truthful: no stub advertised as production capability
7. Secure spool protects telemetry at rest
8. Release artifacts have SBOM, checksum, signature, attestation
9. Dashboard builds against generated contract
10. Documentation and compatibility matrix are updated with the same PR
```

### 4.2 Runtime mode matrix

| Mode | Control plane | Auth | Bundle source | Telemetry sink | OS L4 |
|---|---|---|---|---|---|
| Local Dev | Local Control Plane | Local bearer | LCP signed envelope | LCP SQLite | Linux eBPF optional, WFP/NEFilter stub |
| Mock Cloud | Mock Cloud | mTLS/dev cert | Mock TUF/signed envelope | Mock memory/store | Linux eBPF optional |
| Cloud Beta | Pollen Cloud | mTLS + OAuth/SVID | Cloud signed bundle | Cloud ingest | Linux eBPF GA, WFP/NEFilter preview |
| Enterprise | Pollen Cloud/private | SPIFFE/SPIRE | signed + policy approval | encrypted spool + cloud | all supported OS modules |

---

## 5. Implementation Roadmap

## Phase P0 — Stabilize CI and Contract Correctness

### P0-T1: Fix bundle envelope schema_version

**Files**

- `contracts/schemas/bundle-envelope.v1.schema.json`
- `crates/local-control-plane/src/bundle.rs`
- `crates/mock-cloud/src/bundles.rs` if applicable
- tests

**Decision**

Use canonical value:

```text
bundle-envelope.v1
```

**Acceptance**

```bash
cargo test -p local-control-plane bundle_envelope_schema
cargo test -p mock-cloud bundle_envelope_schema
cd contracts && npm test
```

---

### P0-T2: Fix `/bundles/latest` response wrapper

**Files**

- `contracts/spec/rest/bundles.tsp`
- `crates/local-control-plane/src/bundle.rs`
- `crates/mock-cloud/src/bundles.rs`
- `crates/dek-bundle-sync/src/lib.rs`

**Server response must be:**

```json
{
  "schema_version": "bundle-fetch-response.v1",
  "status": "bundle_ready",
  "generation": 123,
  "envelope": {
    "schema_version": "bundle-envelope.v1",
    "manifest": {},
    "signatures": []
  }
}
```

**Client compatibility**

Update DEK to accept both:

- new `/bundles/latest` response wrapper
- legacy `/bundles/manifest` during migration

```rust
#[derive(serde::Deserialize)]
struct BundleFetchResponse {
    schema_version: String,
    status: String,
    generation: Option<i64>,
    envelope: Option<serde_json::Value>,
}
```

---

### P0-T3: Fix telemetry ingest response

**Files**

- `contracts/spec/rest/telemetry.tsp`
- `crates/local-control-plane/src/telemetry.rs`
- `crates/mock-cloud/src/telemetry.rs`
- `crates/dek-telemetry`

**Response**

```json
{
  "schema_version": "telemetry-ingest-response.v1",
  "accepted": 10,
  "rejected": 0,
  "retry_after_seconds": null
}
```

**Acceptance**

```bash
cargo test -p local-control-plane telemetry_contract
cargo test -p mock-cloud telemetry_contract
```

---

### P0-T4: Enforce semantic contract lint in CI

**Patch**

```yaml
# .github/workflows/contract-ci.yml
- run: npm ci
- run: npm test
- run: npm run check:generated
```

or:

```yaml
- run: npm run lint:semantic
```

**Acceptance**

A PR that removes `deny` from `decision-enums.yaml` must fail CI.

---

### P0-T5: Add Dashboard CI

**New file:** `.github/workflows/dashboard-ci.yml`

```yaml
name: Dashboard CI

on:
  push:
    branches: [main]
    paths:
      - "apps/local-admin-dashboard/**"
      - "contracts/generated/typescript/**"
      - ".github/workflows/dashboard-ci.yml"
  pull_request:
    branches: [main]
    paths:
      - "apps/local-admin-dashboard/**"
      - "contracts/generated/typescript/**"
      - ".github/workflows/dashboard-ci.yml"

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
          node-version: "22"
          cache: npm
          cache-dependency-path: apps/local-admin-dashboard/package-lock.json
      - run: npm ci
      - run: npm run typecheck
      - run: npm run lint
      - run: npm test
      - run: npm run build
```

If `package-lock.json` is absent, commit it.

---

### P0-T6: Fix eBPF userspace compile blockers

**Files**

- `crates/dek-ebpfd/src/dns_cache.rs`
- `crates/dek-ebpfd/src/lib.rs`

Patch imports:

```rust
#[cfg(target_os = "linux")]
use aya::{
    maps::{HashMap, LruHashMap, MapData},
    Ebpf,
};
```

or refactor estimate function to only use pinned `LruHashMap`.

Make BPFFS path visible:

```rust
pub(crate) const BPFFS_PATH: &str = "/sys/fs/bpf/pollen-dek";
```

Acceptance:

```bash
cargo check --manifest-path crates/dek-ebpfd/Cargo.toml
cargo +nightly build --manifest-path crates/dek-ebpf-prog/Cargo.toml --target bpfel-unknown-none -Z build-std=core
```

---

### P0-T7: Stop hiding WFP stub failures in tests

Current test ignores `manager.start()` error. That is OK for compile-only CI but unsafe as security test.

Split tests:

```rust
#[test]
fn wfp_stub_reports_unavailable() {
    let mut manager = WfpFilterManager::new();
    assert!(manager.start().is_err());
}
```

Then add Windows-only integration test behind feature:

```rust
#[cfg(all(windows, feature = "wfp-native"))]
#[test]
fn wfp_native_starts_engine() {
    let mut manager = WfpFilterManager::new();
    manager.start().expect("WFP engine should open");
}
```

---

## Phase P1 — Contract Conformance and No Drift

### P1-T1: Add provider conformance tests

Create shared test helper crate:

```text
crates/contract-conformance/
```

Responsibilities:

- load `contracts/generated/openapi/pollen.v1.yaml`
- load relevant JSON Schemas
- start LCP in-memory Axum router
- start Mock-Cloud in-memory Axum router
- call every important route
- validate response schema and status codes

Example:

```rust
#[tokio::test]
async fn lcp_bundles_latest_conforms() {
    let app = test_lcp_router().await;
    let res = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/v1/tenants/local/devices/device-001/bundles/latest")
            .header("content-type", "application/json")
            .body(Body::from(r#"{
                "tenant_id":"local",
                "device_id":"device-001",
                "capabilities":["bundle.signed-envelope.v1"]
            }"#))
            .unwrap()
    ).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    validate_schema("bundle-fetch-response.v1", &body).unwrap();
}
```

Acceptance:

```bash
cargo test -p contract-conformance
```

---

### P1-T2: Make Dashboard contract-generated

Add dependency:

```bash
cd apps/local-admin-dashboard
npm install openapi-fetch
```

Add generated API usage and remove hand-written path strings gradually.

Acceptance:

```bash
grep -R '"/v1/' apps/local-admin-dashboard/src
```

Allowed raw paths only in one central file:

```text
apps/local-admin-dashboard/src/lib/api.ts
```

---

### P1-T3: Make Contract Hub consumable by Pollen Cloud repo

Add GitHub Release artifact on every tag:

```yaml
- name: Package Contract Artifacts
  run: |
    tar -czf pollen-contracts-${{ github.ref_name }}.tar.gz \
      contracts/spec contracts/schemas contracts/catalog \
      contracts/generated/openapi contracts/generated/asyncapi \
      contracts/generated/typescript contracts/generated/rust \
      contracts/COMPATIBILITY.md contracts/CHANGELOG.md
```

Add checksum + cosign signing for contract tarball.

Add `contracts/docs/CLOUD_CONSUMER_GUIDE.md`:

```md
# Pollen Cloud Contract Consumer Guide

Cloud repo MUST NOT copy schemas manually.
It MUST consume one of:
1. Git submodule at a tag
2. GitHub Release artifact
3. Rust dependency pinned to tag
4. npm package pinned to version

Provider CI MUST run:
- OpenAPI conformance
- JSON Schema validation
- Pact/consumer contract test
- N-1 DEK compatibility test
```

---

### P1-T4: Add compatibility matrix enforcement

Add script:

```ts
// contracts/scripts/check-compatibility.ts
import fs from "node:fs";

const compat = fs.readFileSync("COMPATIBILITY.md", "utf8");
if (!compat.includes("1.0")) {
  console.error("COMPATIBILITY.md must mention current contract 1.0");
  process.exit(1);
}

const changelog = fs.readFileSync("CHANGELOG.md", "utf8");
if (!changelog.includes("1.0.0-beta.6")) {
  console.error("CHANGELOG.md must mention current workspace version");
  process.exit(1);
}
```

CI:

```yaml
- run: npm run check:compatibility
```

---

## Phase P2 — Runtime Enforcement Completeness

### P2-T1: Capability truthfulness

Create runtime capability module:

```rust
pub struct RuntimeCapabilities {
    pub contract: Vec<String>,
    pub pdp: Vec<String>,
    pub pep: Vec<String>,
    pub os: Vec<String>,
}

pub fn collect_runtime_capabilities() -> RuntimeCapabilities {
    RuntimeCapabilities {
        contract: vec![
            "contract.discovery.v1".into(),
            "bundle.signed-envelope.v1".into(),
            "telemetry.batch.v1".into(),
        ],
        pdp: compiled_pdp_capabilities(),
        pep: compiled_pep_capabilities(),
        os: os_capabilities(),
    }
}
```

Local discovery should call this instead of hard-coded capabilities.

---

### P2-T2: Activation must reject unsupported bundle requirements

If bundle manifest says:

```json
"required_os_modules": {
  "windows": ["os.l4.wfp.v1"]
}
```

and runtime only has `os.l4.wfp.stub`, activation must fail.

```rust
pub fn ensure_required_caps(bundle: &PollenPolicyBundle, caps: &RuntimeCapabilities) -> Result<()> {
    for cap in bundle.required_caps_for_current_os() {
        if !caps.all().contains(&cap) {
            return Err(anyhow::anyhow!("missing required capability: {cap}"));
        }
    }
    Ok(())
}
```

Acceptance:

- bundle requiring unsupported WFP fails activation
- failure does not replace LKG
- telemetry event `activation.missing_capability` emitted

---

### P2-T3: Linux eBPF runtime mode management

Current eBPF program has:

```rust
RUNTIME_MODE.get(0).map(|m| m.default_action).unwrap_or(1)
```

Add userspace setter:

```rust
pub fn set_runtime_default_action(default_action: u32) -> Result<()> {
    let pin_path = format!("{}/RUNTIME_MODE", crate::linux::BPFFS_PATH);
    let map_data = aya::maps::MapData::from_pin(&pin_path)?;
    let mut map: aya::maps::PerCpuArray<_, dek_ebpf_common::DekRuntimeMode> =
        aya::maps::PerCpuArray::try_from(map_data)?;

    let value = dek_ebpf_common::DekRuntimeMode {
        default_action,
        generation: current_generation(),
    };

    map.set(0, value, 0)?;
    Ok(())
}
```

`fail_closed()` in Linux backend must call:

```rust
set_runtime_default_action(0)?;
```

`apply()` with valid rules may call:

```rust
set_runtime_default_action(1)?;
```

or keep deny by default for protected workloads.

---

### P2-T4: Implement Windows WFP user-mode v1

Add dependencies:

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_NetworkManagement_WindowsFilteringPlatform",
  "Win32_Security",
  "Win32_System_Rpc"
] }
```

Skeleton:

```rust
#[cfg(windows)]
mod native {
    use anyhow::{Context, Result};
    use windows::Win32::NetworkManagement::WindowsFilteringPlatform::*;

    pub struct WfpEngine {
        handle: HANDLE,
    }

    impl WfpEngine {
        pub fn open() -> Result<Self> {
            let mut handle = HANDLE::default();
            unsafe {
                FwpmEngineOpen0(
                    None,
                    RPC_C_AUTHN_WINNT,
                    None,
                    None,
                    &mut handle,
                ).ok().context("FwpmEngineOpen0")?;
            }
            Ok(Self { handle })
        }
    }

    impl Drop for WfpEngine {
        fn drop(&mut self) {
            unsafe { let _ = FwpmEngineClose0(self.handle); }
        }
    }
}
```

Implement at least:

- open engine
- create sublayer
- clear own filters by provider/sublayer
- add ALE_AUTH_CONNECT_V4/V6 block filters for CIDR/port
- fail_closed deny-all with control-plane allow exception

Acceptance:

```powershell
cargo test -p dek-windows-wfp --features wfp-native
```

Manual Windows test:

```powershell
pollen-dek doctor --network
pollen-dek network publish --deny 1.1.1.1:443
curl https://1.1.1.1 # should fail
```

---

### P2-T5: Implement macOS NEFilter v1

Create:

```text
platform/macos/PollenDEKNetworkExtension/
  PollenFilterDataProvider.swift
  PollenFilterControlProvider.swift
  IPCServer.swift
  Entitlements.plist
```

Rust crate `dek-macos-nefilter` remains IPC client.

Swift sketch:

```swift
import NetworkExtension

final class PollenFilterDataProvider: NEFilterDataProvider {
    private var rules = RuleStore()

    override func startFilter(completionHandler: @escaping (Error?) -> Void) {
        IPCServer.shared.onRulesUpdate = { [weak self] newRules in
            self?.rules.replace(newRules)
        }
        IPCServer.shared.start()
        completionHandler(nil)
    }

    override func handleNewFlow(_ flow: NEFilterFlow) -> NEFilterNewFlowVerdict {
        let meta = FlowMetadata.from(flow)
        if rules.shouldDeny(meta) {
            return .drop()
        }
        return .allow()
    }
}
```

Rust IPC client:

```rust
pub fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
    let payload = serde_json::to_vec(&rules)?;
    self.socket.send(&payload)?;
    Ok(())
}
```

Acceptance:

- System Extension installs
- user approval documented
- flow allow/drop test passes
- DEK does not advertise `os.l4.nefilter.v1` until extension is active

---

## Phase P3 — Secure Telemetry Spool Integration

### P3-T1: Wire `dek-secure-spool` into `dek-telemetry`

Target behavior:

```text
emit_async(event)
  -> redact
  -> validate schema
  -> try send cloud
  -> if fail: append encrypted spool segment
  -> background flusher replay encrypted segments
```

Example integration:

```rust
pub async fn emit_async(&self, event: serde_json::Value, priority: Priority) {
    let redacted = redact_event(event);
    match self.try_send(&redacted).await {
        Ok(_) => return,
        Err(e) => {
            tracing::warn!("telemetry send failed; spooling encrypted event: {e}");
            self.secure_spool.append(redacted, priority).await?;
        }
    }
}
```

### P3-T2: Add replay and ack semantics

Frame lifecycle:

```text
pending segment
  -> sealed segment
  -> replaying
  -> acknowledged
  -> deleted
```

Never delete a segment until Cloud/LCP returns:

```json
{
  "schema_version": "telemetry-ingest-response.v1",
  "accepted": N,
  "rejected": 0
}
```

### P3-T3: Add quota and retention

Config:

```toml
[telemetry.spool]
max_bytes = 536870912
max_segment_bytes = 8388608
max_retention_hours = 72
drop_policy = "drop_low_priority_first"
```

Acceptance:

- offline 10k events creates encrypted spool only
- no plaintext PII appears in spool files
- replay after reconnect sends all events
- corrupt frame rejected and quarantined

---

## Phase P4 — Policy Authoring/Compile/Deploy Completeness

### P4-T1: PPI as single intermediate representation

All policy authoring must output PPI first:

```json
{
  "schema_version": "pollen-policy-intent.v1",
  "policy_id": "pol-001",
  "intent_type": "network_egress",
  "targets": {},
  "conditions": {},
  "effect": "deny",
  "obligations": []
}
```

Cloud Natural Language Editor:

```text
natural language -> AI parser -> PPI -> validation -> compiler routing -> artifacts
```

Local Dashboard:

```text
structured forms / YAML expert mode -> PPI -> validation -> compiler routing -> artifacts
```

DEK:

```text
never authors, never compiles; only verifies, activates, enforces
```

### P4-T2: Compiler routing

Rules:

| Policy kind | Compiler |
|---|---|
| ABAC/RBAC app/tool decision | Cedar |
| complex content/request logic | OPA/Rego/WASM |
| relationship authorization | OpenFGA |
| simple CIDR/port/process egress | OS L4 artifact |
| transform/redaction | WASM plugin config |

Example router:

```rust
pub fn select_compiler(intent: &PolicyIntent) -> Vec<CompilerTarget> {
    match intent.intent_type.as_str() {
        "relationship_access" => vec![CompilerTarget::OpenFga],
        "network_egress" if intent.is_kernel_safe() => vec![CompilerTarget::OsL4],
        "data_redaction" => vec![CompilerTarget::OpaWasm, CompilerTarget::WasmPlugin],
        "tool_authorization" => vec![CompilerTarget::Cedar],
        _ => vec![CompilerTarget::OpaWasm],
    }
}
```

### P4-T3: Dry-run simulator must use same runtime path

Current dashboard includes dry-run simulation. Ensure:

```text
Dashboard simulate -> LCP /simulate endpoint -> policy compiler -> same PDP adapter -> no telemetry mutation -> response uses DecisionResult schema
```

Acceptance:

- dry-run uses `evaluate_dry_run`
- does not mutate circuit breaker stats
- returns `adapter_results` and `obligations`

---

## Phase P5 — CI/CD and Release Readiness

### P5-T1: Consolidated CI gates

Main branch required checks:

```text
Rust default workspace: fmt, clippy, test, release build
Linux eBPF: build prog, build daemon, verifier smoke if possible
WASM plugins: build wasm32-wasip1
Contract CI: npm test, generated drift, breaking gate, docs update
Dashboard CI: typecheck, lint, unit test, build
Conformance: LCP + Mock-Cloud OpenAPI/JSON Schema
Security: cargo audit, cargo deny, secret scan
Release dry-run: package but do not publish
```

### P5-T2: cargo-deny

Add `deny.toml`:

```toml
[licenses]
allow = ["Apache-2.0", "MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016"]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
```

Workflow:

```yaml
- uses: EmbarkStudios/cargo-deny-action@v2
```

### P5-T3: Release workflow hardening

Current release workflow is good but should add:

- `cargo test --locked` before build
- fail if any staging binary missing
- build eBPF before release binaries or include object properly
- sign `SHA256SUMS` too
- attach contract artifacts
- upload SBOM per target
- generate installer packages only after smoke test

Patch excerpt:

```yaml
- name: Preflight test
  run: |
    cargo fmt --all -- --check
    cargo test --workspace --exclude dek-ebpf-prog --exclude pii-redactor-plugin --locked

- name: Ensure staged binaries exist
  shell: bash
  run: |
    test -f staging/dek-core || test -f staging/dek-core.exe
    test -f staging/dek-cli || test -f staging/dek-cli.exe
```

Sign `SHA256SUMS`:

```bash
cosign sign-blob --yes \
  --output-signature SHA256SUMS.sig \
  --output-certificate SHA256SUMS.pem \
  SHA256SUMS
```

---

## 6. Production Acceptance Criteria

### 6.1 Local Mode

```bash
local-control-plane
dek-cli profile set local --trusted-key <key>
dek-cli enroll --cloud-url http://127.0.0.1:17891
dek-core
dek-cli doctor
```

Must pass:

- LCP discovery works
- Dashboard loads
- policy created in Dashboard
- bundle signed
- DEK fetches `/bundles/latest`
- envelope schema validates
- sidecar API reloads
- MCP decision returns valid DecisionResult
- telemetry appears in Dashboard
- offline spool encrypts events
- reconnect replays telemetry

### 6.2 Mock Cloud Mode

Must pass:

- enrollment
- mTLS
- bundle fetch
- SSE BundleReady
- trusted key rotation
- rollback blocked
- tampered bundle rejected
- telemetry ingest
- chaos outage triggers LKG then StrictDeny

### 6.3 Linux L4

Must pass:

- eBPF object loads
- maps pinned
- `RUNTIME_MODE` default action changes on strict deny
- CIDR deny blocks test IP
- DNS observe updates LRU map
- TTL janitor deletes expired entries
- no verifier rejection

### 6.4 Windows L4

Beta can be one of:

- `stub` not advertised as `os.l4.wfp.v1`, or
- native WFP starts and enforces ALE filters

No intermediate state where system claims WFP support but cannot enforce.

### 6.5 macOS L4

Beta can be one of:

- `stub` not advertised as `os.l4.nefilter.v1`, or
- System Extension installed and `NEFilterDataProvider` enforces flow drop/allow

No intermediate state where system claims NEFilter support but cannot enforce.

### 6.6 Release

A release is valid only if:

```text
- all CI required checks pass
- release artifacts exist for target OS/arch
- SHA256SUMS validates
- cosign verify succeeds
- GitHub provenance attestation exists
- SBOM exists
- contract tarball exists
- CHANGELOG/COMPATIBILITY updated
```

---

## 7. AI Agent Execution Plan

Use this exact order.

### Sprint 1 — Contract and CI Truth

1. Fix bundle envelope schema value mismatch
2. Fix bundle latest response
3. Fix telemetry response
4. Enforce `lint:semantic`
5. Add Dashboard CI
6. Add conformance test crate

### Sprint 2 — Linux eBPF Hardening

1. Fix `dns_cache.rs` compile/import issues
2. Make `BPFFS_PATH` accessible
3. Harden eBPF build path
4. Add runtime mode setter
5. Add DNS LRU/TTL tests
6. Add eBPF map pressure metrics

### Sprint 3 — Runtime Capability and Activation

1. Add runtime capability collector
2. Wire capabilities into discovery
3. Reject unsupported required capabilities at activation
4. Add telemetry events for missing capability
5. Remove/rename misleading `os.l4.wfp.v1`/`nefilter.v1` from static discovery unless active

### Sprint 4 — Telemetry Secure Spool

1. Wire secure spool into `dek-telemetry`
2. Add replay/ack lifecycle
3. Add retention/quota
4. Add key rotation
5. Add corruption/quarantine tests

### Sprint 5 — Windows/macOS Native Preview

1. Implement WFP engine open/close + sublayer
2. Implement allow/control-plane exemption + deny filter
3. Implement NEFilter System Extension project skeleton
4. Implement Rust IPC client for NEFilter
5. Add OS-specific docs and manual tests

### Sprint 6 — Release

1. Harden release workflow
2. Add release dry run
3. Add artifact/contract signing
4. Generate installers
5. Update quickstarts
6. Tag `v1.0.0-beta.7` or next beta

---

## 8. Recommended Repo Documentation Updates

Add or update:

```text
docs/release/RELEASE_CHECKLIST.md
docs/release/VERIFY_RELEASE.md
docs/contracts/CLOUD_CONSUMER_GUIDE.md
docs/contracts/MOCK_CLOUD_CONFORMANCE.md
docs/security/SECURE_SPOOLING.md
docs/security/SUPPLY_CHAIN.md
docs/os/linux-ebpf.md
docs/os/windows-wfp.md
docs/os/macos-nefilter.md
docs/dashboard/CONTRACT_CLIENT.md
docs/runbooks/STRICT_DENY.md
docs/runbooks/BUNDLE_ROLLBACK.md
```

Each doc must be enforced by CI when related source changes.

Example docs gate:

```bash
if git diff --name-only origin/main...HEAD | grep -E 'crates/dek-ebpfd|crates/dek-ebpf-prog'; then
  git diff --name-only origin/main...HEAD | grep -E 'docs/os/linux-ebpf.md|docs/runbooks/STRICT_DENY.md' \
    || { echo "eBPF changes require docs update"; exit 1; }
fi
```

---

## 9. Final Recommendation

Pollen DEK is now in a strong beta architecture state, but the next step must focus on **truthful production behavior**, not adding more features.

Highest-priority fixes:

1. Contract conformance
2. OS capability truthfulness
3. Linux eBPF compile/runtime hardening
4. Secure spool integration
5. Dashboard generated client
6. Release verification

Do **not** tag a production-like release until:

```text
contracts/source == generated artifacts == LCP behavior == Mock-Cloud behavior == Dashboard types == DEK client behavior
```

The main anti-drift rule should be:

> Pollen Cloud in another repo must consume the same released contract artifact; it must never copy schemas manually.

---

## 10. External References Used

- OpenAPI Specification v3.1.0: https://spec.openapis.org/oas/v3.1.0.html
- AsyncAPI Specification v3.0.0: https://www.asyncapi.com/docs/reference/specification/v3.0.0
- OPA Bundles: https://www.openpolicyagent.org/docs/management-bundles
- Cedar validation: https://docs.cedarpolicy.com/policies/validation.html
- OpenFGA concepts: https://openfga.dev/docs/concepts
- Wasmtime fast instantiation: https://docs.wasmtime.dev/examples-fast-instantiation.html
- Linux eBPF LRU maps: https://docs.kernel.org/bpf/map_hash.html
- Windows WFP ALE: https://learn.microsoft.com/en-us/windows/win32/fwp/application-layer-enforcement--ale-
- Apple NetworkExtension traffic filtering: https://developer.apple.com/documentation/networkextension/filtering-network-traffic
- GitHub Actions OIDC: https://docs.github.com/en/actions/concepts/security/openid-connect
- Sigstore/cosign: https://docs.sigstore.dev/cosign/
