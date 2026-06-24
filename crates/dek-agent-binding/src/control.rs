use crate::capability::{CapabilityDescriptor, Surface};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlBindingSpec {
    pub surface_selector: String,
    pub strategy: ControlStrategy,
    pub reversible: bool,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStrategy {
    StdioWrapperInjection { wrapper_path: String },
    HttpProxyRedirect { local_proxy: String },
    NetworkEgressInterception,
    ObserveOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlLevel {
    Observe,
    Enforce,
}

pub fn plan_control_binding(
    cap: &CapabilityDescriptor,
    device_peps: &[String],
    requested_level: ControlLevel,
) -> Vec<ControlBindingSpec> {
    cap.interaction_surfaces
        .iter()
        .map(|s| match s {
            Surface::McpStdio { command, args } => {
                if requested_level == ControlLevel::Enforce
                    && device_peps.contains(&"mcp-stdio".to_string())
                {
                    ControlBindingSpec {
                        surface_selector: "mcp_stdio".into(),
                        strategy: ControlStrategy::StdioWrapperInjection {
                            wrapper_path: format!(
                                "dek-mcp-stdio-wrapper --target {} {}",
                                command,
                                args.join(" ")
                            ),
                        },
                        reversible: true,
                        requires_approval: false,
                    }
                } else {
                    ControlBindingSpec {
                        surface_selector: "mcp_stdio".into(),
                        strategy: ControlStrategy::ObserveOnly,
                        reversible: true,
                        requires_approval: false,
                    }
                }
            }
            Surface::McpHttp { url } | Surface::McpSse { url } => {
                if requested_level == ControlLevel::Enforce
                    && device_peps.contains(&"mcp-http".to_string())
                {
                    ControlBindingSpec {
                        surface_selector: "mcp_http".into(),
                        strategy: ControlStrategy::HttpProxyRedirect {
                            local_proxy: format!("http://127.0.0.1:8787/proxy?upstream={url}"),
                        },
                        reversible: true,
                        requires_approval: false,
                    }
                } else {
                    ControlBindingSpec {
                        surface_selector: "mcp_http".into(),
                        strategy: ControlStrategy::ObserveOnly,
                        reversible: true,
                        requires_approval: false,
                    }
                }
            }
            Surface::OpenAiCompatApi { .. } => {
                if requested_level == ControlLevel::Enforce
                    && (device_peps.contains(&"linux-ebpf".to_string())
                        || device_peps.contains(&"windows-wfp".to_string())
                        || device_peps.contains(&"macos-nefilter".to_string()))
                {
                    ControlBindingSpec {
                        surface_selector: "openai_api".into(),
                        strategy: ControlStrategy::NetworkEgressInterception,
                        reversible: true,
                        requires_approval: true,
                    }
                } else {
                    ControlBindingSpec {
                        surface_selector: "openai_api".into(),
                        strategy: ControlStrategy::ObserveOnly,
                        reversible: true,
                        requires_approval: false,
                    }
                }
            }
            _ => ControlBindingSpec {
                surface_selector: "native".into(),
                strategy: ControlStrategy::ObserveOnly,
                reversible: true,
                requires_approval: false,
            },
        })
        .collect()
}
