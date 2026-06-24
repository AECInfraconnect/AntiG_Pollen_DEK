use crate::NetworkEnforcer;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EnforcementRouter {
    enforcers: Vec<Arc<RwLock<dyn NetworkEnforcer>>>,
}

impl Default for EnforcementRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl EnforcementRouter {
    pub fn new() -> Self {
        Self {
            enforcers: Vec::new(),
        }
    }

    pub fn register_enforcer(&mut self, enforcer: Arc<RwLock<dyn NetworkEnforcer>>) {
        self.enforcers.push(enforcer);
    }

    pub async fn start_all(&self) -> Result<()> {
        for e in &self.enforcers {
            e.write().await.start()?;
        }
        Ok(())
    }

    pub async fn stop_all(&self) -> Result<()> {
        for e in &self.enforcers {
            e.write().await.stop()?;
        }
        Ok(())
    }

    pub async fn apply_to_all(
        &self,
        rules: &dek_domain_schema::CompiledNetworkRules,
    ) -> Result<()> {
        for e in &self.enforcers {
            e.read().await.apply_rules(rules)?;
        }
        Ok(())
    }

    pub async fn clear_all(&self) -> Result<()> {
        for e in &self.enforcers {
            e.read().await.clear_rules()?;
        }
        Ok(())
    }
}
