# Compliance Control Mapping

Pollen DEK satisfies several compliance frameworks (NIST 800-53, SOC2, PDPA, HIPAA) through its edge-enforcement architecture, audit logs, and telemetry.

| Control (framework) | DEK evidence | Source |
|---|---|---|
| **AC-3 Access Enforcement (NIST 800-53)** | Every decision (allow/deny + reason + policy_id) | `TelemetryEvent::Decision` -> decision-logs |
| **AC-4 Information Flow (network egress)** | Network guardrail enforce/block events | `os_guardrail`/`ebpf_guardrail` |
| **AU-9 Protection of Audit Info** | Hash-chain tamper-evidence (prev_digest+seq) | `dek-policy-syncer::audit` |
| **AU-2/AU-3 Audit Events** | Bundle sync/reload/rotation/unsigned-push | Audit events |
| **SC-12 Key Management** | Key rotation chain-of-trust + trusted-keys | `keys.rs` rotation |
| **SC-13 Crypto Protection** | ed25519 bundle sig + mTLS (rustls) | Bundle-sync + transport |
| **SI-7 Software Integrity** | TUF + cosign verify before apply | Updater + bundle verify |
| **PDPA/HIPAA** | Minimum necessary access + per-decision principal | Decision-logs |
| **SOC2 CC6.1/CC7.2** | Access enforcement + monitoring evidence | Decision-logs + telemetry |
