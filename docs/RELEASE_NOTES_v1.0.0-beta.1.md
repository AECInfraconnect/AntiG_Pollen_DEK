# Release Notes: v1.0.0-beta.1 (OSS Launch)

Pollen DEK v1.0.0-beta.1 represents the first official Open Source Software (OSS) candidate, providing enterprise-grade distributed authorization natively on the edge.

## Highlights
- **Cosign Integration:** The release workflow now enforces sigstore keyless OIDC signatures for all artifacts. `dek-updater` fully verifies the `cosign` cryptographic signature prior to executing atomic swaps, blocking maliciously signed or foreign binaries.
- **Fail-Closed Auto-Update:** The updater ensures robust SHA256 matches and cosign verification before writing the new executable payload, maintaining service uptime through `apply_with_rollback`.
- **First Run Documentation:** We've introduced `FIRST_RUN_UX.md` for a streamlined quickstart guide covering CA compilation, mock enrollment, and local policy testing.
- **Socket Resilience:** Addressed Windows ephemeral port exhaustion under heavy scale by swapping default HTTP bindings to native `TcpSocket` implementations explicitly tuned with `SO_REUSEADDR`.

## Compliance
- This release is compliant with our internal matrix validations across A to K evaluation sequences.
- SPDX licensing headers will be progressively rolled out in P1 (OSS ecosystem alignment).
- All Linux, Windows, and macOS targets are verifiably built using `cargo-auditable`.

## Usage
To download and install the agent manually, fetch the appropriate OS archive below. For automated deployment, use `dek-updater upgrade`.
