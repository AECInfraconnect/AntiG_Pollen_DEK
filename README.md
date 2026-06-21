# Pollen DEK — Open-Source Device Enforcement Kit

[![CI](https://github.com/AECInfraconnect/AntiG_Pollen_DEK/actions/workflows/ci.yml/badge.svg)](https://github.com/AECInfraconnect/AntiG_Pollen_DEK/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/tag/AECInfraconnect/AntiG_Pollen_DEK?include_prereleases)](https://github.com/AECInfraconnect/AntiG_Pollen_DEK/releases)
[![Compatibility](https://img.shields.io/badge/Compatibility-Matrix-success.svg)](contracts/COMPATIBILITY.md)

**Pollen DEK** is an Apache-2.0 runtime that **enforces and observes AI-agent, MCP,
API, and tool-call activity at the desktop/edge** — a Policy Enforcement Point
(PEP) with a local Policy Decision Point (PDP).

It runs **fully locally** with the built-in Local Admin Dashboard, or connects to
**Pollen Cloud** (commercial) for managed multi-tenant policy, observability, and
compliance. The DEK speaks **one contract** to both — switching targets changes
only the endpoint + trust store, never the enforcement code.

---

## Why

- **Enforce, don't just observe** — allow/deny/redact MCP tool calls and network
  egress against signed policy, fail-closed by default.
- **Policy your way** — Cedar (ABAC/RBAC), OPA/Rego (complex logic), OpenFGA
  (ReBAC); the router auto-selects the right engine per request.
- **Kernel-grade network control** — eBPF on Linux today; Windows WFP / macOS
  System Extension in progress. Complex rules are kept out of the kernel to avoid
  instability (kernel handles only simple, exact matches).
- **Local-first, Cloud-ready** — same schema, bundle format, and telemetry
  envelope in both modes.

## Quickstart

### Local mode (single machine, no Cloud)

```bash
# 1) start the Local Control Plane + dashboard (http://127.0.0.1:3000)
local-control-plane &

# 2) point the DEK at it and enroll
dek-cli profile set local --trusted-key <key-from-local-cp-log>
dek-cli enroll --cloud-url http://127.0.0.1:3000

# 3) run the enforcement point (PEP on :43890)
dek-core &
dek-cli doctor && dek-cli status
```

See **[docs/quickstart_local_en.md](docs/quickstart_local_en.md)** (TH: `_th`).

### Pollen Cloud mode

```bash
dek-cli profile set cloud --url https://cloud.<your-cloud-domain> --tenant-id <tenant>
dek-cli enroll --cloud-url https://cloud.<your-cloud-domain>
dek-core &
```

## Download & verify

Binaries for Linux/macOS/Windows (both x86_64 and arm64/aarch64) are on **[GitHub Releases](https://github.com/AECInfraconnect/AntiG_Pollen_DEK/releases)**.
Each asset ships with `SHA256SUMS`, GitHub Artifact Attestations (`actions/attest-build-provenance`), and a Sigstore cosign signature; verify before running:

```bash
# 1) Check SHA256SUMS
sha256sum -c SHA256SUMS

# 2) Verify Cosign Keyless Signature
cosign verify-blob --certificate <asset>.pem --signature <asset>.sig \
  --certificate-identity-regexp "https://github.com/AECInfraconnect/AntiG_Pollen_DEK/.*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" <asset>

# 3) Verify GitHub Artifact Attestation
gh attestation verify <asset> -o AECInfraconnect
```

Update in place (verifies cosign before applying, with rollback):

```bash
dek-cli update --channel beta
```

## Architecture (at a glance)

```
 Local Admin Dashboard            Pollen Cloud (commercial)
  SQLite · tenant=local            MySQL/TiDB · multi-tenant
  HTTP 127.0.0.1 · Bearer          mTLS + OAuth + SPIFFE/SPIRE
            \                         /
             \  same schema/bundle/  /
              \  telemetry + reload /
               ▼                   ▼
                ┌───────────────┐
                │   DEK (PEP)   │  profile: local | cloud
                │  enforce +    │
                │  fail-closed  │
                └───────────────┘
```

Full detail: **[ARCHITECTURE.md](ARCHITECTURE.md)**.

## Plugin / Adapter SDK

Policy engines are adapters built on **`dek-pdp-sdk`** (Apache-2.0); bundled
adapters (Cedar/OPA/OpenFGA) are feature-gated, and you can add your own without
touching the core. Transform plugins (e.g. PII redaction via `dek-pii-wasm`) and
telemetry sinks plug in the same way. See **[examples/policies](examples/policies/)**.

## Documentation

Start at **[docs/README.md](docs/README.md)** — install guides, user/developer
guides, runbooks, security model, compliance mapping, and the
[DEK↔Cloud contract](docs/contracts/pollen-cloud-dek-api.md).

## License

DEK runtime, CLI, agent, SDK, adapters, and example policies are **Apache-2.0**.
**Pollen Cloud is commercial.** See [LICENSE](LICENSE) and [NOTICE](NOTICE).
