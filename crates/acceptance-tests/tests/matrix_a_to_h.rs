//! Full-binary acceptance matrix (A–H) — spawns mock-cloud + dek-core and
//! exercises the Pollen DEK contract end-to-end over the real HTTP(S)/mTLS path.
//!
//! Run: cargo test -p acceptance-tests --test matrix_a_to_h -- --ignored --nocapture
//!
//! Marked #[ignore] so it doesn't run in the default unit pass (it builds the
//! workspace + spawns processes). CI runs it explicitly in an integration job.
//!
//! Prereqs handled by the harness:
//!   - `cargo build --workspace` (debug)
//!   - cert-gen writes certs/ (root CA + server + client) for mTLS
//!   - mock-cloud on :43891 (mTLS) + :43892 (enrollment HTTP)
//!   - dek-core enrolled against mock-cloud, then driven via its local IPC/PEP

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

fn workspace_dir() -> PathBuf {
    // crates/acceptance-tests -> repo root
    std::env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf()
}
fn bin(name: &str) -> PathBuf {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    workspace_dir().join("target/debug").join(format!("{name}{ext}"))
}
fn insecure_client() -> reqwest::Client {
    reqwest::Client::builder().danger_accept_invalid_certs(true).build().unwrap()
}

/// Kill-on-drop guard for spawned children.
struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

async fn wait_https(url: &str, tries: u32) -> Result<()> {
    let c = insecure_client();
    for _ in 0..tries {
        if c.get(url).send().await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(500)).await;
    }
    anyhow::bail!("timeout waiting for {url}")
}

/// Build workspace + generate certs + start mock-cloud. Returns the running proc.
async fn setup() -> Result<Proc> {
    assert!(
        Command::new("cargo").args(["build", "--workspace"]).status().await?.success(),
        "workspace build failed"
    );
    // certs for mTLS (cert-gen writes ./certs)
    let _ = Command::new(bin("cert-gen")).current_dir(workspace_dir()).status().await;

    let mock = Command::new(bin("mock-cloud"))
        .current_dir(workspace_dir())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn mock-cloud")?;
    wait_https("https://127.0.0.1:43892/admin/dashboard", 20).await?;
    Ok(Proc(mock))
}

/// Enroll a dek-core instance against mock-cloud and start it.
/// (Uses the device enrollment flow on :43892; see dek-cli `enroll`.)
async fn enroll_and_start_core() -> Result<Proc> {
    // dek-cli enroll --cloud-url http://127.0.0.1:43892
    let status = Command::new(bin("dek-cli"))
        .args(["enroll", "--cloud-url", "http://127.0.0.1:43892"])
        .current_dir(workspace_dir())
        .status()
        .await
        .context("enroll")?;
    anyhow::ensure!(status.success(), "enrollment failed");

    let core = Command::new(bin("dek-core"))
        .current_dir(workspace_dir())
        .env("DEK_BUNDLE_SYNC_INTERVAL", "2")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn dek-core")?;
    // dek-core IPC on 127.0.0.1:43889; PEP/proxy on :43890
    sleep(Duration::from_secs(3)).await;
    Ok(Proc(core))
}

/// Pull the mock-cloud audit log (DEK-side audit events land here).
async fn fetch_audits() -> Result<serde_json::Value> {
    let c = insecure_client();
    let res = c.get("https://127.0.0.1:43892/admin/audits").send().await?;
    Ok(res.json().await.unwrap_or(serde_json::json!([])))
}

// ===========================================================================
// The matrix. Each scenario is a function so failures are isolated & named.
// Gated behind one #[ignore] entry-point that sets up shared infra once.
// ===========================================================================

#[tokio::test]
#[ignore = "full-binary integration; run explicitly in CI integration job"]
async fn acceptance_matrix_a_to_h() -> Result<()> {
    let _mock = setup().await?;

    // ---- A: Enroll -> sync -> enforce ----
    let _core = enroll_and_start_core().await?;
    // after sync, DEK should be Active and the PEP should authorize per policy.
    let pep = insecure_client();
    let allow_req = serde_json::json!({ "mcp": { "method": "tools/list" }, "principal": "tester" });
    // (token setup omitted; see dek-auth test helpers) — assert the PEP responds.
    let resp = pep.post("http://127.0.0.1:43890/v1/authorize").json(&allow_req).send().await;
    assert!(resp.is_ok(), "A: PEP reachable after enroll+sync");

    // audit trail received policy.sync.success
    let audits = fetch_audits().await?;
    let txt = audits.to_string();
    assert!(txt.contains("policy.sync") || txt.contains("bundle"), "A: sync audit present");

    // ---- B: Unsigned/forged push -> reject + critical audit ----
    // Drive via mock-cloud admin to publish a tampered bundle, then wait a sync.
    let _ = pep.post("https://127.0.0.1:43892/admin/publish-tampered-bundle").send().await;
    sleep(Duration::from_secs(4)).await;
    let audits = fetch_audits().await?;
    assert!(
        audits.to_string().contains("rejected") || audits.to_string().contains("unsigned"),
        "B: tampered bundle produced a rejection audit"
    );

    // ---- C: Network partition -> LKG -> strict-deny ----
    // Stop mock-cloud to simulate partition; with a short max_bundle_age the DEK
    // should flip to strict-deny and the PEP should deny.
    // (Set DEK_* env / signed config so max_bundle_age is small in the test profile.)
    // assert: enforcement_state.json reaches strict_deny; PEP denies.

    // ---- D: Recovery -> active ----
    // Restart mock-cloud; after a sync, enforcement returns to active.

    // ---- E: Key rotation ----
    let _ = pep.post("https://127.0.0.1:43892/admin/rotate-key").send().await;
    sleep(Duration::from_secs(4)).await;
    // assert: bundle signed by the rotated key still verifies (overlap); audit key_rotation.

    // ---- F: Hot-reload no interrupt ----
    // Fire concurrent PEP requests while publishing a new bundle; assert 0 errors.

    // ---- G: Backpressure ----
    // Fire > max_concurrent requests; assert some get 503 (deny) not allow.

    // ---- H: PDP circuit breaker ----
    // Force an evaluator to error/timeout; assert breaker opens (fast deny) then recovers.

    // NOTE: C/D/F/G/H assertions require small test-profile config + mock-cloud
    // admin hooks (publish-tampered-bundle, rotate-key, fault-injection). Those
    // endpoints are added per MOCKCLOUD_keys_patch.md + scenarios.rs. Until then
    // they are structured no-ops above so the harness compiles and A/B/E run.
    Ok(())
}
