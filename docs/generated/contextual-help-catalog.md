# Generated Contextual Help Catalog

This file is generated from `apps/local-admin-dashboard/src/data/contextual-help.compact.json`.
Edit the compact catalog, then run `node scripts/docs/build-contextual-help-docs.mjs`.

## Dashboard overview

Topic ID: `overview.dashboard`

Use this page to confirm the current device, local mode, registered entity counts, observation sources, and control capability readiness.

- Start here after launching the Local Control Plane.
- If a capability shows setup needed, open the related setup action before expecting enforcement.
- Entity counts come from registry endpoints; capability readiness comes from the local capability snapshot.

Source: `docs/FIRST_RUN_UX.md#dashboard-overview`
Related: `capability.control_methods`, `discovery.auto_scan`

## Current device

Topic ID: `overview.current_device`

This card identifies the local device and OS posture that Pollek is observing and enforcing from.

- Device ID and OS properties are local-control-plane data.
- Privilege level affects which OS control methods can enforce.
- Enterprise Cloud can aggregate many devices, but this card describes only this local node.

Source: `docs/design/POLICY_FIRST_OS_OBSERVE_ENFORCE_WASM.md#local-capabilities`
Related: `capability.control_methods`

## Control capabilities

Topic ID: `capability.control_methods`

Control methods are OS, wrapper, proxy, or protocol integration points that can observe or enforce policy.

- Available means the method is installed and warmed up on this device.
- Max level shows the strongest behavior this method can currently provide.
- A policy can only enforce where at least one matching method can enforce.

Source: `docs/design/POLICY_FIRST_OS_OBSERVE_ENFORCE_WASM.md#control-methods`
Related: `policy.deploy`

## Auto Discovery deep scan

Topic ID: `discovery.auto_scan`

Deep Scan collects local metadata from known safe discovery sources to find AI agents, tools, resources, and model endpoints.

- A deep scan can take longer because it waits for slower sources to finish.
- Pollek records metadata and evidence pointers, not private file contents.
- Review candidates before relying on policy suggestions for newly discovered agents.

Source: `docs/design/AUTO_DISCOVERY_COVERAGE_EXPANSION.md#deep-scan`
Related: `entity.agent`, `entity.known_capabilities`

## Discovery candidate

Topic ID: `discovery.candidate`

A candidate is an observed entity that Pollek can explain and optionally register as a governed record.

- Confidence combines matched signals such as process names, configs, windows, endpoints, or known definitions.
- Registered candidates become durable registry records.
- Capabilities are detected from local evidence and compared with reference definitions.

Source: `docs/design/AUTO_DISCOVERY_COVERAGE_EXPANSION.md#candidate-model`
Related: `entity.reference_intel`, `entity.known_capabilities`

## Agent record

Topic ID: `entity.agent`

An agent record represents a local or cloud AI workload that can use tools, access resources, and produce telemetry.

- Identity fields should prefer SPIFFE or token-bound identities when available.
- Reference intel enriches context, but only local evidence proves activity.
- Use related records to inspect governed tools, resources, policies, and activity.

Source: `docs/design/identity-binding-and-cloud-control.md#agent-identity`
Related: `entity.reference_intel`, `entity.known_capabilities`

## Tool record

Topic ID: `entity.tool`

A tool record describes a callable function, MCP tool, wrapper action, API, or local integration point an agent can use.

- Tool ownership links the tool to the agent or server that exposes it.
- Call counts and last used values come from telemetry or registry aggregation.
- Tool output should be guarded before returning to the agent or host.

Source: `docs/DEFINITION_FILE.md#tools`
Related: `entity.resource`, `policy.deploy`

## Data resource record

Topic ID: `entity.resource`

A resource record describes local files, folders, cloud drives, APIs, databases, tables, or other data sources.

- Prefer exact local metadata such as file path, folder path, host, database, schema, or table when the OS/source can provide it.
- Sensitivity is advisory until confirmed by local evidence or policy classification.
- Use activity to verify which agent accessed the resource.

Source: `docs/RESOURCE_TRACE_DEPTH.md#resource-detail-depth`
Related: `activity.timeline`

## Policy record

Topic ID: `entity.policy`

A policy record defines the intended governance behavior and tracks deployment, engine, mode, and affected entities.

- Policy status and deployment history are evidence fields.
- Mode explains whether the policy observes, asks, warns, or enforces.
- Use the relationship map to confirm affected agents, tools, and resources.

Source: `docs/POLICY_FIRST_FLOW.md#policy-records`
Related: `policy.deploy`, `activity.timeline`

## Reference intel

Topic ID: `entity.reference_intel`

Reference intel is curated external context matched from observed names, vendors, hosts, URIs, paths, or protocols.

- Reference intel helps classify and explain a known entity.
- It is not proof that access occurred and must not drive allow or deny decisions by itself.
- Source labels and reviewed dates show where the summary came from.

Source: `docs/reference-intel-definitions.md#contract`
Related: `entity.known_capabilities`

## Known capability checklist

Topic ID: `entity.known_capabilities`

Known capabilities compare standard expected behavior from reference definitions with what this device actually observed.

- Green detected items require local evidence, registry values, telemetry, or wrapper/proxy observations.
- Not observed yet means the entity may support it, but this device has not seen proof.
- Use this checklist to decide what additional probes or integrations are worth enabling.

Source: `docs/reference-intel-definitions.md#display-rules`
Related: `discovery.auto_scan`

## Policy suggestions

Topic ID: `policy.suggestions`

Suggestions are generated from discovery, observed behavior, capability readiness, and policy templates.

- Review target agents and feasibility before deployment.
- A suggestion is not active policy until it is deployed or converted into a policy record.
- Setup requirements explain why enforcement may be limited on this device.

Source: `docs/POLICY_FIRST_FLOW.md#policy-suggestions`
Related: `policy.deploy`

## Deploy or apply policy

Topic ID: `policy.deploy`

Deployment binds policy intent to the available control methods on this local device.

- The Local Control Plane chooses the best available method for each domain.
- If enforcement is unavailable, Pollek should explain the gap instead of silently downgrading.
- Deployment evidence should appear in policy details and activity.

Source: `docs/design/policy-enforcement-flows.md#deployment`
Related: `capability.control_methods`, `activity.timeline`

## Activity timeline

Topic ID: `activity.timeline`

Activity shows chronological evidence across agents, tools, resources, policies, decisions, PEP/PDP, cost, and traces.

- Use filters to isolate a single entity or decision outcome.
- Exact usage and cost should come from wrappers, proxies, provider usage fields, or logs before estimates.
- Trace IDs connect local activity to Cloud aggregation when Enterprise Cloud is enabled.

Source: `docs/COST_TOKEN_USAGE_RESEARCH.md#exact-first`
Related: `entity.resource`, `entity.policy`

