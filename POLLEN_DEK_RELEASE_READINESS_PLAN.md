# AntiG_Pollen_DEK — แนวทางปรับปรุงระบบให้พร้อม Release จริง

**ฉบับวิเคราะห์เชิงลึก** | วันที่วิเคราะห์: 12 มิถุนายน 2026 | HEAD: `29b4d16` ("fix: eBPF target path and missing certs folder in tests")
**Repo:** `AECInfraconnect/AntiG_Pollen_DEK` — Rust workspace 53 crates + dashboard React/TS + eBPF + WFP + WASM plugins
**วิธีวิเคราะห์:** clone repo จริง, อ่าน workflow ทั้ง 12 ไฟล์, ดึง annotation จาก GitHub Actions run จริง, ตรวจ tag/release history ย้อนหลัง 18 release attempts, และทดสอบพฤติกรรม cargo บางส่วนในเครื่อง

---

## 1. Executive Summary

โปรเจกต์เดินทางมาไกลมาก — ประวัติ 146 commits ครอบคลุม Phase 0 ถึง GA Readiness แต่ติดอยู่ใน **วงจร "fix(ci)" whack-a-mole** มานาน (มากกว่า 40 commits เป็นการไล่แก้ CI ทีละจุด) สาเหตุรากของปัญหา "CI Failing" และ "No release found" ที่แก้ไม่หายสักทีคือ **ปัญหาเชิงโครงสร้าง 3 ข้อ** ไม่ใช่ bug รายตัว:

| # | ปัญหาราก | อาการที่เห็น | สถานะปัจจุบัน |
|---|----------|--------------|----------------|
| R1 | **Tag ชี้ผิด commit** — tag v1.0.0 / beta.1 / beta.2 เดิมชี้ไป commit เก่าที่ไม่อยู่บน main เลย (orphaned history) | Release รัน 18 ครั้ง fail ทุกครั้ง เพราะ build โค้ดเก่าที่พังเสมอ ไม่ว่าจะแก้ main กี่ครั้ง | ✅ **แก้แล้ว** — tag ทั้ง 3 ถูกย้ายมาที่ `29b4d16` แต่เกิดปัญหาใหม่ (ดู P0-1) |
| R2 | **Test harness build workspace ข้างใน test** — `matrix_a_to_k.rs` สั่ง `cargo build --workspace` เองใน `setup()` โดยเดิมใช้ชื่อ `--exclude` ผิด (`pii_redactor` แทน `pii-redactor-plugin` ซึ่ง cargo แค่เตือนแล้ว build ต่อ) | Integration Tests fail ด้วย exit code 100 ภายใน 4m38s ("workspace build failed") | ✅ ชื่อ exclude แก้แล้ว แต่ **สถาปัตยกรรม build-inside-test ยังอยู่** (ดู P0-2) |
| R3 | **E2E ขาด prerequisite** — ไม่มี `playwright install` และ `DEK_DASHBOARD_DIR` ไม่ถูก set (default `../../apps/...` ชี้ออกนอก repo เมื่อรันจาก root) | local-admin-e2e fail ที่ step "Run local admin E2E" ทุกครั้ง | ✅ **แก้แล้ว** ใน commit `524a747` |

**สถานะ ณ เวลาวิเคราะห์:** การ push tag 3 ตัวพร้อมกันบน commit เดียว ทำให้เกิด workflow storm (Release Supply Chain ×3 + Release Gate ×2 + Integration + CI + e2e + Docs + Security พร้อมกัน 15+ jobs) ผลยังอยู่ในสถานะ Queued / In progress — **ยังไม่มี release ออก** และมีความเสี่ยงเหลืออยู่หลายจุดที่จะทำให้รอบนี้ fail อีก (รายละเอียดส่วนที่ 3)

---

## 2. สิ่งที่แก้สำเร็จแล้ว (ยืนยันจากโค้ดบน main)

เพื่อให้เห็นภาพว่าเหลืออะไรจริง ๆ รายการนี้คือสิ่งที่ commit ชุดล่าสุด (`524a747` → `29b4d16`) จัดการไปแล้วและผมตรวจยืนยันในโค้ดแล้ว:

- **E2E script** (`scripts/e2e-local-admin.sh`): เพิ่ม `npx playwright install --with-deps`, export `DEK_DASHBOARD_DIR="$(pwd)/apps/local-admin-dashboard/dist"`, ตัด fmt/clippy/test ที่ซ้ำกับ workflow ออก
- **Integration workflow**: เพิ่ม `fail-fast: false`, `Swatinem/rust-cache@v2`, `timeout-minutes: 30`
- **Harness excludes**: `matrix_a_to_k.rs` ใช้ชื่อ `pii-redactor-plugin` ถูกต้องแล้ว
- **dek-opa-wasm กลับมาเป็น first-class**: commit `8a8a83a` คืน `adapter-opa` เป็น default feature และถอด `--exclude dek-opa-wasm` ออกจาก workflow ส่วนใหญ่ — ถูกต้องตามหลัก เพราะเดิม exclude ก็ไร้ผลอยู่แล้ว (`dek-router-builder` ดึงมันเข้ามาเป็น dependency ผ่าน default feature เสมอ ซึ่ง `dek-core`, `dek-mcp-proxy`, `dek-ext-authz` ใช้ทั้งหมด)
- **Repo hygiene**: untrack ไฟล์ขยะ/อันตรายแล้ว — `cargo-binstall.exe` (16 MB), `clippy_output.json` (1.5 MB), `master.key.wrapped`, `crates/mock-cloud/rewrite.exe`
- **Release pipeline**: ใช้ `cargo-binstall` ติดตั้ง cargo-auditable, แก้ path eBPF artifact เป็น `crates/dek-ebpf-prog/target/...` (ถูกต้อง เพราะ crate นี้ถูก exclude จาก workspace จึงมี target dir ของตัวเอง), ใส่ `+nightly` ใน eBPF build
- **Security Audit**: เปลี่ยน `continue-on-error: true → false` — ผล "เขียว" ของ cargo audit เป็นของจริงแล้ว ไม่ใช่เขียวปลอม
- **Tags**: ย้ายทั้ง 3 tag มาที่ HEAD ปัจจุบัน

---

## 3. ความเสี่ยงที่ยังเหลือ — เรียงตามลำดับความรุนแรง

### 🔴 P0-1: Tag 3 ตัวบน commit เดียว = 3 Release ซ้อนกัน + workflow storm

ตอนนี้ `v1.0.0`, `v1.0.0-beta.1`, `v1.0.0-beta.2` ชี้ไป `29b4d16` ทั้งหมด ผลที่จะตามมาถ้าทุกอย่างผ่าน:

1. GitHub จะมี **3 releases ของโค้ดเดียวกัน** — และ `v1.0.0` (stable) จะออกพร้อม beta ซึ่งผิด semantics ร้ายแรง: ผู้ใช้จะเข้าใจว่ามี GA แล้วทั้งที่ acceptance matrix เพิ่งจะผ่านครั้งแรก
2. Workflow ทุกตัวที่ trigger `on: push` โดยไม่จำกัด ref (Docs Validation, Security Audit ฯลฯ) ถูกยิงซ้ำต่อ tag → คิวงานยาว, cache ตีกัน, สับสนเวลาไล่ดูผล

**วิธีแก้ (ทำทันที ไม่ต้องรอผลรอบนี้):**

```bash
# เก็บ beta.2 ไว้เป็น release candidate ของรอบนี้ ลบอีกสองตัวทิ้ง
git push origin :refs/tags/v1.0.0
git push origin :refs/tags/v1.0.0-beta.1
git tag -d v1.0.0 v1.0.0-beta.1

# ยกเลิก workflow runs ของ tag ที่ลบ (ผ่าน UI: Actions → เลือก run → Cancel)
# v1.0.0 ค่อยตัดใหม่จาก commit ที่ผ่าน beta-bake period แล้วเท่านั้น
```

และจำกัด trigger ของ workflow ที่ไม่เกี่ยวกับ release ไม่ให้รันบน tag:

```yaml
# docs.yml, security.yml, ci.yml ฯลฯ — ระบุ branches ให้ชัด (มีอยู่แล้วใน ci.yml แต่ docs.yml ไม่มี)
on:
  push:
    branches: [ "main" ]   # ไม่ใส่ tags → tag push จะไม่ trigger
```

### 🔴 P0-2: Acceptance harness ยัง build ทั้ง workspace อยู่ "ข้างใน" test

`matrix_a_to_k.rs::setup()` ยังรัน `cargo build --workspace ...` เป็นส่วนแรกของ test ปัญหาคือ:

- บน runner เย็น (cache miss/เปลี่ยน toolchain) การ build 50+ crates รวม wasmtime, cedar, sqlx ใน debug ใช้เวลา 20–40 นาที → ชน `timeout-minutes: 30` ที่เพิ่งใส่ และ rust-cache จะช่วยได้เฉพาะรอบที่ dependency ไม่เปลี่ยน
- nextest compile test binary หนึ่งรอบ แล้ว test ไป build อีกรอบ — งานซ้ำซ้อนและ debug ยากมากเพราะ build error ไปโผล่เป็น "test failure exit 100" แทนที่จะเป็น build step ที่อ่าน log ตรง ๆ ได้ (นี่คือเหตุผลที่ไล่แก้กันอยู่หลายสิบ commit)

**วิธีแก้:** ย้าย build ออกมาเป็น workflow step แล้วให้ harness แค่ตรวจว่า binary มีอยู่:

```yaml
# .github/workflows/integration.yml — เพิ่มก่อน step nextest
    - name: Pre-build workspace binaries
      timeout-minutes: 40
      run: cargo build --workspace --exclude dek-ebpf-prog --exclude dek-ebpfd --exclude pii-redactor-plugin

    - name: Run Acceptance Tests (JUnit & JSON)
      timeout-minutes: 25
      env:
        DEK_SKIP_HARNESS_BUILD: "1"
      run: cargo nextest run -p acceptance-tests --test matrix_a_to_k --run-ignored ignored-only --profile ci
```

```rust
// matrix_a_to_k.rs::setup() — ครอบส่วน build เดิม
if std::env::var("DEK_SKIP_HARNESS_BUILD").is_err() {
    // ... cargo build เดิม (เก็บไว้ให้รัน local ได้สะดวก)
}
```

ผลพลอยได้: ถ้า build พังจะเห็นทันทีที่ step "Pre-build" พร้อม compiler error เต็ม ๆ ไม่ต้องเดาจาก exit 100 อีก

### 🔴 P0-3: Gitleaks ใน Release Gate จะสะดุด history ของ `master.key.wrapped`

ไฟล์ถูก untrack จาก HEAD แล้วก็จริง แต่ **ยังอยู่ใน git history** (เคย commit และ repo เป็น public) Release Gate มี step `gitleaks/gitleaks-action@v2` ซึ่ง scan commit history → มีโอกาสสูงที่จะ flag แล้ว fail gate นอกจากนี้ action ตัวนี้ต้องการ `GITHUB_TOKEN` ใน env ซึ่งใน release-gate.yml ไม่ได้ส่งให้ (ใน security.yml ถึงกับ comment ทิ้งไว้พร้อม token — แสดงว่าเคยเจอปัญหานี้แล้ว)

**วิธีแก้ 3 ชั้น:**

1. **หมุนกุญแจจริงทันที** — ต่อให้เป็น wrapped key การที่มันเคย public แปลว่าต้องถือว่า compromised ตามหลัก zero-trust ที่ Pollen เองก็ propagate
2. **ล้าง history** ด้วย `git filter-repo` (ทำครั้งเดียว ประสานทีมก่อนเพราะ rewrite ทั้ง repo):
   ```bash
   pip install git-filter-repo
   git filter-repo --invert-paths \
     --path master.key.wrapped --path cargo-binstall.exe \
     --path clippy_output.json --path crates/mock-cloud/rewrite.exe
   git push origin --force --all && git push origin --force --tags
   ```
   ผลพลอยได้: repo เบาลง ~18 MB และ clone เร็วขึ้น
3. **แก้ workflow** ให้ gitleaks ทำงานได้และมี baseline:
   ```yaml
   - name: Gitleaks (Detect secrets)
     uses: gitleaks/gitleaks-action@v2
     env:
       GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
   ```
   ถ้ายังไม่พร้อม rewrite history ให้เพิ่ม `.gitleaksignore` ระบุ fingerprint ของ finding เก่าไว้ชั่วคราว (พร้อม comment ว่า key หมุนแล้ว)

### 🟠 P1-1: Release Gate `build-artifacts` job มี landmine ที่ยังไม่เคยรันถึง

Job นี้อยู่หลัง gate ผ่าน เลยยังไม่เคยถูกทดสอบจริง พบปัญหาจากการอ่านโค้ด:

- **aarch64 .deb จะ fail แน่นอน**: ติดตั้ง `cross` แต่ไม่เคยเรียกใช้ — `cargo deb --target aarch64-unknown-linux-gnu` บน runner x86 ไม่มี aarch64 linker → link error ทันที ต้องเปลี่ยนเป็น `cross build` ก่อนแล้ว `cargo deb --no-build` หรือใช้ runner `ubuntu-24.04-arm`
- **WiX (`candle.exe`/`light.exe`)**: เรียกตรง ๆ โดยไม่ติดตั้ง — image `windows-2025` (ที่ windows-latest กำลัง redirect ไป ตาม notice ใน annotation) ไม่ pre-install WiX v3 บน PATH แล้ว ต้องเพิ่ม `dotnet tool install --global wix` (WiX v4/v5 ซึ่ง syntax .wxs ต่าง) หรือ `choco install wixtoolset` (v3)
- **macOS .pkg ไม่ได้ sign/notarize** — ติดตั้งบนเครื่องผู้ใช้จริงจะโดน Gatekeeper บล็อก (ยอมรับได้สำหรับ beta แต่ต้องเขียนบอกใน release notes)
- ติดตั้ง `cross` จาก git ทุกรอบ (`cargo install cross --git ...`) ช้าและไม่ pin version

**คำแนะนำเชิงกลยุทธ์:** สำหรับ beta อย่าเพิ่งเอา .msi/.pkg/aarch64 มาเป็น blocking path — ตัด job `build-artifacts` ออกจาก release-gate (หรือใส่ `continue-on-error: true` ชั่วคราว) ให้ release.yml ที่ผลิต tar.gz/zip/deb x86_64 + cosign + SBOM เป็นเส้นทางหลักเส้นเดียว แล้วค่อยทำ installer ครบทุก platform ใน milestone GA

### 🟠 P1-2: macOS notarization จะพังทันทีที่ใส่ secrets

`release.yml` ตอนนี้ตัด `--team-id "YOUR_TEAM_ID"` ทิ้งทั้งก้อน แต่ `xcrun notarytool submit` เมื่อ auth ด้วย `--apple-id/--password` **บังคับต้องมี `--team-id`** — step นี้รอดมาตลอดเพราะ `if: env.APPLE_CERT_B64 != ''` ประเมินจาก env ระดับ job (ว่างเสมอ — env ที่ประกาศใน step มองไม่เห็นจาก `if` ของ step ตัวเอง) จึง **ถูก skip โดยบังเอิญ** วันไหนทีมใส่ secrets แล้วย้าย env ขึ้น job level จะระเบิดทันที แก้ให้ถูกตั้งแต่ตอนนี้:

```yaml
      - name: Sign macOS Binaries
        if: matrix.os == 'macos-latest' && secrets.APPLE_CERT_B64 != ''
        env:
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}   # เพิ่ม secret ใหม่
          # ... env เดิม
        run: |
          # ... เดิม
          xcrun notarytool submit staging.zip --apple-id "$APPLE_DEV_ID" \
            --password "$APPLE_APP_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait
```

### 🟠 P1-3: ci.yml ยัง exclude `dek-opa-wasm` — ไม่ consistent กับ workflow อื่น

หลัง commit `8a8a83a` ทุก workflow ถอด exclude นี้ออกแล้ว **ยกเว้น ci.yml** (clippy/test/build ยัง exclude อยู่ทั้ง 3 บรรทัด) ผลคือ unit tests และ clippy `--all-features` ของ crate นี้ไม่เคยถูกรันใน CI หลักทั้งที่มันเป็น default dependency ของ binary หลักทุกตัว — ถ้ามี regression จะไปโผล่ที่ Release Gate แทน (ช้าเกินไป) ลบ `--exclude dek-opa-wasm` ออกจาก ci.yml ทั้ง 3 จุดให้ตรงกัน

### 🟠 P1-4: `cargo install` ยังเป็นคอขวด/จุดเปราะในหลาย workflow

จุดที่ยังคอมไพล์เครื่องมือจาก source ทุกรอบ: `bpf-linker` (release.yml + ci.yml — ตัวนี้หนักเป็นพิเศษ), `cargo-audit` (security.yml, release-gate), `cargo-deny`, `cargo-cyclonedx`, `cargo-deb`, `cargo-machete`, `cross` ทุกตัวกินเวลา 3–10 นาที/ตัว และพังได้เมื่อ toolchain ใหม่ชนกับ MSRV ของเครื่องมือ มาตรฐานเดียวที่ควรใช้ทั้ง repo:

```yaml
      - uses: cargo-bins/cargo-binstall@main
      - run: cargo binstall -y bpf-linker cargo-audit cargo-deny cargo-cyclonedx cargo-deb
```

(หรือ `taiki-e/install-action` ซึ่ง pin version ได้ — เลือกอย่างใดอย่างหนึ่งแล้วใช้ให้เหมือนกันทุกไฟล์)

### 🟡 P2: รายการเก็บกวาดคุณภาพ

- **`sleep 5` ใน e2e script**: แทนที่ด้วย poll loop `for i in $(seq 1 30); do curl -fsS http://127.0.0.1:3000/health && break; sleep 1; done` — `cargo run` รอบที่ binary stale อาจ recompile นานกว่า 5 วินาที
- **playwright.config.ts ไม่มี**: สร้างไฟล์ระบุ `testDir: './e2e'`, `retries: 1`, `reporter: [['junit', {...}]]`, `use: { baseURL: 'http://127.0.0.1:3000' }` เพื่อให้ผล e2e เข้า test report เดียวกับ nextest
- **โครงสร้างซ้ำซ้อน**: `dek-windows-wfp/` ที่ root ซ้ำกับ `crates/dek-windows-wfp` (อันแรกคือ native driver + service แยก workspace — ควร rename เป็น `native/windows-wfp/` พร้อม README อธิบาย), `ui/mock-cloud` (React) vs `crates/mock-cloud` (Rust), service file 3 ที่ (`pollen-dek.service` root / `deploy/` / `packaging/`) — เลือก `packaging/` เป็น source of truth ที่เดียว
- **ไฟล์ root ที่ควรย้าย**: `apply_license*.{ps1,py,sh}`, `check_jobs.py` → `scripts/`; `INTEGRATION_p1_p2_guide.html` → `docs/`
- **`Acceptance Matrix (A-H)`** ชื่อ job ล้าหลัง test จริงที่เป็น A–K แล้ว — แก้ชื่อกันสับสนเวลาอ่านผล
- **Branch protection ยังไม่มี**: เปิด required checks (CI, Integration, local-admin-e2e, Security) บน main เมื่อทุกตัวเขียวเสถียร เพื่อปิดวงจร "push ตรงเข้า main แล้วค่อยมาไล่แก้"
- **Node 20 deprecation warning**: actions/checkout@v4 และ upload-artifact@v4 จะถูกบังคับ Node 24 วันที่ 16 มิ.ย. 2026 (อีก 4 วัน!) — อัปเกรดเป็น `actions/checkout@v5` / `actions/upload-artifact@v5` ทุก workflow ก่อนถึงวันนั้น มิฉะนั้นอาจมี breakage แทรกมาอีกระลอกโดยไม่เกี่ยวกับโค้ดเราเลย

---

## 4. Roadmap สู่ v1.0.0 GA

### Phase A — Stabilize (เป้า: ภายใน 1–2 วัน)
ลบ tag ส่วนเกิน (P0-1) → รอผลรอบ beta.2 ปัจจุบัน → ถ้า Integration ยัง fail ให้ทำ P0-2 (ย้าย build ออกจาก harness) ซึ่งเป็นตัวการที่เหลืออยู่ที่น่าจะชน timeout → อัปเกรด actions เป็น Node 24 ready (P2 ข้อสุดท้าย — มี deadline จริง 16 มิ.ย.)
**Definition of Done:** CI + Integration + local-admin-e2e + Security เขียวพร้อมกันบน main 3 commits ติดต่อกัน

### Phase B — First Real Release (เป้า: ภายใน 3–5 วัน)
แก้ P0-3 (gitleaks + key rotation + history rewrite), P1-2 (notarytool), P1-3 (ci.yml consistency), P1-4 (binstall ทุก workflow), ตัด build-artifacts ออกจาก blocking path (P1-1) → tag `v1.0.0-beta.3` จาก main ที่เขียว → ตรวจว่า Release Supply Chain ผลิตครบ: tar.gz/zip/deb + `dek-ebpf-prog.bpf.o` + SBOM (CycloneDX) + SHA256SUMS + cosign .sig/.pem ทุกไฟล์
**Definition of Done:** หน้า Releases มี beta พร้อม asset ครบ และคำสั่ง verify ตามท้าย release.yml ใช้ได้จริงจากเครื่องนอก

### Phase C — Beta Bake (เป้า: 2–3 สัปดาห์)
Nightly soak (มีอยู่แล้ว ตรวจว่ารันผ่านจริง), ติดตั้งจริงบน Windows/Linux/macOS อย่างละเครื่องผ่าน artifact ที่ release ออกมา (ไม่ใช่ build local), ทดสอบ dual Local/Cloud mode + SPIRE flow ตามแผนที่วางไว้, รวบรวม known issues → ออก beta.4/rc.1 ตามรอบ
**Definition of Done:** zero P0/P1 bugs ค้าง 1 สัปดาห์เต็ม + soak 30 นาทีผ่านติดต่อกัน 7 คืน

### Phase D — GA v1.0.0
ทำ installer ครบ (msi ผ่าน WiX ที่ติดตั้งถูกต้อง, pkg signed+notarized, aarch64 ผ่าน cross หรือ ARM runner), เปิด branch protection, เขียน RELEASE_PROCESS.md ลง repo (ขั้นตอน tag → verify → publish เป็นลายลักษณ์อักษร), แล้วจึงตัด `v1.0.0` จาก rc ตัวสุดท้ายแบบ commit เดียวกันเป๊ะ

---

## 5. Checklist ก่อนกด tag ทุกครั้ง (พิมพ์แปะข้างจอได้)

```text
[ ] main เขียวครบ 4 workflows (CI / Integration / e2e / Security) ที่ commit ที่จะ tag
[ ] tag ใหม่ชี้ HEAD ของ main เท่านั้น — ห้าม tag local commit ที่ยังไม่ push
[ ] มี tag เดียวต่อหนึ่ง release event — ไม่ push หลาย tag พร้อมกัน
[ ] CHANGELOG.md อัปเดต version + วันที่
[ ] เวอร์ชันใน Cargo.toml workspace ตรงกับ tag (ตอนนี้ 1.0.0-beta.1 — ต้อง bump ให้ตรงกับ tag ที่จะออก)
[ ] รอ Release Supply Chain จบแล้วตรวจ asset ครบก่อนประกาศ
```

---

## 6. ภาคผนวก: หลักฐานการวิเคราะห์

- **Integration fail (run #86, `0007850`)**: annotation จริง "Process completed with exit code 100" (nextest = test failure) ที่ 4m38s — เร็วเกินกว่า workspace build จะเสร็จ → inner build fail; ทดสอบยืนยันแล้วว่า cargo ปฏิบัติกับ `--exclude` ชื่อผิดเป็นแค่ warning (`warning: excluded package(s) ... not found in workspace`) จึง build crate ที่ตั้งใจกันออกต่อไปเงียบ ๆ
- **e2e fail (run #69)**: step "Run local admin E2E" — script เดิมไม่มี playwright install และ `DEK_DASHBOARD_DIR` default `../../apps/local-admin-dashboard/dist` (config.rs) ชี้นอก repo เมื่อ CWD = root
- **Release fail 18 ครั้ง**: ตรวจ `git merge-base --is-ancestor` แล้ว tag เดิมทั้ง 3 ไม่เป็น ancestor ของ main (orphaned) — run ล่าสุดก่อนแก้ fail ที่ "Build Binaries (ubuntu) exit 101" + "Build eBPF Bytecode exit 101" ซึ่งคือ bug เก่าที่ main แก้ไปแล้วแต่ tag ยังชี้ของเก่า
- **dek-router-builder feature graph**: `default = ["adapter-cedar", "adapter-opa", "adapter-openfga"]` → `dep:dek-opa-wasm` ถูกดึงเสมอจาก dek-core/mcp-proxy/ext-authz/mcp-stdio-wrapper — เป็นเหตุผลทางเทคนิคที่การ "exclude" ที่ทำกันมาหลายสิบ commit ไม่เคยมีผลจริงต่อ dependency build

*เอกสารนี้สร้างจากการวิเคราะห์ repo สาธารณะ ณ commit `29b4d16` — สถานะ workflow ที่กำลังรันอยู่อาจเปลี่ยนผลบางข้อ แนะนำเช็คหน้า Actions อีกครั้งหลังคิวงานปัจจุบันจบ*
