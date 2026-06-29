//! Consent ledger: an append-only, user-visible, revocable record of exactly
//! which deep-observation capabilities the user enabled, when, and at what
//! requested level. Supports the transparency/record-keeping story and lets the
//! UI show or revoke grants.

use crate::types::{AchievedLevel, Sensor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub sensor: Sensor,
    /// The level the user was asked to consent to.
    pub requested_level: AchievedLevel,
    /// Free-form scope note shown to the user, such as "project folder only".
    pub scope: String,
    /// Monotonic logical timestamp supplied by the caller to keep this crate
    /// dependency-light.
    pub at_seq: u64,
    pub granted: bool,
    /// Set when a previously granted consent is revoked.
    pub revoked_at_seq: Option<u64>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConsentLedger {
    records: Vec<ConsentRecord>,
}

impl ConsentLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_decision(
        &mut self,
        sensor: Sensor,
        requested_level: AchievedLevel,
        scope: &str,
        granted: bool,
        at_seq: u64,
    ) {
        self.records.push(ConsentRecord {
            sensor,
            requested_level,
            scope: scope.into(),
            at_seq,
            granted,
            revoked_at_seq: None,
        });
    }

    /// Returns whether the latest consent decision for this sensor is active.
    pub fn is_granted(&self, sensor: Sensor) -> bool {
        self.records
            .iter()
            .rfind(|r| r.sensor == sensor)
            .map(|r| r.granted && r.revoked_at_seq.is_none())
            .unwrap_or(false)
    }

    /// Revoke the latest active grant for a sensor. Returns true when changed.
    pub fn revoke(&mut self, sensor: Sensor, at_seq: u64) -> bool {
        if let Some(rec) = self
            .records
            .iter_mut()
            .rfind(|r| r.sensor == sensor && r.granted && r.revoked_at_seq.is_none())
        {
            rec.revoked_at_seq = Some(at_seq);
            true
        } else {
            false
        }
    }

    pub fn history(&self) -> &[ConsentRecord] {
        &self.records
    }
}
