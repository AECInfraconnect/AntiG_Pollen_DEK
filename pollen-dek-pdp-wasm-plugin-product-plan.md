# Pollen DEK PDP, WASM, Flow, and Plugin Product Implementation Plan

Date: 2026-06-23

Repository reviewed: `AECInfraconnect/AntiG_Pollen_DEK`

Purpose: provide an implementation-ready product and engineering plan for improving PDP runtimes, WASM policy files, decision flows, and plugin architecture so Pollen DEK can be used in real local-user and enterprise environments.

## 1. Executive Recommendation

Pollen DEK should position itself as a **Local AI Agent Governance Runtime**:

- **Observe** every agent/tool/resource event where possible.
- **Control** agent registration, PDP routing, policy deployment, and plugin permissions from the Local Dashboard.
- **Enforce** only through PEP types that can actually intercept a given action on the user's machine.
- **Explain** every decision with policy, PDP, PEP, latency, fallback, and telemetry status.
- **Sync** signed policy bundles and redacted telemetry to Pollen Cloud when enabled.

The PDP design should not be a single engine choice. It should be a routed runtime layer:

1. **Local PDP first** for low latency and offline use.
2. **WASM PDP for portable policy execution** where OPA/Rego or custom policy plugins are needed.
3. **Cedar local PDP** for application/resource authorization and schema-driven policies.
4. **OpenFGA/SpiceDB-style remote PDP** for relationship-based authorization.
5. **Pollen Cloud PDP** for managed policies, shared enterprise rules, cross-device governance, and compliance reporting.

## 2. User Segments and Required Flows

| User segment | Primary job | Required PDP/PEP behavior | Product priority |
|---|---|---|---:|
| Individual developer | See what local AI agents are doing and block risky tools | Local PDP, MCP/stdio wrapper PEP, file/network guard where available | P0 |
| Power user / local-first user | Run agents offline with personal privacy | OPA WASM or Cedar local, local-only telemetry, no cloud dependency | P0 |
| Enterprise security admin | Deploy policy presets to employee devices | Signed bundles, cloud-managed policy, local enforcement, audit logs | P0 |
| Compliance / regulated team | Prove policy enforcement and data protection | Tamper-evident audit, redaction, signed telemetry, policy version history | P1 |
| AI agent vendor | Integrate Pollen controls into their agent | Plugin SDK, PEP SDK, decision API, test harness | P1 |
| Platform / MSP | Manage many tenants/devices | Cloud PDP, routing profiles, fleet health, policy rollout/rollback | P1 |
| Research / plugin developer | Build custom policy detectors and resource classifiers | WASM plugin sandbox, capability manifest, test fixtures | P2 |

## 3. Current Repo Assessment

Based on the latest repository structure and previously inspected crates:

### 3.1 Existing Strengths

- Local Control Plane already has routes for PDP runtime, routing, cloud PDP, policy presets, discovery, telemetry, and bundles.
- OPA WASM adapter exists and starts in the right direction: default deny, policy loading, fuel/resource constraints, and capability-style metadata.
- Cedar adapter exists and supports authorization decisions, cache, and default-deny error behavior.
- OpenFGA adapter exists as a remote relationship PDP with connection tests and cache.
- Policy router concepts already exist: route ordering, failover, circuit breaker, fallback behavior, shadowing.
- Secure telemetry and spooler concepts exist.
- Plugin-related crates exist in the repo landscape, which is a good base for a real plugin SDK.

### 3.2 Product Gaps To Fix

| Area | Gap | User impact | Priority |
|---|---|---:|---:|
| PDP runtime UX | Local Engines tab can appear empty or simulated | Users cannot trust whether local PDP is real | P0 |
| OPA WASM lifecycle | Need signed WASM artifact manifest, ABI validation, bundle metadata, warm instance/cache | Slow or unsafe policy activation | P0 |
| Cedar lifecycle | Schema must be real, versioned, validated, and mapped to Pollen resources | Cedar decisions may not match real resources | P0 |
| OpenFGA lifecycle | Needs store/model/tuple bootstrap wizard and relationship sync | Hard to configure correctly | P1 |
| Pollen Cloud PDP | Needs login-based auto-discovery plus manual override for enterprise networks | Confusing setup | P1 |
| Plugin system | Needs manifest, permissions, sandbox, signing, versioning, and telemetry hooks | Unsafe extension surface | P0 |
| PEP selection | Policy presets must deploy only to PEPs that can enforce them | False sense of security | P0 |
| Decision explanations | Need consistent decision trace across PDPs | Hard to debug and audit | P0 |
| Telemetry mapping | Need event schema linking agent, tool, resource, PDP, PEP, policy, decision | Incomplete audit | P0 |

## 4. Comparable Software and Lessons

| System | Lesson for Pollen DEK |
|---|---|
| OPA | Keep Rego-to-WASM as a portable local PDP option. Use bundles, discovery-style config, signed policy artifacts, and decision logs. |
| Cedar / AWS Verified Permissions | Strong fit for schema-driven application/resource permissions. Require schema validation before policy activation. |
| OpenFGA / Zanzibar-style systems | Best fit for relationship-based access control. Use for user-resource-tool relationship graphs, not as the only PDP. |
| Envoy ext_authz | Good model for PEP/PDP separation: PEP asks PDP for allow/deny with context and gets structured response. |
| Cerbos / Topaz / Permify / SpiceDB | Product lesson: developers need a local PDP that is easy to run, inspect, test, and promote to cloud/fleet mode. |
| Wasmtime | Use fuel, epoch interruption, memory limits, and capability-based WASI access for plugin isolation. |
| OpenTelemetry | Use consistent event schemas and processors for redaction/filtering before export. |

## 5. Target Product Model

### 5.1 PDP Runtime Types

```ts
type PdpRuntimeKind =
  | "opa_wasm"
  | "cedar_local"
  | "openfga_remote"
  | "spicedb_remote"
  | "pollen_cloud"
  | "plugin_wasm";

type PdpRuntimeCategory =
  | "local_engine"
  | "remote_connector"
  | "cloud_managed"
  | "plugin";

interface PdpRuntime {
  id: string;
  name: string;
  kind: PdpRuntimeKind;
  category: PdpRuntimeCategory;
  status: "ready" | "degraded" | "unreachable" | "misconfigured" | "disabled";
  capabilities: string[];
  policyLanguages: Array<"rego" | "cedar" | "openfga" | "plugin_abi_v1">;
  endpoint?: string;
  artifactRef?: string;
  schemaRef?: string;
  modelRef?: string;
  version: string;
  lastHealthAt?: string;
  lastError?: string;
}
```

### 5.2 PEP Types

```ts
type PepType =
  | "mcp_proxy"
  | "stdio_wrapper"
  | "http_gateway"
  | "browser_extension"
  | "fs_guard"
  | "os_network"
  | "ide_extension"
  | "agent_sdk"
  | "audit_only";

interface PepCapability {
  pepType: PepType;
  os: "windows" | "macos" | "linux" | "any";
  canObserve: boolean;
  canEnforce: boolean;
  resources: string[];
  tools: string[];
  requiredPrivileges: "none" | "user" | "admin" | "kernel";
  compatiblePdpKinds: PdpRuntimeKind[];
}
```

Key rule: if a PEP cannot actually intercept a resource, the UI must show **Observe only** or **Not enforceable**, not "Protected".

## 6. Recommended User Flows

### 6.1 First-Run Flow

1. Detect OS, user privileges, installed AI agents, MCP servers, IDEs, browsers, and common agent config files.
2. Show discovered agents with confidence score and enforceability:
   - Observe only
   - Enforce via MCP proxy
   - Enforce via stdio wrapper
   - Enforce via browser extension
   - Enforce via OS PEP
3. Seed local PDP runtimes:
   - `local-opa-wasm`
   - `local-cedar`
4. Ask user to choose profile:
   - Personal privacy
   - Developer guardrails
   - Enterprise managed
   - Compliance strict
5. Deploy signed baseline bundle.
6. Start telemetry collection locally.
7. Offer Pollen Cloud login for sync, fleet policy, and cloud PDP.

### 6.2 Policy Deployment Wizard

Wizard steps:

1. **Choose goal**
   - Content guard
   - PII protection
   - File/folder protection
   - Email/drive protection
   - Internet/domain guard
   - Tool approval
   - Token/cost budget
   - Shadow AI block
2. **Choose targets**
   - Agents
   - Tools
   - Resources
   - File paths
   - Domains
   - Email/drive accounts
3. **Choose enforcement surface**
   - Recommended PEP
   - Available alternatives
   - Observe-only warning if no enforceable PEP exists
4. **Choose PDP**
   - Recommended local PDP
   - Remote connector
   - Pollen Cloud PDP
   - Shadow/fallback PDP
5. **Preview generated policy**
   - Rego/Cedar/OpenFGA/plugin config
   - Decision examples
6. **Test with simulator**
   - Allow case
   - Deny case
   - Redaction case
   - Fallback case
7. **Deploy signed bundle**
8. **Observe live decisions**

### 6.3 PDP Routing Flow

Default route recommendation:

```text
1. local-opa-wasm      -> content guard, tool guard, cost/token guard
2. local-cedar         -> resource authorization, file/folder/email/drive policy
3. openfga-remote      -> relationship checks if configured
4. pollen-cloud        -> enterprise managed fallback or fleet policy
5. deny                -> fail closed for enforce mode
```

Routing decision must include:

- matched route ID
- primary PDP
- fallback PDP used or not
- PDP latency
- policy bundle version
- PEP type
- enforcement mode
- final decision
- explain reason

## 7. PDP Runtime Improvements

### 7.1 OPA WASM Runtime

OPA WASM should be treated as a first-class local PDP for:

- content guard policies
- tool allow/deny
- PII rule evaluation
- cost/token budget checks
- HTTP/domain allowlist
- MCP tool permission checks

Required improvements:

1. Compile Rego to WASM in build/deploy pipeline, not at decision time.
2. Store WASM with a signed artifact manifest.
3. Validate policy ABI before activation.
4. Cache prepared instances or use a warm instance pool.
5. Limit fuel, memory, input size, and evaluation timeout.
6. Record per-policy decision latency and errors.
7. Support external data documents in bundle.
8. Make builtin support explicit; unsupported builtins fail deployment, not runtime.

Artifact manifest:

```json
{
  "schema_version": "pollen.wasm.policy.v1",
  "artifact_id": "policy.content_guard.v1",
  "pdp_kind": "opa_wasm",
  "language": "rego",
  "entrypoint": "pollen/allow",
  "wasm_sha256": "hex...",
  "data_sha256": "hex...",
  "abi": "opa-wasm-abi-v1",
  "builtins": ["glob.match", "regex.match"],
  "limits": {
    "max_input_bytes": 262144,
    "max_memory_bytes": 16777216,
    "max_fuel": 5000000,
    "timeout_ms": 25
  },
  "issued_at": "2026-06-23T00:00:00Z",
  "expires_at": "2026-12-23T00:00:00Z",
  "signature": "base64..."
}
```

Rust activation example:

```rust
pub struct WasmPolicyManifest {
    pub artifact_id: String,
    pub wasm_sha256: String,
    pub abi: String,
    pub entrypoint: String,
    pub limits: WasmLimits,
    pub signature: String,
}

pub fn validate_wasm_policy_artifact(
    manifest: &WasmPolicyManifest,
    wasm_bytes: &[u8],
    trusted_key: &[u8],
) -> anyhow::Result<()> {
    if manifest.abi != "opa-wasm-abi-v1" {
        anyhow::bail!("unsupported wasm policy ABI: {}", manifest.abi);
    }

    let actual = sha256_hex(wasm_bytes);
    if actual != manifest.wasm_sha256 {
        anyhow::bail!("wasm hash mismatch");
    }

    verify_manifest_signature(manifest, trusted_key)?;
    Ok(())
}
```

Wasmtime runtime limits:

```rust
pub struct WasmEvalLimits {
    pub fuel: u64,
    pub timeout_ms: u64,
    pub max_input_bytes: usize,
}

pub async fn evaluate_wasm_policy(
    pool: &WasmInstancePool,
    input: serde_json::Value,
    limits: WasmEvalLimits,
) -> anyhow::Result<Decision> {
    let input_bytes = serde_json::to_vec(&input)?;
    if input_bytes.len() > limits.max_input_bytes {
        anyhow::bail!("policy input too large");
    }

    let eval = async {
        let mut instance = pool.checkout().await?;
        instance.add_fuel(limits.fuel)?;
        instance.evaluate_json(&input_bytes).await
    };

    tokio::time::timeout(
        std::time::Duration::from_millis(limits.timeout_ms),
        eval,
    )
    .await
    .map_err(|_| anyhow::anyhow!("wasm policy evaluation timed out"))?
}
```

### 7.2 Cedar Local Runtime

Cedar should be used for:

- resource authorization
- file/folder/email/drive access rules
- user/agent/tool/resource relationship in local context
- policies where schema validation is important

Required improvements:

1. Replace mock schema with generated Pollen resource schema.
2. Store schema version with every policy bundle.
3. Validate policies before activation.
4. Normalize Pollen event context into Cedar entities.
5. Cache schema and entity templates, not raw decisions only.
6. Explain deny reasons consistently.

Cedar input mapping:

```rust
pub struct PollenAuthzEvent {
    pub principal_agent_id: String,
    pub principal_user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub resource_path: Option<String>,
    pub context: serde_json::Value,
}

pub fn to_cedar_request(event: &PollenAuthzEvent) -> cedar_policy::Request {
    let principal = cedar_policy::EntityUid::from_str(
        &format!("Agent::\"{}\"", event.principal_agent_id)
    ).expect("validated agent id");

    let action = cedar_policy::EntityUid::from_str(
        &format!("Action::\"{}\"", event.action)
    ).expect("validated action");

    let resource = cedar_policy::EntityUid::from_str(
        &format!("{}::\"{}\"", event.resource_type, event.resource_id)
    ).expect("validated resource");

    cedar_policy::Request::new(
        Some(principal),
        Some(action),
        Some(resource),
        cedar_policy::Context::empty(),
        None,
    ).expect("validated cedar request")
}
```

Example Cedar policy for folder protection:

```cedar
forbid (
  principal,
  action in [Action::"file.read", Action::"file.write"],
  resource is Folder
)
when {
  resource.sensitivity == "private" &&
  !(principal in resource.allowed_agents)
};
```

### 7.3 OpenFGA Remote Runtime

OpenFGA should be used only when relationship graphs matter:

- user owns project
- agent belongs to workspace
- tool can access resource because user granted it
- team has role on drive/email/data source

Required wizard:

1. Endpoint and auth token.
2. Store selection or creation.
3. Authorization model import.
4. Tuple bootstrap.
5. Check API test.
6. Consistency and timeout setting.
7. Local fallback behavior.

Example model concept:

```dsl
model
  schema 1.1

type user

type agent
  relations
    define owner: [user]
    define operator: [user]

type resource
  relations
    define owner: [user]
    define viewer: [user, agent]
    define editor: [user, agent]
    define can_read: viewer or editor or owner
    define can_write: editor or owner
```

Decision request:

```rust
pub struct RelationshipCheck {
    pub user: String,
    pub relation: String,
    pub object: String,
}

pub async fn check_openfga(
    client: &OpenFgaClient,
    check: RelationshipCheck,
) -> anyhow::Result<bool> {
    let response = tokio::time::timeout(
        std::time::Duration::from_millis(150),
        client.check(check.user, check.relation, check.object),
    )
    .await
    .map_err(|_| anyhow::anyhow!("openfga check timeout"))??;

    Ok(response.allowed)
}
```

### 7.4 Pollen Cloud PDP

Pollen Cloud PDP should not require manual endpoint entry for normal users after login.

Recommended behavior:

- If user is logged in: auto-discover cloud PDP endpoints from tenant contract.
- If enterprise network/proxy requires override: allow advanced manual endpoint.
- If offline: local PDP remains active.
- Cloud PDP can be primary only if user explicitly enables managed mode.
- Cloud PDP should usually run as:
  - fleet policy source
  - shadow evaluator
  - fallback PDP for complex enterprise rules
  - telemetry/compliance aggregator

Cloud PDP tab fields:

| Field | Default |
|---|---|
| Login status | Auto |
| Tenant | Auto |
| PDP endpoint | Auto-discovered |
| Manual override | Advanced only |
| Mode | Disabled / Shadow / Fallback / Primary |
| Offline behavior | Use local PDP / deny / allow audit-only |
| Last sync | Auto |
| Policy bundle version | Auto |

## 8. Plugin Architecture

### 8.1 Plugin Types

```ts
type PluginType =
  | "detector"
  | "policy_evaluator"
  | "resource_classifier"
  | "telemetry_processor"
  | "pep_adapter"
  | "pdp_connector";
```

Examples:

- detector: detect Claude Desktop, Cursor, VS Code agents, MCP servers.
- policy_evaluator: custom WASM PDP.
- resource_classifier: classify file/email/drive sensitivity.
- telemetry_processor: redact and enrich telemetry.
- pep_adapter: connect to a specific agent SDK.
- pdp_connector: connect to a third-party PDP.

### 8.2 Plugin Manifest

```json
{
  "schema_version": "pollen.plugin.v1",
  "id": "com.pollen.plugins.github-resource-classifier",
  "name": "GitHub Resource Classifier",
  "version": "0.1.0",
  "type": "resource_classifier",
  "runtime": "wasm32-wasi-preview2",
  "entrypoint": "plugin.wasm",
  "permissions": [
    "telemetry.read.redacted",
    "resource.classify",
    "http.fetch:api.github.com"
  ],
  "limits": {
    "memory_bytes": 33554432,
    "fuel": 10000000,
    "timeout_ms": 100,
    "max_output_bytes": 65536
  },
  "signing": {
    "algorithm": "ed25519",
    "signature": "base64..."
  }
}
```

### 8.3 Plugin Permission Model

Plugins must be denied by default. Grant only declared and approved capabilities.

Permission examples:

```text
telemetry.read.redacted
telemetry.write.enriched
resource.classify
decision.explain
pdp.evaluate
http.fetch:<host>
filesystem.read:<path>
agent.detect
pep.register
```

### 8.4 Plugin Host Flow

```text
install plugin
  -> verify manifest signature
  -> validate permissions against policy
  -> compile/cache WASM
  -> run capability self-test
  -> enable plugin
  -> emit plugin.lifecycle telemetry
```

Rust trait for plugin host:

```rust
#[async_trait::async_trait]
pub trait PollenPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn permissions(&self) -> &[PluginPermission];

    async fn call(
        &self,
        operation: &str,
        input: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value>;
}
```

Plugin invocation guard:

```rust
pub async fn call_plugin_checked(
    plugin: &dyn PollenPlugin,
    operation: &str,
    input: serde_json::Value,
    policy: &PluginPolicy,
) -> anyhow::Result<serde_json::Value> {
    policy.ensure_allowed(plugin.id(), operation, plugin.permissions())?;

    let result = tokio::time::timeout(
        std::time::Duration::from_millis(policy.timeout_ms(plugin.id())),
        plugin.call(operation, input),
    )
    .await
    .map_err(|_| anyhow::anyhow!("plugin call timeout"))??;

    policy.validate_output(plugin.id(), operation, &result)?;
    Ok(result)
}
```

## 9. Unified Decision Envelope

Every PDP should return the same decision envelope.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionEnvelope {
    pub decision_id: String,
    pub allowed: bool,
    pub mode: EnforcementMode,
    pub reason: String,
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub pep_type: String,
    pub pdp_runtime_id: String,
    pub route_id: String,
    pub policy_bundle_id: String,
    pub policy_version: String,
    pub latency_ms: u64,
    pub fallback_used: bool,
    pub obligations: Vec<DecisionObligation>,
    pub redactions: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DecisionObligation {
    RedactField { path: String },
    RequireApproval { approver_group: String },
    LogOnly,
    MaskOutput,
    LimitTokens { max_tokens: u64 },
    BlockNetwork { host: String },
}
```

## 10. WASM File and Bundle Layout

Recommended local bundle:

```text
bundle/
  manifest.json
  signature.sig
  opa/
    content_guard.wasm
    cost_budget.wasm
    data.json
  cedar/
    schema.cedarschema
    policies.cedar
    entities.json
  openfga/
    model.fga
    seed-tuples.json
  plugins/
    resource_classifier/
      manifest.json
      plugin.wasm
  routing/
    routes.json
  examples/
    allow.json
    deny.json
```

Bundle manifest:

```json
{
  "schema_version": "pollen.bundle.v1",
  "bundle_id": "baseline-personal-privacy",
  "version": "1.0.0",
  "tenant": "local",
  "created_at": "2026-06-23T00:00:00Z",
  "expires_at": "2026-12-23T00:00:00Z",
  "artifacts": [
    {
      "id": "opa.content_guard",
      "kind": "opa_wasm",
      "path": "opa/content_guard.wasm",
      "sha256": "hex..."
    },
    {
      "id": "cedar.file_guard",
      "kind": "cedar",
      "path": "cedar/policies.cedar",
      "sha256": "hex..."
    }
  ],
  "routes": "routing/routes.json",
  "signature": "base64..."
}
```

## 11. Dashboard Improvements

### 11.1 Local Engines Tab

Show real runtime status:

| Column | Description |
|---|---|
| Engine | OPA WASM, Cedar Local |
| Status | Ready / Not installed / Misconfigured |
| Policies loaded | Count |
| Bundle version | Active version |
| p95 latency | Local decision latency |
| Last error | Safe error message |
| Actions | Probe, View policies, Run test |

No fake "reachable" state. Local engines should show "Ready" only when:

- runtime initialized
- schema/policy loaded
- test decision succeeds
- active bundle signature verified

### 11.2 Remote Connectors Tab

For OpenFGA/SpiceDB/OPA Server/Cerbos:

- endpoint
- auth method
- health endpoint
- decision endpoint
- timeout
- consistency mode
- fallback behavior
- test check
- discovered capabilities

### 11.3 Pollen Cloud PDP Tab

Main user path:

```text
Login -> Auto-discover tenant PDP -> Show endpoint -> Enable Shadow/Fallback/Primary -> Sync bundle
```

Advanced path:

```text
Manual endpoint override -> test TLS/auth -> save as enterprise override
```

### 11.4 Routing and Failover Tab

Required form fields:

| Field | Type |
|---|---|
| Name | text |
| Priority | number |
| Match agents | multi-select |
| Match actions | multi-select |
| Match resource types | multi-select |
| Match sensitivity | select |
| Primary PDP | select |
| Fallback PDP | select |
| Mode | enforce / shadow / audit |
| Failure behavior | deny / allow / fallback / use-cache |
| Timeout | ms |
| Shadow PDP | optional |

Route example:

```json
{
  "id": "route-content-guard",
  "priority": 10,
  "match": {
    "actions": ["llm.prompt", "tool.call"],
    "resource_types": ["prompt", "tool"]
  },
  "primary_pdp": "local-opa-wasm",
  "fallback_pdp": "pollen-cloud",
  "mode": "enforce",
  "failure_behavior": "deny",
  "timeout_ms": 25,
  "shadow_pdp": "pollen-cloud"
}
```

## 12. API Additions

```http
GET  /v1/tenants/:tenant/pdp/runtimes
POST /v1/tenants/:tenant/pdp/runtimes/:id/probe
POST /v1/tenants/:tenant/pdp/runtimes/:id/load-bundle
POST /v1/tenants/:tenant/pdp/evaluate

GET  /v1/tenants/:tenant/plugins
POST /v1/tenants/:tenant/plugins/install
POST /v1/tenants/:tenant/plugins/:id/enable
POST /v1/tenants/:tenant/plugins/:id/disable
POST /v1/tenants/:tenant/plugins/:id/test

POST /v1/tenants/:tenant/policies/deploy/preview
POST /v1/tenants/:tenant/policies/deploy/commit
POST /v1/tenants/:tenant/policies/deploy/rollback
```

Evaluation API:

```json
{
  "principal": "agent:claude-desktop",
  "action": "file.read",
  "resource": "file:/Users/alice/private/tax.pdf",
  "context": {
    "pep_type": "fs_guard",
    "user": "alice",
    "process": "Claude Desktop",
    "sensitivity": "private"
  }
}
```

## 13. Implementation Roadmap

### P0 - Make Local PDP and Policy Deployment Real

- OPA WASM artifact manifest, signature verification, ABI check.
- Cedar schema generation and validation.
- Unified decision envelope.
- Real Local Engines tab status and probe.
- Policy Deployment Wizard compatibility gate.
- Bundle activation with atomic write and rollback.
- Plugin manifest/signature/permission validation.

### P1 - Make Enterprise and Remote PDP Useful

- Pollen Cloud PDP auto-discovery after login.
- OpenFGA wizard with model/store/tuple bootstrap.
- PDP routing simulator and failover testing.
- Shadow PDP comparison reports.
- Decision explanation UI.
- Fleet-ready policy bundle versioning.

### P2 - Expand Plugin Ecosystem

- WASM plugin SDK.
- Example plugins:
  - local file sensitivity classifier
  - GitHub resource classifier
  - Slack/email redactor
  - custom cost budget evaluator
- Plugin marketplace metadata.
- Plugin test harness and sandbox benchmarks.

## 14. Acceptance Criteria

The feature is ready when:

- Local Engines tab shows real OPA WASM and Cedar local status.
- A policy preset can compile to a signed bundle and deploy to a compatible PEP/PDP pair.
- OPA WASM policies are hash-verified, ABI-checked, and resource-limited before activation.
- Cedar policies are schema-validated before activation.
- OpenFGA connector can create/test a real relationship check.
- Pollen Cloud PDP is auto-discovered after login, with advanced manual override.
- Routing simulator shows primary, fallback, shadow, timeout, and final decision.
- Plugins cannot run unless signed, permission-approved, resource-limited, and tested.
- Every decision creates a decision envelope and telemetry event.
- UI clearly distinguishes Observe, Enforce, Shadow, and Not Enforceable.

## 15. References For AI Agents

Use the official documentation for:

- Open Policy Agent WASM, bundles, discovery, and decision logs.
- Cedar policy language, schema, entities, and validation.
- OpenFGA authorization model, stores, tuples, and Check API.
- Envoy external authorization pattern.
- Wasmtime fuel, epoch interruption, memory limits, and WASI component model.
- OpenTelemetry collector processors and telemetry pipeline design.
- Comparable products: Cerbos, SpiceDB/Authzed, Permify, Aserto Topaz.
