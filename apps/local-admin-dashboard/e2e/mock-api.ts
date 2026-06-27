import type { Page, Route } from "@playwright/test";

const externalServer = process.env.DEK_PLAYWRIGHT_EXTERNAL_SERVER === "1";
const now = "2026-06-27T10:15:00Z";

const json = (route: Route, body: unknown, status = 200) =>
  route.fulfill({
    status,
    contentType: "application/json",
    body: JSON.stringify(body),
  });

const objectMeta = (source = "discovery", status = "registered") => ({
  schema_version: "pollek.registry.meta.v1",
  tenant_id: "local",
  workspace_id: "local",
  environment_id: "desktop",
  created_at: "2026-06-27T10:00:00Z",
  updated_at: now,
  created_by: "playwright-fixture",
  updated_by: "playwright-fixture",
  source,
  status,
  tags: ["e2e", "governance-loop"],
});

const agent = {
  meta: objectMeta(),
  agent_id: "agent-antigravity",
  name: "Antigravity",
  agent_type: "openai_agent",
  vendor: "Google",
  runtime: {
    runtime_name: "native",
    version: "1.0.0",
  },
  entrypoints: [
    {
      command: "antigravity.exe",
      args: [],
    },
  ],
  declared_tools: ["tool-workspace-files"],
  declared_resources: ["resource-workspace-src"],
  identity: {
    spiffe_id: "spiffe://local.pollek/device/dev-win/agent/antigravity",
    process_path: "C:\\Program Files\\Google\\Antigravity\\antigravity.exe",
    user_subject: "DELL\\LocalAdmin",
    token_bindings: [
      {
        kind: "oidc_id_token",
        provider: "Pollek Cloud",
        issuer: "https://cloud.pollek.ai",
        subject: "agent-antigravity",
        audience: ["pollek-cloud"],
        scopes: ["telemetry.write"],
        confirmation: "spiffe_svid",
        expires_at: "2026-06-27T12:00:00Z",
        last_rotated_at: "2026-06-27T10:00:00Z",
      },
    ],
  },
  trust_level: "medium",
  capabilities: [
    "workspace_file_access",
    "terminal_execution",
    "browser_control",
    "tool_calling",
    "mcp_client",
  ],
  labels: {
    reference_intel: "google-antigravity",
    source: "discovery",
  },
  enforcement_mode: "Enforce",
};

const tool = {
  tool_id: "tool-workspace-files",
  name: "Workspace Files",
  type: "filesystem",
  vendor: "Pollek",
  description: "Observed file and folder access from the local agent process.",
  agent_id: agent.agent_id,
  source: "local-observer",
  status: "active",
  call_count: 3,
  last_used: now,
};

const resource = {
  resource_id: "resource-workspace-src",
  name: "repo/src",
  type: "folder",
  uri: "file:///C:/Users/DELL/Documents/Codex/repo/apps/local-admin-dashboard/src",
  path: "C:\\Users\\DELL\\Documents\\Codex\\repo\\apps\\local-admin-dashboard\\src",
  host: "DELL-WINDOWS",
  description: "Source folder observed through the local filesystem telemetry plane.",
  sensitivity: "internal_source",
  source: "local-observer",
  status: "active",
  last_accessed: now,
};

const policy = {
  policy_id: "policy-protect-workspace-files",
  name: "Protect workspace source files",
  description: "Require local policy evaluation before Antigravity reads source folders.",
  engine: "opa_wasm",
  status: "published",
  mode: "enforce",
  scope: agent.agent_id,
  created_at: "2026-06-27T10:05:00Z",
  updated_at: now,
  rules_count: 2,
  source: "policy-suggestions",
  last_deployed_at: now,
  bundle_id: "bundle-local-1",
};

const candidate = {
  schema_version: "discovery.candidate.v2",
  candidate_id: agent.agent_id,
  tenant_id: "local",
  device_id: "DELL-WINDOWS",
  status: "registered",
  display_name: agent.name,
  vendor: agent.vendor,
  product: "Antigravity",
  inferred_agent_type: "openai_agent",
  confidence: 0.96,
  risk_score: 42,
  first_seen: "2026-06-27T10:00:00Z",
  last_seen: now,
  scan_ids: ["scan-e2e-1"],
  last_scan_id: "scan-e2e-1",
  evidence: [
    {
      evidence_id: "ev-proc-antigravity",
      source: "process",
      confidence: 0.98,
      observed_at: now,
      privacy_class: "metadata_only",
      redacted: true,
      merge_key: "Google/Antigravity",
      source_path_redacted: "C:\\Program Files\\Google\\Antigravity\\antigravity.exe",
      data: {
        process_name: "antigravity.exe",
        window_title: "Antigravity - Pollek Workspace",
      },
    },
    {
      evidence_id: "ev-file-src",
      source: "filesystem",
      confidence: 0.9,
      observed_at: now,
      privacy_class: "metadata_only",
      redacted: true,
      data: {
        path: resource.path,
        access: "read",
      },
    },
  ],
  matched_signals: [
    { kind: "process_name", detail: "antigravity.exe", weight: 0.45 },
    { kind: "well_known_vendor", detail: "Google Antigravity", weight: 0.35 },
  ],
  capability_tags: agent.capabilities,
  discovered_configs: [],
  discovered_endpoints: [],
  discovered_mcp_servers: [
    {
      server_name: "workspace-files",
      transport: "stdio",
    },
  ],
  suggested_registration: {
    agent_id: agent.agent_id,
    display_name: agent.name,
  },
  suggested_observation_profile: {
    exact_usage_first: true,
    sources: ["process", "filesystem", "usage_logs"],
  },
  suggested_control_bindings: [
    {
      binding_id: "binding-workspace-files",
      kind: "tool",
      target_candidate_id: agent.agent_id,
      action: "enforce",
      requires_user_approval: false,
      risk: "medium",
      reversible: true,
      summary: "Apply filesystem policy before source folder access.",
    },
  ],
  telemetry_plan: {
    exact_usage_sources: ["wrapper", "provider_usage_logs"],
    fallback_sources: ["process_metadata"],
  },
  labels: {
    reference_intel: "google-antigravity",
  },
};

const canonicalCapability = {
  capability_id: "cap-workspace-file-access",
  candidate_id: agent.agent_id,
  capability_kind: "tool_access",
  name: "Workspace file access",
  description: "Local discovery observed filesystem metadata for source folder reads.",
  modality: ["filesystem"],
  actions: ["read", "list"],
  source: "filesystem observer",
  confidence: 0.92,
  risk_tags: ["source_code", "local_file"],
  evidence_ids: ["ev-file-src"],
  privacy_class: "metadata_only",
};

const capabilityInventory = {
  schema_version: "discovery.capability-inventory.v1",
  candidate_id: agent.agent_id,
  entity: {
    schema_version: "discovery.entity.v1",
    candidate_id: agent.agent_id,
    tenant_id: "local",
    device_id: "DELL-WINDOWS",
    entity_kind: "agent",
    display_name: agent.name,
    vendor: agent.vendor,
    product: "Antigravity",
    confidence: 0.96,
    risk_score: 42,
    status: "registered",
    capabilities: [canonicalCapability],
    evidence: candidate.evidence,
    relationships: [
      {
        relationship_id: "rel-agent-tool",
        subject_candidate_id: agent.agent_id,
        relation: "uses_tool",
        object_candidate_id: tool.tool_id,
        confidence: 0.9,
        evidence_ids: ["ev-file-src"],
      },
    ],
    suggested_registration: candidate.suggested_registration,
    suggested_control_bindings: candidate.suggested_control_bindings,
    privacy_profile: "metadata_only",
    performance_cost_class: "passive_metadata",
    first_seen: candidate.first_seen,
    last_seen: candidate.last_seen,
  },
  capabilities: [canonicalCapability],
  relationships: [
    {
      relationship_id: "rel-agent-tool",
      subject_candidate_id: agent.agent_id,
      relation: "uses_tool",
      object_candidate_id: tool.tool_id,
      confidence: 0.9,
      evidence_ids: ["ev-file-src"],
    },
  ],
  retrieval_status: "derived",
  source: "local discovery fixture",
  privacy_note: "Metadata-only fixture. No file content is read.",
};

const graphNodes = [
  {
    id: "agent:agent-antigravity",
    type: "agent",
    entity_id: agent.agent_id,
    label: agent.name,
    subtitle: "Google native coding agent",
    status: "enforcing",
    risk: "medium",
    mode: "enforce",
    badges: ["Registered", "SPIFFE bound"],
    metrics: [
      { label: "Tools", value: "1" },
      { label: "Resources", value: "1" },
    ],
    href: `/agents?id=${agent.agent_id}`,
    raw: agent,
  },
  {
    id: "tool:tool-workspace-files",
    type: "tool",
    entity_id: tool.tool_id,
    label: tool.name,
    subtitle: tool.type,
    status: "active",
    risk: "medium",
    mode: "enforce",
    badges: ["Observed"],
    metrics: [{ label: "Calls", value: "3" }],
    href: `/tools?id=${tool.tool_id}`,
    raw: tool,
  },
  {
    id: "resource:resource-workspace-src",
    type: "resource",
    entity_id: resource.resource_id,
    label: resource.name,
    subtitle: resource.path,
    status: "active",
    risk: "medium",
    mode: "enforce",
    badges: ["Observed"],
    metrics: [{ label: "Sensitivity", value: resource.sensitivity }],
    href: `/tools?id=${resource.resource_id}`,
    raw: resource,
  },
  {
    id: "policy:policy-protect-workspace-files",
    type: "policy",
    entity_id: policy.policy_id,
    label: policy.name,
    subtitle: policy.engine,
    status: "enforcing",
    risk: "medium",
    mode: "enforce",
    badges: ["Published"],
    metrics: [{ label: "Rules", value: "2" }],
    href: `/policies?id=${policy.policy_id}`,
    raw: policy,
  },
];

const graphEdges = [
  {
    id: "edge-agent-tool",
    source: "agent:agent-antigravity",
    target: "tool:tool-workspace-files",
    relation: "uses_tool",
    label: "uses",
    evidence: "filesystem observer",
    observed: true,
    enforced: true,
  },
  {
    id: "edge-tool-resource",
    source: "tool:tool-workspace-files",
    target: "resource:resource-workspace-src",
    relation: "accesses_resource",
    label: "reads",
    evidence: "resource telemetry",
    observed: true,
    enforced: true,
  },
  {
    id: "edge-policy-agent",
    source: "policy:policy-protect-workspace-files",
    target: "agent:agent-antigravity",
    relation: "governs",
    label: "governs",
    evidence: "deployed policy bundle",
    observed: true,
    enforced: true,
  },
];

const graphResponse = {
  schema_version: "entity-graph.v1",
  tenant_id: "local",
  generated_at: now,
  center: null,
  nodes: graphNodes,
  edges: graphEdges,
  summaries: [
    { kind: "agents", label: "Agents", count: 1, tone: "info" },
    { kind: "observed_links", label: "Observed Links", count: 3, tone: "success" },
    { kind: "enforced_links", label: "Enforced Links", count: 3, tone: "success" },
  ],
  warnings: [],
};

const activityItem = {
  event_id: "evt-governance-loop-1",
  timestamp: now,
  actor: {
    id: "agent:agent-antigravity",
    type: "agent",
    entity_id: agent.agent_id,
    label: agent.name,
  },
  action: "filesystem.read",
  tool: {
    id: "tool:tool-workspace-files",
    type: "tool",
    entity_id: tool.tool_id,
    label: tool.name,
  },
  resource: {
    id: "resource:resource-workspace-src",
    type: "resource",
    entity_id: resource.resource_id,
    label: resource.name,
  },
  policies: [
    {
      id: "policy:policy-protect-workspace-files",
      type: "policy",
      entity_id: policy.policy_id,
      label: policy.name,
    },
  ],
  decision: "allow",
  enforcement_mode: "enforce",
  pep_plane: "windows_user_mode_observer",
  pdp_engine: "opa_wasm",
  trace_id: "trace-governance-loop-1",
  cost: {
    total_cost_usd: 0.0012,
    total_tokens: 128,
    provider: "local-observer",
    model: "exact-usage-fixture",
  },
  explanation: "Policy allowed source folder read after local PDP evaluation.",
  raw: {
    evidence_id: "ev-file-src",
    capture_quality: "exact",
  },
};

const activityTimeline = {
  schema_version: "activity-timeline.v1",
  tenant_id: "local",
  generated_at: now,
  items: [activityItem],
  next_cursor: null,
};

const activitySummary = {
  activity_sets: [
    {
      id: "governance-loop",
      label: "Governance loop",
      items: [activityItem],
    },
  ],
};

const capabilitySnapshot = {
  schema_version: "local-capability-snapshot.v2",
  tenant_id: "local",
  device_id: "DELL-WINDOWS",
  os: {
    family: "windows",
    version: "11 24H2",
    arch: "x86_64",
    is_server: false,
    elevated: true,
  },
  mode: "desktop_advanced",
  generated_at: now,
  control_methods: [
    {
      method_id: "windows_process_observer",
      display_name_en: "Windows Process Observer",
      display_name_th: "Windows Process Observer",
      status: "available",
      domains: ["process", "filesystem"],
      max_level: "enforce",
      maturity: "beta",
      install_state: "installed",
      warm_check: "passed",
      setup_action_ids: [],
      limitations_en: ["User-mode fixture for E2E. Kernel WFP driver is not required."],
      limitations_th: [],
    },
    {
      method_id: "browser_extension",
      display_name_en: "Browser Extension",
      display_name_th: "Browser Extension",
      status: "needs_install",
      domains: ["browser"],
      max_level: "observe",
      maturity: "alpha",
      install_state: "not_installed",
      warm_check: "not_run",
      setup_action_ids: ["install-browser-extension"],
      limitations_en: ["Browser message body capture requires user installation."],
      limitations_th: [],
    },
  ],
  observation_sources: [
    {
      source_id: "process",
      display_name_en: "Process Table",
      display_name_th: "Process Table",
      status: "available",
      domains: ["process"],
      privacy_note_en: "Reads process metadata only.",
      privacy_note_th: "",
      setup_action_ids: [],
    },
    {
      source_id: "filesystem",
      display_name_en: "Filesystem Metadata",
      display_name_th: "Filesystem Metadata",
      status: "available",
      domains: ["filesystem"],
      privacy_note_en: "Captures file and folder paths, not contents.",
      privacy_note_th: "",
      setup_action_ids: [],
    },
  ],
  setup_actions: [
    {
      action_id: "install-browser-extension",
      title_en: "Install browser extension",
      title_th: "Install browser extension",
      detail_en: "Required for exact browser AI prompt and response telemetry.",
      detail_th: "",
      estimated_minutes: 3,
      requires_admin: false,
      requires_restart: false,
      safe_to_skip: true,
    },
  ],
  contract: {
    local_contract_version: "pollek.local.v1",
    compatible_cloud_contracts: ["pollek.cloud.v1"],
    status: "compatible",
    reason_code: null,
  },
};

const usageSummary = {
  schema_version: "ai-usage-summary.v1",
  tenant_id: "local",
  generated_at: now,
  currency: "USD",
  totals: {
    total_cost_usd: 0.0012,
    total_tokens: 128,
    input_tokens: 80,
    output_tokens: 48,
    cached_input_tokens: 0,
    reasoning_output_tokens: 0,
    tool_tokens: 18,
    multimodal_tokens: 0,
    calls: 1,
  },
  by_agent: [
    {
      agent_id: agent.agent_id,
      total_cost_usd: 0.0012,
      total_tokens: 128,
      calls: 1,
      budget: { status: "ok" },
    },
  ],
  by_provider: [
    {
      provider: "local-observer",
      total_cost_usd: 0.0012,
      total_tokens: 128,
      calls: 1,
    },
  ],
  by_model: [
    {
      model: "exact-usage-fixture",
      total_cost_usd: 0.0012,
      total_tokens: 128,
      calls: 1,
    },
  ],
  buckets: [],
};

const usageEvents = {
  schema_version: "ai-usage-event-page.v1",
  tenant_id: "local",
  items: [
    {
      event_id: "usage-exact-1",
      occurred_at: now,
      agent_id: agent.agent_id,
      provider: "local-observer",
      model: "exact-usage-fixture",
      surface: "tool",
      tokens: {
        input_tokens: 80,
        output_tokens: 48,
        cached_input_tokens: 0,
        reasoning_output_tokens: 0,
        tool_tokens: 18,
        multimodal_tokens: 0,
        total_tokens: 128,
        estimated: false,
      },
      cost: {
        amount_usd: 0.0012,
        estimated: false,
      },
      cloud_sync_status: "pending",
      metadata: {
        capture_quality: "exact",
        source: "wrapper telemetry fixture",
      },
    },
  ],
  next_cursor: null,
};

const policySuggestion = {
  suggestion_id: "suggest-protect-workspace-files",
  title: "Protect workspace source files",
  summary: "Antigravity was observed reading the source folder. Deploy an enforce policy for local file access.",
  severity: "medium",
  status: "ready",
  feasibility: "can_enforce_now",
  recommended_policy_type: "filesystem_access_guard",
  recommended_control_level: "enforce",
  confidence: 0.91,
  target_agent_id: agent.agent_id,
  created_at: now,
  setup_required: [],
};

function entity360(entityType: string | null, entityId: string | null) {
  const entity =
    graphNodes.find(
      (node) => node.type === entityType && node.entity_id === entityId,
    ) ?? graphNodes[0];
  return {
    schema_version: "entity-360.v1",
    tenant_id: "local",
    generated_at: now,
    entity,
    graph: {
      ...graphResponse,
      center: entity,
    },
    summaries: [
      { kind: "entity", label: entity.label, count: 1, tone: "info" },
      { kind: "observed_links", label: "Observed Links", count: 3, tone: "success" },
      { kind: "enforced_links", label: "Enforced Links", count: 3, tone: "success" },
    ],
    activity: [activityItem],
    warnings: [],
  };
}

function routeUrl(route: Route) {
  return new URL(route.request().url());
}

export async function installMockApi(page: Page) {
  if (externalServer) {
    return;
  }

  let scanStarted = false;
  let suggestionsGenerated = false;
  const policies = [policy];

  await page.route("**/.well-known/pollek-contract", (route) =>
    json(route, {
      schema_version: "contract-discovery.v1",
      preferred: "pollek.v1",
      supported: ["pollek.v1"],
      capabilities: ["local-admin-dashboard", "policy-publish"],
    }),
  );

  await page.route("**/v1/tenants/local/devices/local/capability-snapshot-v2**", (route) =>
    json(route, capabilitySnapshot),
  );
  await page.route("**/v1/tenants/local/devices/local/capability-refresh**", (route) =>
    json(route, capabilitySnapshot),
  );

  await page.route("**/v1/tenants/local/registry/agents", (route) =>
    json(route, { items: scanStarted ? [agent] : [] }),
  );
  await page.route("**/v1/tenants/local/registry/mcp-servers", (route) =>
    json(route, { items: [] }),
  );
  await page.route("**/v1/tenants/local/registry/tools", (route) =>
    json(route, { items: scanStarted ? [tool] : [] }),
  );
  await page.route("**/v1/tenants/local/registry/resources", (route) =>
    json(route, { items: scanStarted ? [resource] : [] }),
  );
  await page.route("**/v1/tenants/local/registry/entities", (route) =>
    json(route, { items: scanStarted ? [agent, tool, resource] : [] }),
  );
  await page.route("**/v1/tenants/local/registry/relationships", (route) =>
    json(route, { items: scanStarted ? graphEdges : [] }),
  );

  await page.route("**/v1/tenants/local/discovery/candidates", (route) => {
    if (route.request().method() === "DELETE") {
      scanStarted = false;
      suggestionsGenerated = false;
      return json(route, { ok: true });
    }
    return json(route, { items: scanStarted ? [candidate] : [] });
  });
  await page.route("**/v1/tenants/local/discovery/entities", (route) =>
    json(route, { items: scanStarted ? [capabilityInventory.entity] : [] }),
  );
  await page.route("**/v1/tenants/local/discovery/candidates/*/capabilities", (route) =>
    json(route, capabilityInventory),
  );
  await page.route(
    "**/v1/tenants/local/discovery/candidates/*/retrieve-capabilities",
    (route) => json(route, capabilityInventory),
  );
  await page.route("**/v1/tenants/local/discovery/candidates/*/register", (route) => {
    scanStarted = true;
    return json(route, agent);
  });
  await page.route("**/v1/tenants/local/discovery/scans", (route) => {
    if (route.request().method() === "POST") {
      scanStarted = true;
      return json(route, {
        scan_id: "scan-e2e-1",
        tenant_id: "local",
        status: "completed",
        started_at: "2026-06-27T10:14:50Z",
        finished_at: now,
        sources: ["process", "filesystem", "mcp_config"],
        candidates_found: 1,
      });
    }
    return json(route, {
      items: scanStarted
        ? [
            {
              scan_id: "scan-e2e-1",
              tenant_id: "local",
              status: "completed",
              started_at: "2026-06-27T10:14:50Z",
              finished_at: now,
              sources: ["process", "filesystem", "mcp_config"],
              candidates_found: 1,
            },
          ]
        : [],
    });
  });
  await page.route("**/v1/tenants/local/discovery/scans/scan-e2e-1", (route) =>
    json(route, {
      scan_id: "scan-e2e-1",
      tenant_id: "local",
      status: "completed",
      started_at: "2026-06-27T10:14:50Z",
      finished_at: now,
      sources: ["process", "filesystem", "mcp_config"],
      candidates_found: 1,
    }),
  );

  await page.route("**/v1/tenants/local/entity-graph**", (route) => {
    const url = routeUrl(route);
    if (url.pathname.endsWith("/entity-graph/node")) {
      return json(
        route,
        entity360(url.searchParams.get("entity_type"), url.searchParams.get("entity_id")),
      );
    }
    return json(
      route,
      scanStarted ? graphResponse : { ...graphResponse, nodes: [], edges: [] },
    );
  });
  await page.route("**/v1/tenants/local/activity-timeline**", (route) =>
    json(route, scanStarted ? activityTimeline : { ...activityTimeline, items: [] }),
  );
  await page.route("**/v1/tenants/local/activity", (route) =>
    json(route, scanStarted ? activitySummary : { activity_sets: [] }),
  );

  await page.route("**/v1/tenants/local/policy-suggestions", (route) =>
    json(route, { items: suggestionsGenerated ? [policySuggestion] : [] }),
  );
  await page.route("**/v1/tenants/local/policy-suggestions/generate", (route) => {
    suggestionsGenerated = true;
    return json(route, { items: [policySuggestion] });
  });
  await page.route("**/v1/tenants/local/v1/policy/suggestions", (route) =>
    json(route, [
      {
        id: "pol_workspace_file_guard",
        title_en: "Protect workspace file access",
        title_th: "Protect workspace file access",
        domains: ["filesystem"],
        recommended_level: "enforce",
      },
    ]),
  );
  await page.route("**/v1/tenants/local/policies", (route) => {
    const method = route.request().method();
    if (method === "GET") {
      return json(route, policies);
    }
    if (method === "POST") {
      const nextPolicy = route.request().postDataJSON();
      policies.push(nextPolicy);
      return json(route, nextPolicy, 201);
    }
    return json(route, { error: "unsupported method" }, 405);
  });
  await page.route("**/v1/tenants/local/policies/feasibility", (route) =>
    json(route, {
      policy_id: policy.policy_id,
      requested_level: "enforce",
      achievable_level: "enforce",
      verdict: "fully_enforceable",
      per_domain: [
        {
          domain: "filesystem",
          chosen_method: "windows_process_observer",
          level: "enforce",
          reason_en: "Local process observer is active.",
          reason_th: "",
        },
      ],
      gaps: [],
      friendly_en: "This policy can be fully enforced on this device.",
      friendly_th: "",
    }),
  );
  await page.route("**/v1/tenants/local/deployment-sessions", (route) =>
    json(route, {
      id: "deploy-session-1",
      status: "ready",
      feasibility: {
        policy_id: policy.policy_id,
        requested_level: "enforce",
        achievable_level: "enforce",
        verdict: "fully_enforceable",
        per_domain: [],
        gaps: [],
        friendly_en: "Ready to deploy.",
        friendly_th: "",
      },
    }),
  );
  await page.route("**/v1/tenants/local/v1/deploy/session", (route) =>
    json(route, {
      id: "deploy-session-1",
      status: "ready",
      feasibility: {
        policy_id: policy.policy_id,
        requested_level: "enforce",
        achievable_level: "enforce",
        verdict: "fully_enforceable",
        per_domain: [],
        gaps: [],
        friendly_en: "Ready to deploy.",
        friendly_th: "",
      },
    }),
  );
  await page.route("**/v1/tenants/local/v1/deploy/session/deploy-session-1/confirm", (route) =>
    json(route, {
      policy_id: policy.policy_id,
      bindings: [
        {
          domain: "filesystem",
          method_id: "windows_process_observer",
          effective_level: "enforce",
          maturity: "beta",
        },
      ],
      fallbacks: [],
      auto_selected: true,
    }),
  );
  await page.route("**/v1/tenants/local/v1/deploy/session/deploy-session-1/apply", (route) =>
    json(route, { applied: true, policy_id: policy.policy_id }),
  );
  await page.route(/\/v1\/tenants\/local\/policies\/[^/]+\/publish$/, (route) =>
    json(route, {
      published: true,
      bundle_id: "bundle-local-1",
      build_number: 1,
    }),
  );

  await page.route("**/v1/tenants/local/telemetry/decision-logs", (route) =>
    json(route, {
      count: 1,
      decisions: [
        {
          timestamp: now,
          event_id: "decision-e2e-1",
          payload: {
            decision: "allow",
            reason: "Policy allowed source folder read after local PDP evaluation.",
            request_id: "req-e2e-1",
            matched_policy_ids: [policy.policy_id],
            latency_ms: 7,
            resource: resource.path,
          },
        },
      ],
    }),
  );
  await page.route("**/v1/tenants/local/usage/summary**", (route) =>
    json(route, usageSummary),
  );
  await page.route("**/v1/tenants/local/usage/events**", (route) =>
    json(route, usageEvents),
  );
  await page.route("**/v1/tenants/local/local-observe/refresh", (route) =>
    json(route, {
      schema_version: "local-observe-refresh.v1",
      tenant_id: "local",
      scan_id: "scan-e2e-1",
      candidates_found: 1,
      resource_events: 1,
      identity_events: 1,
      tool_events: 1,
      usage_events: 1,
      exact_usage_events: 1,
      estimated_usage_events: 0,
      capture_quality: ["exact"],
      limitations: [],
      next_steps: [],
    }),
  );

  await page.route("**/v1/tenants/local/connectors", (route) => {
    if (route.request().method() === "GET") {
      return json(route, []);
    }
    return json(route, { id: "mock-connector", ok: true });
  });
  await page.route("**/v1/tenants/local/policy-presets", (route) =>
    json(route, []),
  );
  await page.route("**/v1/tenants/local/telemetry/cost-ledger", (route) =>
    json(route, []),
  );
  await page.route("**/v1/tenants/local/telemetry/alerts", (route) =>
    json(route, []),
  );
  await page.route("**/v1/tenants/local/bundles", (route) => json(route, []));
  await page.route("**/v1/tenants/local/settings", (route) =>
    json(route, { ok: true }),
  );
  await page.route("**/v1/tenants/local/discovery/scan", (route) => {
    scanStarted = true;
    return json(route, { status: "completed", findings: [candidate] });
  });
}
