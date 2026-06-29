//! The capability broker: a single place that drives every sensor through the
//! same lifecycle and enforces legal transitions. Sensors register an adapter;
//! the broker handles probe -> consent -> install -> verify -> rollback and
//! records consent in the ledger.

use crate::adapter::SensorAdapter;
use crate::api::{build_report, CapabilityReport};
use crate::ledger::ConsentLedger;
use crate::probe::ProbeReport;
use crate::types::{AchievedLevel, CapabilityState, Sensor};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum BrokerError {
    NotRegistered(Sensor),
    IllegalTransition {
        sensor: Sensor,
        from: String,
        action: &'static str,
    },
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokerError::NotRegistered(s) => write!(f, "no adapter registered for {s:?}"),
            BrokerError::IllegalTransition {
                sensor,
                from,
                action,
            } => {
                write!(
                    f,
                    "illegal transition for {sensor:?}: cannot '{action}' from state '{from}'"
                )
            }
        }
    }
}
impl std::error::Error for BrokerError {}

pub struct CapabilityBroker {
    adapters: HashMap<Sensor, Box<dyn SensorAdapter>>,
    states: HashMap<Sensor, CapabilityState>,
    probes: HashMap<Sensor, ProbeReport>,
    rollback_tokens: HashMap<Sensor, String>,
    ledger: ConsentLedger,
    seq: u64,
}

impl Default for CapabilityBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityBroker {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            states: HashMap::new(),
            probes: HashMap::new(),
            rollback_tokens: HashMap::new(),
            ledger: ConsentLedger::new(),
            seq: 0,
        }
    }

    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    pub fn register(&mut self, adapter: Box<dyn SensorAdapter>) {
        let s = adapter.sensor();
        self.adapters.insert(s, adapter);
        self.states.insert(s, CapabilityState::Unknown);
    }

    pub fn state(&self, sensor: Sensor) -> Option<&CapabilityState> {
        self.states.get(&sensor)
    }

    pub fn ledger(&self) -> &ConsentLedger {
        &self.ledger
    }

    fn adapter(&self, sensor: Sensor) -> Result<&dyn SensorAdapter, BrokerError> {
        self.adapters
            .get(&sensor)
            .map(|b| b.as_ref())
            .ok_or(BrokerError::NotRegistered(sensor))
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

    fn current_state(&self, sensor: Sensor) -> Result<&CapabilityState, BrokerError> {
        self.states
            .get(&sensor)
            .ok_or(BrokerError::NotRegistered(sensor))
    }

    /// Step 1 — preflight probe. Legal from any state (idempotent re-probe).
    pub fn probe(&mut self, sensor: Sensor) -> Result<&CapabilityState, BrokerError> {
        let report = self.adapter(sensor)?.probe();
        let achievable = report.achievable_level();
        self.probes.insert(sensor, report);
        self.states
            .insert(sensor, CapabilityState::Probed { achievable });
        self.current_state(sensor)
    }

    /// Step 2 — ask the user. Legal only after a probe.
    pub fn request_consent(&mut self, sensor: Sensor) -> Result<&CapabilityState, BrokerError> {
        match self.states.get(&sensor) {
            Some(CapabilityState::Probed { achievable }) => {
                let a = *achievable;
                self.states
                    .insert(sensor, CapabilityState::ConsentRequested { achievable: a });
                self.current_state(sensor)
            }
            other => Err(BrokerError::IllegalTransition {
                sensor,
                from: other.map(Self::state_tag).unwrap_or_else(|| "none".into()),
                action: "request_consent",
            }),
        }
    }

    /// Step 3a — user grants. Records the ledger and moves to Consented.
    pub fn grant_consent(
        &mut self,
        sensor: Sensor,
        scope: &str,
    ) -> Result<&CapabilityState, BrokerError> {
        let achievable = match self.states.get(&sensor) {
            Some(CapabilityState::ConsentRequested { achievable }) => *achievable,
            other => {
                return Err(BrokerError::IllegalTransition {
                    sensor,
                    from: other.map(Self::state_tag).unwrap_or_else(|| "none".into()),
                    action: "grant_consent",
                })
            }
        };
        let seq = self.next_seq();
        self.ledger
            .record_decision(sensor, achievable, scope, true, seq);
        self.states
            .insert(sensor, CapabilityState::Consented { achievable });
        self.current_state(sensor)
    }

    /// Step 3b — user denies. Records the ledger and returns to Probed.
    pub fn deny_consent(&mut self, sensor: Sensor) -> Result<&CapabilityState, BrokerError> {
        let achievable = match self.states.get(&sensor) {
            Some(CapabilityState::ConsentRequested { achievable }) => *achievable,
            other => {
                return Err(BrokerError::IllegalTransition {
                    sensor,
                    from: other.map(Self::state_tag).unwrap_or_else(|| "none".into()),
                    action: "deny_consent",
                })
            }
        };
        let seq = self.next_seq();
        self.ledger
            .record_decision(sensor, achievable, "", false, seq);
        self.states
            .insert(sensor, CapabilityState::Probed { achievable });
        self.current_state(sensor)
    }

    /// Step 4 — install/apply. Legal only after consent. Produces the honest
    /// Active{level} (possibly an observe-only fallback) or Blocked.
    pub fn install(&mut self, sensor: Sensor) -> Result<&CapabilityState, BrokerError> {
        match self.states.get(&sensor) {
            Some(CapabilityState::Consented { .. }) => {}
            other => {
                return Err(BrokerError::IllegalTransition {
                    sensor,
                    from: other.map(Self::state_tag).unwrap_or_else(|| "none".into()),
                    action: "install",
                })
            }
        }
        self.states.insert(sensor, CapabilityState::Installing);

        let new_state = match self.adapter(sensor)?.install() {
            Ok(outcome) => {
                self.rollback_tokens
                    .insert(sensor, outcome.rollback_token.clone());
                match outcome.achieved {
                    AchievedLevel::None => CapabilityState::Blocked {
                        reason: "install completed but no observation could be activated".into(),
                        achieved: AchievedLevel::None,
                    },
                    level => CapabilityState::Active { level },
                }
            }
            Err(e) => CapabilityState::Blocked {
                reason: e.to_string(),
                achieved: AchievedLevel::None,
            },
        };
        self.states.insert(sensor, new_state);
        self.current_state(sensor)
    }

    /// Roll back to a clean state. Legal from Active/Blocked/Installing. Also
    /// revokes the consent in the ledger.
    pub fn rollback(&mut self, sensor: Sensor) -> Result<&CapabilityState, BrokerError> {
        match self.states.get(&sensor) {
            Some(CapabilityState::Active { .. })
            | Some(CapabilityState::Blocked { .. })
            | Some(CapabilityState::Installing) => {}
            other => {
                return Err(BrokerError::IllegalTransition {
                    sensor,
                    from: other.map(Self::state_tag).unwrap_or_else(|| "none".into()),
                    action: "rollback",
                })
            }
        }
        if let Some(token) = self.rollback_tokens.get(&sensor).cloned() {
            // Best-effort revert; a real adapter surfaces hard failures.
            let _ = self.adapter(sensor)?.rollback(&token);
        }
        let seq = self.next_seq();
        self.ledger.revoke(sensor, seq);
        self.states.insert(sensor, CapabilityState::RolledBack);
        self.current_state(sensor)
    }

    /// Build the API report for the `observe-capabilities` endpoint.
    pub fn report(&self, sensor: Sensor) -> Result<CapabilityReport, BrokerError> {
        let probe = self
            .probes
            .get(&sensor)
            .ok_or(BrokerError::NotRegistered(sensor))?;
        let state = self
            .states
            .get(&sensor)
            .cloned()
            .unwrap_or(CapabilityState::Unknown);
        Ok(build_report(probe, &state, self.ledger.is_granted(sensor)))
    }
}
