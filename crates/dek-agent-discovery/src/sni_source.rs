use crate::web_ai_scan::{SniFlow, SniFlowSource};
use std::sync::Arc;
use std::time::Duration;

pub struct NetworkEventRecord {
    pub pid: Option<u32>,
    pub sni_host: Option<String>,
    pub ts: u64,
}

pub trait FlowStore: Send + Sync {
    fn recent_network_events(&self, since: Duration) -> Vec<NetworkEventRecord>;
}

/// Reads SNI flows from the DEK telemetry spool
pub struct SpoolFlowSource {
    store: Arc<dyn FlowStore>,
}

impl SpoolFlowSource {
    pub fn new(store: Arc<dyn FlowStore>) -> Self {
        Self { store }
    }
}

impl SniFlowSource for SpoolFlowSource {
    fn recent_flows(&self, since: Duration) -> Vec<SniFlow> {
        self.store
            .recent_network_events(since)
            .into_iter()
            .filter_map(|ev| {
                Some(SniFlow {
                    browser_pid: ev.pid,
                    sni_host: ev.sni_host?,
                    ts: ev.ts,
                })
            })
            .collect()
    }
}
