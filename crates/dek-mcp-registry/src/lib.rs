use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRecord {
    pub id: String,
    pub transport: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecord {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

pub trait McpRegistry {
    fn register_agent(&self, agent: AgentRecord) -> Result<()>;
    fn register_server(&self, server: McpServerRecord) -> Result<()>;
    fn register_tools(&self, server_id: &str, tools: Vec<ToolRecord>) -> Result<()>;
    fn get_server(&self, server_id: &str) -> Result<Option<McpServerRecord>>;
    fn get_tools(&self, server_id: &str) -> Result<Option<Vec<ToolRecord>>>;
}

pub struct InMemoryRegistry {
    agents: Arc<RwLock<HashMap<String, AgentRecord>>>,
    servers: Arc<RwLock<HashMap<String, McpServerRecord>>>,
    tools: Arc<RwLock<HashMap<String, Vec<ToolRecord>>>>,
}

impl InMemoryRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            servers: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl McpRegistry for InMemoryRegistry {
    fn register_agent(&self, agent: AgentRecord) -> Result<()> {
        let mut map = self
            .agents
            .write()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        map.insert(agent.id.clone(), agent);
        Ok(())
    }

    fn register_server(&self, server: McpServerRecord) -> Result<()> {
        let mut map = self
            .servers
            .write()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        map.insert(server.id.clone(), server);
        Ok(())
    }

    fn register_tools(&self, server_id: &str, tools: Vec<ToolRecord>) -> Result<()> {
        let mut map = self
            .tools
            .write()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        map.insert(server_id.to_string(), tools);
        Ok(())
    }

    fn get_server(&self, server_id: &str) -> Result<Option<McpServerRecord>> {
        let map = self
            .servers
            .read()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        Ok(map.get(server_id).cloned())
    }

    fn get_tools(&self, server_id: &str) -> Result<Option<Vec<ToolRecord>>> {
        let map = self
            .tools
            .read()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        Ok(map.get(server_id).cloned())
    }
}
