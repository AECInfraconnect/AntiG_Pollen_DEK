# Pollen Cloud ↔ DEK Integration Specification

This document provides a comprehensive specification for developing **Pollen Cloud** to connect, manage, and enforce policies on the **Data Execution Kernel (DEK)**. It outlines the core contracts, authentication mechanisms, policy lifecycles, and telemetry protocols required for seamless integration.

## 1. Authentication & Trust Model

Pollen Cloud and DEK communicate securely over the public internet using a rigorous identity and trust model.

### 1.1 mTLS & SPIFFE/SPIRE (X.509-SVID)
- **Transport**: All Cloud-DEK communication happens over HTTPS with Mutual TLS (mTLS).
- **Identity**: DEK uses `dek-spire-node` to perform node attestation. It exchanges a short-lived join token for an X.509-SVID (SPIFFE Verifiable Identity Document).
- **SPIFFE ID**: DEK identifies itself via a SPIFFE ID (e.g., `spiffe://pollen.ai/tenant/{tenant_id}/device/{device_id}`).
- **JWT-SVID**: For Application-level authentication (API layer), DEK acquires a JWT-SVID from the SPIRE agent, allowing Pollen Cloud to verify the exact workload identity cryptographically.

### 1.2 OAuth & Enrollment
- **Initial Enrollment**: DEK uses the OAuth 2.0 Device Authorization Grant flow (`dek-cli enroll`).
- DEK prompts the user with a code to enter at `https://cloud.pollen.ai/device`.
- Upon approval, DEK receives long-lived refresh tokens and initial trust bundles.

### 1.3 Trust Store & Pinned Keys
- DEK operates on a **Fail-Closed** trust model. It must possess the public keys (Trust Bundle) to verify policy signatures.
- Pollen Cloud provides rotatable trust bundles containing public keys.
- **Verification**: DEK verifies every policy bundle's signature using these keys. If the signature is invalid or the bundle is expired, DEK enforces a strict-deny.

---

## 2. API Endpoints & Contract Enforcements

All API calls between DEK and Pollen Cloud must adhere to the shared contract defined in `dek-control-plane-api`. 

### 2.1 Enforced HTTP Headers
Pollen Cloud must expect and validate the following headers on every request from DEK:
- `Authorization: Bearer <jwt-svid or oauth-token>`
- `Content-Type: application/json`
- `X-Pollen-Device-Id: <device_id>`
- `X-Pollen-Tenant-Id: <tenant_id>`

### 2.2 Core Endpoint Groups
Pollen Cloud must implement the following REST/SSE endpoints (matching the Local Control Plane):
1. **Bundle APIs** (`/v1/tenants/{tenant_id}/devices/{device_id}/bundles/...`): For DEK to fetch TUF-lite metadata and signed policy artifacts.
2. **Telemetry APIs** (`/v1/telemetry/...`): To ingest decision logs and metrics from DEK.
3. **Registry APIs** (`/v1/tenants/{tenant_id}/registry/...`): To sync entities, AI agents, and MCP servers.

---

## 3. Policy & Hot Reload Lifecycle

The PDP (Policy Decision Point) inside DEK must be updated without downtime. Pollen Cloud must support the **Hot Reload** mechanism.

### 3.1 Bundle Format (PollenPolicyBundleManifestV2)
Policies are packaged into "Bundles". A bundle contains:
- `build_number`: Incremental versioning.
- `artifacts`: Compiled policies for various engines (Cedar, OPA, OpenFGA).
- `signatures`: Cryptographic signatures of the bundle.
- `activation_strategy`: Defines how DEK applies the update (e.g., `AtomicAllOrNothing`).

### 3.2 Server-Sent Events (SSE) Push
- DEK connects to Pollen Cloud via an SSE stream to listen for bundle updates.
- When an admin publishes a new policy on Pollen Cloud, the Cloud emits a `BundleReady` event over SSE to the specific `tenant_id` / `device_id`.
- DEK receives the event, pulls the new bundle, verifies the signature, and atomically hot-reloads the PDP engines (`dek-activation` crate).

### 3.3 Enforcement States (Fail-Closed)
Pollen Cloud must be aware of DEK's state machine:
- **Active**: DEK has a valid, fresh, signed bundle.
- **GracePeriod**: DEK cannot reach the Cloud but the bundle is still within the `max_bundle_age` threshold. It falls back to Last-Known-Good (LKG).
- **StrictDeny**: Bundle is expired, signature is invalid, or identity SVID is revoked. DEK blocks all requests.

---

## 4. Entity Registry Syncing

Pollen Cloud acts as the central directory for AI Agents, MCP Servers, Tools, and Resources.

### 4.1 Schema Validation
Entities sent from Pollen Cloud to DEK must adhere to JSON Schemas (e.g., `ai-agent.schema.json`, `mcp-server.schema.json`).

### 4.2 Registration Flow
- **Cloud-to-DEK**: When an entity is registered on Pollen Cloud, its snapshot is included in the next Policy Bundle update (`registry_snapshot_sha256`).
- **Status Enum**: Entities have a `RegistryStatus` (e.g., `Active`, `Suspended`, `Published`). DEK only loads `Active` or `Published` entities into its router.

---

## 5. Telemetry & Observability

DEK enforces policies and queues the results in `dek-secure-spool` (disk-backed queue) before shipping them to Pollen Cloud.

### 5.1 Telemetry Event Envelope
All events use the `TelemetryEventEnvelope` schema. Key fields:
- `event_type`: Identifies the payload (e.g., `DecisionLog`, `SecurityEvent`, `RuntimeMetric`).
- `trace_id` & `span_id`: Distributed tracing linking the AI agent's request to the PEP decision.
- `redaction_applied`: Boolean indicating if `dek-pii-wasm` redacted sensitive data.

### 5.2 Decision Results (`DecisionLog`)
When DEK evaluates a request, it sends a `DecisionResult` payload containing:
- `decision`: The outcome (`Allow`, `Deny`, `Redact`, `RequireApproval`, etc.).
- `matched_policy_ids`: Which specific rules were triggered.
- `adapter_results`: The breakdown from specific engines (e.g., Cedar allowed, but OpenFGA denied).
- `obligations`: Actions the client must take (e.g., Step-Up MFA).

### 5.3 Batching & Ingestion
Pollen Cloud must implement high-throughput ingestion endpoints (e.g., Kafka/Kinesis backed) for the `/v1/telemetry/*` routes, as thousands of DEKs will batch-send telemetry simultaneously.

---

## 6. PDP Engines & OS Guardrails

Pollen Cloud needs to compile policies into the correct IR (Intermediate Representation) or native text for the engines inside DEK.

- **User-mode Engines**: Cedar (ABAC), OPA/Rego, OpenFGA. Pollen Cloud must compile the user's high-level intent into these formats and attach them as `BundleArtifactV2`.
- **Kernel Guardrails (eBPF / WFP)**: Pollen Cloud must ensure that only "simple" rules (e.g., exact IP/Port blocks) are targeted for OS-level enforcement. Complex conditional logic must remain in user-mode engines to prevent kernel verifier rejections.
