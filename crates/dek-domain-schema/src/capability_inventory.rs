// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlMode {
    Observe,
    Enforce,
    StrictDeny,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentCapabilityInventory {
    pub schema_version: String,
    pub tenant_id: String,
    pub device_id: String,
    pub agent_id: String,
    pub candidate_id: Option<String>,
    pub display_name: String,
    pub agent_type: AgentKind,
    pub trust_level: String,
    pub confidence: f64,
    pub risk_score: u32,
    pub process: Option<ProcessSurface>,
    pub config_surfaces: Vec<ConfigSurface>,
    pub mcp_surfaces: Vec<McpSurface>,
    pub model_endpoints: Vec<ModelEndpointSurface>,
    pub browser_surfaces: Vec<BrowserSurface>,
    pub file_surfaces: Vec<FileSurface>,
    pub network_surfaces: Vec<NetworkSurface>,
    pub supported_pep_bindings: Vec<PepBindingOption>,
    pub supported_pdp_routes: Vec<PdpRouteOption>,
    pub telemetry_capabilities: TelemetryCapabilities,
    pub last_scan_id: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    DesktopAgent,
    IdeAgent,
    CliAgent,
    BrowserAgent,
    WebAiApp,
    McpClient,
    McpServer,
    LocalModelServer,
    IdeExtension,
    CustomScriptAgent,
    AutomationAgent,
    UnknownAiProcess,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessSurface {
    pub pid: u32,
    pub exe_path: String,
    pub command_line: String,
    pub owner_user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigSurface {
    pub config_id: String,
    pub path_hash: String,
    pub path_redacted: String,
    pub owner_client: String,
    pub format: String,
    pub editable: bool,
    pub backup_supported: bool,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpSurface {
    pub server_name: String,
    pub client_hint: String,
    pub transport: McpTransportKind,
    pub command_template: Option<Vec<String>>,
    pub endpoint_domain: Option<String>,
    pub has_auth_header: bool,
    pub env_key_names: Vec<String>,
    pub tools_known: Vec<String>,
    pub resources_known: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McpTransportKind {
    Stdio,
    StreamableHttp,
    SseLegacy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelEndpointSurface {
    pub endpoint_url: String,
    pub protocol: String,
    pub models_known: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BrowserSurface {
    pub browser_type: String,
    pub extension_id: Option<String>,
    pub host_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileSurface {
    pub path_hash: String,
    pub access_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NetworkSurface {
    pub local_port: Option<u16>,
    pub remote_domain: Option<String>,
    pub remote_ip: Option<String>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PepBindingOption {
    pub pep_type: PepType,
    pub action: PepBindingAction,
    pub coverage: PepCoverage,
    pub mode_supported: Vec<ControlMode>,
    pub requires_admin: bool,
    pub requires_user_approval: bool,
    pub reversible: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PepType {
    McpProxy,
    StdioWrapper,
    HttpGateway,
    BrowserExtension,
    LocalModelProxy,
    LinuxEbpf,
    WindowsWfp,
    MacosNetworkExtension,
    FileSystemPep,
    EmbeddedSdk,
    TelemetryOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PepBindingAction {
    RewriteConfig,
    WrapCommand,
    ProxyEndpoint,
    InstallLocalService,
    InstallOsModule,
    EnableBrowserExtension,
    ObserveOnly,
    ManualInstruction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PepCoverage {
    ToolCalls,
    ToolCallsAndResources,
    HttpEgress,
    NetworkEgress,
    FileAccess,
    BrowserSaas,
    LocalModelApi,
    ProcessMetadataOnly,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PdpRouteOption {
    pub route_id: String,
    pub primary_pdp: String,
    pub fallback_pdp: Option<String>,
    pub shadow_pdp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelemetryCapabilities {
    pub emits_tool_logs: bool,
    pub emits_resource_logs: bool,
    pub emits_decision_logs: bool,
    pub emits_network_logs: bool,
    pub format: String,
}
