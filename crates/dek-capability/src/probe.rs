//! Preflight probe.
//!
//! `HostFacts` is what a real probe discovers via OS calls (driver signature
//! state, entitlement presence, kernel features, current privileges). It is the
//! single injection point: production fills it from the OS; tests fill it
//! directly. `derive_requirements` then encodes the *real* per-OS rules that
//! decide the achievable level — independent of how the facts were gathered.

use crate::types::{AchievedLevel, GateStatus, Os, Remediation, Requirement, Sensor};
use serde::{Deserialize, Serialize};

/// Facts about the host that gate sensor installation. Unknown fields default
/// to the conservative value (false / unsupported).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostFacts {
    pub os: Option<Os>,
    pub is_admin_or_root: bool,

    // Windows
    pub win_driver_signed: bool,
    pub win_test_signing: bool,

    // macOS
    pub mac_es_entitlement_present: bool,
    pub mac_system_extension_approved: bool,
    pub mac_full_disk_access: bool,
    pub mac_notarized: bool,

    // Linux
    pub linux_kernel_supports_ebpf: bool,
    pub linux_kernel_supports_fanotify: bool,
    pub linux_has_cap_sys_admin: bool,
}

fn require(
    code: &str,
    desc: &str,
    status: GateStatus,
    blocking: bool,
    rem: Option<Remediation>,
) -> Requirement {
    Requirement {
        code: code.into(),
        description: desc.into(),
        status,
        blocking,
        remediation: rem,
    }
}

fn rem(action: &str, deep_link: Option<&str>, admin: bool) -> Option<Remediation> {
    Some(Remediation {
        action: action.into(),
        deep_link: deep_link.map(Into::into),
        requires_admin: admin,
    })
}

/// Result of a preflight probe for one (sensor, os).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeReport {
    pub sensor: Sensor,
    pub os: Os,
    pub requirements: Vec<Requirement>,
    /// Can we observe at all (perhaps via a no-privilege fallback)?
    pub observe_supported: bool,
}

impl ProbeReport {
    /// Best level achievable *if the user consents*.
    ///
    /// Enforce requires every blocking requirement to PASS. If a blocking
    /// requirement is merely Remediable/Unsupported but observation is still
    /// supported, the honest answer is ObserveOnly.
    pub fn achievable_level(&self) -> AchievedLevel {
        let all_blocking_pass = self
            .requirements
            .iter()
            .filter(|r| r.blocking)
            .all(|r| r.status == GateStatus::Pass);
        if all_blocking_pass && self.observe_supported {
            AchievedLevel::Enforce
        } else if self.observe_supported {
            AchievedLevel::ObserveOnly
        } else {
            AchievedLevel::None
        }
    }

    /// Blocking requirements not yet satisfied (the "missing" list for the API).
    pub fn missing(&self) -> Vec<&Requirement> {
        self.requirements
            .iter()
            .filter(|r| r.blocking && r.status != GateStatus::Pass)
            .collect()
    }
}

/// Encode the real per-OS gate rules. This is the reviewable "what does each OS
/// actually require?" table. Fact-gathering lives elsewhere; this is pure logic.
pub fn derive_requirements(sensor: Sensor, facts: &HostFacts) -> ProbeReport {
    let os = facts.os.unwrap_or(Os::Linux);

    // L0 MCP proxy and L4 browser/L5 content have no kernel gate: they run in
    // user space (proxy / extension / native messaging). Consent is the only gate.
    if matches!(sensor, Sensor::McpProxy) {
        return ProbeReport {
            sensor,
            os,
            requirements: vec![],
            observe_supported: true,
        };
    }

    let reqs: Vec<Requirement> = match (sensor, os) {
        // ---- Windows ----
        (Sensor::File, Os::Windows)
        | (Sensor::Network, Os::Windows)
        | (Sensor::Process, Os::Windows) => {
            let signed_status = if facts.win_driver_signed || facts.win_test_signing {
                GateStatus::Pass
            } else {
                // A signed kernel driver is required to ENFORCE; it cannot be
                // produced by a user clicking a popup (needs an EV cert +
                // Microsoft attestation). Remediable only by shipping a signed
                // build / admin install.
                GateStatus::Remediable
            };
            vec![
                require(
                    "windows.admin",
                    "Administrator rights to install the driver/service",
                    if facts.is_admin_or_root {
                        GateStatus::Pass
                    } else {
                        GateStatus::Remediable
                    },
                    true,
                    rem("Re-run the installer as Administrator", None, true),
                ),
                require(
                    "windows.driver_signed",
                    "A Microsoft-attested signed kernel driver (minifilter/WFP/ETW) to enforce",
                    signed_status,
                    true,
                    rem(
                        "Install the signed Pollek driver build (EV-signed, WHQL-attested)",
                        None,
                        true,
                    ),
                ),
            ]
        }

        // ---- macOS ----
        (Sensor::File, Os::Macos) | (Sensor::Process, Os::Macos) => {
            // Endpoint Security needs an Apple-granted entitlement. If it is
            // absent we cannot use ES at all -> enforce Unsupported, but file
            // observation can still fall back to FSEvents (observe-only).
            let ent = if facts.mac_es_entitlement_present {
                GateStatus::Pass
            } else {
                GateStatus::Unsupported
            };
            vec![
                require(
                    "macos.es_entitlement",
                    "Apple-granted Endpoint Security client entitlement (to enforce)",
                    ent,
                    true,
                    None, // not user-remediable on device; requires Apple approval + signed build
                ),
                require(
                    "macos.system_extension_approved",
                    "User approval of the System Extension in System Settings",
                    if facts.mac_system_extension_approved { GateStatus::Pass } else { GateStatus::Remediable },
                    true,
                    rem("Approve the Pollek System Extension", Some("x-apple.systempreferences:com.apple.preference.security"), false),
                ),
                require(
                    "macos.full_disk_access",
                    "Full Disk Access for file visibility",
                    if facts.mac_full_disk_access { GateStatus::Pass } else { GateStatus::Remediable },
                    false,
                    rem("Grant Full Disk Access to Pollek", Some("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles"), false),
                ),
            ]
        }
        (Sensor::Network, Os::Macos) => vec![
            require(
                "macos.network_extension_entitlement",
                "Network Extension content-filter entitlement (to block)",
                if facts.mac_es_entitlement_present {
                    GateStatus::Pass
                } else {
                    GateStatus::Unsupported
                },
                true,
                None,
            ),
            require(
                "macos.system_extension_approved",
                "User approval of the System/Network Extension",
                if facts.mac_system_extension_approved {
                    GateStatus::Pass
                } else {
                    GateStatus::Remediable
                },
                true,
                rem(
                    "Approve the Pollek Network Extension",
                    Some("x-apple.systempreferences:com.apple.preference.security"),
                    false,
                ),
            ),
        ],

        // ---- Linux ----
        (Sensor::File, Os::Linux) => {
            let kernel = if facts.linux_kernel_supports_fanotify {
                GateStatus::Pass
            } else {
                GateStatus::Unsupported
            };
            vec![
                require(
                    "linux.kernel_fanotify",
                    "Kernel support for fanotify permission events (to block)",
                    kernel,
                    true,
                    None,
                ),
                require(
                    "linux.cap_sys_admin",
                    "CAP_SYS_ADMIN (usually root) for fanotify",
                    if facts.linux_has_cap_sys_admin {
                        GateStatus::Pass
                    } else {
                        GateStatus::Remediable
                    },
                    true,
                    rem(
                        "Run the Pollek service with CAP_SYS_ADMIN (or as root)",
                        None,
                        true,
                    ),
                ),
            ]
        }
        (Sensor::Network, Os::Linux) | (Sensor::Process, Os::Linux) => {
            let kernel = if facts.linux_kernel_supports_ebpf {
                GateStatus::Pass
            } else {
                GateStatus::Unsupported
            };
            vec![
                require(
                    "linux.kernel_ebpf",
                    "Kernel eBPF support (to observe/enforce)",
                    kernel,
                    true,
                    None,
                ),
                require(
                    "linux.cap_bpf",
                    "CAP_BPF / CAP_SYS_ADMIN to load eBPF programs",
                    if facts.linux_has_cap_sys_admin {
                        GateStatus::Pass
                    } else {
                        GateStatus::Remediable
                    },
                    true,
                    rem(
                        "Grant CAP_BPF/CAP_SYS_ADMIN to the Pollek service",
                        None,
                        true,
                    ),
                ),
            ]
        }

        // ---- Browser / Content (user space on every OS) ----
        (Sensor::Browser, _) => vec![require(
            "browser.extension_installed",
            "Pollek browser extension / native messaging host installed",
            GateStatus::Remediable,
            true,
            rem("Install the Pollek browser extension", None, false),
        )],
        (Sensor::Content, _) => vec![],

        // Fallback: unknown combo -> observe-only, nothing to enforce.
        _ => vec![],
    };

    // Observation fallback: nearly everything can observe at *some* fidelity
    // without the enforce gate (FSEvents, connect-tracing, procfs polling,
    // proxy/extension). Content/MCP always observe.
    let observe_supported = !matches!((sensor, os), (Sensor::Network, Os::Macos))
        || facts.mac_system_extension_approved;

    ProbeReport {
        sensor,
        os,
        requirements: reqs,
        observe_supported,
    }
}
