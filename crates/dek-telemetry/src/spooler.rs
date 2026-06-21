use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Priority {
    pub fn from_i32(v: i32) -> Self {
        match v {
            3 => Priority::Critical,
            2 => Priority::High,
            1 => Priority::Normal,
            _ => Priority::Low,
        }
    }
}

pub struct Spooler {
    conn: Mutex<Connection>,
}

impl Spooler {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                priority INTEGER NOT NULL,
                payload TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn push(&self, priority: Priority, payload: &Value) -> Result<()> {
        let payload_str = serde_json::to_string(payload)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events (priority, payload) VALUES (?1, ?2)",
            params![priority as i32, payload_str],
        )?;
        Ok(())
    }

    pub fn pop_batch(&self, limit: usize) -> Result<Vec<(i64, Value)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, payload FROM events ORDER BY priority DESC, id ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            let id: i64 = row.get(0)?;
            let payload_str: String = row.get(1)?;
            Ok((id, payload_str))
        })?;

        let mut batch = Vec::new();
        for r in rows {
            if let Ok((id, p_str)) = r {
                if let Ok(v) = serde_json::from_str(&p_str) {
                    batch.push((id, v));
                }
            }
        }
        Ok(batch)
    }

    pub fn delete_batch(&self, ids: &[i64]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        // Construct the query
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("DELETE FROM events WHERE id IN ({})", placeholders);
        let mut stmt = conn.prepare(&query)?;
        
        let params = rusqlite::params_from_iter(ids.iter());
        stmt.execute(params)?;
        Ok(())
    }
    
    pub fn len(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM events")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count as usize)
    }
}
