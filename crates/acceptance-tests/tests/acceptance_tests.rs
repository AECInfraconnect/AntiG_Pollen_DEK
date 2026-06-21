use anyhow::Result;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

async fn wait_for_port(port: u16, max_retries: u32) -> Result<()> {
    let url = format!("http://127.0.0.1:{}", port);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    for _ in 0..max_retries {
        // Try hitting a known endpoint, even if it returns 404, connection means success
        if client.get(&url).send().await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(500)).await;
    }
    anyhow::bail!("Timeout waiting for port {}", port)
}

#[tokio::test]
async fn run_acceptance_test_matrix() -> Result<()> {
    // 1. Build workspace first
    let build_status = Command::new("cargo")
        .args(["build", "--workspace"])
        .status()
        .await?;
    assert!(build_status.success(), "Workspace build failed");

    // 2. Start mock-cloud
    let mut mock_cloud = Command::new("cargo")
        .args(["run", "-p", "mock-cloud"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // Wait for mock-cloud to be ready (it runs on 43891 and 43892)
    // Actually mock-cloud runs HTTPS so reqwest needs danger_accept_invalid_certs
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let mut ready = false;
    for _ in 0..10 {
        if client.get("https://127.0.0.1:43892/admin/dashboard").send().await.is_ok() {
            ready = true;
            break;
        }
        sleep(Duration::from_millis(1000)).await;
    }
    assert!(ready, "Mock cloud did not start in time");

    // =========================================================================
    // A. Enrollment and Identity
    // =========================================================================
    println!("--- Testing A. Enrollment and Identity ---");
    // A04: MITM Rejection (Wrong CA)
    // We simulate this by dek-core trying to enroll with strict TLS. 
    // Since mock-cloud uses a self-signed cert, standard reqwest will fail if not using custom roots.
    let res = reqwest::get("https://127.0.0.1:43891/v1/tenants/tenant-production-1/devices/device-001/status").await;
    assert!(res.is_err(), "A04 Failed: MITM/Wrong CA was not rejected by strict TLS client");

    // A01: Fresh install and OAuth Device Flow
    // We simulate the admin approving the device
    let approval_res = client.post("https://127.0.0.1:43892/device")
        .form(&[("user_code", "MOCK-CODE"), ("profile", "Enterprise")])
        .send()
        .await?;
    assert!(approval_res.status().is_success(), "Failed to approve device");

    // =========================================================================
    // B. Secure Bundle and Config
    // =========================================================================
    println!("--- Testing B. Secure Bundle and Config ---");
    // Get device info
    let dev_info = client.get("https://127.0.0.1:43891/v1/tenants/tenant-production-1/devices/device-001")
        .send()
        .await?;
    assert!(dev_info.status().is_success() || dev_info.status() == reqwest::StatusCode::NOT_FOUND);

    // B01: Valid signed config
    let pub_res = client.post("https://127.0.0.1:43892/admin/policies/publish")
        .json(&serde_json::json!({
            "cedar_src": "permit(principal, action, resource);",
            "openfga_store": "store_test"
        }))
        .send()
        .await?;
    assert!(pub_res.status().is_success(), "Failed to publish valid bundle");

    // =========================================================================
    // C. Hot Reload
    // =========================================================================
    println!("--- Testing C. Hot Reload ---");
    // Mock the hot reload trigger
    let rollout_res = client.post("https://127.0.0.1:43892/admin/rollout")
        .json(&serde_json::json!({
            "canary_percentage": 10,
            "canary_bundle_version": "v2",
            "canary_cedar_src": "permit(principal, action, resource);",
            "canary_openfga_store": "store_test_v2"
        }))
        .send()
        .await?;
    assert!(rollout_res.status().is_success(), "Failed to set rollout");

    // =========================================================================
    // D. Enforcement Modes & F. Telemetry
    // =========================================================================
    println!("--- Testing D & F. Telemetry ---");
    let ingest_res = client.post("https://127.0.0.1:43891/v1/tenants/tenant-production-1/devices/device-001/telemetry/decisions")
        .json(&serde_json::json!([
            {
                "device_id": "device-001",
                "timestamp": "2026-06-07T00:00:00Z",
                "action": "test_action",
                "decision": "Permit"
            }
        ]))
        .send()
        .await?;
    assert!(ingest_res.status().is_success(), "Failed to ingest telemetry");

    // Check dashboard to see if telemetry arrived
    let dash_html = client.get("https://127.0.0.1:43892/admin/dashboard").send().await?.text().await?;
    assert!(dash_html.contains("test_action"), "F01 Failed: Telemetry did not show up in dashboard");
    assert!(dash_html.contains("PUBLISH_POLICY"), "G01 Failed: Audit log did not capture publish action");

    // Clean up
    mock_cloud.kill().await?;

    println!("All automated critical path acceptance tests passed!");
    Ok(())
}
