# ADR-0001: Prompt Guard Response Data Plane

## Status

Accepted

## Context

Prompt Guard runs in the local product boundary first: MCP proxy, local wrappers,
and future browser or SDK PEPs must be able to scan model-bound requests and
agent-bound responses without requiring Pollek Cloud. Pollek Cloud can still
push policy and guard configuration through the shared contract path when
Enterprise Cloud mode is connected.

The implementation has two possible output-guard paths:

- in-process response scanning inside `/mcp`
- a standard response filter endpoint shared by every PEP

## Decision

Use `/v1/filter/response` as the standard response data plane.

`/mcp` may call the same `GuardPipeline` in process for fast local decisions,
but every PEP that handles tool output, browser output, model output, or wrapper
output should be able to call `/v1/filter/response` before returning content to
an agent or host UI.

## Consequences

- Output handling is consistent across MCP, browser extension, SDK, and wrapper
  integrations.
- Prompt Guard telemetry uses the shared `guard_incident` telemetry envelope, so
  Local Dashboard and Pollek Cloud can aggregate the same evidence.
- Policy presets render `guard-pipeline-config.v1` with a `data_plane` value of
  `/v1/filter/response`.
- `/mcp` remains optimized for local operation, but it must not become the only
  place where response guard logic exists.

## Control Mode Mapping

| Preset control mode | Guard mode |
| --- | --- |
| `observe` | `observe` |
| `warn` | `warn` |
| `approval` | `warn` |
| `enforce` | `enforce` |
| `strict_deny` | `strict_deny` |

`approval` maps to `warn` because Prompt Guard itself does not own the approval
workflow. Approval remains an obligation handled by policy and workflow PEPs,
while Guard keeps the response filter deterministic.
