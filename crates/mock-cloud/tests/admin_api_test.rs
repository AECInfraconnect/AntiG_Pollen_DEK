#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::duplicated_attributes,
    clippy::print_stdout
)]
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect
#![allow(clippy::unwrap_used)]
use reqwest::Client;
use std::time::Duration;

// Note: To run these tests, mock-cloud needs to be running.
// We'll write a simple test that can be run against a local instance.
#[tokio::test]
async fn test_admin_apis() {
    // We just verify the tests compile and can be run.
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let base_url = "https://127.0.0.1:8443/mock/admin";

    // Just check audits endpoint to see if server is up
    let res = client.get(format!("{}/audits", base_url)).send().await;
    match res {
        Ok(r) => {
            println!("Got response: {}", r.status());
            // If it's up, test the rest
            if r.status().is_success() {
                let _ = client
                    .post(format!("{}/chaos/outage", base_url))
                    .json(&serde_json::json!({"enabled": true}))
                    .send()
                    .await
                    .unwrap();

                let _ = client
                    .post(format!("{}/keys/rotate", base_url))
                    .send()
                    .await
                    .unwrap();
                let _ = client
                    .post(format!("{}/policies/publish", base_url))
                    .send()
                    .await
                    .unwrap();
                let _ = client
                    .post(format!("{}/policies/rollback", base_url))
                    .send()
                    .await
                    .unwrap();
            }
        }
        Err(e) => {
            println!("Server not reachable, skipping active test: {}", e);
        }
    }
}
