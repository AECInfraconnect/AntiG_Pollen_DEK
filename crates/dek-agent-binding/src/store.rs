use crate::binding::AgentBinding;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait AgentBindingStore: Send + Sync {
    async fn upsert(&self, binding: AgentBinding) -> Result<(), String>;
    async fn get(&self, binding_id: &str) -> Result<Option<AgentBinding>, String>;
    async fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<AgentBinding>, String>;
    async fn delete(&self, binding_id: &str) -> Result<bool, String>;
}

pub struct InMemoryBindingStore {
    bindings: Arc<RwLock<HashMap<String, AgentBinding>>>,
}

impl Default for InMemoryBindingStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryBindingStore {
    pub fn new() -> Self {
        Self {
            bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl AgentBindingStore for InMemoryBindingStore {
    async fn upsert(&self, binding: AgentBinding) -> Result<(), String> {
        let mut map = self.bindings.write().await;
        map.insert(binding.binding_id.clone(), binding);
        Ok(())
    }

    async fn get(&self, binding_id: &str) -> Result<Option<AgentBinding>, String> {
        let map = self.bindings.read().await;
        Ok(map.get(binding_id).cloned())
    }

    async fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<AgentBinding>, String> {
        let map = self.bindings.read().await;
        let filtered: Vec<_> = map
            .values()
            .filter(|b| b.tenant_id == tenant_id)
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn delete(&self, binding_id: &str) -> Result<bool, String> {
        let mut map = self.bindings.write().await;
        Ok(map.remove(binding_id).is_some())
    }
}
