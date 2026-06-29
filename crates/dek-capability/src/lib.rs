//! # dek-capability
//!
//! Pollek's capability broker. It drives every sensor through one honest
//! lifecycle that respects the two independent gates required to observe/enforce
//! on an end-user machine:
//!
//! 1. **Consent gate** (app-level) — the user agrees. We control this.
//! 2. **OS gate** (kernel/entitlement) — the OS permits the sensor to load
//!    (driver signing on Windows, Endpoint Security entitlement on macOS,
//!    eBPF/fanotify capabilities + kernel support on Linux). We do NOT control
//!    this; a consent popup cannot override it.
//!
//! The achievable outcome is a *level* — `Enforce`, `ObserveOnly`, or `None` —
//! and the broker always reports the **honest achieved level** plus the
//! concrete remediation needed to improve it.
//!
//! Flow: `probe -> request_consent -> grant_consent -> install -> (Active | Blocked)`,
//! with `rollback` available and a `ConsentLedger` recording every decision.

pub mod adapter;
pub mod api;
pub mod broker;
pub mod ledger;
pub mod probe;
pub mod types;

pub use adapter::{AdapterError, DefaultAdapter, InstallOutcome, SensorAdapter};
pub use api::{build_report, CapabilityReport};
pub use broker::{BrokerError, CapabilityBroker};
pub use ledger::{ConsentLedger, ConsentRecord};
pub use probe::{derive_requirements, HostFacts, ProbeReport};
pub use types::{AchievedLevel, CapabilityState, GateStatus, Os, Remediation, Requirement, Sensor};
