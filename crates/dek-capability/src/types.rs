//! Core domain types for the capability broker.
//!
//! The central idea (see the companion doc): enabling a sensor requires passing
//! **two independent gates** —
//!   1. **Consent gate** (app-level): the user agrees. Fully under our control.
//!   2. **OS gate** (kernel/entitlement-level): the OS *permits* the sensor to
//!      load (driver signing, ES entitlement, root/CAP_* + kernel support).
//!      NOT under our control — a consent popup cannot override it.
//!
//! The achievable outcome is therefore a *level*, not a boolean:
//! `Enforce` (both gates pass), `ObserveOnly` (consent passes, OS enforce gate
//! does not), or `None`.

use serde::{Deserialize, Serialize};

/// A sensor family Pollek can install (maps to doc #2 observation layers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensor {
    /// L0 — MCP/HTTP/SDK proxy (no OS gate; highest fidelity).
    McpProxy,
    /// L1 — process observer (ETW / Endpoint Security / eBPF).
    Process,
    /// L2 — file observer (minifilter / fanotify / Endpoint Security).
    File,
    /// L3 — network observer (WFP / eBPF / Network Extension).
    Network,
    /// L4 — browser/email/cloud observer (extension / native messaging).
    Browser,
    /// L5 — content guard / AMSI-like.
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Windows,
    Macos,
    Linux,
}

/// What was actually achieved after running the flow. The product must report
/// this honestly (enforceability honesty — never claim Enforce when ObserveOnly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievedLevel {
    /// Sensor not active at all.
    None,
    /// Can see/record activity, but cannot block.
    ObserveOnly,
    /// Can both observe and enforce (block/ask/redact).
    Enforce,
}

/// The two gates, evaluated independently by the probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    /// Requirement satisfied on this device right now.
    Pass,
    /// Not satisfied, but the user can fix it (we provide remediation steps).
    Remediable,
    /// Not satisfiable on this device/OS build at all.
    Unsupported,
}

/// A concrete OS-level requirement discovered by the preflight probe.
/// `blocking` requirements gate Enforce; non-blocking ones only degrade quality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    /// Stable machine code, e.g. "windows.driver_signed".
    pub code: String,
    /// Plain-language description for the UI.
    pub description: String,
    pub status: GateStatus,
    /// True if this requirement must pass for Enforce.
    pub blocking: bool,
    /// User-facing remediation (only meaningful when status == Remediable).
    pub remediation: Option<Remediation>,
}

/// A step the user (or an admin) must take outside our popup to satisfy a
/// requirement — e.g. approve a System Extension, install a signed driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remediation {
    pub action: String,
    /// Optional deep link / settings URI to take the user straight there.
    pub deep_link: Option<String>,
    /// True if it needs admin/root, so the UI can warn up front.
    pub requires_admin: bool,
}

/// The lifecycle state of a single (sensor, os) capability.
///
/// ```text
/// Unknown ─probe→ Probed ─requestConsent→ ConsentRequested ─grant→ Consented
///    Consented ─install→ Installing ─verify→ { Active(Enforce) | Active(ObserveOnly) }
///    any ─fail→ Blocked{reason, remediation}    (OS gate refused / install failed)
///    any ─rollback→ RolledBack ─reset→ Unknown
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CapabilityState {
    /// Nothing known yet — must probe first.
    Unknown,
    /// Probe done; we know which gates pass and the best achievable level.
    Probed { achievable: AchievedLevel },
    /// We asked the user for consent; awaiting their decision.
    ConsentRequested { achievable: AchievedLevel },
    /// User consented (app gate passed); ready to install.
    Consented { achievable: AchievedLevel },
    /// Install/apply in progress.
    Installing,
    /// Live. `level` is the *honest* achieved level.
    Active { level: AchievedLevel },
    /// Could not reach the intended level. `reason` + remediation explain why.
    Blocked {
        reason: String,
        achieved: AchievedLevel,
    },
    /// Cleanly reverted; safe to retry from Unknown.
    RolledBack,
}

impl CapabilityState {
    /// The level currently in effect (Active or Blocked carry a real level).
    pub fn effective_level(&self) -> AchievedLevel {
        match self {
            CapabilityState::Active { level } => *level,
            CapabilityState::Blocked { achieved, .. } => *achieved,
            _ => AchievedLevel::None,
        }
    }
}
