#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::duplicated_attributes)]
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

fn workspace_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn bin(name: &str) -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(name)
        .with_extension(std::env::consts::EXE_EXTENSION)
}

struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

async fn wait_http(url: &str, tries: u32) -> Result<()> {
    let c = reqwest::Client::new();
    for _ in 0..tries {
        if c.get(url).send().await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(500)).await;
    }
    anyhow::bail!("timeout waiting for {url}")
}

async fn setup_local_cp() -> Result<Proc> {
    assert!(
        Command::new("cargo")
            .args(["build", "--workspace"])
            .status()
            .await?
            .success(),
        "workspace build failed"
    );

    let dash_dir = workspace_dir()
        .join("apps")
        .join("local-admin-dashboard")
        .join("dist");

    let lcp = Command::new(bin("local-control-plane"))
        .current_dir(workspace_dir())
        .env("DEK_DASHBOARD_DIR", dash_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn local-control-plane")?;

    wait_http("http://127.0.0.1:3000/v1/tenants/local/registry/agents", 20).await?;
    Ok(Proc(lcp))
}

#[tokio::test]
#[ignore = "runs the local control plane e2e"]
async fn local_control_plane_e2e() -> Result<()> {
    let _lcp = setup_local_cp().await?;

    let c = reqwest::Client::new();

    // Test 1: Verify the UI is being served at the root (fallback to index.html)
    let ui_res = c.get("http://127.0.0.1:3000/").send().await?;
    assert_eq!(
        ui_res.status().as_u16(),
        200,
        "Dashboard UI must be served at root"
    );
    let html = ui_res.text().await?;
    assert!(html.contains("<html"), "Response must be HTML");

    // Test 2: Create a dummy agent in the registry
    let create_req = serde_json::json!({
        "meta": {
            "schema_version": "1.0",
            "tenant_id": "local",
            "workspace_id": "default",
            "environment_id": "dev",
            "created_at": "2026-06-09T00:00:00Z",
            "updated_at": "2026-06-09T00:00:00Z",
            "created_by": "local-admin",
            "updated_by": "local-admin",
            "source": "manual",
            "status": "active",
            "tags": []
        },
        "agent_id": "agent-e2e-1",
        "name": "e2e-test-agent",
        "agent_type": "claude_desktop",
        "runtime": { "runtime_name": "local" },
        "entrypoints": [],
        "declared_tools": [],
        "declared_resources": [],
        "identity": {},
        "trust_level": "system",
        "capabilities": [],
        "labels": {}
    });

    let res = c
        .post("http://127.0.0.1:3000/v1/tenants/local/registry/agents")
        .json(&create_req)
        .send()
        .await?;

    assert_eq!(
        res.status().as_u16(),
        201,
        "Agent creation should return HTTP 201 Created"
    );

    // Test 3: List agents and ensure the new agent is in the response
    let list_res = c
        .get("http://127.0.0.1:3000/v1/tenants/local/registry/agents")
        .send()
        .await?;
    assert_eq!(list_res.status().as_u16(), 200);
    let agents: Vec<serde_json::Value> = list_res.json().await?;
    assert!(agents.iter().any(|a| a["agent_id"] == "agent-e2e-1"));

    Ok(())
}
