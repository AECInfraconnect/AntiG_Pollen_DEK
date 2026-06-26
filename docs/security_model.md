# Pollek Local Enforcement Kit Security Model

This document outlines the security architecture and threat models addressed by Pollek Local Enforcement Kit `v1.0.0-beta`.

## 1. Supply Chain Security

Pollek Local Enforcement Kit ensures cryptographic trust from compilation to execution:

- **Reproducible Builds**: Built using GitHub Actions with `cargo auditable` provenance.
- **SBOM**: Full CycloneDX Software Bill of Materials provided with every release.
- **Signatures**: Binaries and installers are signed using Sigstore Cosign (keyless OIDC). Users should verify the signature against the release certificates before installation.

## 2. Secure Cloud Communication

Communication between Local Enforcement Kit and Pollek Cloud (or Mock-Cloud) is heavily secured:

- **Enrollment Phase**: Performed over TLS 1.2+ using a short-lived device code.
- **Active Phase (mTLS)**: Once enrolled, all subsequent telemetry, config pulls, and bundle synchronizations occur over strictly authenticated mTLS.
- **SPIFFE/SPIRE (Roadmap)**: Full integration with SPIRE for identity issuance and rotation is planned. The current beta uses mock SPIFFE certificates simulating this workflow.

## 3. Dynamic Policy Bundle Security

Local Enforcement Kit enforces policies dynamically through signed WebAssembly (WASM) bundles.

- **Tamper Evidence**: Every bundle manifest contains SHA-256 hashes of the policies and WASM artifacts.
- **Signatures**: The entire bundle manifest is cryptographically signed by the Cloud control plane. Local Enforcement Kit rejects any bundle with a mismatched or missing signature.
- **Monotonic Versioning**: Rollback attacks are prevented. Local Enforcement Kit rejects bundles with a `version` less than or equal to the currently active bundle version.
- **Isolation**: Policies are compiled into WASI modules and executed inside Wasmtime with strict CPU fuel limits, memory limits, and no system access.

## 4. Fallback and Resilience

When Local Enforcement Kit cannot reach the Cloud (e.g., network outage or DDoS):

- **Last Known Good**: Local Enforcement Kit will continue enforcing the latest verified policy bundle in memory.
- **Fail-Closed Option**: If specifically configured or if the bundle expires during an extended outage, Local Enforcement Kit falls back to a strict "Default Deny" mode to prevent unprotected traffic egress.

## 5. Local State Protection

Local Enforcement Kit stores its bootstrap identity and telemetry spool locally.

- **Windows**: Encrypted using DPAPI.
- **macOS**: Encrypted and stored in the Keychain.
- **Linux**: Secured using system file permissions (e.g., `0600`) and optional Secret Service integration.

## 6. Known Beta Scope Limitations

- During the `v1.0.0-beta` phase, mock keys are used for testing scenarios. To deploy securely, ensure you do not use `--dev-fixed-keys` in production environments.
- Windows WFP and macOS NetworkExtension enforcement are beta. They are reported
  as real `Enforce` only when the local component is installed, approved/elevated,
  and the active warm-check passes.
- Browser AI session enforcement requires the Pollek browser extension. Window,
  title, process, or SNI signals are observation evidence only.
- Cross-OS demo profiles are fixture data only. They are disabled by default,
  marked with `contract.reason_code=demo_fixture`, and must not be used as
  compliance evidence.

## 7. Local Content and Output Guarding

The local MCP proxy applies request-side and response-side content guard checks:

- Request-side checks focus on prompt injection, policy override attempts, and
  sensitive-data exfiltration indicators.
- Response-side checks focus on LLM05 improper output handling, including secret
  echo, unsafe HTML/Markdown injection, and hidden prompt leakage before a tool
  response is returned to an agent.
- Guard output includes score, confidence, categories, and normalization steps
  while retaining the legacy allow/redact/deny recommendation.
