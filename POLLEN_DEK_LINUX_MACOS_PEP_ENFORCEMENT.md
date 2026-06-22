# Pollen DEK — Enforce + Observe AI Agents บน Linux (eBPF) และ macOS (NetworkExtension)

**วันที่:** 21 มิถุนายน 2026 | **HEAD:** `da8ed28`
**โจทย์:** ทำให้ PEP enforce + observe AI agents + ส่ง telemetry ได้จริงบน Linux และ macOS (macOS เตรียม code ไว้ก่อน พัฒนาเต็มภายหลัง)
**วิธี:** ตรวจโค้ด PEP จริง + deep research Linux eBPF cgroup + macOS NetworkExtension ปี 2026

---

## 0. สถานะปัจจุบัน: Linux พร้อมจริง / macOS ยังเป็น stub

ตรวจโค้ดแล้วต่างจาก Windows อย่างชัดเจน:

| OS | PEP | สถานะ |
|----|-----|-------|
| **Linux** | `dek-ebpf-prog` + `dek-ebpfd` (cgroup) | 🟢 **enforce จริงแล้ว** — `dek_connect4` มี verdict 0=block/1=allow + 3-tier policy |
| **macOS** | `dek-macos-nefilter` | 🔴 **stub ทั้งหมด** — `apply_rules()` แค่ log, ไม่มี Network Extension จริง |

**Linux — หลักฐานว่าทำงานจริง** (`dek-ebpf-prog/src/main.rs`):
```rust
#[cgroup_sock_addr(connect4)]
pub fn dek_connect4(ctx: SockAddrContext) -> i32 {
    // 3-tier policy resolution:
    // 1) cgroup-specific:  CGROUP_POLICY_MAP.get(&cgroup_id)
    // 2) LPM trie (IP/CIDR): VERDICT_MAP.get(&key)
    // 3) port policy:        PORTS_MAP.get(&dest_port)
    // return 0 = BLOCK connection, 1 = ALLOW
}
```
นี่คือ PEP ที่ enforce จริง — บล็อก outbound connection **ก่อนถูกสร้าง** ที่ cgroup/connect4 hook โดย agent ไม่ต้อง cooperate (transparent) — เป็นโมเดลที่ Windows WFP ควรเลียนแบบ (ตามเอกสารก่อนหน้า)

**macOS — หลักฐานว่าเป็น stub** (`dek-macos-nefilter/src/lib.rs`):
```rust
// Stub implementation for cross-compilation testing.
fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
    info!("OS Enforcement (macOS): pushing compiled rules...");  // ← log อย่างเดียว
    Ok(())
}
```

---

## 1. Linux — ปรับปรุงให้สมบูรณ์ (eBPF มีฐานดีแล้ว)

eBPF enforcement มีแล้ว แต่ research + ตรวจโค้ดพบจุดที่ควรเสริมให้ครบวงจร observe + telemetry:

### 1.1 ช่องว่างที่เหลือ

| ส่วน | สถานะ | ต้องเสริม |
|------|-------|-----------|
| connect4 enforcement | ✅ มี | — |
| connect6 (IPv6) | 🟡 ตรวจว่ามี handler ไหม | เพิ่มถ้าขาด |
| DNS capture (observe) | ✅ observe-only (returns 1) | resolve domain → policy |
| EgressEvent → telemetry | 🟡 มี RingBuf EVENTS | ต่อ ringbuf → spool |
| domain-level enforce | 🔴 มีแต่ IP/port | DNS→IP correlation |

### 1.2 ต่อ eBPF EgressEvent → telemetry spool (observe + ส่งจริง)

eBPF ส่ง `EgressEvent` ผ่าน RingBuf `EVENTS` แล้ว แต่ user-space ต้องอ่านแล้ว forward เข้า spool:

```rust
// dek-ebpfd/src/lib.rs เสริม — อ่าน ringbuf แล้วส่ง telemetry
use aya::maps::ring_buf::RingBuf;
use std::sync::Arc;

pub async fn pump_egress_events(
    bpf: &mut aya::Ebpf,
    spool: Arc<dek_secure_spool::Spool>,
) -> anyhow::Result<()> {
    let mut events: RingBuf<_> = RingBuf::try_from(bpf.map_mut("EVENTS")
        .ok_or_else(|| anyhow::anyhow!("EVENTS map not found"))?)?;

    loop {
        while let Some(item) = events.next() {
            // parse EgressEvent (pid, cgroup_id, dest_ip, dest_port, verdict)
            let ev: dek_ebpf_common::EgressEvent = parse_event(&item)?;
            let envelope = dek_domain_schema::TelemetryEventEnvelope {
                event_type: "decision_log".into(),
                data: serde_json::json!({
                    "decision": if ev.verdict == 0 { "block" } else { "allow" },
                    "pid": ev.pid,
                    "cgroup_id": ev.cgroup_id,
                    "dest_ip": ipv4_string(ev.dest_ip),
                    "dest_port": ev.dest_port,
                    "enforcement_plane": "ebpf_cgroup_linux",
                    "ts": chrono::Utc::now().to_rfc3339(),
                }),
                ..Default::default()
            };
            // ส่งผ่าน spool ที่มีอยู่ (offline-safe)
            spool.enqueue(serde_json::to_vec(&envelope)?).await?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
```

> EgressEvent มีอยู่แล้วในโค้ด — เหลือแค่ pump เข้า spool ทำให้ทุก block/allow ที่ eBPF ตัดสิน ส่ง telemetry ได้จริงเหมือน decision log อื่น

### 1.3 Domain-level enforcement (DNS → IP correlation)

ตอนนี้ enforce ได้แค่ IP/port — แต่ AI agent ต่อ claude.ai ผ่าน CDN (IP เปลี่ยน) ต้อง correlate DNS:

```rust
// dek-ebpfd/src/dns_cache.rs เสริม — เมื่อ DNS capture เห็น claude.ai → A record
// → push IP ที่ resolve ได้เข้า VERDICT_MAP ให้ตรง policy
pub fn on_dns_response(
    domain: &str, resolved_ips: &[u32],
    policy: &DomainPolicy,           // จาก blackbox AI signature (เอกสารก่อนหน้า)
    verdict_map: &mut aya::maps::LpmTrie<MapData, u32, PolicyVerdict>,
) -> anyhow::Result<()> {
    if let Some(v) = policy.verdict_for(domain) {
        for &ip in resolved_ips {
            // push IP → verdict ลง map ที่ eBPF connect4 อ่าน
            let key = aya::maps::lpm_trie::Key::new(32, ip);
            verdict_map.insert(&key, v, 0)?;
        }
        info!(domain, count = resolved_ips.len(), "domain policy → IP verdicts pushed");
    }
    Ok(())
}
```

> flow: DNS capture (observe) เห็น query `claude.ai` → resolve → push IP เข้า VERDICT_MAP → connect4 บล็อก IP นั้นได้ตาม policy domain — ทำให้ "block claude.ai" ทำงานแม้ IP เปลี่ยน (DEK มี `dns_cache.rs` + hickory-proto อยู่แล้ว)

### 1.4 Linux PEP architecture (ครบวงจร)

```
AI agent ต่อ claude.ai
   │
   ├─ DNS query → dek_dns_capture (cgroup_skb, observe) → resolve → push IP เข้า VERDICT_MAP
   │
   ▼
connect4 hook (cgroup_sock_addr) ◄── enforce จริงที่นี่
   │  3-tier: cgroup → LPM trie (IP) → port
   ├─ verdict 0 → BLOCK (connection ไม่เกิด)
   └─ verdict 1 → ALLOW
   │
   ▼ EgressEvent → RingBuf EVENTS
pump_egress_events → dek-secure-spool → Cloud (telemetry จริง)
```

---

## 2. macOS — เตรียม Code ให้พร้อม (Network Extension)

จาก research macOS PEP ที่ถูกต้องคือ **NEFilterDataProvider** (ไม่ใช่ pf ที่ deprecated)

### 2.1 PEP type ที่ถูกต้องสำหรับ macOS

จาก research NEFilterDataProvider ให้ TCP/UDP flow + IP traffic อื่น (ICMP) และ filter provider **modify traffic ไม่ได้ แค่ให้ verdict allow/drop flow** และ macOS 26 มี URL filter API ใหม่ที่ตัดสินด้วย **ทั้ง URL** ไม่ใช่แค่ hostname โดยไม่ละเมิด privacy

**โครงสร้าง 2 ส่วน (ตาม Apple):**
- **Container app** (Rust/Swift) — ใช้ `NEFilterManager` config + push rule ผ่าน IPC ไป provider
- **System Extension** (`NEFilterDataProvider`) — รับ flow → ตัดสิน allow/drop → emit telemetry

> ข้อจำกัดที่ต้องรู้จาก research: in-app traffic บางตัวไม่ถึง Data Provider ถ้า config ไม่ครบ (`filterBrowsers=true` + `filterSockets=true`) ต้องตั้งทั้งคู่; และ flow source IP อาจเป็น 0.0.0.0 ตอน handleNewFlow ต้อง resolve ภายหลัง

### 2.2 เตรียม Rust IPC client (เชื่อม container app ↔ extension)

```rust
// dek-macos-nefilter/src/lib.rs — แทน stub ด้วย IPC client จริง (เตรียมไว้)
#![allow(unsafe_code)]
use anyhow::Result;
use dek_domain_schema::CompiledNetworkRules;
use dek_enforcement_api::NetworkEnforcer;
use std::sync::Arc;

pub struct NeFilterClient {
    connected: bool,
    socket_path: String,
    spool: Option<Arc<dek_secure_spool::Spool>>,  // Optional (consistency กับ WFP)
    #[cfg(target_os = "macos")]
    stream: Option<std::os::unix::net::UnixStream>,
}

impl NeFilterClient {
    pub fn new(spool: Option<Arc<dek_secure_spool::Spool>>) -> Self {
        Self {
            connected: false,
            socket_path: "/var/run/pollen/nefilter.sock".into(),
            spool,
            #[cfg(target_os = "macos")]
            stream: None,
        }
    }
}

impl NetworkEnforcer for NeFilterClient {
    fn start(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::net::UnixStream;
            warn!("macOS NE: requires signing + entitlement + notarization + MDM (dev prototype)");
            // เชื่อม UDS ที่ NEFilterDataProvider เปิดไว้
            match UnixStream::connect(&self.socket_path) {
                Ok(s) => { self.stream = Some(s); self.connected = true;
                           info!("connected to PollenDEKNetworkExtension"); Ok(()) }
                Err(e) => anyhow::bail!("NE socket connect failed: {e}"),
            }
        }
        #[cfg(not(target_os = "macos"))]
        anyhow::bail!("macOS NE not compiled on this OS");
    }

    fn apply_rules(&self, rules: &CompiledNetworkRules) -> Result<()> {
        if !self.connected {
            warn!("NE not connected; rules not pushed");
            return Ok(());
        }
        #[cfg(target_os = "macos")]
        {
            use std::io::Write;
            // serialize rules → ส่งผ่าน UDS ไป provider
            let payload = serde_json::to_vec(&NeRuleMessage::from_compiled(rules))?;
            if let Some(stream) = &self.stream {
                let mut s = stream.try_clone()?;
                // framed message: [u32 len][json]
                s.write_all(&(payload.len() as u32).to_be_bytes())?;
                s.write_all(&payload)?;
                info!(policy = %rules.policy_id, "NE rules pushed via IPC");
            }
        }
        Ok(())
    }

    fn clear_rules(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        if let Some(stream) = &self.stream {
            use std::io::Write;
            let mut s = stream.try_clone()?;
            let clear = serde_json::to_vec(&NeRuleMessage::clear())?;
            let _ = s.write_all(&(clear.len() as u32).to_be_bytes());
            let _ = s.write_all(&clear);
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.connected = false;
        #[cfg(target_os = "macos")]
        { self.stream = None; }
        Ok(())
    }
}

/// message format ระหว่าง container app ↔ NEFilterDataProvider
#[derive(serde::Serialize, serde::Deserialize)]
pub struct NeRuleMessage {
    pub action: String,              // "apply" | "clear"
    pub policy_id: String,
    pub block_domains: Vec<String>,  // claude.ai, openai.com...
    pub block_cidrs: Vec<String>,
    pub block_ports: Vec<u16>,
}

impl NeRuleMessage {
    #[cfg(target_os = "macos")]
    fn from_compiled(rules: &CompiledNetworkRules) -> Self {
        Self {
            action: "apply".into(), policy_id: rules.policy_id.clone(),
            block_domains: vec![], // map จาก rules
            block_cidrs: vec![], block_ports: vec![],
        }
    }
    fn clear() -> Self {
        Self { action: "clear".into(), policy_id: String::new(),
               block_domains: vec![], block_cidrs: vec![], block_ports: vec![] }
    }
}
```

### 2.3 เตรียม System Extension skeleton (Swift — verdict logic)

```swift
// macos/PollenDEKNetworkExtension/FilterDataProvider.swift (เตรียมไว้)
import NetworkExtension

class FilterDataProvider: NEFilterDataProvider {
    // rule ที่ push มาจาก container app ผ่าน IPC
    private var blockedDomains: Set<String> = []
    private var blockedPorts: Set<UInt16> = []

    override func startFilter(completionHandler: @escaping (Error?) -> Void) {
        // listen UDS /var/run/pollen/nefilter.sock รับ rule จาก container
        startIpcListener()
        completionHandler(nil)
    }

    // ตัดสินทุก flow ใหม่ — allow หรือ drop (research: filter ให้ verdict ไม่ modify)
    override func handleNewFlow(_ flow: NEFilterFlow) -> NEFilterNewFlowVerdict {
        guard let socketFlow = flow as? NEFilterSocketFlow,
              let endpoint = socketFlow.remoteEndpoint as? NWHostEndpoint else {
            return .allow()
        }
        let host = endpoint.hostname
        let port = UInt16(endpoint.port) ?? 0

        // emit telemetry ทุก decision → เขียนลง shared spool / XPC
        let blocked = blockedDomains.contains(where: { host.contains($0) })
                      || blockedPorts.contains(port)
        emitTelemetry(host: host, port: port, decision: blocked ? "block" : "allow")

        return blocked ? .drop() : .allow()
    }
}
```

> macOS 26 ขั้นสูง: ใช้ **NEURLFilterManager** (URL filter API ใหม่) ตัดสินด้วยทั้ง URL ผ่าน Bloom filter — เหมาะกับ blackbox AI ที่อยากคุมระดับ path ไม่ใช่แค่ domain (เตรียมไว้ phase หลัง)

### 2.4 ข้อกำหนด deploy macOS (เตรียมเอกสาร)

จาก stub เดิมระบุถูกแล้ว — macOS NE ต้อง: **signing + entitlement approval (com.apple.developer.networking.networkextension) + notarization + MDM deployment** ไม่เหมือน Linux/Windows ที่รันได้ทันที — นี่คือเหตุผลที่ "เตรียม code ก่อน พัฒนาเต็มภายหลัง" ถูกต้อง

---

## 3. PEP Type สรุปต่อ OS (เทียบ 3 platform)

| | Linux | macOS | Windows (เอกสารก่อน) |
|--|-------|-------|---------------------|
| **PEP native** | eBPF cgroup (connect4) | NEFilterDataProvider | WFP (ALE_AUTH_CONNECT) |
| **enforce ระดับ** | IP/port/cgroup + DNS→domain | flow allow/drop + URL (26) | IP/port/app |
| **transparent** | ✅ (ไม่ต้อง config agent) | ✅ | ✅ |
| **deploy** | รันได้ (root/CAP_BPF) | ต้อง sign+entitlement+MDM | admin rights |
| **สถานะ DEK** | 🟢 enforce จริงแล้ว | 🔴 stub → เตรียม code | 🔴 no-op → ต้องแก้ |
| **telemetry** | RingBuf → spool | XPC/UDS → spool | WFP audit → spool |

**หลักการร่วม:** ทุก OS ใช้ **native kernel/system enforcement** ที่ transparent (agent ไม่ต้อง cooperate) + verdict ที่ connection-establishment time + telemetry ผ่าน `dek-secure-spool` เดียวกัน — ต่างกันแค่ API ของแต่ละ OS

---

## 4. แผน Implement

### Phase L — Linux ให้ครบวงจร (มีฐานแล้ว) 🟢
```
L1  pump_egress_events: RingBuf EVENTS → spool (telemetry จริง)  [§1.2]
L2  DNS→IP correlation: domain policy → VERDICT_MAP  [§1.3]
L3  connect6 (IPv6) handler ถ้ายังขาด
L4  ยืนยัน block claude.ai ทำงาน end-to-end
```

### Phase M — macOS เตรียม code (พัฒนาเต็มภายหลัง) 🔴
```
M1  NeFilterClient IPC จริง (UDS framed message) + Optional spool  [§2.2]
M2  NeRuleMessage format (container ↔ extension)
M3  System Extension skeleton (Swift, handleNewFlow verdict)  [§2.3]
M4  เอกสาร deploy: signing/entitlement/notarization/MDM  [§2.4]
M5  (phase หลัง) NEURLFilterManager สำหรับ URL-level
```

### Acceptance
**Linux:**
1. policy "block claude.ai" → agent ต่อไม่ติดจริง (DNS→IP→connect4 block)
2. ทุก block/allow → telemetry เข้า spool → dashboard เห็น
3. enforce ทำงานต่อ cgroup เฉพาะ (คุม agent รายตัว)

**macOS (เตรียม code):**
4. NeFilterClient compile + cross-compile ผ่าน (cfg-gated)
5. IPC message format ครบ (apply/clear)
6. System Extension skeleton มี handleNewFlow verdict logic
7. เอกสาร deploy requirement ครบ
8. `cargo test -p dek-ebpfd -p dek-macos-nefilter` + clippy `-D warnings` ผ่าน

---

## 5. สรุป

**Linux:** eBPF **enforce จริงแล้ว** — `dek_connect4` มี 3-tier verdict (cgroup/LPM-trie/port) บล็อก connection ก่อนสร้างได้แบบ transparent เหลือเสริม 2 จุด: pump `EgressEvent` (RingBuf ที่มีอยู่) เข้า spool เพื่อ telemetry จริง + DNS→IP correlation เพื่อ enforce ระดับ domain (claude.ai แม้ IP เปลี่ยน) — DEK มี `dns_cache.rs` + hickory อยู่แล้ว ต่อได้เลย

**macOS:** PEP ที่ถูกคือ **NEFilterDataProvider** (TCP/UDP flow verdict allow/drop, ไม่ใช่ pf ที่ deprecated) — เตรียม code ครบ: Rust IPC client (UDS framed) + Swift System Extension skeleton (handleNewFlow) + message format + เอกสาร deploy (sign/entitlement/notarize/MDM) ตามที่ stub เดิมระบุถูก พัฒนาเต็มภายหลังเพราะ deploy ซับซ้อนกว่า Linux/Windows มาก ส่วน macOS 26 มี URL filter API ใหม่สำหรับคุมระดับ path

**หลักการร่วมทั้ง 3 OS:** native enforcement ที่ transparent + verdict ตอน connect + telemetry ผ่าน spool เดียวกัน — Linux พร้อม, Windows ต้องแก้ WFP no-op (เอกสารก่อน), macOS เตรียม code ไว้ — code ต่อกับ type จริง (`NetworkEnforcer`, `CompiledNetworkRules`, `EgressEvent`, `dek-secure-spool`)

---

*วิเคราะห์จาก repo ณ `da8ed28` + research Linux eBPF cgroup + macOS NetworkExtension ปี 2026 — code เป็น reference; eBPF ต้อง test บน Linux จริง, macOS NE ต้อง sign+entitlement; ต้อง view ไฟล์ก่อนแก้ + clippy -D warnings + Rust 1.85*
