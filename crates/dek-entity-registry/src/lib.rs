use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityRegistrySnapshot {
    pub tenant_id: String,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub r#type: String,
    pub id: String,
    pub state: EntityState,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityState {
    Discovered,
    Registered,
    Attested,
    Approved,
    Active,
    Suspended,
    Retired,
}
