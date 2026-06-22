use std::net::Ipv4Addr;

/// Convert an IPv4 `u32` (host byte order) back to `Ipv4Addr`.
fn ipv4_from_u32(ip: u32) -> Ipv4Addr {
    Ipv4Addr::from(ip)
}

fn ipv4_string(ip: u32) -> String {
    ipv4_from_u32(ip).to_string()
}

/// แปลง connection observation → telemetry envelope
/// caller (ETW listener หรือ FwpmNetEventsSubscribe0 ในอนาคต) เรียก function นี้
pub fn observe_connection(
    remote_ip: u32,
    remote_port: u16,
    app_id: &str,
    domain_hint: Option<String>,
    decision: &str,
) -> serde_json::Value {
    let data = serde_json::json!({
        "remote_ip": ipv4_string(remote_ip),
        "remote_port": remote_port,
        "app": app_id,
        "domain_hint": domain_hint,
        "enforcement_plane": "wfp_windows",
        "decision": decision,
        "ts": chrono::Utc::now().to_rfc3339(),
    });

    serde_json::json!({
        "event_type": "network_observation",
        "data": data,
    })
}
