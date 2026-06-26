#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::duplicated_attributes
)]

use reqwest::Client;

mod common;

#[tokio::test]
async fn demo_capability_profiles_are_opt_in_and_isolated() {
    std::env::set_var("POLLEK_ENABLE_DEMO_PROFILES", "1");

    let harness = common::LocalControlPlaneHarness::start().await;
    let base = harness.base_url.clone();
    let client = Client::new();

    let real = client
        .get(format!(
            "{base}/v1/tenants/local/devices/local/capability-snapshot-v2?mode=desktop_advanced"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_ne!(real["contract"]["reason_code"], "demo_fixture");
    assert!(!real["device_id"]
        .as_str()
        .unwrap_or_default()
        .starts_with("demo_"));

    let demo = client
        .get(format!(
            "{base}/v1/tenants/local/devices/local/capability-snapshot-v2?mode=desktop_advanced&demo_os=windows&demo_profile=ready"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(demo["device_id"], "demo_windows_ready");
    assert_eq!(demo["os"]["family"], "windows");
    assert_eq!(demo["contract"]["reason_code"], "demo_fixture");

    let windows_wfp = demo["control_methods"]
        .as_array()
        .unwrap()
        .iter()
        .find(|method| method["method_id"] == "windows_wfp")
        .expect("windows_wfp demo method");
    assert_eq!(windows_wfp["status"], "available");
    assert_eq!(windows_wfp["warm_check"], "passed");
    assert!(windows_wfp["limitations_en"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap_or_default().contains("Demo fixture")));

    let after_demo = client
        .get(format!(
            "{base}/v1/tenants/local/devices/local/capability-snapshot-v2?mode=desktop_advanced"
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_ne!(after_demo["contract"]["reason_code"], "demo_fixture");
    assert!(!after_demo["device_id"]
        .as_str()
        .unwrap_or_default()
        .starts_with("demo_"));

    std::env::remove_var("POLLEK_ENABLE_DEMO_PROFILES");
}
