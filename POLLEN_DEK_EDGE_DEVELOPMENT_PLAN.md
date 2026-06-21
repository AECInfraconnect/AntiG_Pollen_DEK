# Pollen DEK — แผนพัฒนาฝั่ง Edge (Agent-Ready Implementation Guide)

**เป้าหมาย:** เอกสารนี้ออกแบบให้ AI Agent นำไปพัฒนาต่อได้ครบถ้วนด้วยตัวเอง — ทุก task มี (1) ไฟล์เป้าหมายจริงในrepo (2) สถานะปัจจุบันที่ตรวจแล้ว (3) code ตัวอย่างที่ต่อยอดจากของจริง (4) acceptance criteria ที่ทดสอบได้
**Repo:** `AECInfraconnect/AntiG_Pollen_DEK` @ `bb0d01f` | **อ้างอิง:** Pollen Datasheet v2.0
**ขอบเขต:** เฉพาะฝั่ง Edge = `crates/local-control-plane`, `crates/dek-*`, `apps/local-admin-dashboard`

---

## คำสั่งสำหรับ AI Agent (อ่านก่อนเริ่ม)

```
บทบาท: คุณคือ senior Rust + React engineer พัฒนา Pollen DEK ฝั่ง edge
กติกา:
1. ทำ task ตามลำดับ P0 → P1 → P2 ห้ามข้าม P0
2. ก่อนแก้ไฟล์ใด ให้ `view` ไฟล์นั้นจริงก่อนเสมอ — code ตัวอย่างในเอกสารนี้
   อิงโครงสร้าง ณ bb0d01f อาจมี drift
3. ทุก crate ห้ามใช้ unwrap()/expect() ในโค้ด production (workspace lint
   `#![deny(clippy::unwrap_used, clippy::expect_used)]` บังคับอยู่)
4. หลังแก้แต่ละ task: รัน `cargo build --workspace` + `cargo clippy` +
   (ถ้าแตะ dashboard) `npm run build` ใน apps/local-admin-dashboard ให้ผ่านก่อนไป task ถัดไป
5. commit เล็ก ๆ ต่อ task พร้อม conventional commit message
6. ทุก task มี "Acceptance" — ถือว่าเสร็จเมื่อผ่านครบ
```

---

# Phase P0 — กู้ CI ให้เขียว (ต้องเสร็จก่อนทุกอย่าง) 🔴

> ขณะนี้ commit `bb0d01f` ทำ build พังทั้ง workspace + Spectral lint fail ถ้าไม่กู้ก่อน task อื่นจะ build ไม่ผ่านตามไปหมด

## P0-T1 — คืน field `adapter_results` + `obligations` ใน decision schema

**ปัญหา:** `contracts/schemas/decision-result.v1.schema.json` ตัด 2 field นี้ออก แต่โค้ด DEK ยังใช้ `.obligations` และ datasheet §5.2 ระบุ `adapter_results`/`obligations` เป็น field บังคับ → compile fail + เสีย observability

**ไฟล์:** `contracts/schemas/decision-result.v1.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://contracts.pollen.dev/schemas/decision-result.v1.schema.json",
  "title": "PollenDecisionResultV1",
  "type": "object",
  "additionalProperties": false,
  "required": ["request_id", "trace_id", "decision", "reason", "matched_policy_ids", "latency_ms"],
  "properties": {
    "request_id": { "type": "string" },
    "trace_id": { "type": "string" },
    "decision": { "type": "string", "enum": ["Allow","Deny","Redact","Mask","Warn","RequireApproval","BreakGlassAllow"] },
    "reason": { "type": "string" },
    "matched_policy_ids": { "type": "array", "items": { "type": "string" } },
    "matched_route_id": { "type": "string" },
    "latency_ms": { "type": "integer", "minimum": 0 },
    "selected_engine": { "type": "string" },
    "enforcement_plane": { "type": "string" },
    "adapter_results": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["engine", "decision"],
        "properties": {
          "engine":     { "type": "string" },
          "decision":   { "type": "string", "enum": ["Allow","Deny","Redact","Mask","Warn","RequireApproval","BreakGlassAllow"] },
          "reason":     { "type": "string" },
          "latency_ms": { "type": "integer", "minimum": 0 }
        }
      }
    },
    "obligations": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["type"],
        "properties": {
          "type":       { "type": "string" },
          "parameters": { "type": "object", "additionalProperties": true }
        }
      }
    }
  }
}
```

จากนั้น regenerate Rust types:
```bash
cd contracts && npm run gen   # หรือ task ที่ build.rs ทำตอน cargo build
cargo build -p pollen-contract
```

**Acceptance:** `cargo build -p dek-control-plane-api` ผ่าน, และ struct `DecisionResult` ที่ generate มี field `adapter_results` + `obligations`

## P0-T2 — แก้ type mismatch `latency_ms` (u64 vs i64)

typify map `integer` → `i64` แต่โค้ดเดิมหลายจุดเป็น `u64`

**วิธีที่แนะนำ (น้อย churn):** ค้นทุกจุดที่ assign/อ่าน แล้วปรับ:
```bash
grep -rn 'latency_ms' crates/ --include='*.rs' | grep -v target
```
จุดที่สร้าง `DecisionResult` ให้ cast: `latency_ms: elapsed.as_millis() as i64,`
(ถ้า field อื่นที่ generate เป็น `i64` เช่นกัน ทำแบบเดียวกัน)

**Acceptance:** `cargo build --workspace --exclude dek-ebpf-prog --exclude dek-ebpfd` ผ่านบนเครื่อง CI-equivalent (Rust ≥ 1.85)

## P0-T3 — เติม doc/tag/error/info ใน TypeSpec ให้ผ่าน Spectral

**ปัญหา:** `contract-ci` fail ที่ `validate-build-and-drift` — operation ขาด `tags`/`description`, ขาด error response, `info` ขาด `description`/`contact`

**ไฟล์:** `contracts/spec/main.tsp` + `contracts/spec/rest/*.tsp`

```typescript
// main.tsp — เติม service metadata
import "@typespec/http";
import "@typespec/openapi";
using TypeSpec.Http;
using TypeSpec.OpenAPI;

@service(#{ title: "Pollen DEK Control-Plane API" })
@info(#{
  version: "1.0.0",
  description: "Contract between Pollen Cloud / Local Control Plane and the DEK edge enforcement plane.",
  contact: #{ name: "Pollen Platform Team", url: "https://pollen.ai", email: "platform@pollen.ai" }
})
namespace PollenContract;
```

```typescript
// rest/bundles.tsp — ทุก operation ต้องมี @doc + @tag + error response
@tag("Bundles")
@route("/v1/tenants/{tenant_id}/devices/{device_id}/bundles")
namespace Bundles {
  @doc("Fetch the latest signed policy bundle manifest for a device. Returns 304 if the device already holds the current build_number.")
  @get
  op getLatest(
    @path tenant_id: string,
    @path device_id: string,
    @header("If-None-Match") etag?: string,
  ): BundleManifestV2 | NotModifiedResponse | PollenError;   // <- error union
}
```

นิยาม `PollenError` ครั้งเดียวใน `common.tsp` แล้ว `$ref` ทุกที่ (ruleset `pollen-error-envelope-required` ต้องการ):
```typescript
// rest/common.tsp
@error
model PollenError {
  @statusCode statusCode: 400 | 401 | 403 | 404 | 409 | 426 | 500;
  code: string;
  message: string;
  trace_id?: string;
}
model NotModifiedResponse { @statusCode statusCode: 304; }
```

**Acceptance:** `cd contracts && npx tsp compile spec/ && npx @stoplight/spectral-cli lint generated/openapi/pollen.v1.yaml --fail-severity=warn` ผ่าน 0 error; และ `git diff --exit-code generated/openapi/` (no drift)

## P0-T4 — แก้ metadata version + MSRV

**ไฟล์:** `Cargo.toml` (root)
```toml
[workspace.package]
version = "1.0.0-beta.5"     # sync กับ tag ที่จะออก (ตอนนี้ค้าง beta.1)
rust-version = "1.85"         # ของเดิม 1.70 ผิด — workspace ใช้ edition2024 ต้อง ≥1.85
```

**Acceptance:** ไม่มี crate ใดประกาศ `edition = "2024"` โดยที่ `rust-version < 1.85`; CI ทั้ง 6 workflow เขียวที่ commit นี้

---

# Phase P1 — ปิด Gap ที่ Datasheet โฆษณาแต่ Edge ยังขาด 🟠

> หมายเหตุจากการตรวจจริง: หลาย feature "มีโครงแล้ว" มากกว่าที่คิด — DecisionLogs page wired จริง, circuit breaker + FailoverStrategy (Priority/HealthBased/RoundRobin/LeastLatency) implement ครบ, PDP pool รองรับ N-tier อยู่แล้ว ดังนั้น P1 โฟกัสเฉพาะ "ส่วนที่ขาดจริง"

## P1-T1 — Dry-run Simulation Engine (datasheet §1 Blast Radius, §8 Sandbox)

**สถานะจริง:** `local-control-plane/src/policy.rs:251 simulate_policy()` คืน mock (`"log_output": ["mock simulate"]`) — endpoint มี แต่ไม่มีเครื่องยนต์จริง สำคัญเพราะ datasheet โฆษณา self-hosted/air-gapped ซึ่งไม่มี Cloud ให้พึ่ง → ต้องมี dry-run บน edge จริง

**แนวทาง:** ใช้ PDP evaluator เดิม (`dek-policy-router` / `dek-decision`) ในโหมด `dry_run` ที่ประเมินจริงแต่ไม่ enforce + ไม่เขียน telemetry จริง

**ไฟล์:** `crates/local-control-plane/src/policy.rs`

```rust
use pollen_contract::PollenDecisionResultV1 as DecisionResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    /// policy draft ที่จะทดสอบ (ยังไม่ publish); ถ้า None ใช้ active bundle
    pub draft_policy_id: Option<String>,
    /// ชุด request สมมติ (What-If) — datasheet §8 batch-test
    pub scenarios: Vec<ScenarioInput>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScenarioInput {
    pub scenario_id: String,
    pub subject: serde_json::Value,   // agent / user
    pub action: String,
    pub resource: serde_json::Value,
    pub context: serde_json::Value,
    /// ผลที่คาดหวัง (Regression/Pass-Fail) — datasheet §8
    pub expected_decision: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SimulateResponse {
    pub results: Vec<ScenarioResult>,
    pub summary: SimulateSummary,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub decision: DecisionResult,
    pub passed: Option<bool>,   // เทียบ expected_decision
}

#[derive(Debug, Serialize)]
pub struct SimulateSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    /// Blast radius: จำนวน scenario ที่ผล "เปลี่ยน" เทียบ active policy
    pub changed_vs_active: usize,
}

async fn simulate_policy(
    Path((tenant, _policy_id)): Path<(String, String)>,
    State(st): State<AppState>,
    Json(req): Json<SimulateRequest>,
) -> impl IntoResponse {
    // 1) โหลด draft (ถ้ามี) → compile ชั่วคราวด้วย compiler.rs ที่มีอยู่
    // 2) สร้าง evaluator แบบ ephemeral (ไม่ผูก enforcement plane จริง)
    // 3) ประเมินแต่ละ scenario ทั้งกับ draft และ active เพื่อหา blast radius
    let evaluator = match st.build_dry_run_evaluator(&tenant, req.draft_policy_id.as_deref()).await {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
    };
    let active = st.active_evaluator(&tenant).await.ok();

    let mut results = Vec::with_capacity(req.scenarios.len());
    let (mut passed, mut failed, mut changed) = (0, 0, 0);

    for sc in &req.scenarios {
        let decision = evaluator.evaluate_dry_run(sc).await;   // ไม่ enforce, ไม่ spool telemetry
        if let Some(active) = &active {
            let active_dec = active.evaluate_dry_run(sc).await;
            if active_dec.decision != decision.decision { changed += 1; }
        }
        let passed_flag = sc.expected_decision.as_ref().map(|exp| {
            format!("{:?}", decision.decision).eq_ignore_ascii_case(exp)
        });
        match passed_flag { Some(true) => passed += 1, Some(false) => failed += 1, None => {} }
        results.push(ScenarioResult { scenario_id: sc.scenario_id.clone(), decision, passed: passed_flag });
    }

    let summary = SimulateSummary {
        total: req.scenarios.len(), passed, failed, changed_vs_active: changed,
    };
    (StatusCode::OK, Json(SimulateResponse { results, summary })).into_response()
}
```

> Agent note: `evaluate_dry_run` ต้องเพิ่มใน evaluator ของ `dek-policy-router` — wrap `evaluate()` เดิมแต่ตั้ง flag ปิด side-effect (ไม่เรียก enforcement plane, ไม่ push telemetry) ดูฟังก์ชัน `evaluate` ปัจจุบันใน `crates/dek-policy-router/src/lib.rs` ก่อน

**Acceptance:** POST `/v1/tenants/local/policies/{id}/simulate` ด้วย 3 scenarios คืน `summary.passed/failed/changed_vs_active` จริง (ไม่ใช่ mock); unit test ใน `policy.rs` ครอบ pass/fail/blast-radius

## P1-T2 — Scenario Sandbox UI (datasheet §8)

**สถานะจริง:** `Simulator.tsx` (130 LOC) เรียก `PolicyApi.simulate` แล้ว แต่ payload เป็น `any` และ backend ยัง mock

**ไฟล์:** `apps/local-admin-dashboard/src/services/api.ts` (ปรับ type) + `pages/Simulator.tsx`

```typescript
// services/api.ts — แทน simulate(req: any)
export interface ScenarioInput {
  scenario_id: string;
  subject: Record<string, unknown>;
  action: string;
  resource: Record<string, unknown>;
  context: Record<string, unknown>;
  expected_decision?: string;
}
export interface SimulateResponse {
  results: { scenario_id: string; decision: DecisionResult; passed: boolean | null }[];
  summary: { total: number; passed: number; failed: number; changed_vs_active: number };
}

export const PolicyApi = {
  // ...
  simulate: (policyId: string, scenarios: ScenarioInput[]): Promise<SimulateResponse> =>
    defaultClient.simulatePolicy(policyId, scenarios),
};
```

```typescript
// ControlPlaneClient — แก้ให้ตรง route จริง (/policies/{id}/simulate ไม่ใช่ /policies/simulate)
async simulatePolicy(policyId: string, scenarios: ScenarioInput[]): Promise<SimulateResponse> {
  return this.fetchApi(`/policies/${policyId}/simulate`, {
    method: 'POST',
    body: JSON.stringify({ scenarios }),
  });
}
```

UI ต้องมี: เพิ่ม/ลบ scenario, ปุ่ม Run, ตาราง Pass/Fail (เขียว/แดง), badge "Blast radius: N decisions change", ปุ่ม Export CSV (ใช้ helper จาก P1-T3), บันทึก/โหลด scenario set ลง `localStorage` (datasheet "save, load, share")

**Acceptance:** กรอก scenario → Run → เห็นผล Pass/Fail จริงจาก backend + blast-radius count + Export CSV ได้

## P1-T3 — Audit Export CSV/JSON (datasheet §7)

**สถานะจริง:** `DecisionLogs.tsx` แสดง logs จริงแล้ว (live refresh 5s) แต่**ไม่มีปุ่ม export**

**ไฟล์ใหม่:** `apps/local-admin-dashboard/src/lib/export.ts`
```typescript
export function toCsv<T extends Record<string, unknown>>(rows: T[], columns: (keyof T)[]): string {
  const esc = (v: unknown) => {
    const s = v == null ? "" : typeof v === "object" ? JSON.stringify(v) : String(v);
    return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
  };
  const header = columns.join(",");
  const body = rows.map((r) => columns.map((c) => esc(r[c])).join(",")).join("\n");
  return `${header}\n${body}`;
}

export function download(filename: string, content: string, mime = "text/csv") {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url; a.download = filename; a.click();
  URL.revokeObjectURL(url);
}
```

ใน `DecisionLogs.tsx` เพิ่มปุ่ม:
```typescript
import { toCsv, download } from "../lib/export";

const exportCsv = () => {
  const rows = decisions.map(({ env, d }) => ({
    timestamp: env.timestamp, decision: d.decision, reason: d.reason,
    request_id: d.request_id, matched_policy_ids: d.matched_policy_ids,
    selected_engine: d.selected_engine, latency_ms: d.latency_ms,
  }));
  download(`pollen-audit-${Date.now()}.csv`,
    toCsv(rows, ["timestamp","decision","reason","request_id","matched_policy_ids","selected_engine","latency_ms"]));
};
const exportJson = () =>
  download(`pollen-audit-${Date.now()}.json`, JSON.stringify(decisions.map(x=>x.d), null, 2), "application/json");
```

**Acceptance:** ปุ่ม Export CSV + Export JSON บนหน้า Audit ดาวน์โหลดไฟล์ที่มี decision logs ปัจจุบันครบทุกคอลัมน์

## P1-T4 — Connector Config + Test Connection (datasheet §5)

**สถานะจริง:** `Settings.tsx` มีแค่ profile switch (local/mock-cloud) — **ไม่มี** การตั้งค่า PDP endpoint หรือ Test Connection ทั้งที่ adapters (Cedar/OPA/OpenFGA) เชื่อมต่อจริงได้

**Backend ใหม่:** `crates/local-control-plane/src/connectors.rs`
```rust
use axum::{extract::State, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectorConfig {
    pub id: String,
    pub kind: ConnectorKind,        // Opa | Cedar | OpenFga
    pub endpoint: String,
    pub health_interval_secs: u32,
    pub mtls_enabled: bool,
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind { Opa, Cedar, OpenFga }

#[derive(Serialize)]
pub struct TestResult { pub ok: bool, pub latency_ms: u64, pub detail: String }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/tenants/:tenant/connectors", get(list).post(upsert))
        .route("/v1/tenants/:tenant/connectors/:id/test", post(test_connection))
}

async fn test_connection(
    axum::extract::Path((_t, id)): axum::extract::Path<(String, String)>,
    State(st): State<AppState>,
) -> Json<TestResult> {
    let cfg = match st.connector_store.get(&id).await {
        Ok(Some(c)) => c,
        _ => return Json(TestResult{ ok:false, latency_ms:0, detail:"connector not found".into() }),
    };
    let start = std::time::Instant::now();
    // health-probe ตาม kind — OPA: GET {endpoint}/health, OpenFGA: GET /healthz, Cedar: local => ok
    let ok = match cfg.kind {
        ConnectorKind::Cedar => true,
        _ => reqwest::Client::new()
                .get(format!("{}/health", cfg.endpoint.trim_end_matches('/')))
                .timeout(std::time::Duration::from_secs(3))
                .send().await.map(|r| r.status().is_success()).unwrap_or(false),
    };
    Json(TestResult {
        ok, latency_ms: start.elapsed().as_millis() as u64,
        detail: if ok { "reachable".into() } else { "unreachable".into() },
    })
}
# // list/upsert: อ่าน/เขียน connector_store (เพิ่มใน store.rs แบบเดียวกับ policy_store)
```
แล้ว merge ใน `app.rs`: `.merge(connectors::router())`

**Frontend:** เพิ่ม section ใน `Settings.tsx` — ตาราง connector + ปุ่ม "Test Connection" ที่เรียก `/connectors/{id}/test` แล้วโชว์ ✓ reachable (latency) หรือ ✗

**Acceptance:** เพิ่ม OPA connector → กด Test → ได้ผล reachable/unreachable จริงพร้อม latency; config persist ข้าม restart

## P1-T5 — Failover: Manual Override + Auto-recovery delay (datasheet §6)

**สถานะจริง:** circuit breaker + FailoverStrategy + N-tier pool **มีครบแล้ว** สิ่งที่**ขาด**: (1) manual override ให้ operator force-switch ตอน maintenance (2) config `auto_recovery_delay` ที่ชัดเจน

**ไฟล์:** `crates/dek-policy-router/src/lib.rs` (+ `dek-resilience/src/breaker.rs`)

```rust
// เพิ่มใน router state: override map
pub struct ManualOverride {
    pub pdp_id: String,
    pub forced_state: ForcedState,  // ForceDown (maintenance) | ForceUp
    pub until: Option<Instant>,
}
pub enum ForcedState { ForceDown, ForceUp }

impl PolicyRouter {
    pub fn set_override(&self, pdp_id: &str, forced: ForcedState, ttl: Option<Duration>) {
        // เก็บใน self.overrides (Mutex<HashMap>); honored ใน select_pdp_from_pool
    }
}

// แก้ select_pdp_from_pool: เคารพ override ก่อน breaker
let available: Vec<&String> = pool.iter().filter(|p| {
    match self.override_for(p) {
        Some(ForcedState::ForceDown) => false,           // maintenance → ข้าม
        Some(ForcedState::ForceUp) => true,              // operator ยืนยันใช้งานได้
        None => self.breakers.get(*p)
                    .map(|b| matches!(b.permitted(), Admit::Allow)).unwrap_or(false),
    }
}).collect();
```

auto-recovery delay เพิ่มใน `CircuitConfig` (มี `cooldown` แล้ว — ตั้งชื่อ alias `auto_recovery_delay` ใน config schema ให้ตรง datasheet) และ expose ผ่าน LCP endpoint `POST /v1/tenants/local/pdp/{id}/override`

**Acceptance:** unit test: เมื่อ set ForceDown, `select_pdp_from_pool` ข้าม pdp นั้นแม้ breaker เป็น Closed; เมื่อ ttl หมด กลับมาเลือกได้

## P1-T6 — i18n EN/TH (datasheet competitive: EN/TH/ZH/AR)

**สถานะจริง:** dashboard **ไม่มี i18n เลย** string hardcode ทั้งหมด ทำ en/th ให้ครบก่อน (zh/ar เป็น stretch — ใส่ scaffold ว่างไว้)

```bash
cd apps/local-admin-dashboard && npm i react-i18next i18next
```

**ไฟล์ใหม่:** `src/lib/i18n.ts`
```typescript
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "../locales/en.json";
import th from "../locales/th.json";

i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, th: { translation: th } },
  lng: localStorage.getItem("dek_lang") || "en",
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});
export default i18n;
```
`src/locales/en.json` / `th.json` — แยก string ทุกหน้า; ใช้ `const { t } = useTranslation()` แทน hardcode; เพิ่ม language switcher ใน Settings (`i18n.changeLanguage` + persist)

**Acceptance:** สลับ EN↔TH ใน Settings เปลี่ยนภาษาทั้ง UI ทันที; ไม่มี string ที่ตกหล่นเป็นภาษาเดิม

---

# Phase P2 — เก็บงาน + เตรียม Release

## P2-T1 — Alerts page (ตอนนี้เป็น `<h1>Alerts (WIP)</h1>` ใน App.tsx:34)
Edge เก็บ event อยู่แล้ว — ทำหน้า rule แบบเบา: match `decision==Deny` เกิน N ครั้ง/นาที → แสดง banner (notify orchestration จริงเป็นงาน Cloud) อย่างน้อยให้ไม่เป็น WIP เปล่า

## P2-T2 — Update เอกสารทั้งหมด
- `CHANGELOG.md` ไล่ถึง bb0d01f (Contract Hub) + งาน P0/P1
- `RELEASE_NOTES_v1.0.0-beta.5.md` (ของเดิมค้าง beta.1)
- `ARCHITECTURE.md` เพิ่มชั้น Contract Hub + dry-run/connector/override ใหม่
- `docs/USER_GUIDE_{EN,TH}.md` เพิ่มวิธีใช้ Simulator/Export/Connector Test
- `README.md` เพิ่ม compatibility badge ชี้ `contracts/COMPATIBILITY.md`

## P2-T3 — Local build gate (กันพลาดแบบ bb0d01f ซ้ำ)
**ไฟล์ใหม่:** `.githooks/pre-push`
```bash
#!/usr/bin/env bash
set -e
echo "[pre-push] cargo build + clippy..."
cargo build --workspace --exclude dek-ebpf-prog --exclude dek-ebpfd
cargo clippy --workspace --exclude dek-ebpf-prog -- -D warnings
( cd contracts && npx tsp compile spec/ && npx @stoplight/spectral-cli lint generated/openapi/pollen.v1.yaml --fail-severity=warn )
( cd apps/local-admin-dashboard && npm run build )
echo "[pre-push] OK"
```
`git config core.hooksPath .githooks` + เปิด branch protection บน main เมื่อ CI เสถียร

---

# สรุปลำดับงาน (Dependency Order)

```
P0-T1 schema fields ─┐
P0-T2 latency type ──┼─> P0-T4 version/MSRV ─> [CI เขียว] ─┐
P0-T3 typespec lint ─┘                                      │
                                                            ▼
P1-T1 dry-run engine ──> P1-T2 sandbox UI                  │
P1-T3 export ──────────> (DecisionLogs + Simulator ใช้ร่วม) │
P1-T4 connector test                                        │
P1-T5 failover override                                     │
P1-T6 i18n                                                  ▼
                                          P2 docs + gate ─> [tag beta.5 จาก main เขียว] ─> Release
```

# Acceptance รวมของทั้งแผน
1. `cargo build --workspace` + `cargo clippy -D warnings` + `npm run build` ผ่านทั้งหมด
2. CI ทั้ง 6 workflow เขียวที่ commit สุดท้าย
3. Simulator คืนผลจริง (pass/fail + blast radius) ไม่ใช่ mock
4. Audit export CSV/JSON ได้
5. Connector Test Connection ได้ผลจริง
6. Failover honor manual override
7. สลับภาษา EN/TH ได้ทั้ง UI
8. เอกสารหลัก current ถึง beta.5
9. Release แรกออกจริงจาก main ที่เขียว (หน้า Releases ไม่ว่างอีกต่อไป)
```

---

*แผนนี้อิงโครงสร้างจริง ณ `bb0d01f` — AI Agent ควร `view` ไฟล์เป้าหมายก่อนแก้ทุกครั้ง เพราะ code ตัวอย่างเป็น reference ไม่ใช่ไฟล์สมบูรณ์ และ field/signature อาจ drift หลัง P0 regenerate types*
