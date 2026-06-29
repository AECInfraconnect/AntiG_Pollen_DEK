//! Demo: drive the File sensor on a Windows host whose driver is not signed,
//! and emit the honest `observe-capabilities` report the dashboard would show.

use dek_capability::{CapabilityBroker, DefaultAdapter, HostFacts, Os, Sensor};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: false,
        ..Default::default()
    };
    let mut broker = CapabilityBroker::new();
    broker.register(Box::new(DefaultAdapter::new(Sensor::File, facts)));

    broker.probe(Sensor::File)?;
    broker.request_consent(Sensor::File)?;
    broker.grant_consent(Sensor::File, "project folder only")?;
    broker.install(Sensor::File)?;

    let report = broker.report(Sensor::File)?;
    let json = serde_json::to_string_pretty(&report)?;
    std::io::stdout().write_all(json.as_bytes())?;
    std::io::stdout().write_all(b"\n")?;
    Ok(())
}
