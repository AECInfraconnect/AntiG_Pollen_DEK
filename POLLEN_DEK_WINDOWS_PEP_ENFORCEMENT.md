# Pollen DEK — Enforce + Observe AI Agents บน Windows ที่ไม่มี Proxy/Sidecar

**วันที่:** 21 มิถุนายน 2026 | **HEAD:** `ed30a30`
**โจทย์:** เครื่อง Windows ของ user ไม่มี proxy/sidecar setup — ทำอย่างไรให้ enforce + observe AI agents + ส่ง telemetry ได้จริง ควรใช้ PEP type แบบไหน
**วิธี:** ตรวจโค้ด PEP จริงใน DEK + deep research Windows endpoint enforcement (WFP modes) ปี 2026

---

## 1. สาเหตุที่แท้จริง: PEP บน Windows ยังไม่ enforce จริง (ตรวจโค้ดยืนยัน)

DEK มี PEP 4 แบบ — ตรวจแล้วว่าบน Windows ที่ไม่มี setup เพิ่ม **ไม่มีตัวไหน enforce ได้จริง**:

| PEP Type | crate | สถานะบน Windows (user เดี่ยว) |
|----------|-------|-------------------------------|
| **WFP network** | `dek-windows-wfp` | 🔴 เปิด engine ได้ แต่ `apply_rules()` **เป็น no-op** — แค่ log ไม่ add filter จริง |
| **MCP HTTP proxy** | `dek-mcp-proxy` | 🟡 ทำงานได้ (TcpListener 127.0.0.1:43890) แต่ agent ต้อง **ตั้งค่าให้ชี้มาที่ proxy เอง** — ไม่ transparent |
| **Envoy ext-authz** | `dek-ext-authz` | 🔴 ต้องมี Envoy sidecar — ไม่มีบนเครื่อง user ทั่วไป |
| **MCP stdio wrapper** | `dek-mcp-stdio-wrapper` | 🟡 ทำงานได้ แต่ต้อง **rewrite config ของ MCP client** ให้เรียกผ่าน wrapper |

**หลักฐานจาก `dek-windows-wfp/src/lib.rs`:**

```rust
fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
    info!("OS Enforcement (Windows): inserting WFP exact block filters for policy '{}'...", ...);
    Ok(())   // ← ไม่มี FwpmFilterAdd0 จริง — แค่ log แล้ว return Ok
}
fn clear_rules(&self) -> Result<()> {
    info!("Clearing all active WFP exact block filters");
    Ok(())   // ← เช่นกัน
}
```

WFP เปิด engine (`FwpmEngineOpen0`) สำเร็จ แต่ **ไม่เคย add filter** — จึงไม่บล็อก/ควบคุมอะไรเลย นี่คือเหตุผลที่ enforce บน Windows ไม่เกิดผลจริง

**สรุปปัญหา:** PEP ที่ "transparent" (ไม่ต้องให้ agent ตั้งค่าเอง) มีแค่ WFP — แต่ WFP ยัง implement ไม่ครบ ส่วน proxy/wrapper ต้องให้ agent cooperate (ตั้ง config ชี้มา) ซึ่ง blackbox AI ในเครื่อง user ไม่ทำให้

---

## 2. Deep Research: PEP Type ไหน enforce ได้จริงบน Windows endpoint

จาก research Windows endpoint enforcement ปี 2026 — **WFP คือคำตอบ** เพราะเป็น native platform ที่ EDR/firewall ทุกตัวใช้ มี 3 ระดับความสามารถ:

### 2.1 WFP Block Mode (user-mode, ไม่ต้องมี driver) ✅ เริ่มที่นี่

จาก research WFP คือกลไกพื้นฐานที่ทำให้ component ต่าง ๆ block/permit/audit network traffic ได้ — filter ประกอบด้วย conditions (IP, port, application, protocol, user) + decision: permit/block/callout และ filter ส่วนใหญ่อยู่ที่ layer FWPM_LAYER_ALE_AUTH_CONNECT — เป็น stateful filter ก่อน outbound connection จะถูกสร้าง

- ทำได้จาก **user-mode** ด้วย `FwpmFilterAdd0` ที่ ALE_AUTH_CONNECT layer — **ไม่ต้องเขียน kernel driver**
- block ได้ตาม: remote IP/port, **application (browser/agent process)**, protocol
- นี่คือสิ่งที่ `apply_rules()` ควรทำแต่ยังไม่ทำ → **แก้ที่นี่ได้ผลทันที**

### 2.2 WFP SNI Inspection (callout, observe domain) ✅ สำหรับ observe

จาก research Win11 Enhanced Phishing Protection ใช้ WFP callout ใน WTD.sys ที่ watch SNI extension ใน TLS ClientHello เพื่อรู้ว่า process เชื่อมต่อไป host ไหน

- ทำให้ DEK เห็น **domain ที่ browser/agent คุย** (claude.ai, openai.com) โดยไม่ต้อง decrypt — อ่านจาก SNI ใน ClientHello
- เป็น signal เดียวกับที่ใช้ใน browser AI discovery (เอกสารก่อนหน้า) — observe + enforce ใช้ source เดียวกัน

### 2.3 WFP Connect-Redirect (callout driver + transparent proxy) 🔵 ขั้นสูง

จาก research connect/bind redirection ของ WFP ให้ ALE callout driver inspect และ redirect connection — redirect application ให้ต่อไป proxy service แทน destination เดิม โดย proxy มี 2 socket: redirected original + new proxied outbound และ WFP driver intercept traffic แล้ว redirect ไป local proxy ของ agent แบบ transparent — ไม่ต้อง config proxy ใน setting เพราะ traffic ทุกอย่างถูก reroute ทำให้ proxy วิเคราะห์ application protocol เช่น TLS ได้

- นี่คือวิธีที่ทำให้ **mcp-proxy ที่มีอยู่ "transparent"** — agent ไม่ต้องตั้งค่า แต่ traffic ถูก redirect มาเอง
- ต้องมี **kernel callout driver** (signed) — effort สูง เก็บไว้ phase หลัง

### 2.4 Telemetry via ETW (observe ที่ Windows ใช้เอง)

จาก research MsSecWfp driver ของ MDE filter network connection ตาม rule + produce audit event เป็น ETW event — DEK ควร emit telemetry รูปแบบเดียวกัน (WFP audit → ETW → telemetry spool)

**ข้อสรุป PEP strategy บน Windows:**

1. **Enforce** = WFP Block Mode (user-mode `FwpmFilterAdd0`) — ทำได้ทันที ไม่ต้อง driver
2. **Observe** = WFP SNI callout — เห็น domain ที่ agent คุย
3. **Telemetry** = WFP audit event → spool → Cloud
4. **(ขั้นสูง)** Transparent proxy = WFP connect-redirect callout driver → mcp-proxy ที่มีอยู่

---

## 3. แก้ไข: ทำให้ WFP enforce จริง (Phase 1 — ไม่ต้องมี driver)

### 3.1 `apply_rules()` ที่ add filter จริง

```rust
// dek-windows-wfp/src/lib.rs — แทน no-op ด้วย FwpmFilterAdd0 จริง
#[cfg(windows)]
use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FwpmFilterAdd0, FWPM_FILTER0, FWPM_FILTER_CONDITION0,
    FWP_CONDITION_VALUE0, FWPM_LAYER_ALE_AUTH_CONNECT_V4,
    FWP_ACTION_BLOCK, FWP_ACTION_PERMIT, FWP_MATCH_EQUAL,
    FWPM_CONDITION_IP_REMOTE_ADDRESS, FWPM_CONDITION_IP_REMOTE_PORT,
    FWPM_CONDITION_ALE_APP_ID,
};

impl WfpFilterManager {
    /// add block filter ที่ ALE_AUTH_CONNECT — บล็อก outbound ไป remote IP/port
    /// หรือบล็อกตาม application (browser/agent process)
    #[cfg(windows)]
    fn add_block_filter(&self, remote_ip: u32, remote_port: u16, weight: u8) -> Result<u64> {
        let Some(engine) = self.engine_handle else {
            anyhow::bail!("WFP engine not open");
        };
        // เงื่อนไข: remote IP + port ตรง → block
        let conditions = [
            FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_ADDRESS,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: ip_to_condition_value(remote_ip),
            },
            FWPM_FILTER_CONDITION0 {
                fieldKey: FWPM_CONDITION_IP_REMOTE_PORT,
                matchType: FWP_MATCH_EQUAL,
                conditionValue: port_to_condition_value(remote_port),
            },
        ];
        let mut filter = FWPM_FILTER0::default();
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
        filter.action.r#type = FWP_ACTION_BLOCK;
        filter.weight = weight_value(weight);
        filter.numFilterConditions = conditions.len() as u32;
        filter.filterCondition = conditions.as_ptr() as *mut _;

        let mut filter_id: u64 = 0;
        // SAFETY: engine handle valid (ตรวจด้านบน), filter+conditions มีอายุครอบคลุม call นี้
        let status = unsafe {
            FwpmFilterAdd0(engine, &filter, None, Some(&mut filter_id))
        };
        if status != 0 {
            anyhow::bail!("FwpmFilterAdd0 failed: {status}");
        }
        info!(filter_id, remote_ip, remote_port, "WFP block filter added");
        Ok(filter_id)
    }
}

impl NetworkEnforcer for WfpFilterManager {
    fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
        if !self.is_active {
            warn!("WFP not active");
            return Ok(());
        }
        #[cfg(windows)]
        {
            let mut added = Vec::new();
            for rule in &rules.block_rules {   // CompiledNetworkRules มี rule list
                match self.add_block_filter(rule.remote_ip, rule.remote_port, rules.risk_tier) {
                    Ok(id) => added.push(id),
                    Err(e) => warn!(?e, "failed to add filter, continuing"),
                }
            }
            // เก็บ filter_id ไว้ clear ทีหลัง (interior mutability หรือ &mut self)
            info!(count = added.len(), policy = %rules.policy_id, "WFP filters applied (REAL)");
        }
        Ok(())
    }

    fn clear_rules(&self) -> Result<()> {
        #[cfg(windows)]
        {
            // FwpmFilterDeleteById0 ต่อ filter_id ที่เก็บไว้
            for id in self.active_filter_ids() {
                // SAFETY: id มาจาก FwpmFilterAdd0 ที่ engine เดียวกัน
                let _ = unsafe { FwpmFilterDeleteById0(self.engine_handle.unwrap_or_default(), id) };
            }
        }
        Ok(())
    }
}
```

> สำคัญ: filter ที่ ALE_AUTH_CONNECT บล็อก **ก่อน connection ถูกสร้าง** ตาม research — agent ต่อ claude.ai ไม่ติดถ้า policy block domain นั้น โดย agent ไม่รู้ตัวและไม่ต้อง config อะไร นี่คือ "transparent enforcement" ที่โจทย์ต้องการ

### 3.2 บล็อกตาม application (process) — คุม agent เฉพาะตัว

```rust
// บล็อก/อนุญาตตาม executable path (เช่น คุมเฉพาะ browser หรือ python agent)
#[cfg(windows)]
fn add_app_filter(&self, app_path: &str, action: FilterAction) -> Result<u64> {
    // FwpmGetAppIdFromFileName0 → app_id blob → ใช้เป็น condition
    let app_id = get_app_id(app_path)?;  // FwpmGetAppIdFromFileName0
    let condition = FWPM_FILTER_CONDITION0 {
        fieldKey: FWPM_CONDITION_ALE_APP_ID,
        matchType: FWP_MATCH_EQUAL,
        conditionValue: blob_to_condition_value(&app_id),
    };
    // ... add filter ด้วย action BLOCK/PERMIT
    Ok(0)
}
```

> ทำให้ policy แบบ "python agent ห้ามต่อ external LLM" หรือ "browser ต่อได้เฉพาะ approved AI domain" เป็นจริง — คุมระดับ process

---

## 4. Observe: WFP SNI callout (เห็น domain ที่ agent คุย)

```rust
// dek-windows-wfp/src/sni_observe.rs — อ่าน SNI จาก ClientHello (observe)
// หมายเหตุ: SNI inspection ที่ลึกต้อง callout driver (kernel) — แต่ระดับ observe
// เริ่มจาก WFP audit event ที่มี remote IP + app_id ก่อน แล้วต่อ SNI ใน phase ขั้นสูง

/// emit observation event เมื่อ agent เปิด connection (จาก WFP ALE_AUTH_CONNECT audit)
#[cfg(windows)]
pub fn observe_connection(remote_ip: u32, remote_port: u16, app_id: &str)
    -> dek_domain_schema::TelemetryEventEnvelope
{
    dek_domain_schema::TelemetryEventEnvelope {
        event_type: "network_observation".into(),
        // ... resolve remote_ip → domain (reverse/SNI), map app_id → agent
        data: serde_json::json!({
            "remote_ip": ipv4_string(remote_ip),
            "remote_port": remote_port,
            "app": app_id,
            "enforcement_plane": "wfp_windows",
        }),
        ..Default::default()
    }
}
```

> ตาม research SNI ใน ClientHello อ่านได้ผ่าน WFP callout (แบบที่ Win11 EPP ทำ) — phase แรกใช้ remote IP + app_id (user-mode) ก่อน, SNI callout (kernel) เป็น phase ขั้นสูงเมื่อต้องการ domain แม่นยำโดยไม่พึ่ง reverse DNS

---

## 5. Telemetry: WFP audit → spool → Cloud (ส่งได้จริง)

```rust
// เชื่อม WFP enforcement event → dek-secure-spool (offline-safe) → Cloud
// ตาม research MsSecWfp ของ MDE ก็ emit ETW event แบบนี้

pub async fn emit_enforcement_telemetry(
    spool: &dek_secure_spool::Spool,
    decision: &str,         // "block" | "permit"
    remote: &str, app: &str, policy_id: &str,
) -> anyhow::Result<()> {
    let event = dek_domain_schema::TelemetryEventEnvelope {
        event_type: "decision_log".into(),
        data: serde_json::json!({
            "decision": decision,
            "remote": remote, "app": app,
            "policy_id": policy_id,
            "enforcement_plane": "wfp_windows",
            "ts": chrono::Utc::now().to_rfc3339(),
        }),
        ..Default::default()
    };
    // ผ่าน spool ที่มีอยู่ — ทนเครือข่ายแย่/offline (GracePeriod/LKG)
    spool.enqueue(serde_json::to_vec(&event)?).await?;
    Ok(())
}
```

> ใช้ `dek-secure-spool` ที่มีอยู่ — telemetry ไหลเข้า pipeline เดียวกับ decision log อื่น ไม่แตกเส้นทางใหม่ และ offline-safe (เครื่อง user ปิด ๆ เปิด ๆ ได้)

---

## 6. Architecture: PEP flow บน Windows (ครบวงจร)

```
AI agent (browser/python) เปิด connection ไป claude.ai
        │
        ▼
WFP ALE_AUTH_CONNECT layer  ◄── DEK เพิ่ม filter ที่นี่ (user-mode, ไม่ต้อง driver)
        │
        ├─ filter match? ──► PDP decision (dek-policy-router)
        │                         │
        │         ┌───────────────┼───────────────┐
        │      BLOCK            PERMIT          REDIRECT (ขั้นสูง)
        │         │               │               │
        │    drop conn       allow conn      → mcp-proxy (transparent)
        │         │               │               │
        ▼         ▼               ▼               ▼
   OBSERVE: remote IP + app_id + (SNI ขั้นสูง)
        │
        ▼
   TELEMETRY: → dek-secure-spool → Cloud (offline-safe)
```

**3 ระดับการ deploy (เลือกตามสิทธิ์/ความพร้อม):**

| ระดับ | ต้องการ | enforce | observe | effort |
|-------|---------|---------|---------|--------|
| **L1 — WFP user-mode** | admin rights (ไม่ต้อง driver) | block IP/port/app ✅ | remote IP + app ✅ | ต่ำ — แก้ apply_rules |
| **L2 — WFP + SNI callout** | signed kernel driver | + block by domain ✅ | + SNI domain ✅ | กลาง |
| **L3 — Transparent proxy** | callout driver + redirect | + DPI/parameter-level ✅ | + full payload ✅ | สูง |

**เริ่มที่ L1** — แก้ `apply_rules()` ให้ add filter จริง enforce ได้ทันทีโดยไม่ต้องเขียน driver (ตาม research user-mode WFP ทำ block ที่ ALE_AUTH_CONNECT ได้เลย)

---

## 7. ทำไมไม่ใช้ PEP type อื่นบน Windows

- **mcp-proxy เดี่ยว ๆ**: ต้องให้ agent ตั้ง config ชี้มา proxy — blackbox AI ในเครื่อง user ไม่ทำให้ (ไม่ transparent) ใช้ได้เฉพาะ MCP client ที่เรา rewrite config ได้
- **Envoy ext-authz**: ต้องมี Envoy sidecar — ไม่มีบนเครื่อง user ทั่วไป เหมาะ server/k8s ไม่ใช่ endpoint
- **eBPF**: เป็น Linux — Windows ไม่มี (มี eBPF-for-Windows แต่ยังไม่ stable พอ production)
- **WFP คือคำตอบเดียวที่ transparent + native บน Windows** — ตาม research เป็น platform ที่ EDR/firewall ทุกตัวใช้

---

## 8. แผน Implement

### Phase 1 — WFP enforce จริง (L1, ไม่ต้อง driver) 🔴 ทำก่อน

```
P1-1  apply_rules() → FwpmFilterAdd0 จริง (block IP/port)  [§3.1]
P1-2  add_app_filter() — block/permit ตาม process  [§3.2]
P1-3  clear_rules() → FwpmFilterDeleteById0 + เก็บ filter_id
P1-4  emit telemetry → spool  [§5]
P1-5  เชื่อม CompiledNetworkRules จาก PDP → WFP filter
```

### Phase 2 — Observe + domain

```
P2-1  observe_connection → telemetry (remote IP + app)  [§4]
P2-2  resolve IP → domain (reuse blackbox AI signature)
P2-3  (L2) SNI callout driver — block by domain
```

### Phase 3 — Transparent proxy (L3, ขั้นสูง)

```
P3-1  WFP connect-redirect callout driver (signed)
P3-2  redirect → mcp-proxy ที่มีอยู่ → DPI/parameter-level
```

### Acceptance

1. policy "block claude.ai" → agent ต่อไม่ติดจริง (ไม่ใช่แค่ log)
2. policy "browser ต่อได้เฉพาะ approved AI" → app filter ทำงาน
3. clear_rules ถอน filter จริง (ตรวจด้วย `netsh wfp show filters`)
4. ทุก block/permit → telemetry เข้า spool → เห็นใน dashboard
5. ทำงานโดยไม่ต้องตั้ง proxy/sidecar ในเครื่อง user (transparent)
6. ต้องการแค่ admin rights (L1) ไม่ต้อง kernel driver
7. `cargo test -p dek-windows-wfp` + clippy `-D warnings` ผ่าน (cross-compile windows target)

---

## 9. สรุป

**สาเหตุ:** บน Windows ที่ไม่มี proxy/sidecar PEP เดียวที่ transparent คือ WFP แต่ `apply_rules()` ของ `dek-windows-wfp` **เป็น no-op** — เปิด engine ได้แต่ไม่เคย add filter จริง จึงไม่ enforce อะไรเลย ส่วน proxy/wrapper ต้องให้ agent cooperate (ตั้ง config) ซึ่ง blackbox AI ไม่ทำให้

**แก้:** ทำ WFP ให้ enforce จริงตาม research — **L1 (user-mode `FwpmFilterAdd0` ที่ ALE_AUTH_CONNECT)** บล็อก IP/port/app ได้ทันทีโดยไม่ต้องเขียน kernel driver, observe ด้วย remote IP + app_id (+ SNI callout ใน L2), telemetry ผ่าน `dek-secure-spool` ที่มีอยู่ และขั้นสูง L3 ใช้ WFP connect-redirect ทำให้ mcp-proxy ที่มีอยู่ transparent

WFP คือคำตอบเดียวที่ native + transparent บน Windows — eBPF เป็น Linux, Envoy ต้อง sidecar, proxy เดี่ยวไม่ transparent เริ่ม Phase 1 แก้ `apply_rules()` ได้ผลทันทีเพราะ user-mode WFP block ที่ ALE_AUTH_CONNECT ได้เลย (เป็นสิ่งที่ EDR/firewall ทุกตัวใช้) — code ต่อกับ type จริง (`NetworkEnforcer`, `CompiledNetworkRules`, `dek-secure-spool`)

---

*วิเคราะห์จาก repo ณ `ed30a30` + research Windows WFP enforcement ปี 2026 — code เป็น reference (Windows API ต้อง test บน target จริง) ต้อง view ไฟล์ก่อนแก้ และรัน clippy -D warnings; WFP filter ต้อง admin rights, L2/L3 ต้อง signed driver*
