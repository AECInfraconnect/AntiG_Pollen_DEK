# Pollen DEK Local Policy Deployment Wizard: Deep Research and Implementation Plan

Version: 2026-06-22  
Target repo: `AECInfraconnect/AntiG_Pollen_DEK`  
Observed repo version from `Cargo.toml`: `1.0.0-beta.10`  
Scope: Local-first policy deployment, AI-agent discovery, PEP/PDP selection, enforcement, control, observe, telemetry to Local Dashboard and Pollen Cloud.

---

## 1. Objective

Pollen DEK should let a local user do the full governance loop on their own computer:

```text
Discover AI agents -> understand tools/resources/capabilities -> choose policies ->
select feasible PEP/PDP -> preview changes -> deploy -> enforce/control/observe ->
log tool/resource access -> send telemetry to Local Dashboard and Pollen Cloud
```

The system must work even when the user's Windows/Linux/macOS machine has no preinstalled sidecar, no proxy, and no OS-level driver. In that case Pollen DEK must degrade honestly:

- Use config-based wrapping/proxying when possible.
- Use local process/network/file observation when possible.
- Use manual instructions when automation cannot safely edit a config.
- Use `ObserveOnly` when no enforceable PEP exists.
- Never label stub or observe-only mode as enforce.

---

## 2. Repo Findings

### 2.1 Strong foundations already present

The repo already has the right major building blocks:

- `local-control-plane` exposes registry, discovery, policy, bundle, telemetry, PDP runtime, PDP routing, PEP capabilities, and policy preset APIs.
- `dek-agent-discovery` has source modules for process scan, MCP config scan, local model probe, IDE extension scan, CLI scan, container scan, browser scan, web AI scan, and aggregation.
- `DiscoveredAgentCandidateV2` already includes discovered configs, endpoints, MCP servers, suggested control bindings, and a telemetry plan.
- `PolicyRouter` already supports routes, local/remote PDP modes, failover, circuit breakers, dry-run authorization, and evaluator auto-selection.
- `bundle.rs` can build signed local policy bundles and expose local bundle endpoints.
- `telemetry.rs` accepts local telemetry on the same split endpoints as Pollen Cloud.
- `PolicyPresets`, `PresetWizard`, `preset_deploy_api`, and `pep_capabilities_api` exist, so the correct product surface is already emerging.

### 2.2 Critical gaps to fix before production

1. `apply_control_binding` and `rollback_control_binding` are stubs.
2. `PresetWizard` hardcodes `target_os: "linux"` and does not inspect the real device capability.
3. `pep_capabilities_api` returns optimistic/static capabilities such as available `mcp_proxy`/`stdio_wrapper` without proving a live PEP process or usable config path.
4. `preset_deploy_api` maps artifact language to PEP-like IDs such as `opa_rego`, `aws_cedar`, `openfga`; those are PDP/runtime kinds, not PEP types.
5. `deploy_to_pep` writes `active_bundle.json`, but does not guarantee live PEP reload, health probe, or rollback.
6. `policy_presets` has v1/v2 model drift: `validate.rs` refers to v2 types while `catalog.rs` and `render.rs` still expose v1-style presets in the fetched source.
7. Preset simulation supports Cedar more concretely than Rego/OpenFGA. Rego/OpenFGA simulation still needs real local/remote runtime integration.
8. Discovery stores evidence, but does not yet persist a first-class capability inventory that deployment can rely on.
9. Telemetry stores decision logs but lacks typed, queryable logs for tool invocation, resource access, PEP binding status, and policy deployment audit.

Primary architectural fix:

> Add a durable `Agent Capability Inventory` and make both Auto Discovery and Policy Deployment Wizard consume it.

---

## 3. External Technical Constraints

### 3.1 MCP

MCP tools are model-invoked operations against external systems. The MCP specification recommends human-visible tool exposure, visible invocation indicators, and confirmation prompts for operations where human approval matters. MCP resources expose data such as files, database schemas, and application-specific information to clients. MCP standard transports include stdio and Streamable HTTP.

Pollen implication:

- MCP is the highest-value local enforcement point because many AI agents already route tool/resource access through MCP.
- `mcp_proxy` and `stdio_wrapper` should be first-class PEPs.
- For high-risk tool calls, the PEP must support approval workflows and visible audit.

### 3.2 OPA/Rego

OPA can compile Rego into Wasm modules with `opa build -t wasm` and a required entrypoint. OPA Wasm is a compiled policy evaluation path, not an OPA server running inside Wasm.

Pollen implication:

- `local.opa_wasm` should mean real OPA-Wasm ABI support with bundle data and entrypoint evaluation.
- If the current implementation is only generic WASI/plugin execution, rename it `wasm_plugin` or implement the OPA-Wasm ABI properly.

### 3.3 Cedar

Cedar uses schema for validation at policy creation/update time. Cedar evaluation itself does not use schema, so Pollen must validate policies against schema before publish.

Pollen implication:

- Cedar is the right local PDP for ABAC/RBAC-style authorization over agent, tool, resource, context, approval tickets, and trust level.
- Policy Deployment Wizard must show schema validation errors before deploy.

### 3.4 OpenFGA

OpenFGA is a relationship-based authorization engine with check APIs and a `/healthz` endpoint for server health.

Pollen implication:

- OpenFGA is a remote/local server PDP, not a built-in local engine unless Pollen runs a managed sidecar.
- Use OpenFGA for owner/team/project/folder delegation, not content scanning or token budgets.

### 3.5 OS-level PEPs

- Linux: eBPF/libbpf supports program types such as cgroup ingress/egress/connect hooks, useful for network-level observation/enforcement.
- Windows: WFP callout drivers can inspect, block, modify, and log TCP/IP traffic, but require driver/service installation and permissions.
- macOS: Network Extension content filters can pass/block flows, but require entitlement, system extension packaging, and user/admin approval.

Pollen implication:

- OS-level PEPs are optional high-coverage enforcement surfaces.
- The wizard must not assume they exist.
- When absent, use MCP/stdio/http/browser/config-based PEPs or observe-only.

---

## 4. Target Architecture

```text
                          +---------------------------+
                          | Local Admin Dashboard     |
                          | Discovery + Deployment UI |
                          +-------------+-------------+
                                        |
                                        v
+--------------------+       +----------+----------+       +----------------------+
| Discovery Sources  | ----> | Agent Capability    | ----> | Deployment Wizard    |
| process/config/MCP |       | Inventory (ACI)     |       | PEP/PDP selection    |
+--------------------+       +----------+----------+       +----------+-----------+
                                        |                             |
                                        v                             v
                              +---------+---------+        +----------+----------+
                              | Policy/Preset     |        | Control Binding     |
                              | Render + Bundle   |        | Apply/Rollback      |
                              +---------+---------+        +----------+----------+
                                        |                             |
                                        v                             v
                              +---------+-----------------------------+----------+
                              | PEP Runtime: MCP Proxy / Stdio Wrapper / HTTP   |
                              | Gateway / Browser Extension / OS PEP / SDK      |
                              +----------------------+--------------------------+
                                                     |
                                                     v
                              +----------------------+--------------------------+
                              | PDP Runtime: Router -> Cedar / OPA-Wasm /       |
                              | OPA Server / OpenFGA / Pollen Cloud PDP         |
                              +----------------------+--------------------------+
                                                     |
                                                     v
                              +----------------------+--------------------------+
                              | Telemetry: Local SQLite Dashboard + Cloud Sink  |
                              +-------------------------------------------------+
```

---

## 5. Core Data Model

### 5.1 Agent Capability Inventory

Add a new durable inventory model. Discovery candidates are temporary; inventory is what deployment trusts.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentCapabilityInventory {
    pub schema_version: String,
    pub tenant_id: String,
    pub device_id: String,
    pub agent_id: String,
    pub candidate_id: Option<String>,
    pub display_name: String,
    pub agent_type: AgentKind,
    pub trust_level: String,
    pub confidence: f64,
    pub risk_score: u32,
    pub process: Option<ProcessSurface>,
    pub config_surfaces: Vec<ConfigSurface>,
    pub mcp_surfaces: Vec<McpSurface>,
    pub model_endpoints: Vec<ModelEndpointSurface>,
    pub browser_surfaces: Vec<BrowserSurface>,
    pub file_surfaces: Vec<FileSurface>,
    pub network_surfaces: Vec<NetworkSurface>,
    pub supported_pep_bindings: Vec<PepBindingOption>,
    pub supported_pdp_routes: Vec<PdpRouteOption>,
    pub telemetry_capabilities: TelemetryCapabilities,
    pub last_scan_id: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    DesktopAgent,
    IdeAgent,
    CliAgent,
    BrowserAgent,
    WebAiApp,
    McpClient,
    McpServer,
    LocalModelServer,
    IdeExtension,
    CustomScriptAgent,
    AutomationAgent,
    UnknownAiProcess,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigSurface {
    pub config_id: String,
    pub path_hash: String,
    pub path_redacted: String,
    pub owner_client: String,
    pub format: String,
    pub editable: bool,
    pub backup_supported: bool,
    pub discovered_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpSurface {
    pub server_name: String,
    pub client_hint: String,
    pub transport: McpTransportKind,
    pub command_template: Option<Vec<String>>,
    pub endpoint_domain: Option<String>,
    pub has_auth_header: bool,
    pub env_key_names: Vec<String>,
    pub tools_known: Vec<String>,
    pub resources_known: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpTransportKind {
    Stdio,
    StreamableHttp,
    SseLegacy,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PepBindingOption {
    pub pep_type: PepType,
    pub action: PepBindingAction,
    pub coverage: PepCoverage,
    pub mode_supported: Vec<ControlMode>,
    pub requires_admin: bool,
    pub requires_user_approval: bool,
    pub reversible: bool,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepType {
    McpProxy,
    StdioWrapper,
    HttpGateway,
    BrowserExtension,
    LocalModelProxy,
    LinuxEbpf,
    WindowsWfp,
    MacosNetworkExtension,
    FileSystemPep,
    EmbeddedSdk,
    TelemetryOnly,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepBindingAction {
    RewriteConfig,
    WrapCommand,
    ProxyEndpoint,
    InstallLocalService,
    InstallOsModule,
    EnableBrowserExtension,
    ObserveOnly,
    ManualInstruction,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PepCoverage {
    ToolCalls,
    ToolCallsAndResources,
    HttpEgress,
    NetworkEgress,
    FileAccess,
    BrowserSaas,
    LocalModelApi,
    ProcessMetadataOnly,
    Unknown,
}
```

### 5.2 Policy Deployment

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyDeployment {
    pub schema_version: String,
    pub deployment_id: String,
    pub tenant_id: String,
    pub device_id: String,
    pub source: DeploymentSource,
    pub preset_id: Option<String>,
    pub policy_ids: Vec<String>,
    pub bundle_id: Option<String>,
    pub selected_agents: Vec<String>,
    pub selected_peps: Vec<PepDeploymentBinding>,
    pub selected_pdp_route: PdpDeploymentRoute,
    pub control_mode: ControlMode,
    pub status: DeploymentStatus,
    pub rollback_snapshot_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PepDeploymentBinding {
    pub binding_id: String,
    pub pep_type: PepType,
    pub agent_id: String,
    pub target_surface_id: String,
    pub mode: ControlMode,
    pub apply_status: BindingStatus,
    pub health_status: HealthStatus,
    pub config_backup_id: Option<String>,
    pub last_probe: Option<PepProbeResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdpDeploymentRoute {
    pub route_id: String,
    pub primary_pdp_id: String,
    pub fallback_pdp_ids: Vec<String>,
    pub shadow_pdp_ids: Vec<String>,
    pub failure_behavior: String,
}
```

### 5.3 Logs

Add typed logs for dashboard queries:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolInvocationLog {
    pub event_type: String, // tool_invocation
    pub event_id: String,
    pub tenant_id: String,
    pub device_id: String,
    pub agent_id: String,
    pub tool_id: String,
    pub tool_name: String,
    pub mcp_server_id: Option<String>,
    pub pep_type: PepType,
    pub pdp_route_id: Option<String>,
    pub decision: String,
    pub reason: String,
    pub redacted_arguments: serde_json::Value,
    pub raw_arguments_captured: bool,
    pub latency_ms: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceAccessLog {
    pub event_type: String, // resource_access
    pub event_id: String,
    pub tenant_id: String,
    pub device_id: String,
    pub agent_id: String,
    pub resource_id: String,
    pub resource_kind: String,
    pub operation: String,
    pub path_redacted: Option<String>,
    pub path_hash: Option<String>,
    pub account_provider: Option<String>,
    pub pep_type: PepType,
    pub decision: String,
    pub reason: String,
    pub timestamp: String,
}
```

---

## 6. PEP/PDP Selection Logic

### 6.1 PEP priority by discovered surface

| Discovered surface | Best PEP | Fallback PEP | Last resort |
|---|---|---|---|
| MCP stdio config | `stdio_wrapper` via config rewrite | `mcp_proxy` if wrapped command can speak proxy | `telemetry_only` |
| MCP HTTP/SSE config | `mcp_proxy` or HTTP reverse proxy | `http_gateway` | `telemetry_only` |
| OpenAI-compatible local model endpoint | `local_model_proxy` | `http_gateway` | `telemetry_only` |
| Ollama endpoint | `local_model_proxy` | `http_gateway` | `telemetry_only` |
| Browser SaaS AI app | `browser_extension` | OS network PEP | `telemetry_only` |
| Unknown process with LLM egress | OS PEP if installed | HTTP gateway only if traffic can be routed | `observe_only` |
| CLI agent command known | `stdio_wrapper` | shell alias/manual wrapper | `telemetry_only` |
| File access through MCP filesystem server | `mcp_proxy` | `stdio_wrapper` | `telemetry_only` |
| Direct filesystem access by process | `file_system_pep` or OS PEP | process/file observe only | `telemetry_only` |

### 6.2 PDP selection

| Policy need | Best PDP | Fallback |
|---|---|---|
| Tool/resource ABAC, approval, trust level | Local Cedar | Pollen Cloud PDP |
| Budgets, token/cost thresholds, content guard flags | Local OPA-Wasm | Remote OPA Server |
| Ownership/team/folder delegation | OpenFGA Server | Cedar if relationship graph is small |
| Multi-engine routing/failover | Policy Router | Pollen Cloud PDP |
| Central enterprise policy | Pollen Cloud PDP | Local last-known-good bundle |

### 6.3 Selection algorithm

```rust
pub fn recommend_deployment_plan(
    inventory: &AgentCapabilityInventory,
    preset: &PolicyPresetV2,
    device: &DeviceCapabilities,
) -> DeploymentRecommendation {
    let mut candidates = Vec::new();

    for pep in &inventory.supported_pep_bindings {
        if !preset.supported_pep_types.contains(&pep.pep_type) {
            continue;
        }
        let max_mode = max_supported_mode(&pep.mode_supported);
        let can_enforce = pep.mode_supported.contains(&ControlMode::Enforce)
            || pep.mode_supported.contains(&ControlMode::StrictDeny);

        let mut score = 0;
        if preset.recommended_pep_types.contains(&pep.pep_type) {
            score += 50;
        }
        if can_enforce {
            score += 30;
        }
        if pep.reversible {
            score += 10;
        }
        if !pep.requires_admin {
            score += 10;
        }
        if matches!(pep.action, PepBindingAction::RewriteConfig | PepBindingAction::WrapCommand) {
            score += 10;
        }

        candidates.push(PepCandidate {
            pep_type: pep.pep_type.clone(),
            score,
            max_mode,
            reason: pep.reason.clone(),
            requires_user_approval: pep.requires_user_approval,
        });
    }

    candidates.sort_by_key(|c| std::cmp::Reverse(c.score));

    let recommended_pep = candidates.first().cloned().unwrap_or(PepCandidate {
        pep_type: PepType::TelemetryOnly,
        score: 0,
        max_mode: ControlMode::Observe,
        reason: "No enforceable PEP discovered for this agent".into(),
        requires_user_approval: false,
    });

    let pdp_route = recommend_pdp_route(preset, device);

    DeploymentRecommendation {
        recommended_pep,
        alternatives: candidates,
        pdp_route,
        warnings: compute_warnings(inventory, preset, device),
    }
}
```

---

## 7. Policy Deployment Wizard UX

### Step 1: Select Agents

Show:

- Registered agents.
- Newly discovered candidates.
- Agent confidence and risk score.
- MCP servers/tools/resources discovered.
- Whether each agent is enforceable, partially enforceable, or observe-only.

CTA:

- `Scan Again`
- `Register Agent`
- `Continue with Selected Agents`

### Step 2: Select Policy Goal

Goal categories:

- Content Guard
- PII/Secrets Protection
- File/Folder Access
- MCP Tool Control
- Email/Drive/Browser Resource Access
- Provider/Network Egress
- Cost & Token Budget
- Approval Workflow
- Shadow AI Detection

### Step 3: Configure Policy Parameters

Examples:

- Allowed folders and denied folders.
- MCP tool allowlist.
- High-risk tools requiring approval.
- Allowed LLM providers/domains.
- Daily/monthly token and USD budget.
- PII detectors and redaction action.
- Email send approval and external recipient rule.

### Step 4: Select PEP Type

Show a capability matrix:

```text
PEP Type          Status          Coverage                  Max Mode
mcp_proxy         available       MCP tools/resources        enforce
stdio_wrapper     available       wrapped subprocess         enforce
http_gateway      not_configured  HTTP egress                observe
linux_ebpf        not_installed   network egress             observe
telemetry_only    available       logs only                  observe
```

Rules:

- Disable `Enforce` if selected PEP max mode is observe-only.
- Show why OS PEP is unavailable.
- If config rewrite is possible, show exact files to change using redacted paths.
- If user approval is needed, show a confirmation page.

### Step 5: Select PDP Route

Default recommendations:

- Cedar local for tool/resource access and approval.
- OPA-Wasm local for budget/content/provider policies.
- OpenFGA server for relationship/folder delegation if configured.
- Pollen Cloud PDP as optional remote primary/fallback/shadow.

Show route modes:

- `Local Only`
- `Local Primary, Cloud Fallback`
- `Cloud Primary, Local Fallback`
- `Shadow Cloud Audit`
- `Observe Only`

### Step 6: Preview

Preview must include:

- Policy source/artifacts.
- PEP binding changes.
- Config files to modify.
- Backup files to create.
- PDP route.
- Telemetry events that will be emitted.
- Rollback plan.

### Step 7: Simulate

Simulation sources:

- Sample input generated from selected preset.
- Last 24h telemetry.
- Selected agent's discovered tools/resources.

Show:

- Allowed count.
- Denied count.
- Warn count.
- Approval count.
- Top affected tools/resources.
- False-positive candidates.

### Step 8: Deploy

Atomic sequence:

1. Validate preset/policy.
2. Validate PDP runtime.
3. Validate PEP capability.
4. Create rollback snapshot.
5. Render policy artifacts.
6. Publish signed bundle.
7. Apply PEP binding.
8. Probe PEP.
9. Probe PDP decision.
10. Emit deployment telemetry.
11. Show status.

### Step 9: Observe Logs

After deploy, show tabs:

- Decisions
- Tool Calls
- Resource Access
- PEP Health
- Policy Deployment Audit
- Telemetry Sync Status

---

## 8. API Design

### 8.1 Inventory APIs

```http
GET  /v1/tenants/:tenant/agent-inventory
GET  /v1/tenants/:tenant/agent-inventory/:agent_id
POST /v1/tenants/:tenant/agent-inventory/rebuild
POST /v1/tenants/:tenant/discovery/candidates/:candidate_id/promote
```

### 8.2 Recommendation APIs

```http
POST /v1/tenants/:tenant/policy-deployment/recommend
POST /v1/tenants/:tenant/policy-deployment/preview
POST /v1/tenants/:tenant/policy-deployment/simulate
POST /v1/tenants/:tenant/policy-deployment/deploy
POST /v1/tenants/:tenant/policy-deployment/:deployment_id/rollback
```

### 8.3 Control Binding APIs

```http
GET  /v1/tenants/:tenant/control-bindings
POST /v1/tenants/:tenant/control-bindings/:binding_id/apply
POST /v1/tenants/:tenant/control-bindings/:binding_id/probe
POST /v1/tenants/:tenant/control-bindings/:binding_id/rollback
```

### 8.4 Logs APIs

```http
GET /v1/tenants/:tenant/logs/decisions
GET /v1/tenants/:tenant/logs/tool-invocations
GET /v1/tenants/:tenant/logs/resource-access
GET /v1/tenants/:tenant/logs/policy-deployments
GET /v1/tenants/:tenant/logs/pep-health
```

---

## 9. Control Binding Implementation

### 9.1 Reversible config rewrite

For MCP config files, do not directly mutate without backup.

```rust
pub async fn apply_mcp_stdio_wrapper_binding(
    binding: &ControlBindingPlan,
    config_path: &std::path::Path,
    server_name: &str,
    wrapper_exe: &str,
) -> anyhow::Result<AppliedBinding> {
    let original = tokio::fs::read_to_string(config_path).await?;
    let backup_id = format!("backup_{}", uuid::Uuid::new_v4());
    let backup_path = config_path.with_extension(format!("{}.pollen.bak", backup_id));
    tokio::fs::write(&backup_path, &original).await?;

    let mut json: serde_json::Value = serde_json::from_str(&original)?;
    let server = json
        .pointer_mut(&format!("/mcpServers/{}", escape_json_pointer(server_name)))
        .ok_or_else(|| anyhow::anyhow!("MCP server not found: {}", server_name))?;

    let old_command = server.get("command").cloned().unwrap_or_default();
    let old_args = server.get("args").cloned().unwrap_or_else(|| serde_json::json!([]));

    server["command"] = serde_json::json!(wrapper_exe);
    server["args"] = serde_json::json!([
        "--tenant", "local",
        "--binding-id", binding.binding_id,
        "--target-command", old_command,
        "--target-args", old_args
    ]);

    let updated = serde_json::to_string_pretty(&json)?;
    tokio::fs::write(config_path, updated).await?;

    Ok(AppliedBinding {
        binding_id: binding.binding_id.clone(),
        status: "applied".into(),
        backup_path_hash: sha256_path(&backup_path),
        backup_path_redacted: redact_path(&backup_path),
        restart_required: true,
    })
}
```

### 9.2 HTTP/SSE MCP proxy rewrite

```rust
pub fn rewrite_mcp_http_endpoint(
    original_url: &str,
    proxy_base: &str,
    binding_id: &str,
) -> String {
    format!(
        "{}/mcp/proxy/{}?upstream={}",
        proxy_base.trim_end_matches('/'),
        binding_id,
        urlencoding::encode(original_url)
    )
}
```

### 9.3 Manual fallback instruction

If config file is not safely editable:

```json
{
  "binding_id": "bind_123",
  "status": "manual_required",
  "instruction": {
    "title": "Start agent through Pollen stdio wrapper",
    "command": "dek-stdio-wrapper --tenant local --agent agent_123 --target-cmd <original>",
    "reason": "Config file is not writable or format is unsupported"
  }
}
```

---

## 10. PDP Runtime Requirements

### 10.1 Local Cedar

Required:

- Load active Cedar artifacts from signed bundle.
- Validate against Pollen Cedar schema before publish.
- Evaluate normalized events:
  - `tool.invoke`
  - `resource.read`
  - `resource.write`
  - `email.send`
  - `drive.share`
  - `llm.request`

### 10.2 Local OPA-Wasm

Required:

- Compile Rego through `opa build -t wasm -e <entrypoint>`.
- Store entrypoint in bundle metadata.
- Load `data.json` alongside Wasm.
- Evaluate OPA-Wasm ABI, not generic WASI `_start`.
- Return unified `PolicyDecision`.

### 10.3 OpenFGA Server

Required:

- Store endpoint and auth ref.
- Probe `/healthz`.
- Store authorization model ID.
- Evaluate `check` requests for relationship-based decisions.
- Never store raw PII in tuple object IDs.

### 10.4 Pollen Cloud PDP

Required:

- Auto-discover endpoint after login/enrollment.
- Support route modes:
  - Cloud shadow audit.
  - Cloud primary local fallback.
  - Local primary cloud fallback.
- Cache last-known-good local bundle.
- Fail closed for enforce unless user explicitly selects fail-open.

---

## 11. Telemetry Design

### 11.1 Required local/cloud endpoints

Keep current split endpoints:

```http
POST /v1/telemetry/events
POST /v1/telemetry/decision-logs
POST /v1/telemetry/security-events
POST /v1/telemetry/runtime-metrics
POST /v1/telemetry/traces
POST /v1/telemetry/ebpf-events
POST /v1/metrics
```

Add typed event routing for:

```text
tool_invocation       -> /v1/telemetry/decision-logs or /events
resource_access       -> /v1/telemetry/decision-logs or /events
policy_deployment     -> /v1/telemetry/events
pep_binding_status    -> /v1/telemetry/events
agent_inventory       -> /v1/telemetry/events
telemetry_sync_status -> /v1/telemetry/events
```

### 11.2 Example event envelope

```json
{
  "schema_version": "pollen.telemetry.v2",
  "event_type": "tool_invocation",
  "event_id": "evt_01J...",
  "tenant_id": "local",
  "device_id": "device-local",
  "agent_id": "agent_cursor",
  "pep_type": "mcp_proxy",
  "pdp_runtime_id": "local.cedar",
  "policy_ids": ["pol_high_risk_tool_approval"],
  "tool": {
    "tool_id": "tool_filesystem_read",
    "name": "filesystem.read",
    "server": "filesystem"
  },
  "decision": {
    "effect": "deny",
    "reason": "path_not_in_allowed_scope",
    "obligations": []
  },
  "arguments": {
    "redacted": true,
    "shape": {"path": "<PATH_HASH:...>"}
  },
  "latency_ms": 6,
  "timestamp": "2026-06-22T10:00:00Z"
}
```

### 11.3 Privacy defaults

- Do not capture raw prompt/response by default.
- Redact env values, auth headers, bearer tokens, passwords, SSH keys, API keys.
- Store path hashes and redacted paths, not full paths, unless user explicitly enables local-only raw path view.
- Cloud upload must be redacted by default.
- Local Dashboard may show more detail only when user enables local sensitive logging.

---

## 12. Frontend Implementation Sketch

### 12.1 Wizard state machine

```ts
export type WizardStep =
  | "scan"
  | "agents"
  | "goal"
  | "parameters"
  | "pep"
  | "pdp"
  | "preview"
  | "simulate"
  | "deploy"
  | "logs";

export interface DeploymentWizardState {
  step: WizardStep;
  selectedAgentIds: string[];
  selectedPresetId?: string;
  params: Record<string, unknown>;
  selectedPepTypes: PepType[];
  selectedPdpRoute?: PdpRouteSelection;
  recommendation?: DeploymentRecommendation;
  preview?: DeploymentPreview;
  simulation?: DeploymentSimulationResult;
  deployment?: PolicyDeployment;
}
```

### 12.2 API client additions

```ts
export const DeploymentApi = {
  listInventory: () => defaultClient.fetchApi("/agent-inventory"),
  getInventory: (agentId: string) => defaultClient.fetchApi(`/agent-inventory/${agentId}`),
  recommend: (payload: unknown) =>
    defaultClient.fetchApi("/policy-deployment/recommend", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  preview: (payload: unknown) =>
    defaultClient.fetchApi("/policy-deployment/preview", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  simulate: (payload: unknown) =>
    defaultClient.fetchApi("/policy-deployment/simulate", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  deploy: (payload: unknown) =>
    defaultClient.fetchApi("/policy-deployment/deploy", {
      method: "POST",
      body: JSON.stringify(payload),
    }),
  rollback: (deploymentId: string) =>
    defaultClient.fetchApi(`/policy-deployment/${deploymentId}/rollback`, {
      method: "POST",
    }),
};

export const LogApi = {
  decisions: () => defaultClient.fetchApi("/logs/decisions"),
  toolInvocations: () => defaultClient.fetchApi("/logs/tool-invocations"),
  resourceAccess: () => defaultClient.fetchApi("/logs/resource-access"),
  deployments: () => defaultClient.fetchApi("/logs/policy-deployments"),
  pepHealth: () => defaultClient.fetchApi("/logs/pep-health"),
};
```

### 12.3 PEP selector UI logic

```tsx
function canSelectControlMode(
  selectedPep: PepCapability,
  requestedMode: ControlMode,
): boolean {
  if (requestedMode === "observe") return true;
  if (selectedPep.max_mode === "observe") return false;
  if (requestedMode === "strict_deny" && selectedPep.max_mode !== "strict_deny") {
    return false;
  }
  return true;
}
```

---

## 13. Backend API Skeleton

```rust
pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/v1/tenants/:tenant/agent-inventory", get(list_inventory))
        .route("/v1/tenants/:tenant/agent-inventory/rebuild", post(rebuild_inventory))
        .route("/v1/tenants/:tenant/policy-deployment/recommend", post(recommend))
        .route("/v1/tenants/:tenant/policy-deployment/preview", post(preview))
        .route("/v1/tenants/:tenant/policy-deployment/simulate", post(simulate))
        .route("/v1/tenants/:tenant/policy-deployment/deploy", post(deploy))
        .route("/v1/tenants/:tenant/policy-deployment/:id/rollback", post(rollback))
        .route("/v1/tenants/:tenant/logs/tool-invocations", get(list_tool_logs))
        .route("/v1/tenants/:tenant/logs/resource-access", get(list_resource_logs))
}

async fn deploy(
    Path(tenant): Path<String>,
    State(st): State<AppState>,
    Json(req): Json<DeployPolicyWizardRequest>,
) -> ApiResult<Json<DeployPolicyWizardResponse>> {
    let inventory = st.registry_store
        .get_agent_inventory(&tenant, &req.agent_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound(req.agent_id.clone()))?;

    let recommendation = recommend_deployment_plan(&inventory, &req.preset, &req.device_capabilities);
    validate_selected_pep(&recommendation, &req.selected_peps, &req.control_mode)?;
    validate_pdp_route(&st, &tenant, &req.pdp_route).await?;

    let rollback = create_rollback_snapshot(&st, &tenant, &req).await?;
    let rendered = render_policy_artifacts(&req).await?;
    let bundle = publish_signed_bundle(&st, &tenant, rendered, &req.pdp_route).await?;

    let mut bindings = Vec::new();
    for pep in &req.selected_peps {
        let applied = apply_control_binding_for_pep(&st, &tenant, &inventory, pep, &bundle).await?;
        let probe = probe_binding(&applied).await?;
        bindings.push((applied, probe));
    }

    emit_policy_deployment_event(&st, &tenant, &req, &bundle, &bindings).await?;

    Ok(Json(DeployPolicyWizardResponse {
        deployment_id: format!("dep_{}", uuid::Uuid::new_v4()),
        status: "active".into(),
        bundle_id: bundle.bundle_id,
        bindings,
        rollback_snapshot_id: rollback.snapshot_id,
    }))
}
```

---

## 14. Database Additions

```sql
CREATE TABLE IF NOT EXISTS agent_inventory (
  tenant TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  device_id TEXT NOT NULL,
  inventory_json TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, agent_id)
);

CREATE TABLE IF NOT EXISTS control_bindings (
  tenant TEXT NOT NULL,
  binding_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  pep_type TEXT NOT NULL,
  action TEXT NOT NULL,
  status TEXT NOT NULL,
  config_backup_id TEXT,
  binding_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, binding_id)
);

CREATE TABLE IF NOT EXISTS policy_deployments (
  tenant TEXT NOT NULL,
  deployment_id TEXT NOT NULL,
  status TEXT NOT NULL,
  deployment_json TEXT NOT NULL,
  rollback_snapshot_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, deployment_id)
);

CREATE TABLE IF NOT EXISTS tool_invocation_logs (
  tenant TEXT NOT NULL,
  event_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  tool_id TEXT,
  event_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, event_id)
);

CREATE TABLE IF NOT EXISTS resource_access_logs (
  tenant TEXT NOT NULL,
  event_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  resource_id TEXT,
  event_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (tenant, event_id)
);
```

---

## 15. Implementation Phases

### Phase 1: Stabilize contracts

- Unify `PolicyPresetV2`, `DeployPresetRequest`, `ControlMode`, `PepType`.
- Fix `policy_presets_api`, `preset_deploy_api`, and `policy.rs` to use one model.
- Separate PDP kinds from PEP types.
- Remove hardcoded `target_os: "linux"` in UI.

### Phase 2: Build Agent Capability Inventory

- Convert `DiscoveredAgentCandidateV2` into durable inventory after registration.
- Persist MCP servers, endpoints, config surfaces, and binding options.
- Add inventory rebuild endpoint.
- Add capability confidence and coverage fields.

### Phase 3: Implement real control bindings

- Implement reversible MCP config rewrite.
- Implement stdio wrapper binding.
- Implement HTTP/SSE proxy binding.
- Implement local model proxy binding for Ollama/OpenAI-compatible endpoints.
- Keep OS PEP bindings as install/probe/status until real modules are present.

### Phase 4: Implement Deployment Wizard APIs

- Recommendation.
- Preview.
- Simulation.
- Deploy.
- Rollback.
- Logs.

### Phase 5: Runtime enforcement

- Wire PEPs to the policy router.
- Wire local Cedar and OPA-Wasm properly.
- Wire OpenFGA server checks.
- Add health/readiness/decision probes.
- Emit typed tool/resource logs.

### Phase 6: Dashboard UX

- Replace current flat preset modal with full wizard.
- Add capability matrix.
- Add config diff preview.
- Add deployment health page.
- Add tool/resource log pages.

### Phase 7: Cloud sync

- Send redacted inventory summaries to Pollen Cloud.
- Send deployment status and typed telemetry.
- Pull managed policy bundles and cloud PDP route suggestions.
- Preserve local-only mode when user is offline or air-gapped.

---

## 16. Acceptance Criteria

### Discovery

- Deep scan produces candidates with process, MCP config, local model endpoints, browser/IDE/CLI evidence when available.
- Registering a candidate creates durable `AgentCapabilityInventory`.
- Inventory lists enforceable, partially enforceable, and observe-only PEP options.

### Policy Deployment Wizard

- Wizard selects real OS from browser/API, not hardcoded Linux.
- Wizard disables enforce when selected PEP cannot enforce.
- Wizard previews config rewrites and rollback backups.
- Wizard deploys a policy and binding in one transaction.
- Wizard rolls back config and policy state.

### Enforcement

- MCP stdio calls can be wrapped and evaluated before tool execution.
- MCP HTTP/SSE calls can be proxied and evaluated.
- HTTP/local model calls can be routed through gateway/proxy when configured.
- OS PEPs show accurate status and never fake availability.

### Telemetry

- Local Dashboard shows decisions, tool invocations, resource access, deployments, and PEP health.
- Cloud telemetry uses the same endpoint contract.
- Raw secrets are rejected or redacted before persistence.
- Offline spool retries and exposes sync status.

### Testing

- Unit tests for capability recommendation matrix.
- Unit tests for reversible config rewrite and rollback.
- Integration tests for MCP stdio wrapper flow.
- Integration tests for policy deployment + signed bundle + PEP binding.
- Integration tests for telemetry ingestion and dashboard log queries.
- Cross-platform tests for Windows path, macOS path, Linux path normalization.

---

## 17. Non-Negotiable Product Rules

1. Do not claim `Enforce` unless a live PEP can block the action.
2. Do not mutate user config without backup and rollback.
3. Do not send raw secrets or raw personal data to Pollen Cloud by default.
4. Do not confuse PDP runtime with PEP type.
5. Do not treat OpenFGA as local embedded engine unless an OpenFGA process is actually managed.
6. Do not treat generic WASI runtime as OPA-Wasm unless OPA-Wasm ABI is implemented.
7. Do not hide partial coverage. Show exactly what is protected and what is not.

---

## 18. Files to Modify First

Backend:

```text
crates/dek-policy-presets/src/model.rs
crates/dek-policy-presets/src/catalog.rs
crates/dek-policy-presets/src/render.rs
crates/dek-policy-presets/src/validate.rs
crates/local-control-plane/src/pep_capabilities_api.rs
crates/local-control-plane/src/preset_deploy_api.rs
crates/local-control-plane/src/agent_discovery_api.rs
crates/local-control-plane/src/store.rs
crates/local-control-plane/src/telemetry.rs
crates/local-control-plane/src/policy.rs
crates/local-control-plane/src/bundle.rs
```

New backend modules:

```text
crates/local-control-plane/src/agent_inventory_api.rs
crates/local-control-plane/src/control_binding.rs
crates/local-control-plane/src/policy_deployment_wizard_api.rs
crates/local-control-plane/src/tool_resource_logs_api.rs
```

Frontend:

```text
apps/local-admin-dashboard/src/pages/AutoDiscovery.tsx
apps/local-admin-dashboard/src/pages/PolicyPresets.tsx
apps/local-admin-dashboard/src/components/PresetWizard.tsx
apps/local-admin-dashboard/src/services/api.ts
apps/local-admin-dashboard/src/services/types.ts
```

New frontend components:

```text
apps/local-admin-dashboard/src/components/policy-deployment/AgentSelector.tsx
apps/local-admin-dashboard/src/components/policy-deployment/PolicyGoalSelector.tsx
apps/local-admin-dashboard/src/components/policy-deployment/PepCapabilityMatrix.tsx
apps/local-admin-dashboard/src/components/policy-deployment/PdpRouteSelector.tsx
apps/local-admin-dashboard/src/components/policy-deployment/ConfigDiffPreview.tsx
apps/local-admin-dashboard/src/components/policy-deployment/SimulationResults.tsx
apps/local-admin-dashboard/src/components/policy-deployment/DeploymentStatus.tsx
apps/local-admin-dashboard/src/components/logs/ToolInvocationLogs.tsx
apps/local-admin-dashboard/src/components/logs/ResourceAccessLogs.tsx
```

---

## 19. Summary Recommendation

The best path is not to build a generic "Add Connector" page. Pollen DEK should build a local policy deployment system around actual capability surfaces:

- What agent exists?
- What tool/resource surfaces does it expose?
- Which PEP can actually intercept it?
- Which PDP can decide for it?
- What policy bundle is active?
- What logs prove control is working?

This will make Pollen DEK credible as a local Device Enforcement Kit: it can discover, observe, control, enforce, and report telemetry with accurate guarantees instead of optimistic UI state.
