# Pollen DEK Security Model

This document outlines the security architecture and threat models addressed by Pollen DEK `v1.0.0-beta`.

## 1. Supply Chain Security

Pollen DEK ensures cryptographic trust from compilation to execution:
- **Reproducible Builds**: Built using GitHub Actions with `cargo auditable` provenance.
- **SBOM**: Full CycloneDX Software Bill of Materials provided with every release.
- **Signatures**: Binaries and installers are signed using Sigstore Cosign (keyless OIDC). Users should verify the signature against the release certificates before installation.

## 2. Secure Cloud Communication

Communication between DEK and Pollen Cloud (or Mock-Cloud) is heavily secured:
- **Enrollment Phase**: Performed over TLS 1.2+ using a short-lived device code.
- **Active Phase (mTLS)**: Once enrolled, all subsequent telemetry, config pulls, and bundle synchronizations occur over strictly authenticated mTLS.
- **SPIFFE/SPIRE (Roadmap)**: Full integration with SPIRE for identity issuance and rotation is planned. The current beta uses mock SPIFFE certificates simulating this workflow.

## 3. Dynamic Policy Bundle Security

DEK enforces policies dynamically through signed WebAssembly (WASM) bundles.
- **Tamper Evidence**: Every bundle manifest contains SHA-256 hashes of the policies and WASM artifacts.
- **Signatures**: The entire bundle manifest is cryptographically signed by the Cloud control plane. DEK rejects any bundle with a mismatched or missing signature.
- **Monotonic Versioning**: Rollback attacks are prevented. DEK rejects bundles with a `version` less than or equal to the currently active bundle version.
- **Isolation**: Policies are compiled into WASI modules and executed inside Wasmtime with strict CPU fuel limits, memory limits, and no system access.

## 4. Fallback and Resilience

When DEK cannot reach the Cloud (e.g., network outage or DDoS):
- **Last Known Good**: DEK will continue enforcing the latest verified policy bundle in memory.
- **Fail-Closed Option**: If specifically configured or if the bundle expires during an extended outage, DEK falls back to a strict "Default Deny" mode to prevent unprotected traffic egress.

## 5. Local State Protection

DEK stores its bootstrap identity and telemetry spool locally.
- **Windows**: Encrypted using DPAPI.
- **macOS**: Encrypted and stored in the Keychain.
- **Linux**: Secured using system file permissions (e.g., `0600`) and optional Secret Service integration.

## 6. Known Beta Scope Limitations

- During the `v1.0.0-beta` phase, mock keys are used for testing scenarios. To deploy securely, ensure you do not use `--dev-fixed-keys` in production environments.
- Windows and macOS network-level egress enforcement remains opt-in (via proxies) rather than transparent kernel-level enforcement.
