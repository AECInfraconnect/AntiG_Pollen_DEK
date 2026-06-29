//! API DTO for `GET /v1/.../observe-capabilities`.
//!
//! Extends doc #2 §12.1 with the fields that make enforceability *honest*:
//! `achieved_level`, `achievable_level`, `missing`, and `remediation` — so the
//! UI can show "✓ / △ / ✕" and a concrete "what to do next" per sensor.

use crate::probe::ProbeReport;
use crate::types::{AchievedLevel, CapabilityState, Os, Remediation, Requirement, Sensor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub sensor: Sensor,
    pub os: Os,
    /// Current state machine tag (e.g. "active", "blocked").
    pub state: String,
    /// What is live RIGHT NOW (honest; never overstated).
    pub achieved_level: AchievedLevel,
    /// The best level reachable if the user completes remediation + consent.
    pub achievable_level: AchievedLevel,
    pub observe_supported: bool,
    pub consented: bool,
    pub requirements: Vec<Requirement>,
    /// Blocking requirements not yet satisfied.
    pub missing: Vec<Requirement>,
    /// Concrete next steps the user/admin must take (OS-level).
    pub remediation: Vec<Remediation>,
}

fn state_tag(state: &CapabilityState) -> String {
    match state {
        CapabilityState::Unknown => "unknown",
        CapabilityState::Probed { .. } => "probed",
        CapabilityState::ConsentRequested { .. } => "consent_requested",
        CapabilityState::Consented { .. } => "consented",
        CapabilityState::Installing => "installing",
        CapabilityState::Active { .. } => "active",
        CapabilityState::Blocked { .. } => "blocked",
        CapabilityState::RolledBack => "rolled_back",
    }
    .to_string()
}

pub fn build_report(
    probe: &ProbeReport,
    state: &CapabilityState,
    consented: bool,
) -> CapabilityReport {
    let missing: Vec<Requirement> = probe.missing().into_iter().cloned().collect();
    let remediation: Vec<Remediation> = missing
        .iter()
        .filter_map(|r| r.remediation.clone())
        .collect();

    CapabilityReport {
        sensor: probe.sensor,
        os: probe.os,
        state: state_tag(state),
        achieved_level: state.effective_level(),
        achievable_level: probe.achievable_level(),
        observe_supported: probe.observe_supported,
        consented,
        requirements: probe.requirements.clone(),
        missing,
        remediation,
    }
}
