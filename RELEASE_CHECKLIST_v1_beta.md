# Pollen DEK v1.0.0-beta Release Checklist

## A. Version and Tagging
- [ ] Ensure all crates have version bumped to `1.0.0-beta.1`.
- [ ] Tag the commit with `v1.0.0-beta.1`.

## B. Pre-Release Gates
- [ ] Pass `cargo clippy --workspace -- -D warnings` (zero unwrap/expect in prod).
- [ ] Pass `cargo test --workspace` (unit and contract tests).
- [ ] Pass `cargo audit` and `cargo deny check` (Security Gates).
- [ ] Pass `gitleaks detect` (No leaked secrets).

## C. Functional Acceptance (Mock Cloud Parity)
- [ ] Network Guardrail enforces correctly (`malicious.example.com` blocked).
- [ ] Strict-Deny (Fail-Closed) mode triggers upon network sync failure.
- [ ] SPIRE Identity rotation returns honest error if unsupported.
- [ ] Human-in-the-loop Obligations (Approvals Queue) cycle completes (request -> approve -> bundle sync -> allowed).
- [ ] Telemetry events successfully ingest into Cloud Decision Logs.

## D. Reliability
- [ ] Panic-free locking verified (`lock_safe()` used everywhere instead of `unwrap()`).
- [ ] Offline telemetry limits respect max rows (10,000 limit).

## E. Performance
- [ ] `cargo bench -p dek-cedar` demonstrates `< 2ms` evaluation latency (P95).
- [ ] WASM Instance Pooling functions within max memory limits.

## F. Installers
- [ ] Windows Installer (`.ps1`) works and registers DEK as service.
- [ ] macOS Installer (`.sh`) correctly drops LaunchDaemon.
- [ ] Linux Installer (`.sh`) sets up systemd properly.

## G. Soak Testing
- [ ] Pass 24-hour soak test without memory leaks or crash loops.

## H. Artifacts and Release Cutover
- [ ] Build release binaries (`cargo build --release`).
- [ ] Generate SBOM (`cargo sbom`).
- [ ] Sign binaries and installers.
- [ ] Publish Release Notes.
