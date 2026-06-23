# Pollek DEK — Production Hardening PR Checklist (Phase 0–5)

รวมทุก deliverable เป็น roadmap การ merge เดียว — **ลำดับ, Cargo ที่ต้องเปลี่ยน, ไฟล์ที่แตะ, acceptance** ครบ
แนะนำ merge ทีละ PR ตามลำดับ (แต่ละ PR build+test เขียวก่อนไป PR ถัดไป)

> Guardrails ที่ทุก PR ต้องไม่ละเมิด: no local authoring/compiler/dry-run · fail-closed ทุก degradation · fallback = LKG read-only · ไม่ unwrap/panic บน network input · secret 0600 · no `--policy-dir` ใน prod

---

## PR-1 · Config foundation (`SyncerConfig` / `ScaleConfig`)

**ไฟล์ใหม่:** `crates/dek-config/src/scale_config.rs`
**แก้:** `crates/dek-config/src/lib.rs`

- เพิ่ม: `pub mod scale_config;` + `pub use scale_config::{SyncerConfig, ScaleConfig};`
- ใน `struct DekConfig { ... }` เพิ่มท้าย (ก่อนปิด `}`):

  ```rust
      #[serde(default)]
      pub syncer: SyncerConfig,
      #[serde(default)]
      pub scale: ScaleConfig,
  ```

**Cargo:** ไม่ต้องเปลี่ยน (serde/serde_json มีแล้ว)
**Acceptance:** `cargo test -p dek-config` (defaults + missing-fields tests เขียว)

---

## PR-2 · `dek-resilience` crate (Phase 4 primitives)

**ไฟล์ใหม่:** `crates/dek-resilience/{Cargo.toml, src/lib.rs, src/breaker.rs, src/admission.rs}`
**แก้:** root `Cargo.toml` → `members += "crates/dek-resilience"`
**Cargo (ใหม่):** tokio[sync,time,rt,macros], metrics="0.22", tracing; dev: tokio[full,test-util]
**Acceptance:** `cargo test -p dek-resilience` (breaker 3 + admission 2 tests)

---

## PR-3 · `dek-bundle-sync` key rotation (Phase 2 verifier)

**ไฟล์ใหม่:** `crates/dek-bundle-sync/src/keys.rs`  (TrustedKeySet + verify + tests)
**แก้:** `crates/dek-bundle-sync/src/lib.rs` (ดู `BUNDLE_SYNC_keys_patch.md`)

- `pub mod keys;`
- struct field: `pinned_public_key: String` → `key_set: Arc<ArcSwap<keys::TrustedKeySet>>`
- `new(...)`: `TrustedKeySet::from_single_pinned(pinned_public_key)`
- เมธอด: `update_keys(set)`, `key_set_snapshot()`
- แทน verify loop ด้วย `key_set.verify(now, &signed_bytes, &parse_signatures(...))`
- เพิ่ม `BundleError::{SignatureRejected, RollbackBlocked}` (thiserror) + คืน error ชนิดนี้
**Cargo (dek-bundle-sync):** + `arc-swap = "1"`, + `thiserror = "1"` (มี ed25519-dalek/base64/serde อยู่แล้ว)
**Acceptance:** `cargo test -p dek-bundle-sync` (keys: valid/overlap/revoke/forged)

---

## PR-4 · `dek-policy-syncer` crate (Phase 0/1/2/3 orchestrator)

**ไฟล์ใหม่:** `crates/dek-policy-syncer/{Cargo.toml, src/lib.rs, src/state.rs, src/gate.rs, src/keys.rs, src/audit.rs, tests/gate_integration.rs, tests/contract_matrix.rs}`
**แก้:** root `Cargo.toml` → `members += "crates/dek-policy-syncer"`
**Cargo (ใหม่):** dek-bundle-sync, dek-config, dek-telemetry, arc-swap, serde, serde_json, tokio[rt,time,macros,sync], tokio-util, anyhow, tracing, metrics, reqwest[rustls-tls,json], sha2, hex; dev: tempfile, ed25519-dalek, base64, axum, tokio[full]
**Acceptance:**

- `cargo test -p dek-policy-syncer` (state 5 + gate 2 + contract_matrix 5)
- ยืนยัน `evaluate_state` + gate fail-closed + `/v1/keys` chain-of-trust

---

## PR-5 · dek-core wiring (Phase 1/2/3 รันจริงในกระบวนการ core)

**แก้:**

- `crates/dek-core/src/supervisor.rs` — สร้าง `PolicySyncer::new(bundle_agent, Some(telemetry), FreshnessConfig{from cfg.syncer})`, `spawn(poll_interval, cancel)`, เก็บ `SyncerHandle` ใน Supervisor struct (อย่าให้ drop)
- ลบ/แทน `bundle_loop::spawn_bundle_sync_task` เดิมด้วย syncer (ดู `PHASE0_1_wiring.md` + `PHASE2_3_wiring.md`)
- wire AuditTrail + KeyManager เข้า `sync_once` (ดู `PHASE2_3_wiring.md` §2)
**Cargo (dek-core):** + `dek-policy-syncer = { path = "../dek-policy-syncer" }`
**Acceptance:** `cargo build -p dek-core`; unit ของ core เขียว; ไฟล์ `state/enforcement_state.json` ถูกเขียนเมื่อรัน

---

## PR-6 · PEP strict-deny gate + scale (Phase 1 + 4 ที่ proxy/ext-authz)

**แก้:**

- `crates/dek-mcp-proxy/src/main.rs`:
  - ก่อน `router.authorize()` (บรรทัด ~489): `if let Some(reason)=dek_policy_syncer::strict_deny_reason(){ deny }` (ดู `PHASE0_1_wiring.md` §2)
  - entry ของ `handle_mcp_request`: `admission.try_admit(tenant)` → None → 503 deny (ดู `PHASE4_5_wiring.md` §4.1)
  - เก็บ `AdmissionControl` ใน `AppState`
- `crates/dek-ext-authz/src/main.rs`: gate เดียวกันใน `check()`
- `crates/dek-policy-router/src/lib.rs`: circuit breaker รอบ evaluator + timeout (ดู `PHASE4_5_wiring.md` §4.2) + ย้าย `dek_pdp_unavailable_total` มาที่ `metrics` crate (ดู `METRICS_instrumentation_patch.rs`)
**Cargo:**
- dek-mcp-proxy: + dek-policy-syncer, dek-resilience, metrics="0.22"; tokio features += time,macros,sync
- dek-ext-authz: + dek-policy-syncer, dek-resilience, metrics="0.22"
- dek-policy-router: + dek-resilience, metrics="0.22" (ลบ opentelemetry ถ้าไม่ใช้ที่อื่น)
**Acceptance:** `cargo build -p dek-mcp-proxy -p dek-ext-authz -p dek-policy-router`; manual: ลบ status file → PEP deny

---

## PR-7 · Observability recorders (dek-metrics เข้า proxy/ext-authz)

**ไฟล์:** ใช้ `dek-metrics` crate (มีแล้วใน repo)
**แก้:** proxy/ext-authz `main()` — `install_recorder(service)` + `spawn_push(...)` (ดู `RECORDER_wiring.md`)
**Acceptance:** metric ใหม่ปรากฏใน push: `dek_enforcement_state`, `dek_proxy_requests_total{decision}`, `dek_circuit_open`, `dek_admission_rejected_total`, `dek_svid_expiry_seconds`, `dek_telemetry_spool_rows`

---

## PR-8 · mock-cloud test surface (Phase 2/5)

**ไฟล์ใหม่:** `crates/mock-cloud/src/keys.rs` (`/v1/keys` signed) (ดู `MOCKCLOUD_keys_patch.md`)
**แก้:** `crates/mock-cloud/src/main.rs` — `mod keys; .merge(keys::router())`; เพิ่ม admin hooks: `/admin/rotate-key`, `/admin/publish-tampered-bundle`, `/admin/audits`; `AppState` += `trusted_keys: Arc<Mutex<Vec<Value>>>`
**Acceptance:** `cargo build -p mock-cloud`; `GET /v1/keys` คืน payload signed; rotate-key เพิ่ม next key

---

## PR-9 · Acceptance harness (Phase 5 matrix A–H)

**ไฟล์ใหม่:** `crates/acceptance-tests/tests/matrix_a_to_h.rs`
**แก้:** `crates/acceptance-tests/Cargo.toml` (ใช้ deps เดิม: tokio[full], reqwest, serde_json, anyhow)
**CI:** เพิ่ม job `integration` (ubuntu) รัน `cargo test -p acceptance-tests --test matrix_a_to_h -- --ignored`
**Acceptance:** A/B/E ผ่าน (sync+enforce, unsigned-reject, key-rotation); C/D/F/G/H เปิดเมื่อ mock-cloud admin hooks + test-profile config พร้อม

---

## สรุป Cargo changes (รวมทุก PR)

| crate | เพิ่ม |
|---|---|
| root `Cargo.toml` | members: dek-resilience, dek-policy-syncer |
| dek-config | (ไม่มี dep ใหม่) |
| dek-bundle-sync | arc-swap, thiserror |
| dek-resilience (ใหม่) | tokio, metrics, tracing |
| dek-policy-syncer (ใหม่) | dek-bundle-sync, dek-config, dek-telemetry, arc-swap, serde(_json), tokio, tokio-util, anyhow, tracing, metrics, reqwest, sha2, hex |
| dek-core | dek-policy-syncer |
| dek-mcp-proxy | dek-policy-syncer, dek-resilience, metrics, dek-metrics; tokio += time,macros,sync |
| dek-ext-authz | dek-policy-syncer, dek-resilience, metrics, dek-metrics |
| dek-policy-router | dek-resilience, metrics (−opentelemetry) |

## ลำดับ merge แนะนำ

`PR-1 → PR-2 → PR-3 → PR-4 → PR-5 → PR-6 → PR-7 → PR-8 → PR-9`
(1–4 เป็น foundation ไม่กระทบ runtime; 5–6 เปิดใช้งานจริง; 7 observability; 8–9 test)

## Definition of Done (รวม)

- [ ] cold-start / stale / partition (>max_bundle_age หรือ >grace) → PEP **deny** (fail-closed)
- [ ] key rotation ผ่าน mTLS โดยไม่ rebuild; rogue-key ถูกปฏิเสธ (chain-of-trust)
- [ ] audit ทุก policy update + `policy.sync.rejected` critical เมื่อ unsigned (+ hash chain)
- [ ] backpressure (per-tenant) + circuit breaker → overload/PDP-fail = deny ไม่ใช่ allow
- [ ] hot-reload ไม่ interrupt (ArcSwap)
- [ ] acceptance matrix A–H เขียวบน CI integration job
- [ ] ไม่มี local authoring/compiler/dry-run/`--policy-dir` (test ยืนยัน)
- [ ] cutover ไป Pollek Cloud จริง = เปลี่ยน endpoint + trust store เท่านั้น

## อ้างอิงไฟล์ (deliverable ที่ทำไว้แล้ว)

`scale_config.rs` · `dek-resilience/*` · `dek-bundle-sync/src/keys.rs` · `dek-policy-syncer/*` ·
`BUNDLE_SYNC_keys_patch.md` · `PHASE0_1_wiring.md` · `PHASE2_3_wiring.md` · `PHASE4_5_wiring.md` ·
`MOCKCLOUD_keys_patch.md` · `RECORDER_wiring.md` · `METRICS_instrumentation_patch.rs` ·
`acceptance-tests/tests/matrix_a_to_h.rs` · `dek_soak_observability.html` · `release.yml`
