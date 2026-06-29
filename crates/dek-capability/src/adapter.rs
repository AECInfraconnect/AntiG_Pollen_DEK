//! Sensor adapters: the per-(sensor, os) install/verify/rollback implementation.
//!
//! `derive_requirements` (probe.rs) decides *what is achievable*. The adapter
//! *does* the install/apply and reports what was actually reached. Production
//! adapters perform real driver/extension/eBPF operations; the `DefaultAdapter`
//! here drives those transitions from `HostFacts` so the broker + state machine
//! are fully testable without touching the kernel.

use crate::probe::{derive_requirements, HostFacts, ProbeReport};
use crate::types::{AchievedLevel, Sensor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    /// Level actually reached after the install/apply step.
    pub achieved: AchievedLevel,
    /// Opaque token used to roll the change back.
    pub rollback_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    InstallFailed(String),
    RollbackFailed(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::InstallFailed(e) => write!(f, "install failed: {e}"),
            AdapterError::RollbackFailed(e) => write!(f, "rollback failed: {e}"),
        }
    }
}
impl std::error::Error for AdapterError {}

/// One sensor's per-OS capability operations.
pub trait SensorAdapter {
    fn sensor(&self) -> Sensor;
    /// Preflight: which gates pass, what's achievable.
    fn probe(&self) -> ProbeReport;
    /// Install/apply. Returns the honest achieved level (may be a fallback).
    fn install(&self) -> Result<InstallOutcome, AdapterError>;
    /// Verify what is actually live now.
    fn verify(&self) -> AchievedLevel;
    /// Cleanly revert a prior install.
    fn rollback(&self, token: &str) -> Result<(), AdapterError>;
}

/// Fact-driven adapter used for the broker's logic and tests. A real adapter
/// replaces `install`/`verify`/`rollback` with actual OS operations but keeps
/// the same contract (honest level reporting + working rollback).
pub struct DefaultAdapter {
    sensor: Sensor,
    facts: HostFacts,
    /// Simulate an install failure even when the probe says it's achievable
    /// (e.g. transient OS error) — lets tests exercise the Blocked path.
    pub force_install_error: bool,
}

impl DefaultAdapter {
    pub fn new(sensor: Sensor, facts: HostFacts) -> Self {
        Self {
            sensor,
            facts,
            force_install_error: false,
        }
    }
}

impl SensorAdapter for DefaultAdapter {
    fn sensor(&self) -> Sensor {
        self.sensor
    }

    fn probe(&self) -> ProbeReport {
        derive_requirements(self.sensor, &self.facts)
    }

    fn install(&self) -> Result<InstallOutcome, AdapterError> {
        if self.force_install_error {
            return Err(AdapterError::InstallFailed(
                "simulated OS install error".into(),
            ));
        }
        // A real install reaches at most what the probe deemed achievable.
        let achieved = self.probe().achievable_level();
        Ok(InstallOutcome {
            achieved,
            rollback_token: format!("rbk-{:?}", self.sensor),
        })
    }

    fn verify(&self) -> AchievedLevel {
        self.probe().achievable_level()
    }

    fn rollback(&self, _token: &str) -> Result<(), AdapterError> {
        Ok(())
    }
}
