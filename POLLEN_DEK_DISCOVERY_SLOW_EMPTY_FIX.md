# Pollen DEK — แก้ Auto Discovery: สแกนนานแต่ไม่เจอ + ขยาย Footprint Coverage

**วันที่:** 21 มิถุนายน 2026 | **HEAD:** `acddb09`
**อาการจริง:** กด Deep Scan แล้วสแกนนานมาก แต่ไม่เจอ AI สักตัว (user กด cancel เอง เพราะรอนานเกินไป)
**วิธี:** ตรวจโค้ด discovery จริง (หาสาเหตุ "นาน" และ "ไม่เจอ") + deep research footprint forensics ของ local AI ปี 2026

---

## 1. สาเหตุที่แท้จริง (ตรวจโค้ดยืนยันทุกข้อ)

อาการแยกเป็น 2 ปัญหาที่ต้องแก้คนละจุด:

### 1.1 ทำไม "ไม่เจอสักตัว" — scanner ที่ควรเจอถูกปิด/ไม่มี source

| Scanner | สถานะจริง (จากโค้ด) | ผล |
|---------|---------------------|-----|
| **process scan** | เทียบ `fingerprint_process()` กับ catalog 14 ตัว | 🟡 เจอเฉพาะถ้ามี **process แยก** (Ollama/Claude Desktop) — web app ใน browser ไม่เจอ |
| **web AI (browser)** | `enable_browser_history_scan: false`, `enable_browser_session_scan: false` (default) | 🔴 **ปิดอยู่** — Claude/ChatGPT/DeepSeek ใน tab ไม่ถูกสแกน |
| **web AI (network SNI)** | `enable_network_sni_scan: true` **แต่ `SniFlowSource` ไม่มี implementation** | 🔴 `sni_source = None` → ข้าม method นี้ทั้งหมด |
| **local model probe** | probe 11434/1234/8000 | 🟢 เจอ ถ้ารัน Ollama/LM Studio |
| **mcp config** | สแกน config file | 🟢 เจอ ถ้ามี MCP client |

**ข้อสรุป:** ถ้าเครื่องทดสอบเปิดแค่ Claude/ChatGPT/DeepSeek ใน **browser tab** (ไม่มี Ollama/MCP/desktop app) → **ทุก scanner ที่จะเจอมันถูกปิดหรือไม่มี source** → เจอ 0 ตัว ตรงกับอาการเป๊ะ

โค้ดยืนยัน (`config.rs`):
```rust
enable_browser_history_scan: false,  // ← ปิด (privacy guard)
enable_browser_session_scan: false,  // ← ปิด
enable_network_sni_scan: true,       // เปิด แต่...
```
และ `grep impl SniFlowSource` = **ไม่มี** → SNI scan เปิดแต่ไม่มีตัวป้อน flow → ไม่ทำงาน

### 1.2 ทำไม "นานมาก" — scan แบบ sequential + ไม่มี timeout รวม

ตรวจ `orchestrator.rs::run_scan` — สแกนทุก source **เรียงต่อกัน (sequential)** ไม่มี parallel, ไม่มี overall timeout:

```rust
// 1. Process Scan  → วน process ทั้งหมด เทียบ fingerprint ทีละตัว
// 2. MCP Scan       → อ่าน config หลาย path
// 3. Local Model Probe → probe 3 port, แต่ละ port timeout 500ms
// 4. ... web_ai ...
```

จุดที่ทำให้ช้า:
- **local model probe เรียงกัน** — probe 11434 (500ms) + 1234 (500ms) + 8000 (500ms) ถ้า port ปิดต้องรอ timeout ครบ = 1.5s+ เฉพาะ probe (ควร parallel)
- **process scan** วน process ทั้งหมดในเครื่อง (อาจหลายร้อย) เทียบ fingerprint
- **browser history** (ถ้าเปิด) copy SQLite + query 1000 rows ต่อ profile ต่อ browser
- **ไม่มี overall timeout** → ถ้า method ใดค้าง scan ค้างทั้งหมด ไม่มี deadline ให้คืนผลบางส่วน

**ข้อสรุป:** scan ช้าเพราะ sequential + probe timeout สะสม + ไม่มี deadline ให้คืนผลที่เจอแล้วก่อน

---

## 2. แก้ทันที: ให้เจอ + เร็ว

### 2.1 เปิด browser scan + ใส่ SNI source (แก้ "ไม่เจอ")

```rust
// config.rs — เปิด session scan default (privacy-safe กว่า history)
impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            min_fingerprint_confidence: 0.5,
            // session = tab ที่เปิดอยู่ตอนนี้ (ไม่ใช่ประวัติทั้งหมด) — privacy-safe กว่า history
            enable_browser_session_scan: true,   // ← เปิด: เจอ tab ที่เปิดอยู่
            enable_browser_history_scan: false,  // คงปิด (ต้อง consent — มีประวัติทั้งหมด)
            enable_network_sni_scan: true,
            ..Default::default()
        }
    }
}
```

> เหตุผล: **session scan = tab ที่เปิด "อยู่ตอนนี้"** (จาก `Current Session`/`sessionstore`) ซึ่งตรงโจทย์ "Claude เปิดอยู่ใน tab" และ privacy-safe กว่า history (ไม่เห็นประวัติย้อนหลังทั้งหมด) — เปิด default ได้ ส่วน history คงต้อง consent

### 2.2 ใส่ SniFlowSource implementation (จาก spool ที่ตกลงกัน)

```rust
// crates/dek-agent-discovery/src/sni_source.rs — impl ตัวแรก (อ่านจาก spool)
use crate::web_ai_scan::{SniFlow, SniFlowSource};
use std::sync::Arc;
use std::time::Duration;

/// อ่าน SNI/flow จาก telemetry spool ที่ DEK เก็บอยู่แล้ว (ไม่สร้าง IPC ใหม่)
pub struct SpoolFlowSource {
    store: Arc<dyn FlowStore>,  // abstraction เหนือ spool/telemetry store
}

impl SpoolFlowSource {
    pub fn new(store: Arc<dyn FlowStore>) -> Self { Self { store } }
}

impl SniFlowSource for SpoolFlowSource {
    fn recent_flows(&self, since: Duration) -> Vec<SniFlow> {
        // ดึง network_observation event ในหน้าต่างเวลา → map เป็น SniFlow
        self.store.recent_network_events(since).into_iter()
            .filter_map(|ev| Some(SniFlow {
                browser_pid: ev.pid,
                sni_host: ev.sni_host?,   // domain_hint จาก SNI/DNS correlation
                ts: ev.ts,
            }))
            .collect()
    }
}

pub trait FlowStore: Send + Sync {
    fn recent_network_events(&self, since: Duration) -> Vec<NetworkEventRecord>;
}
pub struct NetworkEventRecord { pub pid: Option<u32>, pub sni_host: Option<String>, pub ts: u64 }
```

> ต่อ orchestrator: `DiscoveryOrchestrator::with_sni_source(Arc::new(SpoolFlowSource::new(store)))` — ถ้ายังไม่มี flow ใน spool ก็คืน empty (ไม่ error) แล้วพึ่ง session scan แทน

### 2.3 ทำให้เร็ว: parallel + overall timeout + คืนผลบางส่วน

```rust
// orchestrator.rs — รัน scanner แบบ parallel พร้อม deadline
use tokio::time::{timeout, Duration};

pub async fn run_scan(&self, scan_id: &str, req: &serde_json::Value)
    -> Result<(DiscoveryScanJob, Vec<DiscoveredAgentCandidateV2>)>
{
    let mut job = /* ... Running ... */;
    let overall_deadline = Duration::from_secs(15);  // ← deadline รวม

    // รัน scanner ทั้งหมด "พร้อมกัน" แทน sequential
    let scan_all = async {
        let (process, mcp, model, web) = tokio::join!(
            run_process_scan(),
            run_mcp_scan(),
            run_model_probe_parallel(),   // probe ทุก port พร้อมกัน (ดู 2.4)
            run_web_ai_scan(self.sni_source.clone()),
        );
        let mut all = Vec::new();
        for r in [process, mcp, model, web] {
            if let Ok(mut ev) = r { all.append(&mut ev); }
        }
        all
    };

    // ถ้าเกิน deadline → คืนผลที่เจอแล้ว (ไม่ปล่อยให้ user รอจนกด cancel)
    let all_evidence = match timeout(overall_deadline, scan_all).await {
        Ok(ev) => ev,
        Err(_) => {
            job.error = Some("scan exceeded deadline; returning partial results".into());
            Vec::new()  // หรือเก็บ partial จาก channel ที่ stream มา
        }
    };
    // ... aggregate → candidates ...
}
```

### 2.4 probe port แบบ parallel (แทน sequential timeout สะสม)

```rust
// local_model_probe.rs — probe ทุก port พร้อมกัน
pub async fn probe_all_parallel() -> Vec<DiscoveryEvidenceV2> {
    let probes = vec![
        ("ollama", 11434, "/api/tags"),
        ("openai_compat", 1234, "/v1/models"),  // LM Studio
        ("openai_compat", 8000, "/v1/models"),  // vLLM
        ("openai_compat", 8080, "/v1/models"),  // llama.cpp
        ("openai_compat", 5000, "/v1/models"),  // text-gen-webui
        ("jan", 1337, "/v1/models"),            // Jan
    ];
    // ยิงทุก port พร้อมกัน — รวมแล้วใช้แค่ ~500ms (timeout ตัวเดียว) ไม่ใช่ 3s
    let futures = probes.into_iter().map(|(kind, port, path)| async move {
        probe_one(kind, port, path).await
    });
    futures::future::join_all(futures).await.into_iter().flatten().collect()
}
```

> ผล: probe 6 port พร้อมกัน = ~500ms (timeout เดียว) แทน 3s+ (6 × 500ms sequential) — scan เร็วขึ้นชัดเจน

---

## 3. Deep Research: ขยาย Footprint Coverage ให้ครอบทุกชนิด

จาก research forensic ของ local AI ปี 2026 — งานวิจัย "Forensic Implications of Localized AI" ทำ systematic analysis ของ Ollama, LM Studio, llama.cpp บน Windows/Linux พบ artifact: installation footprint, config file, model cache, prompt history, network activity และ recover plaintext prompt history ใน JSON, model usage log, file signature ที่ใช้ระบุตัวได้

นี่คือ footprint ที่ควรเพิ่มเข้า catalog เพื่อ coverage สูงสุด:

### 3.1 หมวด footprint ที่ครอบ AI ทุกชนิด

| หมวด AI | footprint signature | ตัวอย่าง |
|---------|---------------------|----------|
| **Browser web AI** | session/history + SNI domain | Claude, ChatGPT, DeepSeek, Gemini, Copilot, Perplexity, Poe, HuggingChat, Grok, Mistral |
| **Local model server** | process + port + API + model cache | Ollama, LM Studio, llama.cpp, vLLM, Jan, GPT4All, text-gen-webui, LocalAI |
| **CLI coding agent** | binary in PATH + config | Claude Code, Aider, Goose, Open Interpreter, Cline, Autohand, Claude Engineer |
| **IDE agent/extension** | extension dir + config | Cursor, Windsurf, Continue, Copilot, Cline, Roo Code |
| **Desktop app** | process + config | Claude Desktop, ChatGPT Desktop, LM Studio GUI |
| **Python framework** | venv + import + jupyter | LangChain, LlamaIndex, CrewAI, AutoGen, smolagents, DSPy, Haystack |
| **MCP server** | config (mcpServers/servers/...) | ทุก MCP client |
| **Container AI** | image + port | ollama/vllm/langchain containers |

### 3.2 Footprint forensic ที่เพิ่มความแม่นยำ (จาก research)

```json
// เพิ่มใน signature catalog — artifact ที่ research พบว่าระบุตัวได้แม่น
{
  "id": "ollama",
  "process_names": ["ollama", "ollama.exe"],
  "ports": [11434],
  "forensic_artifacts": {
    "model_cache": {
      "macos": ["~/.ollama/models"],
      "windows": ["%USERPROFILE%/.ollama/models"],
      "linux": ["~/.ollama/models", "/usr/share/ollama/.ollama/models"]
    },
    "config": ["~/.ollama/config.json"],
    "manifests": ["~/.ollama/models/manifests"],  // ระบุ model ที่ติดตั้ง
    "logs": ["~/.ollama/logs/server.log"]          // prompt usage
  }
},
{
  "id": "lmstudio",
  "process_names": ["LM Studio", "lms"],
  "ports": [1234],
  "forensic_artifacts": {
    "models": ["~/.cache/lm-studio/models", "~/.lmstudio/models"],
    "config": ["~/.lmstudio/config.json"],
    "chat_history": ["~/.lmstudio/conversations"]   // research: plaintext JSON
  }
},
{
  "id": "jupyter_ai",
  "agent_type": "notebook_agent",
  "process_names": ["jupyter", "jupyter-lab", "python"],
  "ports": [8888],
  "forensic_artifacts": {
    "config": ["~/.jupyter", "~/.ipython"],
    "ai_extension": ["jupyter_ai_config.json"]      // jupyter-ai magic
  }
}
```

### 3.3 Python framework detection (venv + import scan)

```rust
// crates/dek-agent-discovery/src/python_framework_scan.rs
// research: AI agent framework รันบน python — ตรวจ venv ที่ติดตั้ง library เหล่านี้
const AI_FRAMEWORK_PACKAGES: &[(&str, &str)] = &[
    ("langchain", "LangChain"), ("llama_index", "LlamaIndex"),
    ("crewai", "CrewAI"), ("autogen", "AutoGen"),
    ("smolagents", "smolagents"), ("dspy", "DSPy"),
    ("haystack", "Haystack"), ("openai", "OpenAI SDK"),
    ("anthropic", "Anthropic SDK"), ("ollama", "Ollama Python"),
];

/// สแกน virtual env + site-packages หา AI framework
pub fn scan_python_frameworks() -> Vec<DiscoveryEvidenceV2> {
    let mut evidence = Vec::new();
    for venv in find_python_environments() {
        let site_packages = venv.join("lib");  // หา site-packages
        for (pkg, name) in AI_FRAMEWORK_PACKAGES {
            // เช็คว่ามี package dir หรือ .dist-info
            if has_package(&site_packages, pkg) {
                evidence.push(make_framework_evidence(name, pkg, &venv));
            }
        }
    }
    evidence
}

/// หา venv: ~/.virtualenvs, conda envs, .venv ใน project, pyenv
fn find_python_environments() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let mut envs = vec![];
    for p in ["~/.virtualenvs", "~/miniconda3/envs", "~/anaconda3/envs",
              "~/.conda/envs", "~/.pyenv/versions"] {
        let expanded = expand_tilde(p, &home);
        if let Ok(rd) = std::fs::read_dir(&expanded) {
            for e in rd.flatten() { if e.path().is_dir() { envs.push(e.path()); } }
        }
    }
    // + scan project dirs สำหรับ .venv (best-effort, จำกัด depth)
    envs
}
```

### 3.4 ขยาย local model server (จาก research ปี 2026)

research ปี 2026 ระบุ tool ใหม่: Ollama, LM Studio, text-generation-webui, GPT4All, Jan, LocalAI — เพิ่มทั้งหมดใน catalog พร้อม port:

```rust
const LOCAL_MODEL_SERVERS: &[(&str, u16, &str)] = &[
    ("ollama", 11434, "/api/tags"),
    ("lmstudio", 1234, "/v1/models"),
    ("vllm", 8000, "/v1/models"),
    ("llamacpp", 8080, "/v1/models"),
    ("text-gen-webui", 5000, "/v1/models"),
    ("gpt4all", 4891, "/v1/models"),
    ("jan", 1337, "/v1/models"),
    ("localai", 8080, "/v1/models"),
];
```

---

## 4. UX: แสดงความคืบหน้า (กัน user กด cancel เพราะคิดว่าค้าง)

ปัญหาหนึ่งคือ user ไม่เห็นว่า scan กำลังทำอะไร เลยคิดว่าค้าง → stream progress:

```rust
// orchestrator: ส่ง progress event ระหว่างสแกน (ผ่าน channel → SSE → UI)
pub enum ScanProgress {
    SourceStarted { source: String },
    SourceCompleted { source: String, found: usize },
    CandidateFound { name: String, confidence: f64 },
}
// UI แสดง "Scanning processes... ✓ (2 found)", "Scanning browser tabs... ✓ (3 found)"
// → user เห็นว่าทำงานอยู่ ไม่กด cancel + เห็นผลทยอยขึ้นทันที (ไม่ต้องรอจบ)
```

> design ให้ candidate ทยอยขึ้นทันทีที่เจอ (incremental) ไม่ต้องรอ scan จบทั้งหมด — แก้ทั้ง "นาน" (รู้สึกเร็วขึ้น) และ "ไม่เห็นอะไร" (เห็นผลทันที)

---

## 5. แผน Implement

### Phase A — แก้ "ไม่เจอ" (impact สูงสุด)
```
A1  เปิด enable_browser_session_scan default = true  [§2.1]
A2  impl SpoolFlowSource + wire เข้า orchestrator  [§2.2]
A3  ยืนยัน session scan เจอ Claude/ChatGPT/DeepSeek ใน tab
```

### Phase B — แก้ "นาน"
```
B1  scanner รัน parallel (tokio::join!) + overall timeout 15s  [§2.3]
B2  probe port parallel (join_all)  [§2.4]
B3  stream progress + incremental candidate → UI  [§4]
```

### Phase C — ขยาย coverage
```
C1  เพิ่ม local model servers (jan/gpt4all/localai/text-gen-webui)  [§3.4]
C2  python_framework_scan (langchain/crewai/autogen/...)  [§3.3]
C3  forensic_artifacts ใน catalog (model cache/config/history)  [§3.2]
C4  เพิ่ม CLI agents (aider/goose/open-interpreter/cline)
```

### Acceptance
1. เปิด Claude/ChatGPT/DeepSeek ใน tab → Deep Scan เจอครบใน < 15s
2. scan ไม่เกิน 15s แม้ไม่เจ��อะไร (มี deadline)
3. candidate ทยอยขึ้นทันทีที่เจอ (ไม่ต้องรอจบ)
4. UI แสดง progress ต่อ source (user ไม่คิดว่าค้าง)
5. เจอ local model server 8 ชนิด + python framework + CLI agent
6. probe parallel < 1s (ไม่ใช่ 3s+ sequential)
7. `cargo test -p dek-agent-discovery` + clippy `-D warnings` ผ่าน

---

## 6. สรุป

**สาเหตุ "นานแต่ไม่เจอ" (2 ปัญหาแยกกัน):**
- **ไม่เจอ:** เครื่องทดสอบเปิด AI ใน **browser tab** แต่ `enable_browser_session_scan = false` (ปิด) + `SniFlowSource` ไม่มี implementation → scanner ที่จะเจอ web AI ถูกปิดหมด → เจอ 0
- **นาน:** scan แบบ **sequential** + probe port timeout สะสม (3s+) + ไม่มี overall deadline → user รอจนกด cancel

**แก้:** เปิด session scan (privacy-safe — เห็นแค่ tab ที่เปิดอยู่) + ใส่ SpoolFlowSource + รัน parallel + deadline 15s + stream progress ให้เห็นผลทันที

**ขยาย coverage** จาก research forensic ปี 2026: เพิ่ม local model server 8 ชนิด, python framework (langchain/crewai/...), CLI agent, forensic artifact (model cache/config/chat history ที่ research พบว่าระบุตัวได้แม่น) — ครอบ AI ทุกชนิดบนเครื่อง local

เริ่ม Phase A แก้ "ไม่เจอ" ได้ทันที (เปิด flag + ใส่ source) แล้ว Phase B แก้ "นาน" — code ต่อกับ type จริง (`DiscoveryConfig`, `SniFlowSource`, `DiscoveryEvidenceV2`)

---

*วิเคราะห์จาก repo ณ `acddb09` + research local AI forensic ปี 2026 — code เป็น reference ต้อง view ไฟล์ก่อนแก้ + clippy -D warnings + Rust 1.85; browser session scan privacy-safe แต่ history ยังต้อง consent*
