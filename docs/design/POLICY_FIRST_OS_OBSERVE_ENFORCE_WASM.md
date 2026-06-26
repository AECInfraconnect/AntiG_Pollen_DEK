# Policy-First OS Observe/Enforce and WASM Design

## Scope Boundary

Open source Pollek remains local-first: Local Control Plane, Local Dashboard, local telemetry spool, local capability detection, and local policy evaluation. Pollek Cloud is the commercial central control plane that aggregates telemetry from many Local Control Planes and sends signed policies back over a secure channel.

The shared contract boundary is:

- Local Dashboard reads the same `local-capability-snapshot.v2`, `security-coverage.v1`, and friendly message reason codes that Cloud Dashboard can aggregate.
- Local Control Plane must remain useful without Cloud enrollment.
- Cloud enrollment requires a registered device/agent identity binding: SPIFFE workload identity plus OAuth/OIDC authorization metadata. Token secrets are never part of dashboard or contract payloads.

## OS Capability Strategy

Pollek must not claim real enforcement unless a concrete local PEP is installed, permitted, and warm-checked.

Windows:

- Observe: ETW providers for process/network telemetry where available.
- Enforce: Windows Filtering Platform for network egress. Real blocking requires a service/callout path and administrator-controlled setup.
- Contract status: `needs_install`, `needs_permission`, `available`, or `failed`; never silently `Ok(())`.

Linux:

- Observe: eBPF tracepoints/kprobes where available; fanotify for filesystem notifications.
- Enforce: eBPF/TC or LSM where installed and loaded; fanotify permission events for filesystem decisions where privileges allow.
- Contract status should distinguish root/capability requirements from simulator-only data.

macOS:

- Observe/Enforce: Network Extension for network path control, Endpoint Security for process/file authorization events where entitlement and user approval exist.
- Contract status should expose entitlement or permission requirements as setup actions.

Cross-platform fallback:

- MCP stdio wrapper and MCP HTTP proxy are the practical first enforcement points for agent tool calls.
- Browser extension is the practical browser AI session observation/enforcement point.
- Egress simulator is always labeled `simulator_only` and never produces `enforced_for_real = true`.
- Cross-OS demo profiles are fixture-only and opt-in. They are disabled unless
  `POLLEK_ENABLE_DEMO_PROFILES=1` is set and the request explicitly asks for a
  `demo_os`. Fixture snapshots are marked with `contract.reason_code=demo_fixture`
  and `device_id=demo_*`.
- Real host capability snapshots never use demo profiles unless explicitly
  requested, and demo reads do not replace the latest real host snapshot.

## WASM Usage

WASM is a good fit for deterministic, portable policy evaluation and plugin-style transformation where host capabilities are explicit:

- OPA/Rego compiled to WASM for local decisions.
- WASM plugin ABI for redaction, classification, scoring, and normalization.
- WASI/component-style interfaces for safe, explicit host calls when a plugin needs registry, clock, hash, or telemetry APIs.

WASM is not a direct replacement for OS PEPs. It can decide; OS/MCP/browser control methods enforce.

The current capability snapshot advertises `wasm_policy_evaluator` as `warn` level unless paired with an enforcement-capable method. This avoids overstating sandboxed PDP evaluation as syscall or network blocking.

The current security coverage evidence maps `decision_engine` from the selected
control method instead of assuming OPA for every real enforcement path. MCP tool
control maps to Cedar-style authorization evidence, prompt/content transforms map
to plugin/WASM evidence, relationship engines map to OpenFGA, and network plans
remain OPA/WASM-precomputed unless a more specific engine is wired.

## LLM01 and LLM05 Guardrails

Content Guard now has two local paths:

- Request-side scan for LLM01 prompt injection and sensitive-data exfiltration.
- Response-side scan for LLM05 improper output handling before a tool result is
  returned to an agent.

The local guard performs bounded normalization: zero-width stripping, casefolding
with common homoglyph folding, HTML/entity decoding, percent decoding, and
bounded base64 candidate decoding. Rules are weighted and produce `score`,
`confidence`, `categories`, and `normalization_steps` while keeping the legacy
`injection_detected` and `recommended` fields for compatibility.

## Identity Binding

Registered agents should bind to:

- A stable local `device_id`.
- A SPIFFE ID when workload identity is available.
- OAuth/OIDC metadata when Pollek Cloud enrollment is enabled.
- Token binding references only: issuer, audience, expiry, storage location, and subject hint. No raw token material in contracts or dashboard cards.

This model lets Local Dashboard remain independent while Pollek Cloud can trace workload activity across device, agent, identity, resource, and tool telemetry.

## Primary References

- Windows Filtering Platform: https://learn.microsoft.com/en-us/windows/win32/fwp/windows-filtering-platform-start-page
- Windows Event Tracing: https://learn.microsoft.com/en-us/windows/win32/etw/about-event-tracing
- Linux BPF: https://docs.kernel.org/bpf/
- Linux fanotify: https://man7.org/linux/man-pages/man7/fanotify.7.html
- Apple Network Extension: https://developer.apple.com/documentation/networkextension
- Apple Endpoint Security: https://developer.apple.com/documentation/endpointsecurity
- SPIFFE overview: https://spiffe.io/docs/latest/spiffe-about/overview/
- SPIRE concepts: https://spiffe.io/docs/latest/spire-about/spire-concepts/
- Wasmtime security: https://docs.wasmtime.dev/security.html
- WASI: https://wasi.dev/
