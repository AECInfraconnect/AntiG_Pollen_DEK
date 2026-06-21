# Pollen Contract Hub Implementation Plan

**Document status:** Implementation-ready plan for AI Agent / Dev Team  
**Target systems:** Pollen DEK, Local Control Plane, Local Admin Dashboard, Mock-Cloud, Pollen Cloud  
**Recommended initial location:** `AntiG_Pollen_DEK/contracts/`  
**Future location:** separate repo `AECInfraconnect/pollen-contracts` when Pollen Cloud has an independent release cycle  
**Main goal:** prevent contract drift across Cloud ↔ DEK ↔ Local Dashboard by making API/schema/event/policy-bundle contracts versioned, validated, generated, signed, and continuously updated.

---

## 0. Executive Summary

Pollen needs a **Contract Hub** because API versioning alone is not enough for a security-sensitive edge enforcement system. DEK nodes may remain offline, old, or partially upgraded, while Pollen Cloud, Mock-Cloud, Local Control Plane, and Dashboard evolve independently. A schema mismatch can trigger incorrect enforcement, fail-closed behavior, telemetry ingestion failure, or silent drift.

This implementation plan creates a **single source of truth** for:

1. HTTP API contracts: OpenAPI 3.1 generated from TypeSpec.
2. Event contracts: AsyncAPI for SSE/event-stream flows.
3. Runtime data models: JSON Schema Draft 2020-12.
4. Machine-readable registries: capability flags, headers, error codes, enum catalogs, PEP/PDP/OS enforcement layer identifiers.
5. Generated SDK/type packages:
   - Rust crate: `pollen-contract`
   - TypeScript package: `@pollen/contract`
6. CI gates:
   - schema validation
   - OpenAPI lint
   - breaking-change detection
   - generated artifact drift check
   - provider conformance
   - Pact-style consumer contract tests
   - signed release artifacts
7. Runtime negotiation:
   - `/.well-known/pollen-contract`
   - `X-Pollen-Contract-Version`
   - capability intersection
   - deprecation/sunset headers
   - deterministic GracePeriod/LKG behavior when contract mismatch occurs

**Core rule:** Pollen Cloud must not invent endpoint/schema/event definitions in its own repo. It must consume the same Contract Hub artifacts used by DEK, Local Control Plane, Local Admin Dashboard, and Mock-Cloud.

---

## 1. Guiding Principles

### 1.1 Contract-first, but staging-first

During the beta phase, implement Contract Hub inside the DEK repo:

```text
AntiG_Pollen_DEK/
└── contracts/
```

Move to a standalone repository later:

```text
AECInfraconnect/pollen-contracts
```

Do not split into a separate repository too early unless the Pollen Cloud team already has independent CI/CD, release tagging, and CODEOWNERS. Early separation can slow development.

### 1.2 Canonical artifacts

The canonical runtime artifacts are:

```text
OpenAPI 3.1         → HTTP REST contract
AsyncAPI            → SSE/event contract
JSON Schema 2020-12 → payload, bundle, telemetry, entity, decision models
catalog/*.yaml      → enums, error codes, capability flags, PEP/PDP identifiers
```

TypeSpec is recommended as the authoring layer, but the generated OpenAPI/JSON Schema/AsyncAPI outputs are what downstream repos consume.

### 1.3 Do not hand-edit generated outputs

Generated files must be reproducible from source specs.

CI must fail if:

```text
source spec changed but generated artifact was not updated
or
someone manually edited generated artifact without changing source spec
```

### 1.4 Docs must update with contracts

Every PR that changes contracts must update the relevant documents:

```text
contracts/CHANGELOG.md
contracts/COMPATIBILITY.md
contracts/docs/MIGRATION.md when needed
contracts/docs/CLOUD_CONSUMER_GUIDE.md when Cloud usage changes
contracts/docs/DEK_CONSUMER_GUIDE.md when DEK behavior changes
```

CI must verify that contract-changing PRs also update at least one documentation/changelog file.

### 1.5 Contract drift is a security incident

A response that fails schema validation must be treated like an integrity failure:

```text
- reject artifact
- keep Last Known Good bundle
- emit audit event contract.schema_mismatch
- enter GracePeriod only when required
- never silently accept incompatible policy/telemetry/bundle data
```

---

## 2. Target Repository Layout

### 2.1 Initial layout inside DEK repo

```text
AntiG_Pollen_DEK/
├── contracts/
│   ├── README.md
│   ├── tspconfig.yaml
│   ├── package.json
│   ├── spec/
│   │   ├── main.tsp
│   │   ├── rest/
│   │   │   ├── common.tsp
│   │   │   ├── bundles.tsp
│   │   │   ├── telemetry.tsp
│   │   │   ├── registry.tsp
│   │   │   ├── enrollment.tsp
│   │   │   └── contract-discovery.tsp
│   │   └── events/
│   │       └── sse.asyncapi.yaml
│   ├── schemas/
│   │   ├── bundle-envelope.v1.schema.json
│   │   ├── bundle-manifest.v2.schema.json
│   │   ├── bundle-signature.v1.schema.json
│   │   ├── telemetry-envelope.v1.schema.json
│   │   ├── decision-result.v1.schema.json
│   │   ├── entity.ai-agent.v1.schema.json
│   │   ├── entity.mcp-server.v1.schema.json
│   │   ├── entity.resource.v1.schema.json
│   │   └── contract-discovery.v1.schema.json
│   ├── catalog/
│   │   ├── capabilities.yaml
│   │   ├── headers.yaml
│   │   ├── error-codes.yaml
│   │   ├── audit-events.yaml
│   │   ├── decision-enums.yaml
│   │   ├── entity-kinds.yaml
│   │   ├── pdp-types.yaml
│   │   ├── pep-types.yaml
│   │   └── os-enforcement-layers.yaml
│   ├── generated/
│   │   ├── openapi/pollen.v1.yaml
│   │   ├── asyncapi/pollen-sse.v1.yaml
│   │   ├── rust/pollen-contract/
│   │   └── typescript/
│   ├── pacts/
│   ├── docs/
│   │   ├── CLOUD_CONSUMER_GUIDE.md
│   │   ├── DEK_CONSUMER_GUIDE.md
│   │   ├── MOCK_CLOUD_CONFORMANCE.md
│   │   ├── MIGRATION.md
│   │   ├── RELEASE_PROCESS.md
│   │   └── CONTRACT_GOVERNANCE.md
│   ├── CHANGELOG.md
│   ├── COMPATIBILITY.md
│   └── SECURITY.md
├── .github/workflows/contract-ci.yml
├── .github/workflows/contract-release.yml
└── .spectral.yaml
```

### 2.2 Future standalone repo layout

When Pollen Cloud becomes a separate production repository, split `contracts/` into:

```text
pollen-contracts/
├── spec/
├── schemas/
├── catalog/
├── generated/
├── pacts/
├── docs/
├── .github/workflows/
├── CHANGELOG.md
├── COMPATIBILITY.md
├── CODEOWNERS
└── SECURITY.md
```

CODEOWNERS example:

```text
# pollen-contracts/CODEOWNERS
/spec/       @pollen-dek-leads @pollen-cloud-leads
/schemas/    @pollen-dek-leads @pollen-cloud-leads
/catalog/    @pollen-dek-leads @pollen-cloud-leads
/generated/  @pollen-platform-leads
/docs/       @pollen-docs @pollen-dek-leads @pollen-cloud-leads
```

---

## 3. Versioning Model

Pollen must separate four version layers.

| Layer | Example | Purpose |
|---|---:|---|
| `contract_version` | `1.2` | Wire/API compatibility between Cloud/LCP/DEK/Dashboard |
| `schema_version` | `telemetry-envelope.v1` | Long-lived payload compatibility, especially spool/LKG |
| `bundle_format_version` | `bundle-envelope.v1` | Signed bundle/envelope parsing and verification |
| `artifact_version` | Git tag `contracts-v0.3.0` | Release/package version of Contract Hub |

### 3.1 Compatibility policy

Default policy for beta:

```text
Cloud and Mock-Cloud must support DEK current and N-1 minor contract.
Sunset period for a supported minor contract must be at least 90 days.
Breaking change requires major bump or explicit breaking-approved label.
DEK must never StrictDeny immediately due only to unsupported Cloud contract if LKG bundle is still valid.
```

Recommended production policy:

```text
Cloud supports current, N-1, and N-2 minor contracts.
Sunset period is at least 180 days for enterprise customers.
```

### 3.2 COMPATIBILITY.md template

```md
# Contract Compatibility Matrix

| Contract | Status | Cloud | Mock-Cloud | DEK | LCP/Dashboard | Sunset | Notes |
|---|---|---|---|---|---|---|---|
| 1.0 | current | >= 0.1.0 | >= 0.1.0 | >= 1.0.0-beta.1 | >= 0.1.0 | - | Initial beta contract |
| 0.9 | deprecated | >= 0.1.0 | >= 0.1.0 | 0.9.x | 0.9.x | 2026-10-01 | Pre-v1 migration |
```

---

## 4. Canonical Bundle Envelope

Local Mode, Mock-Cloud Mode, and Cloud Mode must use the same envelope shape:

```json
{
  "manifest": {},
  "signatures": []
}
```

### 4.1 `bundle-envelope.v1.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.pollen.dev/schemas/bundle-envelope.v1.schema.json",
  "title": "PollenBundleEnvelopeV1",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "manifest", "signatures"],
  "properties": {
    "schema_version": {
      "type": "string",
      "const": "bundle-envelope.v1"
    },
    "manifest": {
      "$ref": "bundle-manifest.v2.schema.json"
    },
    "signatures": {
      "type": "array",
      "minItems": 1,
      "items": { "$ref": "bundle-signature.v1.schema.json" }
    }
  }
}
```

### 4.2 `bundle-signature.v1.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.pollen.dev/schemas/bundle-signature.v1.schema.json",
  "title": "PollenBundleSignatureV1",
  "type": "object",
  "additionalProperties": false,
  "required": ["key_id", "alg", "signature", "signed_at"],
  "properties": {
    "key_id": { "type": "string", "minLength": 1 },
    "alg": {
      "type": "string",
      "enum": ["ed25519", "ecdsa-p256-sha256", "cosign-bundle-v1"]
    },
    "signature": { "type": "string", "contentEncoding": "base64" },
    "signed_at": { "type": "string", "format": "date-time" },
    "signer": { "type": "string" },
    "certificate_chain_ref": { "type": "string" }
  }
}
```

### 4.3 `bundle-manifest.v2.schema.json` minimum fields

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.pollen.dev/schemas/bundle-manifest.v2.schema.json",
  "title": "PollenPolicyBundleManifestV2",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "schema_version",
    "bundle_id",
    "tenant_id",
    "generation",
    "created_at",
    "expires_at",
    "target",
    "artifacts"
  ],
  "properties": {
    "schema_version": { "type": "string", "const": "bundle-manifest.v2" },
    "bundle_id": { "type": "string" },
    "tenant_id": { "type": "string" },
    "generation": { "type": "integer", "minimum": 1 },
    "created_at": { "type": "string", "format": "date-time" },
    "expires_at": { "type": "string", "format": "date-time" },
    "target": {
      "type": "object",
      "additionalProperties": false,
      "required": ["device_id", "pep_types", "pdp_types"],
      "properties": {
        "device_id": { "type": "string" },
        "entity_ids": { "type": "array", "items": { "type": "string" } },
        "pep_types": { "type": "array", "items": { "$ref": "../catalog/pep-types.schema.json" } },
        "pdp_types": { "type": "array", "items": { "$ref": "../catalog/pdp-types.schema.json" } },
        "os_layers": { "type": "array", "items": { "type": "string" } }
      }
    },
    "capabilities_required": {
      "type": "array",
      "items": { "type": "string" }
    },
    "artifacts": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["artifact_id", "kind", "sha256", "path"],
        "properties": {
          "artifact_id": { "type": "string" },
          "kind": {
            "type": "string",
            "enum": ["opa-wasm", "rego", "cedar", "openfga-model", "openfga-tuples", "os-l4-rules", "wasm-plugin-config"]
          },
          "sha256": { "type": "string", "pattern": "^[a-fA-F0-9]{64}$" },
          "path": { "type": "string" },
          "entrypoint": { "type": "string" }
        }
      }
    }
  }
}
```

---

## 5. Catalogs

Catalogs prevent string drift. They must be machine-readable YAML and generate Rust/TypeScript enums.

### 5.1 `catalog/capabilities.yaml`

```yaml
version: 1
capabilities:
  - id: contract.discovery.v1
    owner: platform
    description: Supports /.well-known/pollen-contract discovery.
  - id: bundle.signed-envelope.v1
    owner: bundle
    description: Supports signed bundle envelope { manifest, signatures }.
  - id: sse.bundle-ready.v1
    owner: cloud
    description: Supports BundleReady event over SSE.
  - id: telemetry.batch.v1
    owner: telemetry
    description: Supports batched telemetry ingestion.
  - id: policy.opa-wasm.v1
    owner: policy
    description: Supports OPA WASM policy artifacts.
  - id: policy.cedar.v1
    owner: policy
    description: Supports Cedar policy artifacts.
  - id: policy.openfga.v1
    owner: policy
    description: Supports OpenFGA model and tuple artifacts.
  - id: os.l4.ebpf.v1
    owner: dek-ebpfd
    description: Supports Linux L4 enforcement using eBPF.
  - id: os.l4.wfp.v1
    owner: dek-windows-wfp
    description: Supports Windows L4 enforcement using WFP.
  - id: os.l4.nefilter.v1
    owner: dek-macos-nefilter
    description: Supports macOS metadata-first NetworkExtension filtering.
```

### 5.2 `catalog/headers.yaml`

```yaml
version: 1
headers:
  X-Pollen-Contract-Version:
    required: true
    direction: request-response
    description: Contract minor version used on the wire.
  X-Pollen-Device-Id:
    required: true
    direction: request
    description: Stable DEK device identity.
  X-Pollen-Tenant-Id:
    required: true
    direction: request
    description: Tenant identity.
  X-Pollen-Bundle-Generation:
    required: false
    direction: request-response
    description: Last known policy bundle generation.
  Deprecation:
    required: false
    direction: response
    description: Standard HTTP deprecation signal.
  Sunset:
    required: false
    direction: response
    description: Standard HTTP sunset timestamp.
```

### 5.3 `catalog/error-codes.yaml`

```yaml
version: 1
errors:
  CONTRACT_VERSION_UNSUPPORTED:
    http_status: 426
    severity: warning
    dek_behavior: enter_grace_period_keep_lkg
  CONTRACT_SCHEMA_MISMATCH:
    http_status: 422
    severity: security
    dek_behavior: reject_artifact_keep_lkg_emit_audit
  BUNDLE_SIGNATURE_INVALID:
    http_status: 422
    severity: security
    dek_behavior: reject_artifact_keep_lkg_emit_audit
  CAPABILITY_MISSING:
    http_status: 409
    severity: warning
    dek_behavior: reject_artifact_keep_lkg_emit_audit
```

---

## 6. TypeSpec Implementation

### 6.1 `contracts/tspconfig.yaml`

```yaml
emit:
  - "@typespec/openapi3"
options:
  "@typespec/openapi3":
    emitter-output-dir: "generated/openapi"
    output-file: "pollen.v1.yaml"
    openapi-versions:
      - 3.1.0
```

### 6.2 `contracts/spec/main.tsp`

```typespec
import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi";

using TypeSpec.Http;
using TypeSpec.Rest;

@service({ title: "Pollen Cloud-DEK Contract", version: "1.0" })
@server("https://api.pollen.cloud", "Pollen Cloud")
@server("http://127.0.0.1:17891", "Local Control Plane / Mock Cloud")
namespace Pollen.Contract;

import "./rest/common.tsp";
import "./rest/contract-discovery.tsp";
import "./rest/bundles.tsp";
import "./rest/telemetry.tsp";
import "./rest/registry.tsp";
import "./rest/enrollment.tsp";
```

### 6.3 `contracts/spec/rest/common.tsp`

```typespec
namespace Pollen.Contract;

model PollenHeaders {
  @header("X-Pollen-Contract-Version")
  contractVersion: string;

  @header("X-Pollen-Device-Id")
  deviceId?: string;

  @header("X-Pollen-Tenant-Id")
  tenantId?: string;
}

model ErrorEnvelope {
  schema_version: "error-envelope.v1";
  error: PollenError;
}

model PollenError {
  code: string;
  message: string;
  correlation_id?: string;
  details?: Record<unknown>;
}

model Page<T> {
  items: T[];
  next_cursor?: string;
}
```

### 6.4 `contracts/spec/rest/contract-discovery.tsp`

```typespec
namespace Pollen.Contract;

model ContractDiscoveryResponse {
  schema_version: "contract-discovery.v1";
  supported: string[];
  preferred: string;
  minimum_dek_version?: string;
  sunset?: Record<string>;
  capabilities: string[];
}

@route("/.well-known/pollen-contract")
interface ContractDiscoveryApi {
  @get
  getDiscovery(...PollenHeaders): ContractDiscoveryResponse | ErrorEnvelope;
}
```

### 6.5 `contracts/spec/rest/bundles.tsp`

```typespec
namespace Pollen.Contract;

model BundleFetchRequest {
  tenant_id: string;
  device_id: string;
  current_generation?: int32;
  capabilities: string[];
}

model BundleFetchResponse {
  schema_version: "bundle-fetch-response.v1";
  status: "not_modified" | "bundle_ready";
  generation?: int32;
  envelope?: unknown; // JSON Schema validates the exact bundle-envelope.v1 shape.
}

@route("/v1/tenants/{tenantId}/devices/{deviceId}/bundles/latest")
interface BundleApi {
  @post
  fetchLatest(
    ...PollenHeaders,
    @path tenantId: string,
    @path deviceId: string,
    @body body: BundleFetchRequest
  ): BundleFetchResponse | ErrorEnvelope;
}
```

### 6.6 `contracts/spec/rest/telemetry.tsp`

```typespec
namespace Pollen.Contract;

model TelemetryBatchRequest {
  schema_version: "telemetry-batch.v1";
  tenant_id: string;
  device_id: string;
  batch_id: string;
  events: unknown[];
}

model TelemetryIngestResponse {
  schema_version: "telemetry-ingest-response.v1";
  accepted: int32;
  rejected: int32;
  retry_after_seconds?: int32;
}

@route("/v1/telemetry/batches")
interface TelemetryApi {
  @post
  ingestBatch(
    ...PollenHeaders,
    @body body: TelemetryBatchRequest
  ): TelemetryIngestResponse | ErrorEnvelope;
}
```

---

## 7. AsyncAPI for SSE Events

### 7.1 `contracts/spec/events/sse.asyncapi.yaml`

```yaml
asyncapi: 3.0.0
info:
  title: Pollen Cloud-DEK SSE Contract
  version: 1.0.0
servers:
  cloud:
    host: api.pollen.cloud
    protocol: https
  local:
    host: 127.0.0.1:17891
    protocol: http
channels:
  bundleEvents:
    address: /v1/tenants/{tenantId}/devices/{deviceId}/events
    messages:
      BundleReady:
        $ref: '#/components/messages/BundleReady'
      ContractDeprecated:
        $ref: '#/components/messages/ContractDeprecated'
operations:
  receiveBundleEvents:
    action: receive
    channel:
      $ref: '#/channels/bundleEvents'
components:
  messages:
    BundleReady:
      name: BundleReady
      title: BundleReady
      payload:
        $ref: '#/components/schemas/BundleReadyPayload'
    ContractDeprecated:
      name: ContractDeprecated
      title: ContractDeprecated
      payload:
        $ref: '#/components/schemas/ContractDeprecatedPayload'
  schemas:
    BundleReadyPayload:
      type: object
      additionalProperties: false
      required: [schema_version, tenant_id, device_id, generation]
      properties:
        schema_version:
          const: sse.bundle-ready.v1
        tenant_id:
          type: string
        device_id:
          type: string
        generation:
          type: integer
        reason:
          type: string
    ContractDeprecatedPayload:
      type: object
      additionalProperties: false
      required: [schema_version, contract_version, sunset]
      properties:
        schema_version:
          const: sse.contract-deprecated.v1
        contract_version:
          type: string
        sunset:
          type: string
          format: date-time
        migration_url:
          type: string
```

---

## 8. Build and Codegen Tooling

### 8.1 `contracts/package.json`

```json
{
  "name": "@pollen/contracts-source",
  "private": true,
  "scripts": {
    "build": "tsp compile spec/main.tsp && npm run copy:asyncapi && npm run gen:ts",
    "copy:asyncapi": "mkdir -p generated/asyncapi && cp spec/events/sse.asyncapi.yaml generated/asyncapi/pollen-sse.v1.yaml",
    "gen:ts": "openapi-typescript generated/openapi/pollen.v1.yaml -o generated/typescript/api.d.ts",
    "lint:openapi": "spectral lint generated/openapi/pollen.v1.yaml --fail-severity=warn",
    "lint:asyncapi": "asyncapi validate generated/asyncapi/pollen-sse.v1.yaml",
    "validate:schemas": "ajv compile -s 'schemas/*.schema.json' --spec=draft2020",
    "check:generated": "git diff --exit-code generated/",
    "test": "npm run build && npm run lint:openapi && npm run lint:asyncapi && npm run validate:schemas"
  },
  "devDependencies": {
    "@typespec/compiler": "latest",
    "@typespec/http": "latest",
    "@typespec/rest": "latest",
    "@typespec/openapi": "latest",
    "@typespec/openapi3": "latest",
    "@stoplight/spectral-cli": "latest",
    "@asyncapi/cli": "latest",
    "ajv-cli": "latest",
    "ajv-formats": "latest",
    "openapi-typescript": "latest"
  }
}
```

### 8.2 Generated TypeScript package

Create `contracts/generated/typescript/package.json`:

```json
{
  "name": "@pollen/contract",
  "version": "0.1.0",
  "type": "module",
  "files": ["api.d.ts", "catalog", "schemas"],
  "exports": {
    "./api": "./api.d.ts"
  }
}
```

Recommended usage in Dashboard or Pollen Cloud UI:

```ts
import createClient from "openapi-fetch";
import type { paths } from "@pollen/contract/api";

const client = createClient<paths>({
  baseUrl: import.meta.env.VITE_POLLEN_API_BASE_URL,
  headers: {
    "X-Pollen-Contract-Version": "1.0"
  }
});

export async function getContractDiscovery() {
  const { data, error } = await client.GET("/.well-known/pollen-contract");
  if (error) throw error;
  return data;
}
```

---

## 9. Rust Contract Crate

### 9.1 Location

Initial path:

```text
contracts/generated/rust/pollen-contract/
```

Later publish as:

```text
pollen-contract = { git = "https://github.com/AECInfraconnect/pollen-contracts", tag = "v0.1.0" }
```

### 9.2 `Cargo.toml`

```toml
[package]
name = "pollen-contract"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
time = { version = "0.3", features = ["serde", "formatting"] }
thiserror = "1"

[build-dependencies]
serde_json = "1"
typify = "0.4"
schemars = "0.8"
```

### 9.3 `build.rs`

```rust
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let schemas_dir = manifest_dir.join("../../../schemas");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let schema_files = [
        "bundle-envelope.v1.schema.json",
        "telemetry-envelope.v1.schema.json",
        "decision-result.v1.schema.json",
        "contract-discovery.v1.schema.json",
    ];

    let mut generated = String::new();
    generated.push_str("// @generated by pollen-contract build.rs. Do not edit.\n");

    for file in schema_files {
        let content = fs::read_to_string(schemas_dir.join(file))
            .unwrap_or_else(|e| panic!("failed to read schema {file}: {e}"));
        let schema: schemars::schema::RootSchema = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("invalid JSON schema {file}: {e}"));

        let mut type_space = typify::TypeSpace::new(
            typify::TypeSpaceSettings::default()
                .with_struct_builder(false)
                .with_derive("PartialEq".into())
        );
        type_space.add_root_schema(schema).unwrap();
        generated.push_str(&type_space.to_stream().to_string());
        generated.push('\n');
    }

    fs::write(out_dir.join("generated.rs"), generated).unwrap();
    println!("cargo:rerun-if-changed={}", schemas_dir.display());
}
```

### 9.4 `src/lib.rs`

```rust
pub mod generated {
    #![allow(clippy::all)]
    #![allow(non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub const CONTRACT_VERSION: &str = "1.0";
pub const BUNDLE_ENVELOPE_SCHEMA_VERSION: &str = "bundle-envelope.v1";
pub const TELEMETRY_ENVELOPE_SCHEMA_VERSION: &str = "telemetry-envelope.v1";

#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("unsupported contract version: {0}")]
    UnsupportedVersion(String),

    #[error("schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("missing capability: {0}")]
    MissingCapability(String),
}
```

### 9.5 Transition rule for `dek-control-plane-api`

Do not delete old hand-written types immediately. Re-export new types first:

```rust
// crates/dek-control-plane-api/src/lib.rs

#[deprecated(note = "Use pollen_contract::generated types directly where possible")]
pub mod legacy;

pub use pollen_contract::generated::*;
pub use pollen_contract::{CONTRACT_VERSION, ContractError};
```

Migration pattern:

```text
Step 1: add pollen-contract dependency
Step 2: re-export generated types
Step 3: update one module at a time
Step 4: add deprecation warnings
Step 5: remove legacy types after two minor releases
```

---

## 10. Runtime Negotiation

### 10.1 Discovery response schema

`schemas/contract-discovery.v1.schema.json`:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.pollen.dev/schemas/contract-discovery.v1.schema.json",
  "title": "PollenContractDiscoveryV1",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "supported", "preferred", "capabilities"],
  "properties": {
    "schema_version": { "type": "string", "const": "contract-discovery.v1" },
    "supported": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1
    },
    "preferred": { "type": "string" },
    "minimum_dek_version": { "type": "string" },
    "sunset": {
      "type": "object",
      "additionalProperties": { "type": "string", "format": "date-time" }
    },
    "capabilities": {
      "type": "array",
      "items": { "type": "string" }
    }
  }
}
```

### 10.2 Axum route for Local Control Plane / Mock-Cloud / Cloud

```rust
use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ContractDiscoveryResponse {
    schema_version: &'static str,
    supported: Vec<&'static str>,
    preferred: &'static str,
    minimum_dek_version: &'static str,
    sunset: std::collections::BTreeMap<&'static str, &'static str>,
    capabilities: Vec<&'static str>,
}

async fn get_contract_discovery() -> Json<ContractDiscoveryResponse> {
    Json(ContractDiscoveryResponse {
        schema_version: "contract-discovery.v1",
        supported: vec!["1.0"],
        preferred: "1.0",
        minimum_dek_version: "1.0.0-beta.1",
        sunset: Default::default(),
        capabilities: vec![
            "contract.discovery.v1",
            "bundle.signed-envelope.v1",
            "telemetry.batch.v1",
            "policy.opa-wasm.v1",
            "policy.cedar.v1",
            "policy.openfga.v1",
        ],
    })
}

pub fn contract_routes() -> Router {
    Router::new().route("/.well-known/pollen-contract", get(get_contract_discovery))
}
```

### 10.3 DEK client contract check

```rust
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct ContractDiscoveryResponse {
    schema_version: String,
    supported: Vec<String>,
    preferred: String,
    capabilities: Vec<String>,
}

pub async fn negotiate_contract(
    client: &Client,
    base_url: &str,
    local_contract: &str,
    local_caps: &[&str],
) -> anyhow::Result<Vec<String>> {
    let url = format!("{}/.well-known/pollen-contract", base_url.trim_end_matches('/'));
    let res = client
        .get(url)
        .header("X-Pollen-Contract-Version", local_contract)
        .send()
        .await?;

    if !res.status().is_success() {
        anyhow::bail!("contract discovery failed: {}", res.status());
    }

    let discovery: ContractDiscoveryResponse = res.json().await?;

    if !discovery.supported.iter().any(|v| v == local_contract) {
        // DEK behavior: enter GracePeriod but keep LKG if still valid.
        anyhow::bail!("contract version unsupported: {local_contract}");
    }

    let remote: HashSet<_> = discovery.capabilities.into_iter().collect();
    let intersection = local_caps
        .iter()
        .filter(|c| remote.contains(**c))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Ok(intersection)
}
```

### 10.4 HTTP headers middleware pattern

```rust
pub fn add_contract_headers(req: reqwest::RequestBuilder, tenant_id: &str, device_id: &str) -> reqwest::RequestBuilder {
    req.header("X-Pollen-Contract-Version", pollen_contract::CONTRACT_VERSION)
        .header("X-Pollen-Tenant-Id", tenant_id)
        .header("X-Pollen-Device-Id", device_id)
}
```

### 10.5 Runtime behavior table

| Condition | DEK behavior | Audit event |
|---|---|---|
| Supported contract | Active | none |
| Unsupported contract from Cloud | GracePeriod + keep LKG | `contract.version_unsupported` |
| JSON schema mismatch | reject artifact + keep LKG | `contract.schema_mismatch` |
| Signature mismatch | reject artifact + keep LKG | `bundle.signature_invalid` |
| Missing required capability | reject artifact + keep LKG | `contract.capability_missing` |
| LKG expired | StrictDeny if protected mode | `activation.strict_deny` |

---

## 11. CI: Contract Hub

### 11.1 `.github/workflows/contract-ci.yml`

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
          git show origin/${{ github.base_ref }}:contracts/generated/openapi/pollen.v1.yaml > /tmp/base.yaml || true

      - name: Breaking change check
        if: hashFiles('/tmp/base.yaml') != ''
        run: |
          oasdiff breaking /tmp/base.yaml contracts/generated/openapi/pollen.v1.yaml --fail-on ERR

  docs-must-change:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Check docs update for contract changes
        shell: bash
        run: |
          CHANGED=$(git diff --name-only origin/${{ github.base_ref }}...HEAD)
          echo "$CHANGED"
          if echo "$CHANGED" | grep -E '^contracts/(spec|schemas|catalog)/' >/dev/null; then
            if ! echo "$CHANGED" | grep -E '^contracts/(CHANGELOG.md|COMPATIBILITY.md|docs/)' >/dev/null; then
              echo "Contract source changed, but CHANGELOG/COMPATIBILITY/docs were not updated."
              exit 1
            fi
          fi
```

### 11.2 `.spectral.yaml`

```yaml
extends:
  - spectral:oas

rules:
  pollen-operation-id-required:
    description: Every operation must have operationId for stable codegen.
    severity: error
    given: $.paths[*][*]
    then:
      field: operationId
      function: truthy

  pollen-contract-version-header-required:
    description: Every operation must document X-Pollen-Contract-Version.
    severity: warn
    given: $.paths[*][*].parameters
    then:
      function: schema
      functionOptions:
        schema:
          type: array
          contains:
            type: object
            properties:
              name:
                const: X-Pollen-Contract-Version
              in:
                const: header

  pollen-error-envelope-required:
    description: Operations should document standard error response.
    severity: warn
    given: $.paths[*][*].responses
    then:
      field: 'default'
      function: truthy
```

### 11.3 Semantic contract lint script

Create `contracts/scripts/semantic-contract-lint.ts`:

```ts
import fs from "node:fs";
import yaml from "yaml";

const errors = yaml.parse(fs.readFileSync("catalog/error-codes.yaml", "utf8"));
const decisions = yaml.parse(fs.readFileSync("catalog/decision-enums.yaml", "utf8"));

function fail(message: string): never {
  console.error(`semantic-contract-lint: ${message}`);
  process.exit(1);
}

for (const [code, def] of Object.entries(errors.errors ?? {})) {
  if (!/^[A-Z0-9_]+$/.test(code)) fail(`invalid error code format: ${code}`);
  if (!(def as any).dek_behavior) fail(`missing dek_behavior for ${code}`);
}

const requiredDecisions = ["allow", "deny", "observe", "redact"];
const decisionValues = new Set(decisions.decisions ?? []);
for (const d of requiredDecisions) {
  if (!decisionValues.has(d)) fail(`missing canonical decision enum: ${d}`);
}

console.log("semantic contract lint passed");
```

Add script:

```json
{
  "scripts": {
    "lint:semantic": "tsx scripts/semantic-contract-lint.ts"
  },
  "devDependencies": {
    "tsx": "latest",
    "yaml": "latest"
  }
}
```

---

## 12. Provider Conformance Tests

Provider conformance must run in:

```text
- Local Control Plane repo/module
- Mock-Cloud
- Pollen Cloud repo
```

### 12.1 Rust JSON Schema validation helper

```rust
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;

pub fn validate_schema(schema_path: &str, value: &Value) -> anyhow::Result<()> {
    let schema_json: Value = serde_json::from_str(&fs::read_to_string(schema_path)?)?;
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema_json)?;

    if let Err(errors) = compiled.validate(value) {
        let messages = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        anyhow::bail!("schema validation failed: {messages}");
    }
    Ok(())
}
```

### 12.2 Example LCP conformance test

```rust
#[tokio::test]
async fn contract_discovery_matches_schema() -> anyhow::Result<()> {
    let app = local_control_plane::router();
    let response = axum_test::TestServer::new(app)?
        .get("/.well-known/pollen-contract")
        .add_header("X-Pollen-Contract-Version", "1.0")
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();

    validate_schema(
        "contracts/schemas/contract-discovery.v1.schema.json",
        &body,
    )?;
    Ok(())
}
```

### 12.3 Mock-Cloud conformance rule

Mock-Cloud must not use a separate handwritten schema. It must import:

```text
contracts/generated/openapi/pollen.v1.yaml
contracts/generated/asyncapi/pollen-sse.v1.yaml
contracts/schemas/*.schema.json
contracts/catalog/*.yaml
```

Recommended mock server command:

```bash
npx @stoplight/prism-cli mock contracts/generated/openapi/pollen.v1.yaml --port 17892
```

If a Rust Mock-Cloud exists, add provider conformance tests like LCP.

---

## 13. Consumer Contract Tests

### 13.1 DEK consumer expectations

DEK must define tests for interactions it actually depends on:

```text
- GET /.well-known/pollen-contract
- POST /v1/tenants/{tenantId}/devices/{deviceId}/bundles/latest
- POST /v1/telemetry/batches
- SSE BundleReady event payload shape
- Contract deprecated response/header behavior
```

### 13.2 Pact-style test example outline

```rust
// crates/acceptance-tests/tests/pact_bundle_fetch.rs

#[tokio::test]
async fn dek_expects_bundle_not_modified_when_generation_current() {
    // Implementation option:
    // - Use Pact Rust when stable for this repo
    // - Or start with wiremock + JSON Schema validation and export pact files later
    // Requirement:
    // - commit generated pacts to contracts/pacts/
    // - Cloud and Mock-Cloud verify them in provider CI
}
```

### 13.3 Minimum before Pact Broker

Before using a Pact Broker, commit files to:

```text
contracts/pacts/dek-cloud-bundle-fetch.pact.json
contracts/pacts/dek-cloud-telemetry-ingest.pact.json
```

Pollen Cloud repo must copy or fetch these artifacts in CI and verify provider behavior.

---

## 14. Pollen Cloud Repo Consumption Contract

This is mandatory for the separate Pollen Cloud repository.

### 14.1 Cloud repository dependency rule

Pollen Cloud must consume one of:

```text
1. Git tag from pollen-contracts, preferred after split
2. GitHub Release artifacts from AntiG_Pollen_DEK/contracts during staging
3. Git submodule or subtree for contracts/ during early alpha
```

It must not duplicate schemas.

### 14.2 Cloud repo CI requirements

Pollen Cloud must have a workflow like this:

```yaml
name: cloud-contract-conformance

on:
  pull_request:
  push:
    branches: [main]

jobs:
  contract-conformance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Fetch Contract Hub artifacts
        run: |
          git clone --depth 1 --branch contracts-v0.1.0 https://github.com/AECInfraconnect/AntiG_Pollen_DEK /tmp/dek
          mkdir -p .contract-hub
          cp -R /tmp/dek/contracts/generated .contract-hub/
          cp -R /tmp/dek/contracts/schemas .contract-hub/
          cp -R /tmp/dek/contracts/catalog .contract-hub/
          cp -R /tmp/dek/contracts/pacts .contract-hub/ || true

      - name: Build Cloud
        run: cargo test --workspace

      - name: Verify provider conformance
        run: cargo test -p pollen-cloud-contract-tests
```

### 14.3 Cloud code must expose contract discovery

```rust
async fn cloud_contract_discovery() -> Json<ContractDiscoveryResponse> {
    Json(ContractDiscoveryResponse {
        schema_version: "contract-discovery.v1",
        supported: vec!["1.0"],
        preferred: "1.0",
        minimum_dek_version: "1.0.0-beta.1",
        sunset: Default::default(),
        capabilities: vec![
            "contract.discovery.v1",
            "bundle.signed-envelope.v1",
            "sse.bundle-ready.v1",
            "telemetry.batch.v1",
            "registry.delta-sync.v1",
        ],
    })
}
```

### 14.4 Cloud must validate inbound payload schema

Telemetry ingestion example:

```rust
async fn ingest_telemetry(Json(body): Json<serde_json::Value>) -> Result<Json<IngestResult>, ApiError> {
    validate_schema(
        ".contract-hub/schemas/telemetry-envelope.v1.schema.json",
        &body,
    )
    .map_err(|e| ApiError::contract_schema_mismatch(e.to_string()))?;

    // Continue ingestion only after schema validation.
    Ok(Json(IngestResult { accepted: 1, rejected: 0 }))
}
```

---

## 15. Release and Signing

### 15.1 Release artifacts

Every Contract Hub release must include:

```text
pollen.v1.yaml
pollen-sse.v1.yaml
schemas.tar.gz
catalog.tar.gz
pollen-contract-rust.tar.gz
pollen-contract-typescript.tgz
CHANGELOG.md
COMPATIBILITY.md
SHA256SUMS
signature/provenance artifact
```

### 15.2 `contract-release.yml`

```yaml
name: contract-release

on:
  push:
    tags:
      - 'contracts-v*'

permissions:
  contents: write
  id-token: write

jobs:
  release-contracts:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: contracts
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: npm
          cache-dependency-path: contracts/package-lock.json

      - run: npm ci
      - run: npm run test

      - name: Package artifacts
        run: |
          mkdir -p dist
          cp generated/openapi/pollen.v1.yaml dist/
          cp generated/asyncapi/pollen-sse.v1.yaml dist/
          tar -czf dist/schemas.tar.gz schemas
          tar -czf dist/catalog.tar.gz catalog
          tar -czf dist/pollen-contract-rust.tar.gz generated/rust/pollen-contract
          tar -czf dist/pollen-contract-typescript.tgz generated/typescript
          cp CHANGELOG.md COMPATIBILITY.md dist/
          cd dist && sha256sum * > SHA256SUMS

      - name: Install cosign
        uses: sigstore/cosign-installer@v3

      - name: Sign checksums with keyless signing
        run: |
          cosign sign-blob --yes --output-signature dist/SHA256SUMS.sig dist/SHA256SUMS

      - name: Create GitHub release
        uses: softprops/action-gh-release@v2
        with:
          files: contracts/dist/*
```

### 15.3 Release immutability rule

Never modify a release tag. If a contract artifact is wrong:

```text
- create new patch release
- document mistake in CHANGELOG
- mark previous release as withdrawn in COMPATIBILITY.md
```

---

## 16. Renovate / Dependabot Update Flow

### 16.1 Purpose

Every downstream repo must receive an automated PR when Contract Hub changes. This is how documentation and implementation stay synchronized.

### 16.2 Rule

When a new Contract Hub release is created:

```text
DEK repo receives PR to bump contract artifact/package.
Pollen Cloud repo receives PR to bump contract artifact/package.
Dashboard receives PR to bump @pollen/contract.
Mock-Cloud receives PR to bump contract artifact/package.
```

The PR must show:

```text
- CHANGELOG.md diff
- COMPATIBILITY.md diff
- generated SDK diff
- migration notes if required
```

### 16.3 Example `renovate.json`

```json
{
  "extends": ["config:recommended"],
  "packageRules": [
    {
      "matchPackageNames": ["@pollen/contract"],
      "groupName": "Pollen Contract Hub",
      "automerge": false,
      "labels": ["contract-update"]
    }
  ],
  "customManagers": [
    {
      "customType": "regex",
      "fileMatch": ["Cargo.toml$"],
      "matchStrings": ["pollen-contract.*tag = \"(?<currentValue>contracts-v[0-9.]+)\""],
      "depNameTemplate": "AECInfraconnect/pollen-contracts",
      "datasourceTemplate": "github-tags"
    }
  ]
}
```

---

## 17. Documentation That Must Stay Updated

### 17.1 Required docs

```text
contracts/README.md
contracts/CHANGELOG.md
contracts/COMPATIBILITY.md
contracts/docs/CLOUD_CONSUMER_GUIDE.md
contracts/docs/DEK_CONSUMER_GUIDE.md
contracts/docs/MOCK_CLOUD_CONFORMANCE.md
contracts/docs/CONTRACT_GOVERNANCE.md
contracts/docs/MIGRATION.md
contracts/docs/RELEASE_PROCESS.md
```

### 17.2 `CLOUD_CONSUMER_GUIDE.md` minimum content

```md
# Pollen Cloud Consumer Guide for Contract Hub

## Mandatory rule
Pollen Cloud must not define endpoint/schema/event contracts outside Contract Hub.

## Required artifacts
- generated/openapi/pollen.v1.yaml
- generated/asyncapi/pollen-sse.v1.yaml
- schemas/*.schema.json
- catalog/*.yaml
- pacts/*.json

## Required CI
- Build against pinned contract version
- Validate provider responses against OpenAPI/JSON Schema
- Verify Pact interactions from DEK
- Implement /.well-known/pollen-contract
- Return X-Pollen-Contract-Version on every response
- Respect Deprecation and Sunset policy
```

### 17.3 `MOCK_CLOUD_CONFORMANCE.md` minimum content

```md
# Mock-Cloud Conformance

Mock-Cloud is not allowed to drift from Contract Hub.

## CI requirements
- Start Mock-Cloud
- Run OpenAPI conformance tests
- Validate SSE payloads against AsyncAPI/JSON Schema
- Validate bundle envelope responses against bundle-envelope.v1.schema.json
- Run DEK acceptance tests against Mock-Cloud
```

### 17.4 Documentation update CI

Use the `docs-must-change` job in `contract-ci.yml`. Any PR touching:

```text
contracts/spec/
contracts/schemas/
contracts/catalog/
```

must also touch:

```text
contracts/CHANGELOG.md
or contracts/COMPATIBILITY.md
or contracts/docs/
```

---

## 18. Migration Plan from Existing DEK Repo

### Phase 0 — Inventory

AI Agent tasks:

1. Inspect current repo for:
   - `dek-control-plane-api`
   - `docs/contracts/schemas/`
   - `.spectral.yaml`
   - `.github/workflows/schema-contract.yml`
   - `local-control-plane`
   - `mock-cloud`
   - `apps/local-admin-dashboard`
2. List all existing schema/type definitions.
3. Identify duplicate DTOs between Rust and TypeScript.

Deliverable:

```text
contracts/docs/INVENTORY.md
```

### Phase 1 — Create Contract Hub staging directory

AI Agent tasks:

1. Create `contracts/` layout.
2. Move or copy existing JSON Schemas into `contracts/schemas/`.
3. Add catalog YAML files.
4. Add package.json/tspconfig.
5. Add minimal TypeSpec for discovery, bundle, telemetry, registry.
6. Generate OpenAPI.
7. Add contract CI validation only, no breaking gate yet.

Acceptance criteria:

```text
npm run test passes inside contracts/
OpenAPI and TypeScript definitions are generated
Existing CI still passes
```

### Phase 2 — Canonical envelope and schemas

AI Agent tasks:

1. Add `bundle-envelope.v1.schema.json`.
2. Add `bundle-manifest.v2.schema.json`.
3. Add `bundle-signature.v1.schema.json`.
4. Update Local Mode bundle writer to output:

```json
{
  "schema_version": "bundle-envelope.v1",
  "manifest": {},
  "signatures": []
}
```

5. Add validation before activation.

Acceptance criteria:

```text
DEK rejects invalid envelope
DEK accepts valid local envelope
Mock-Cloud returns the same envelope format
```

### Phase 3 — Rust contract crate

AI Agent tasks:

1. Create `contracts/generated/rust/pollen-contract`.
2. Generate or include Rust types from JSON Schemas.
3. Add it to workspace if appropriate.
4. Make `dek-control-plane-api` re-export generated types.
5. Mark old types deprecated.

Acceptance criteria:

```text
cargo test --workspace passes
No handwritten duplicate for migrated DTOs
```

### Phase 4 — TypeScript contract package

AI Agent tasks:

1. Generate `contracts/generated/typescript/api.d.ts`.
2. Create package `@pollen/contract`.
3. Update Local Admin Dashboard API layer to use generated types.
4. Add type-check CI.

Acceptance criteria:

```text
Dashboard cannot call a non-existent path without TypeScript error
Dashboard build passes
```

### Phase 5 — Runtime negotiation

AI Agent tasks:

1. Implement `/.well-known/pollen-contract` in LCP and Mock-Cloud.
2. Add `X-Pollen-Contract-Version` to DEK outbound requests.
3. Add response header parsing for `Deprecation` and `Sunset`.
4. Add audit events:
   - `contract.version_unsupported`
   - `contract.schema_mismatch`
   - `contract.capability_missing`
   - `contract.deprecated`

Acceptance criteria:

```text
DEK can negotiate capabilities
Unsupported contract triggers GracePeriod/LKG behavior, not silent failure
Deprecated contract is visible in telemetry/audit
```

### Phase 6 — CI gates

AI Agent tasks:

1. Add generated artifact drift check.
2. Add Spectral lint.
3. Add schema validation.
4. Add AsyncAPI validation.
5. Add oasdiff breaking check.
6. Add docs-must-change check.
7. Add semantic-contract-lint.

Acceptance criteria:

```text
Breaking OpenAPI change fails CI unless approved
Schema/catalog change without docs fails CI
Generated artifact drift fails CI
```

### Phase 7 — Mock-Cloud conformance

AI Agent tasks:

1. Ensure Mock-Cloud imports Contract Hub artifacts.
2. Add provider conformance tests.
3. Add DEK acceptance test against Mock-Cloud.
4. Add schema validation for mock responses.

Acceptance criteria:

```text
Mock-Cloud cannot return response that violates Contract Hub schema
DEK acceptance tests use Mock-Cloud generated from same contract
```

### Phase 8 — Pollen Cloud repo integration

AI Agent tasks for Cloud repo:

1. Add Contract Hub artifact fetch step.
2. Add provider conformance test suite.
3. Implement discovery endpoint.
4. Implement bundle, telemetry, registry endpoints from OpenAPI.
5. Validate inbound payload schemas.
6. Verify DEK pacts.
7. Add Renovate/Dependabot for contract updates.

Acceptance criteria:

```text
Cloud repo cannot merge endpoint/schema change unless Contract Hub changed first
Cloud implementation passes same conformance tests as Mock-Cloud/LCP
```

### Phase 9 — Release/signing

AI Agent tasks:

1. Add `contract-release.yml`.
2. Package artifacts.
3. Generate SHA256SUMS.
4. Sign checksum or artifact.
5. Publish GitHub Release.
6. Update `RELEASE_PROCESS.md`.

Acceptance criteria:

```text
Every contract release is immutable, downloadable, and verifiable
Cloud repo can pin a contract release tag
```

### Phase 10 — Split repo when ready

Do this only when:

```text
- Pollen Cloud repo exists and uses contract artifacts
- DEK repo and Cloud repo both have conformance CI
- Contract release process is stable
- CODEOWNERS and governance are accepted
```

Then move `contracts/` to `pollen-contracts` and update downstream dependencies.

---

## 19. AI Agent Development Instructions

Use this section as direct instructions for a coding AI Agent.

### 19.1 Work style

1. Do not rewrite existing DEK architecture.
2. Add `contracts/` staging directory first.
3. Migrate one schema/type at a time.
4. Keep backward compatibility with existing `dek-control-plane-api` initially.
5. Do not hand-edit generated output.
6. Every contract change must update docs/changelog/compatibility.
7. Add tests before replacing existing client/server logic.
8. Mock-Cloud and Local Control Plane must share the same Contract Hub artifacts.
9. Pollen Cloud repo must consume artifacts from Contract Hub, not copy-paste schemas.

### 19.2 First pull request scope

Recommended first PR:

```text
PR title: feat(contracts): add Contract Hub staging layout and minimal CI

Files:
- contracts/package.json
- contracts/tspconfig.yaml
- contracts/spec/main.tsp
- contracts/spec/rest/common.tsp
- contracts/spec/rest/contract-discovery.tsp
- contracts/spec/rest/bundles.tsp
- contracts/spec/rest/telemetry.tsp
- contracts/spec/events/sse.asyncapi.yaml
- contracts/schemas/bundle-envelope.v1.schema.json
- contracts/schemas/contract-discovery.v1.schema.json
- contracts/catalog/*.yaml
- contracts/docs/*.md
- contracts/CHANGELOG.md
- contracts/COMPATIBILITY.md
- .github/workflows/contract-ci.yml
```

Out of scope for first PR:

```text
- deleting legacy types
- switching all DEK clients to generated client
- full Pact Broker
- standalone repo split
```

### 19.3 Done criteria for first PR

```text
npm run test passes in contracts/
OpenAPI is generated
AsyncAPI validates
JSON Schemas validate
GitHub Action runs
Docs-must-change rule exists
No existing DEK functionality breaks
```

---

## 20. Security and Failure Handling

### 20.1 Contract mismatch is not a normal retry

If response/event/bundle fails schema validation:

```text
- do not retry endlessly
- do not activate new bundle
- keep LKG
- emit audit/security event
- report telemetry when available
```

### 20.2 Contract release signing

Release artifacts must be signed because DEK and Cloud use them as security-critical behavior definitions. At minimum:

```text
- SHA256SUMS
- signed checksum file
- immutable Git tag
```

Recommended:

```text
- keyless Sigstore/cosign signing
- GitHub artifact attestation
- provenance metadata
```

### 20.3 Runtime trust boundary

DEK must not dynamically download and execute new behavior based only on contract discovery. Discovery is for compatibility negotiation only. Policy activation must still use signed bundle envelope and existing trust rules.

---

## 21. Acceptance Criteria for Pollen Contract Hub v1

Pollen Contract Hub is considered complete for v1.0.0-beta when:

1. `contracts/` exists and generates OpenAPI, AsyncAPI, JSON Schemas, Rust types, and TS types.
2. Local Control Plane implements `/.well-known/pollen-contract`.
3. Mock-Cloud implements the same discovery and bundle/telemetry contracts.
4. DEK adds `X-Pollen-Contract-Version` on outbound Cloud/LCP requests.
5. DEK handles unsupported contract version deterministically.
6. Bundle envelope uses `{ "manifest": {...}, "signatures": [...] }` across Local Mode, Mock-Cloud, Cloud Mode, and LKG.
7. Schema mismatch rejects artifact and keeps LKG.
8. CI fails on generated artifact drift.
9. CI fails on schema/spec/catalog change without docs/changelog update.
10. CI fails on unapproved OpenAPI breaking changes.
11. Pollen Cloud repo can fetch and pin the same Contract Hub artifacts.
12. Mock-Cloud and Cloud provider tests validate responses against the same schemas.
13. `COMPATIBILITY.md` is maintained with every minor/major contract change.
14. Contract release artifacts are immutable and signed or checksum-verifiable.

---

## 22. References

Use these as technical references for implementation decisions:

- OpenAPI Specification 3.1: https://spec.openapis.org/oas/v3.1.0.html
- TypeSpec OpenAPI emitter: https://typespec.io/docs/emitters/openapi3/openapi/
- AsyncAPI Initiative: https://www.asyncapi.com/
- JSON Schema Draft 2020-12: https://json-schema.org/draft/2020-12
- Spectral API linter: https://github.com/stoplightio/spectral-action
- oasdiff OpenAPI breaking change detection: https://github.com/oasdiff/oasdiff
- Pact consumer-driven contract testing: https://docs.pact.io/
- HTTP Deprecation header RFC 9745: https://www.rfc-editor.org/info/rfc9745/
- HTTP Sunset header RFC 8594: https://www.rfc-editor.org/info/rfc8594/
- Sigstore: https://www.sigstore.dev/

