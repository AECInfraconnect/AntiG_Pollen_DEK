import "@typespec/http";
import "@typespec/rest";
using TypeSpec.Http;
using TypeSpec.Rest;

namespace Pollek.Contract;

model ObjectMeta {
  schema_version: string;
  tenant_id: string;
  workspace_id: string;
  environment_id: string;
  created_at: string;
  updated_at: string;
  created_by: string;
  updated_by: string;
  source: string;
  status: string;
  tags: string[];
}

model TokenUsage {
  input_tokens?: int64;
  output_tokens?: int64;
  total_tokens?: int64;
  "model"?: string;
}

model BrowserAiObservationScope {
  base_name?: string;
  display_name?: string;
  browser_id?: string;
  browser_name?: string;
  candidate_id?: string;
  discovery_candidate_id?: string;
}

model AgentObservationEvent {
  event_id: string;
  tenant_id: string;
  trace_id: string;
  agent_id?: string;
  shadow_candidate_id?: string;
  tool_id?: string;
  resource_id?: string;
  surface: string;
  action: string;
  pep_type?: string;
  risk_level?: string;
  timestamp: string;
  payload_json: string;
  token_usage?: TokenUsage;
  browser_scope?: BrowserAiObservationScope;
}

enum SuggestionType {
  RestrictMcpTool,
  RequireApprovalForSensitiveResource,
  RegisterShadowAgent,
  EnforceCostBudget,
  RestrictExternalLlmProvider,
}

enum SuggestedPolicyLanguage {
  Rego,
  Cedar,
  OpenFga,
}

enum SuggestionSeverity {
  Low,
  Medium,
  High,
  Critical,
}

enum SuggestionStatus {
  Draft,
  Suggested,
  Approved,
  Dismissed,
  Applied,
}

model PolicyArtifact {
  name: string;
  content: string;
  language: SuggestedPolicyLanguage;
}

model PolicySuggestion {
  suggestion_id: string;
  tenant_id: string;
  target_agent_id?: string;
  target_resource_id?: string;
  target_tool_id?: string;
  suggestion_type: SuggestionType;
  title: string;
  summary: string;
  severity: SuggestionSeverity;
  confidence: float32;
  recommended_policy_type: SuggestedPolicyLanguage;
  recommended_pep_type: string;
  artifacts: PolicyArtifact[];
  status: SuggestionStatus;
  created_at: string;
}

model DiscoveryCandidate {
  schema_version: string;
  candidate_id: string;
  tenant_id: string;
  device_id: string;
  status: string;
  canonical_service_id: string;
  surface_group_id: string;
  authority_boundary: string;
  entity_role: string;
  duplicate_policy: string;
  control_parent_id?: string;
  grouping_reason?: string;
  observe_scope: string;
  enforce_scope: string;
  related_surfaces: DiscoveryRelatedSurface[];
  instance_count?: int32;
  matched_signature_id?: string;
  display_name: string;
  vendor?: string;
  product?: string;
  inferred_agent_type: string;
  confidence: float32;
  risk_score: int32;
  capability_tags?: string[];
  matched_signals?: unknown[];
  first_seen: string;
  last_seen: string;
  evidence?: unknown[];
  discovered_configs?: unknown[];
  discovered_endpoints?: unknown[];
  discovered_mcp_servers?: unknown[];
  suggested_registration?: unknown;
  suggested_observation_profile?: unknown;
  suggested_control_bindings?: unknown[];
  telemetry_plan?: unknown;
  labels?: Record<string>;
  scan_ids?: string[];
  last_scan_id?: string;
}

model DiscoveryRelatedSurface {
  service_id: string;
  display_name: string;
  entity_role: string;
  authority_boundary: string;
  evidence_sources: string[];
  confidence: float32;
  control_parent_id?: string;
  grouping_reason?: string;
}

model DiscoveryEnrichmentSourcePlan {
  source_id: string;
  label: string;
  allowed: boolean;
  network_access: string;
  safety: string;
}

model DiscoveryEnrichmentFact {
  fact: string;
  value: string;
  confidence: float32;
  source: string;
}

model DiscoveryEnrichmentSession {
  schema_version: "pollek.discovery.enrichment_session.v1";
  session_id: string;
  tenant_id: string;
  candidate_id: string;
  status: string;
  created_at: string;
  consent_required: boolean;
  privacy_guardrails: string[];
  local_evidence_summary: unknown;
  source_plan: DiscoveryEnrichmentSourcePlan[];
  extracted_facts: DiscoveryEnrichmentFact[];
  definition_candidate: unknown;
  accepted_sources?: string[];
  research_result?: unknown;
  learned_profile_id?: string;
}

model DiscoveryEnrichmentApproveRequest {
  accepted_sources: string[];
}

model CanonicalDiscoveryCapability {
  capability_id: string;
  candidate_id: string;
  capability_kind: string;
  name: string;
  description?: string;
  input_schema?: unknown;
  output_schema?: unknown;
  modality: string[];
  actions: string[];
  source: string;
  confidence: float32;
  risk_tags: string[];
  evidence_ids: string[];
  privacy_class: string;
}

model DiscoveredDiscoveryRelationship {
  relationship_id: string;
  subject_candidate_id: string;
  relation: string;
  object_candidate_id: string;
  confidence: float32;
  evidence_ids: string[];
}

model DiscoveryEntityCandidateV1 {
  schema_version: "pollek.discovery_entity_candidate.v1";
  candidate_id: string;
  tenant_id: string;
  device_id: string;
  entity_kind: string;
  display_name: string;
  vendor?: string;
  product?: string;
  confidence: float32;
  risk_score: int32;
  status: string;
  capabilities: CanonicalDiscoveryCapability[];
  evidence: unknown[];
  relationships: DiscoveredDiscoveryRelationship[];
  suggested_registration: unknown;
  suggested_control_bindings: unknown[];
  privacy_profile: string;
  performance_cost_class: string;
  first_seen: string;
  last_seen: string;
}

model DiscoveryCapabilityInventoryResponse {
  schema_version: "discovery-capability-inventory.v1";
  candidate_id: string;
  entity: DiscoveryEntityCandidateV1;
  capabilities: CanonicalDiscoveryCapability[];
  relationships: DiscoveredDiscoveryRelationship[];
  retrieval_status: string;
  source: string;
  privacy_note?: string;
}

model DiscoveryEntityListResponse {
  schema_version: "discovery-entity-list.v1";
  entities: DiscoveryEntityCandidateV1[];
  next_cursor?: int64;
  total: int64;
}

model AgentObservationEventListResponse {
  items: AgentObservationEvent[];
}

model CostLedgerEntry {
  event_id: string;
  agent_id: string;
  provider: string;
  "model"?: string;
  input_tokens: int64;
  output_tokens: int64;
  total_tokens: int64;
  input_cost: float32;
  output_cost: float32;
  total_cost: float32;
  currency: string;
  estimated: boolean;
  timestamp: string;
}

model AgentTrustScore {
  schema_version: "agent-trust-score.v1";
  agent_id: string;
  score: float32;
  reasons: string[];
}

model BudgetPolicy {
  schema_version: "budget-policy.v1";
  agent_id: string;
  daily_cost_cap_usd: float32;
  daily_token_cap: int64;
}

model SandboxConfig {
  schema_version: "sandbox-config.v1";
  timeout_ms: int64;
  max_memory_mb: int32;
  enable_network: boolean;
}

model CostLedgerListResponse {
  items: CostLedgerEntry[];
}

model AgentUsageSummary {
  cost: float32;
  tokens: int64;
}

model CostSummaryResponse {
  schema_version: "cost-summary.v1";
  tenant_id: string;
  period: string;
  total_estimated_cost_usd: float32;
  total_tokens: int64;
  provider_breakdown: Record<float32>;
  agent_breakdown: Record<float32>;
  agent_token_breakdown: Record<int64>;
  agent_usage_breakdown: Record<AgentUsageSummary>;
}

model PolicySuggestionListResponse {
  items: PolicySuggestion[];
}

model DiscoveryCandidateListResponse {
  schema_version: "agent-discovery-candidate-list.v1";
  candidates: DiscoveryCandidate[];
  items?: DiscoveryCandidate[];
  next_cursor?: int64;
  total: int64;
}

model PolicyCoverageSummary {
  status: string;
  policy_ids: string[];
  missing_controls: MissingControl[];
  last_simulated_at?: string;
  last_enforced_at?: string;
  confidence: float32;
}

model MissingControl {
  control_type: string;
  reason: string;
  recommended_policy_type: string;
  recommended_pep_type: string;
}

model PepBinding {
  pep_type: string;
  deployment_status: string;
  rule_ids: string[];
  capabilities: string[];
  limitations: string[];
}

model SchemaFingerprint {
  input_schema_hash: string;
  output_schema_hash?: string;
  descriptor_hash?: string;
  previous_descriptor_hash?: string;
  drift_status: string;
}

model Tool {
  meta: ObjectMeta;
  tool_id: string;
  mcp_server_id?: string;
  name: string;
  description?: string;
  input_schema: unknown;
  output_schema?: unknown;
  side_effect_level: string;
  data_access_level: string;
  risk_level: string;
  category: string;
  source: string;
  declared_by_agent_ids: string[];
  observed_by_agent_ids: string[];
  reachable_resource_ids: string[];
  required_entitlements: string[];
  schema_fingerprint: SchemaFingerprint;
  policy_coverage: PolicyCoverageSummary;
  pep_bindings: PepBinding[];
  last_seen_at?: string;
  first_seen_at?: string;
  observation_count_24h: int64;
  deny_count_24h: int64;
  allow_count_24h: int64;
}

model Resource {
  meta: ObjectMeta;
  resource_id: string;
  resource_type: string;
  name: string;
  uri: string;
  classification: string;
  owner_entity_id?: string;
  attributes: Record<unknown>;
  data_boundary: string;
  data_tags: string[];
  pii_types: string[];
  secret_types: string[];
  allowed_actions: string[];
  observed_actions: string[];
  observed_by_agent_ids: string[];
  reachable_via_tool_ids: string[];
  policy_coverage: PolicyCoverageSummary;
  pep_bindings: PepBinding[];
  first_seen_at?: string;
  last_seen_at?: string;
  access_count_24h: int64;
  violation_count_24h: int64;
  cost_relevant: boolean;
}

model ToolListResponse {
  items: Tool[];
}

model ResourceListResponse {
  items: Resource[];
}

@tag("Registry")
@route("/v1/tenants/{tenant_id}/registry/tools")
interface ToolsApi {
  @doc("List tools")
  @get listTools(@path tenant_id: string, ...PollekHeaders): ToolListResponse | PollekError;

  @doc("Create a tool")
  @post createTool(@path tenant_id: string, ...PollekHeaders, @body body: Tool): Tool | PollekError;
}

@tag("Registry")
@route("/v1/tenants/{tenant_id}/registry/resources")
interface ResourcesApi {
  @doc("List resources")
  @get listResources(@path tenant_id: string, ...PollekHeaders): ResourceListResponse | PollekError;

  @doc("Create a resource")
  @post createResource(@path tenant_id: string, ...PollekHeaders, @body body: Resource): Resource | PollekError;
}

@tag("Observation")
@route("/v1/tenants/{tenant_id}/observations")
interface ObservationApi {
  @doc("Ingest observation events")
  @route("events")
  @post ingestEvents(@path tenant_id: string, ...PollekHeaders, @body body: AgentObservationEvent[]): void | PollekError;

  @doc("Get observations for an agent")
  @route("agents/{agent_id}")
  @get getAgentObservations(@path tenant_id: string, @path agent_id: string, ...PollekHeaders): AgentObservationEventListResponse | PollekError;

  @doc("Get observations for a tool")
  @route("tools/{tool_id}")
  @get getToolObservations(@path tenant_id: string, @path tool_id: string, ...PollekHeaders): AgentObservationEventListResponse | PollekError;

  @doc("Get observations for a resource")
  @route("resources/{resource_id}")
  @get getResourceObservations(@path tenant_id: string, @path resource_id: string, ...PollekHeaders): AgentObservationEventListResponse | PollekError;

  @doc("Get cost summary")
  @route("costs")
  @get getCosts(@path tenant_id: string, ...PollekHeaders): CostSummaryResponse | PollekError;

  @doc("Get policy coverage summary")
  @route("coverage")
  @get getCoverage(@path tenant_id: string, ...PollekHeaders): PolicyCoverageSummary | PollekError;

  @doc("Get trust score for an agent")
  @route("trust/{agent_id}")
  @get getAgentTrustScore(@path tenant_id: string, @path agent_id: string, ...PollekHeaders): AgentTrustScore | PollekError;
}

@tag("Discovery")
@route("/v1/tenants/{tenant_id}/discovery")
interface DiscoveryApi {
  @doc("Start a discovery scan")
  @route("scans")
  @post scan(@path tenant_id: string, ...PollekHeaders, @body body: unknown): unknown | PollekError;

  @doc("List discovery candidates")
  @route("candidates")
  @get listCandidates(@path tenant_id: string, ...PollekHeaders): DiscoveryCandidateListResponse | PollekError;

  @doc("List canonical discovery entities derived from discovered candidates.")
  @route("entities")
  @get listDiscoveryEntities(@path tenant_id: string, ...PollekHeaders): DiscoveryEntityListResponse | PollekError;

  @doc("Get canonical capabilities for a discovery candidate without invoking tools or reading resources.")
  @route("candidates/{candidate_id}/capabilities")
  @get getCandidateCapabilities(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders): DiscoveryCapabilityInventoryResponse | PollekError;

  @doc("Start a safe local definition enrichment session for a discovery candidate.")
  @route("candidates/{candidate_id}/enrichment/start")
  @post startCandidateEnrichment(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders, @body body: unknown): DiscoveryEnrichmentSession | PollekError;

  @doc("Get a discovery enrichment session.")
  @route("enrichment/{session_id}")
  @get getCandidateEnrichment(@path tenant_id: string, @path session_id: string, ...PollekHeaders): DiscoveryEnrichmentSession | PollekError;

  @doc("Approve the safe source plan for a discovery enrichment session.")
  @route("enrichment/{session_id}/approve")
  @post approveCandidateEnrichment(@path tenant_id: string, @path session_id: string, ...PollekHeaders, @body body: DiscoveryEnrichmentApproveRequest): DiscoveryEnrichmentSession | PollekError;

  @doc("Persist the learned local discovery profile from an enrichment session.")
  @route("enrichment/{session_id}/submit")
  @post submitCandidateEnrichment(@path tenant_id: string, @path session_id: string, ...PollekHeaders): DiscoveryEnrichmentSession | PollekError;

  @doc("Retrieve and persist canonical capabilities for a discovery candidate.")
  @route("candidates/{candidate_id}/retrieve-capabilities")
  @post retrieveCandidateCapabilities(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders): DiscoveryCapabilityInventoryResponse | PollekError;

  @doc("Register a candidate")
  @route("candidates/{candidate_id}/register")
  @post registerCandidate(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders): void | PollekError;

  @doc("Ignore a candidate")
  @route("candidates/{candidate_id}/ignore")
  @post ignoreCandidate(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders): void | PollekError;

  @doc("Observe a candidate only")
  @route("candidates/{candidate_id}/observe-only")
  @post observeCandidate(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders): void | PollekError;

  @doc("Block a candidate")
  @route("candidates/{candidate_id}/block")
  @post blockCandidate(@path tenant_id: string, @path candidate_id: string, ...PollekHeaders): void | PollekError;
}

@tag("PolicySuggestions")
@route("/v1/tenants/{tenant_id}/policy-suggestions")
interface PolicySuggestionsApi {
  @doc("Generate policy suggestions")
  @route("generate")
  @post generate(@path tenant_id: string, ...PollekHeaders): void | PollekError;

  @doc("List policy suggestions")
  @get listSuggestions(@path tenant_id: string, ...PollekHeaders): PolicySuggestionListResponse | PollekError;

  @doc("Get a specific policy suggestion")
  @route("{suggestion_id}")
  @get getSuggestion(@path tenant_id: string, @path suggestion_id: string, ...PollekHeaders): PolicySuggestion | PollekError;

  @doc("Create a draft from a suggestion")
  @route("{suggestion_id}/create-draft")
  @post createDraft(@path tenant_id: string, @path suggestion_id: string, ...PollekHeaders): void | PollekError;

  @doc("Dismiss a policy suggestion")
  @route("{suggestion_id}/dismiss")
  @post dismiss(@path tenant_id: string, @path suggestion_id: string, ...PollekHeaders): void | PollekError;

  @doc("Simulate a policy suggestion")
  @route("{suggestion_id}/simulate")
  @post simulate(@path tenant_id: string, @path suggestion_id: string, ...PollekHeaders): void | PollekError;

  @doc("Approve a policy suggestion")
  @route("{suggestion_id}/approve")
  @post approve(@path tenant_id: string, @path suggestion_id: string, ...PollekHeaders): void | PollekError;
}

@tag("PolicyPresets")
@route("/v1/tenants/{tenant_id}/policy-presets")
interface PolicyPresetsApi {
  @doc("List policy presets")
  @get listPresets(@path tenant_id: string, ...PollekHeaders): unknown | PollekError;

  @doc("Get a specific policy preset")
  @route("{preset_id}")
  @get getPreset(@path tenant_id: string, @path preset_id: string, ...PollekHeaders): unknown | PollekError;

  @doc("Preview a policy preset")
  @route("{preset_id}/preview")
  @post previewPreset(@path tenant_id: string, @path preset_id: string, ...PollekHeaders, @body body: unknown): unknown | PollekError;

  @doc("Create a draft from a preset")
  @route("{preset_id}/create-draft")
  @post createDraft(@path tenant_id: string, @path preset_id: string, ...PollekHeaders, @body body: unknown): unknown | PollekError;

  @doc("Simulate a policy preset")
  @route("{preset_id}/simulate")
  @post simulatePreset(@path tenant_id: string, @path preset_id: string, ...PollekHeaders, @body body: unknown): unknown | PollekError;
}

@tag("PepCapabilities")
@route("/v1/tenants/{tenant_id}/pep-capabilities")
interface PepCapabilitiesApi {
  @doc("List PEP capabilities")
  @get listCapabilities(@path tenant_id: string, ...PollekHeaders): unknown | PollekError;

  @doc("Check PEP capabilities")
  @route("check")
  @post checkCapabilities(@path tenant_id: string, ...PollekHeaders, @body body: unknown): unknown | PollekError;
}
