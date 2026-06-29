#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::duplicated_attributes
)]

use reqwest::Client;
use serde_json::json;

mod common;

fn opt(value: Option<&str>) -> Option<String> {
    value.map(Into::into)
}

macro_rules! observed {
    (
        $event_id:expr,
        $ts_ms:expr,
        $agent_id:expr,
        $session_id:expr,
        $activity:expr,
        $action:expr,
        $resource_classification:expr,
        $provenance_taint:expr,
        $path:expr,
        $host_reputation:expr,
        $in_allowlist:expr $(,)?
    ) => {
        json!({
            "event_id": $event_id,
            "ts_ms": $ts_ms,
            "agent_id": $agent_id,
            "session_id": $session_id,
            "activity": $activity,
            "action": $action,
            "resource_classification": opt($resource_classification),
            "provenance_taint": opt($provenance_taint),
            "path": opt($path),
            "host": null,
            "host_reputation": opt($host_reputation),
            "in_allowlist": $in_allowlist
        })
    };
}

#[tokio::test]
async fn detection_coverage_exposes_rules_sensors_and_research_basis() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let client = Client::new();

    let coverage = client
        .get(format!(
            "{}/v1/tenants/local/detections/coverage",
            harness.base_url
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(coverage["schema_version"], "pollek.detection.coverage.v1");
    assert_eq!(coverage["manifest_integrity"], "verified");
    assert_eq!(coverage["rule_count"], 5);
    assert!(coverage["rules"].as_array().unwrap().iter().any(|rule| {
        rule["id"] == "POLLEK-DET-0002"
            && rule["can_stop_next_time"] == true
            && rule["privacy_note"]
                .as_str()
                .unwrap()
                .contains("does not store raw prompt")
    }));
    assert!(
        coverage["sensors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|sensor| {
                sensor["id"] == "mcp_proxy"
                    && sensor["achievable_level"] == "enforce"
                    && sensor["deterministic_decision"]
                        .as_str()
                        .unwrap()
                        .contains("does not depend on this source alone")
            }),
        "{coverage}"
    );
    assert!(
        coverage["research_basis"]
            .as_array()
            .unwrap()
            .iter()
            .any(|basis| basis["framework"] == "OWASP Top 10 for LLM Applications"),
        "{coverage}"
    );
}

#[tokio::test]
async fn detection_evaluate_groups_sequences_by_agent_and_session() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let client = Client::new();

    let cross_agent = json!({
        "events": [
            observed!("r1", 1000, "agent-a", "sess-1", "FileRead", "read", Some("sensitive"), None, Some("/data/customers.csv"), None, false),
            observed!("u1", 1050, "agent-b", "sess-1", "WebUpload", "upload", None, None, None, Some("neutral"), false)
        ]
    });
    let response = client
        .post(format!(
            "{}/v1/tenants/local/detections/evaluate",
            harness.base_url
        ))
        .json(&cross_agent)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(response["evaluated_events"], 2);
    assert!(response["fired"].as_array().unwrap().is_empty());

    let same_session = json!({
        "events": [
            observed!("r2", 1000, "agent-a", "sess-2", "FileRead", "read", Some("sensitive"), None, Some("/data/customers.csv"), None, false),
            observed!("u2", 1050, "agent-a", "sess-2", "WebUpload", "upload", None, None, None, Some("neutral"), false)
        ]
    });
    let response = client
        .post(format!(
            "{}/v1/tenants/local/detections/evaluate",
            harness.base_url
        ))
        .json(&same_session)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert!(
        response["fired"].as_array().unwrap().iter().any(|hit| {
            hit["agent_id"] == "agent-a"
                && hit["session_id"] == "sess-2"
                && hit["rule"]["id"] == "POLLEK-DET-0002"
                && hit["matched_event_ids"] == json!(["r2", "u2"])
        }),
        "{response}"
    );
}

#[tokio::test]
async fn sensor_consent_and_install_record_honest_setup_state() {
    let harness = common::LocalControlPlaneHarness::start().await;
    let client = Client::new();
    let base = harness.base_url;

    let consent = client
        .post(format!(
            "{base}/v1/tenants/local/detections/sensors/browser_ai_extension/consent"
        ))
        .json(&json!({
            "accepted": true,
            "scopes": ["browser_tabs", "prompt_metadata"],
            "note": "E2E user approved browser observe setup"
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(consent["status"], "accepted");
    assert_eq!(consent["record"]["raw_content_stored"], false);

    let install = client
        .post(format!(
            "{base}/v1/tenants/local/detections/sensors/browser_ai_extension/install"
        ))
        .json(&json!({
            "accepted": true,
            "requested_level": "enforce"
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(install["status"], "waiting_for_browser_user_approval");
    assert!(
        install["record"]["fallback"]
            .as_str()
            .unwrap()
            .contains("process metadata"),
        "{install}"
    );

    let sensors = client
        .get(format!("{base}/v1/tenants/local/detections/sensors"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let browser_sensor = sensors["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|sensor| sensor["id"] == "browser_ai_extension")
        .expect("browser sensor should be present");
    assert_eq!(browser_sensor["achieved_level"], "none");
    assert_eq!(browser_sensor["achievable_level"], "observe_only");
    assert_eq!(
        browser_sensor["setup_state"]["schema_version"],
        "pollek.observe.sensor.setup.v1"
    );
    assert!(
        browser_sensor["deterministic_decision"]
            .as_str()
            .unwrap()
            .contains("remaining evidence matrix"),
        "{browser_sensor}"
    );
}
