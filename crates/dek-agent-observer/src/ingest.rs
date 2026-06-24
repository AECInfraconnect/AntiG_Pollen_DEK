use crate::error::ObserverError;
use crate::model::AgentObservationEvent;
use crate::trust::{AgentBaseline, TrustScore};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::sync::Mutex;

#[async_trait]
pub trait ObservationStore: Send + Sync {
    async fn append(&self, event: AgentObservationEvent) -> Result<(), ObserverError>;
    async fn recent_for_agent(
        &self,
        agent_id: &str,
        limit: u32,
    ) -> Result<Vec<AgentObservationEvent>, ObserverError>;
    async fn update_baseline(&self, agent_id: &str) -> Result<TrustScore, ObserverError>;
}

pub struct SqliteObservationStore {
    conn: Mutex<Connection>,
}

impl SqliteObservationStore {
    pub fn new(db_path: &str) -> Result<Self, ObserverError> {
        let conn =
            Connection::open(db_path).map_err(|e| ObserverError::Ingestion(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS observations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT,
                payload TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| ObserverError::Ingestion(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[async_trait]
impl ObservationStore for SqliteObservationStore {
    async fn append(&self, event: AgentObservationEvent) -> Result<(), ObserverError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ObserverError::Ingestion("lock poisoned".into()))?;
        let payload = serde_json::to_string(&event).unwrap_or_default();
        conn.execute(
            "INSERT INTO observations (agent_id, payload) VALUES (?1, ?2)",
            params![event.agent_id, payload],
        )
        .map_err(|e| ObserverError::Ingestion(e.to_string()))?;
        Ok(())
    }

    async fn recent_for_agent(
        &self,
        agent_id: &str,
        limit: u32,
    ) -> Result<Vec<AgentObservationEvent>, ObserverError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ObserverError::Ingestion("lock poisoned".into()))?;
        let mut stmt = conn
            .prepare(
                "SELECT payload FROM observations WHERE agent_id = ?1 ORDER BY id DESC LIMIT ?2",
            )
            .map_err(|e| ObserverError::Ingestion(e.to_string()))?;
        let rows = stmt
            .query_map(params![agent_id, limit as i64], |row| {
                let payload: String = row.get(0)?;
                Ok(payload)
            })
            .map_err(|e| ObserverError::Ingestion(e.to_string()))?;

        let mut events = Vec::new();
        for r in rows.flatten() {
            if let Ok(ev) = serde_json::from_str(&r) {
                events.push(ev);
            }
        }
        Ok(events)
    }

    async fn update_baseline(&self, agent_id: &str) -> Result<TrustScore, ObserverError> {
        let events = self.recent_for_agent(agent_id, 100).await?;
        let mut baseline = AgentBaseline::default();
        for ev in &events {
            baseline.observe(ev);
        }
        Ok(baseline.calculate_trust(agent_id))
    }
}

#[allow(clippy::print_stdout)]
pub fn ingest_event(event: AgentObservationEvent) -> Result<(), String> {
    // Keep this for backward compatibility or default fallback
    if event.event_id.is_empty() {
        return Err("event_id is required".to_string());
    }
    println!("Ingested event: {}", event.event_id);
    Ok(())
}
