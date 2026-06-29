//! End-to-end broker tests: the two-gate model, honest fallback levels,
//! illegal transitions, rollback, and consent-ledger behavior.

use dek_capability::{
    AchievedLevel, CapabilityBroker, CapabilityState, DefaultAdapter, HostFacts, Os, Sensor,
};

fn broker_with(sensor: Sensor, facts: HostFacts) -> CapabilityBroker {
    let mut broker = CapabilityBroker::new();
    broker.register(Box::new(DefaultAdapter::new(sensor, facts)));
    broker
}

fn run_to_install(
    broker: &mut CapabilityBroker,
    sensor: Sensor,
) -> anyhow::Result<CapabilityState> {
    broker.probe(sensor)?;
    broker.request_consent(sensor)?;
    broker.grant_consent(sensor, "test scope")?;
    Ok(broker.install(sensor)?.clone())
}

#[test]
fn mcp_proxy_enforces_with_only_consent() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::McpProxy, facts);
    let state = run_to_install(&mut broker, Sensor::McpProxy)?;
    assert_eq!(
        state,
        CapabilityState::Active {
            level: AchievedLevel::Enforce
        }
    );
    Ok(())
}

#[test]
fn windows_file_happy_path_enforces() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: true,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    let state = run_to_install(&mut broker, Sensor::File)?;
    assert_eq!(
        state,
        CapabilityState::Active {
            level: AchievedLevel::Enforce
        }
    );
    assert!(broker.ledger().is_granted(Sensor::File));
    Ok(())
}

#[test]
fn windows_unsigned_driver_falls_back_to_observe_only() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: false,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    let state = run_to_install(&mut broker, Sensor::File)?;
    assert_eq!(
        state,
        CapabilityState::Active {
            level: AchievedLevel::ObserveOnly
        }
    );

    let report = broker.report(Sensor::File)?;
    assert_eq!(report.achieved_level, AchievedLevel::ObserveOnly);
    assert!(report
        .missing
        .iter()
        .any(|requirement| requirement.code == "windows.driver_signed"));
    assert!(!report.remediation.is_empty());
    Ok(())
}

#[test]
fn macos_without_entitlement_is_observe_only() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Macos),
        mac_es_entitlement_present: false,
        mac_system_extension_approved: true,
        mac_full_disk_access: true,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    let state = run_to_install(&mut broker, Sensor::File)?;
    assert_eq!(
        state,
        CapabilityState::Active {
            level: AchievedLevel::ObserveOnly
        }
    );
    let report = broker.report(Sensor::File)?;
    assert!(report
        .missing
        .iter()
        .any(|requirement| requirement.code == "macos.es_entitlement"));
    Ok(())
}

#[test]
fn linux_capability_gate_controls_enforce() -> anyhow::Result<()> {
    let no_cap = HostFacts {
        os: Some(Os::Linux),
        linux_kernel_supports_fanotify: true,
        linux_has_cap_sys_admin: false,
        ..Default::default()
    };
    let mut broker_without_cap = broker_with(Sensor::File, no_cap);
    assert_eq!(
        run_to_install(&mut broker_without_cap, Sensor::File)?,
        CapabilityState::Active {
            level: AchievedLevel::ObserveOnly
        }
    );

    let with_cap = HostFacts {
        os: Some(Os::Linux),
        linux_kernel_supports_fanotify: true,
        linux_has_cap_sys_admin: true,
        ..Default::default()
    };
    let mut broker_with_cap = broker_with(Sensor::File, with_cap);
    assert_eq!(
        run_to_install(&mut broker_with_cap, Sensor::File)?,
        CapabilityState::Active {
            level: AchievedLevel::Enforce
        }
    );
    Ok(())
}

#[test]
fn install_before_consent_is_rejected() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: true,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    broker.probe(Sensor::File)?;
    let err = broker.install(Sensor::File).err();
    assert!(matches!(
        err,
        Some(dek_capability::BrokerError::IllegalTransition { .. })
    ));
    Ok(())
}

#[test]
fn rollback_clears_state_and_revokes_consent() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: true,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    run_to_install(&mut broker, Sensor::File)?;
    assert!(broker.ledger().is_granted(Sensor::File));

    let rolled = broker.rollback(Sensor::File)?.clone();
    assert_eq!(rolled, CapabilityState::RolledBack);
    assert_eq!(rolled.effective_level(), AchievedLevel::None);
    assert!(!broker.ledger().is_granted(Sensor::File));
    Ok(())
}

#[test]
fn deny_consent_returns_to_probed_and_is_recorded() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: true,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    broker.probe(Sensor::File)?;
    broker.request_consent(Sensor::File)?;
    let state = broker.deny_consent(Sensor::File)?.clone();
    assert!(matches!(state, CapabilityState::Probed { .. }));
    assert!(!broker.ledger().is_granted(Sensor::File));
    assert_eq!(broker.ledger().history().len(), 1);
    assert!(!broker.ledger().history()[0].granted);
    Ok(())
}

#[test]
fn install_failure_yields_blocked_not_silent_success() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: true,
        ..Default::default()
    };
    let mut adapter = DefaultAdapter::new(Sensor::File, facts);
    adapter.force_install_error = true;
    let mut broker = CapabilityBroker::new();
    broker.register(Box::new(adapter));
    broker.probe(Sensor::File)?;
    broker.request_consent(Sensor::File)?;
    broker.grant_consent(Sensor::File, "scope")?;
    let state = broker.install(Sensor::File)?.clone();
    if let CapabilityState::Blocked { achieved, .. } = state {
        assert_eq!(achieved, AchievedLevel::None);
        return Ok(());
    }
    Err(anyhow::anyhow!("expected blocked state"))
}

#[test]
fn report_serializes_with_honest_levels() -> anyhow::Result<()> {
    let facts = HostFacts {
        os: Some(Os::Windows),
        is_admin_or_root: true,
        win_driver_signed: true,
        ..Default::default()
    };
    let mut broker = broker_with(Sensor::File, facts);
    run_to_install(&mut broker, Sensor::File)?;
    let report = broker.report(Sensor::File)?;
    assert_eq!(report.achieved_level, AchievedLevel::Enforce);
    assert_eq!(report.achievable_level, AchievedLevel::Enforce);
    assert!(report.missing.is_empty());
    let json = serde_json::to_string(&report)?;
    assert!(json.contains("\"achieved_level\":\"enforce\""));
    Ok(())
}
