//! spooler.rs — durable, bounded telemetry spool (SQLite-backed).
//!
//! Production hardening (vs. previous):
//!  - Bounded on disk: a hard row cap with drop-oldest/lowest-priority eviction
//!    so a long cloud outage can't fill the disk (the events table previously
//!    grew unbounded).
//!  - WAL + sane PRAGMAs for crash-safety and bounded journal growth.
//!  - INCREMENTAL auto_vacuum + a `vacuum()` hook so disk is reclaimed after
//!    batches are acked.
//!  - Public API is unchanged (new/push/pop_batch/delete_batch/len) plus two
//!    additions (`with_capacity`, `vacuum`); existing callers keep working.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::sync::Mutex;
use tracing::warn;

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

/// Default hard cap on spooled rows. At ~1KB/event this bounds the DB to a few
/// tens of MB. When exceeded, the lowest-priority, oldest rows are evicted.
pub const DEFAULT_MAX_ROWS: i64 = 50_000;

pub struct Spooler {
    conn: Mutex<Connection>,
    max_rows: i64,
}

impl Spooler {
    pub fn new(db_path: &str) -> Result<Self> {
        Self::with_capacity(db_path, DEFAULT_MAX_ROWS)
    }

    /// Like `new`, but with an explicit row cap (drop-oldest beyond it).
    pub fn with_capacity(db_path: &str, max_rows: i64) -> Result<Self> {
        let conn = Connection::open(db_path).context("open spool db")?;

        // PRAGMAs must run before tables are created for auto_vacuum to take
        // effect on a fresh DB without a full VACUUM.
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA auto_vacuum = INCREMENTAL;
             PRAGMA journal_size_limit = 8388608;", // ~8 MiB cap on the WAL
        )
        .context("set spool pragmas")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                priority INTEGER NOT NULL,
                payload TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        // Index matches the drain + eviction ordering.
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_drain ON events (priority DESC, id ASC)",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
            max_rows: max_rows.max(1),
        })
    }

    pub fn push(&self, priority: Priority, payload: &Value) -> Result<()> {
        let payload_str = serde_json::to_string(payload)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events (priority, payload) VALUES (?1, ?2)",
            params![priority as i32, payload_str],
        )?;

        // Drop-oldest enforcement: keep only the newest `max_rows` ranked by
        // (priority DESC, id DESC). Under sustained outage we shed the least
        // important, oldest events rather than exhausting the disk.
        let evicted = conn.execute(
            "DELETE FROM events
             WHERE id NOT IN (
                 SELECT id FROM events
                 ORDER BY priority DESC, id DESC
                 LIMIT ?1
             )",
            params![self.max_rows],
        )?;
        if evicted > 0 {
            warn!(evicted, cap = self.max_rows, "telemetry spool full; evicted oldest/low-priority events");
        }
        Ok(())
    }

    pub fn pop_batch(&self, limit: usize) -> Result<Vec<(i64, Value)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, payload FROM events ORDER BY priority DESC, id ASC LIMIT ?1")?;
        let rows = stmt.query_map([limit as i64], |row| {
            let id: i64 = row.get(0)?;
            let payload_str: String = row.get(1)?;
            Ok((id, payload_str))
        })?;

        let mut batch = Vec::new();
        for r in rows.flatten() {
            let (id, p_str) = r;
            if let Ok(v) = serde_json::from_str(&p_str) {
                batch.push((id, v));
            }
        }
        Ok(batch)
    }

    pub fn delete_batch(&self, ids: &[i64]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("DELETE FROM events WHERE id IN ({})", placeholders);
        let mut stmt = conn.prepare(&query)?;
        stmt.execute(rusqlite::params_from_iter(ids.iter()))?;
        Ok(())
    }

    /// Reclaim free pages after acked batches. Call periodically (e.g. every
    /// few drain cycles) from the telemetry loop — cheap and incremental.
    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("PRAGMA incremental_vacuum;")?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn evicts_oldest_when_over_capacity() {
        let s = Spooler::with_capacity(":memory:", 3).unwrap();
        for i in 0..5 {
            s.push(Priority::Normal, &json!({ "n": i })).unwrap();
        }
        assert_eq!(s.len().unwrap(), 3); // only newest 3 survive
        let batch = s.pop_batch(10).unwrap();
        let ns: Vec<i64> = batch.iter().map(|(_, v)| v["n"].as_i64().unwrap()).collect();
        assert_eq!(ns, vec![2, 3, 4]); // 0,1 evicted
    }

    #[test]
    fn critical_survives_eviction_over_low() {
        let s = Spooler::with_capacity(":memory:", 2).unwrap();
        s.push(Priority::Critical, &json!({ "k": "keep" })).unwrap();
        s.push(Priority::Low, &json!({ "k": "a" })).unwrap();
        s.push(Priority::Low, &json!({ "k": "b" })).unwrap();
        let batch = s.pop_batch(10).unwrap();
        let ks: Vec<String> = batch.iter().map(|(_, v)| v["k"].as_str().unwrap().to_string()).collect();
        assert!(ks.contains(&"keep".to_string()), "critical must survive");
        assert_eq!(batch.len(), 2);
    }
}
