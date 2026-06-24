use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceCapabilities {
    pub device_id: String,
    pub dek_version: String,
    pub os: OsInfo,
    pub pdp: Vec<PdpCapability>,
    pub pep: Vec<PepCapability>,
    pub plugins: Vec<PluginCapability>,
    pub kernel: KernelCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OsInfo {
    pub r#type: String,
    pub version: String,
    pub arch: String,
}

pub mod detect;
use dek_domain_schema::control_level::ControlLevel;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PdpCapability {
    pub r#type: String,
    pub version: Option<String>,
    pub mode: Option<String>,
    pub control_level: ControlLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Active,
    Available,
    MissingDependencies,
    PermissionDenied,
    NotSupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PepCapability {
    pub r#type: String,
    #[serde(default)]
    pub transports: Vec<String>,
    pub control_level: ControlLevel,
    pub status: CapabilityStatus,
    pub status_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCapability {
    pub id: String,
    pub abi: String,
    pub control_level: ControlLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelCapabilities {
    pub linux_ebpf: Option<serde_json::Value>,
    pub windows_wfp: Option<serde_json::Value>,
    pub macos_nefilter: Option<serde_json::Value>,
}

impl DeviceCapabilities {
    pub fn has_os_l4_ready(&self) -> bool {
        self.kernel.linux_ebpf.is_some()
            || self.kernel.windows_wfp.is_some()
            || self.kernel.macos_nefilter.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct CompatibilityRule {
    pub preset_id: String,
    pub pep_types: Vec<String>,
    pub pdp_kinds: Vec<dek_domain_schema::PdpKind>,
    pub required_resources: Vec<String>,
    pub enforceable: bool,
}

pub fn is_compatible(
    rule: &CompatibilityRule,
    pep: &dek_domain_schema::PepBinding,
    pdp: &dek_domain_schema::PdpRuntime,
) -> bool {
    rule.pep_types.contains(&pep.pep_type)
        && rule.pdp_kinds.contains(&pdp.kind)
        && pep.can_observe
        && (!rule.enforceable || pep.can_enforce)
}

pub struct CapabilityRegistry {
    pub device_id: String,
    pub dek_version: String,
}

impl CapabilityRegistry {
    pub fn new(device_id: String, dek_version: String) -> Self {
        Self {
            device_id,
            dek_version,
        }
    }

    pub fn gather(&self) -> DeviceCapabilities {
        let os_version = sysinfo::System::os_version().unwrap_or_else(|| "unknown".to_string());

        let os = OsInfo {
            r#type: std::env::consts::OS.to_string(),
            version: os_version.clone(), // Query via sysinfo
            arch: std::env::consts::ARCH.to_string(),
        };

        let linux_ebpf = if std::env::consts::OS == "linux" {
            Some(serde_json::json!({ "bpf_jit_enable": 1 }))
        } else {
            None
        };

        let windows_wfp = if std::env::consts::OS == "windows" {
            Some(serde_json::json!({ "ale_auth_connect": true }))
        } else {
            None
        };

        let macos_nefilter = if std::env::consts::OS == "macos" {
            Some(serde_json::json!({ "network_extension": true }))
        } else {
            None
        };

        DeviceCapabilities {
            device_id: self.device_id.clone(),
            dek_version: self.dek_version.clone(),
            os,
            pdp: vec![
                PdpCapability {
                    r#type: "wasm".to_string(),
                    version: Some("wasmtime-24".to_string()),
                    mode: Some("sandbox".to_string()),
                    control_level: ControlLevel::Enforce,
                },
                PdpCapability {
                    r#type: "native".to_string(),
                    version: None,
                    mode: None,
                    control_level: ControlLevel::Enforce,
                },
            ],
            pep: detect::detect_pep_capabilities(),
            plugins: vec![],
            kernel: KernelCapabilities {
                linux_ebpf,
                windows_wfp,
                macos_nefilter,
            },
        }
    }
}
