// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalScanSession {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSourceStatus {
    pub source: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalScanSummary {
    pub total_found: u32,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScanSessionV2 {
    pub schema_version: String,
    pub scan_id: String,
    pub tenant_id: String,
    pub device_id: String,
    pub status: ScanStatus,
    pub requested_sources: Vec<DiscoverySourceKind>,
    pub source_results: Vec<DiscoverySourceResult>,
    pub candidates_found: u32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub friendly_summary_en: String,
    pub friendly_summary_th: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Queued,
    Running,
    Completed,
    Partial,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySourceKind {
    ProcessScan,
    McpConfigScan,
    BrowserExtensionScan,
    LocalModelScan,
    ContainerScan,
    NetworkEgress,
    IdeExtensionScan,
    CliAgentScan,
    InstalledAppScan,
    PythonFrameworkScan,
}

impl DiscoverySourceKind {
    pub fn as_api_source(&self) -> &'static str {
        match self {
            Self::ProcessScan => "process",
            Self::McpConfigScan => "mcp_config",
            Self::BrowserExtensionScan => "browser_extension",
            Self::LocalModelScan => "local_model",
            Self::ContainerScan => "container",
            Self::NetworkEgress => "web_ai",
            Self::IdeExtensionScan => "ide_extension",
            Self::CliAgentScan => "cli_agent",
            Self::InstalledAppScan => "installed_app",
            Self::PythonFrameworkScan => "python_framework",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScanSourceState {
    Queued,
    Running,
    Completed,
    Degraded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DiscoverySourceResult {
    pub source: DiscoverySourceKind,
    pub status: ScanSourceState,
    pub candidates_found: u32,
    pub evidence_found: u32,
    pub error_message: Option<String>,
    pub privacy_note_en: String,
    pub privacy_note_th: String,
}
