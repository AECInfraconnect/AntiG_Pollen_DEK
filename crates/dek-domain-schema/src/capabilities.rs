use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EnforcementCapabilities {
    pub mcp_http_pep: bool,
    pub mcp_stdio_pep: bool,
    pub network_filter_user_mode: bool,
    pub network_filter_kernel: bool,
    pub dns_filter: bool,
    pub process_attribution: bool,
    pub ebpf_guardrail: bool,
    pub hot_reload_network_rules: bool,
    pub fail_closed_high_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceRegistrationRequest {
    pub device_id: String,
    pub os: String,
    pub dek_version: String,
    pub capabilities: EnforcementCapabilities,
}
