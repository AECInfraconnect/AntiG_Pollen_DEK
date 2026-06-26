use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

pub const DEFAULT_MAX_ROWS: i64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

pub struct SqliteSpool {
    conn: Mutex<Connection>,
    max_rows: i64,
    cipher: Aes256Gcm,
}

impl SqliteSpool {
    pub fn new(
        db_path: &PathBuf,
        key_bytes: &[u8; 32],
        max_rows: i64,
    ) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA auto_vacuum = INCREMENTAL;
             PRAGMA journal_size_limit = 8388608;",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                priority INTEGER NOT NULL,
                payload BLOB NOT NULL,
                nonce BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_drain ON events (priority DESC, id ASC)",
            [],
        )?;

        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            conn: Mutex::new(conn),
            max_rows: max_rows.max(1),
            cipher,
        })
    }

    pub fn push(&self, priority: Priority, payload: &[u8]) -> Result<(), anyhow::Error> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, payload)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO events (priority, payload, nonce) VALUES (?1, ?2, ?3)",
            params![priority as i32, ciphertext, nonce.as_slice()],
        )?;

        let evicted = conn.execute(
            "DELETE FROM events
             WHERE id NOT IN (
                 SELECT id FROM events
                 ORDER BY priority DESC, id DESC
                 LIMIT ?1
             )",
            params![self.max_rows],
        )?;

        Ok(())
    }

    pub fn pop_batch(&self, limit: usize) -> Result<Vec<(i64, Vec<u8>)>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, payload, nonce FROM events ORDER BY priority DESC, id ASC LIMIT ?1",
        )?;

        let rows = stmt.query_map([limit as i64], |row| {
            let id: i64 = row.get(0)?;
            let payload_blob: Vec<u8> = row.get(1)?;
            let nonce_blob: Vec<u8> = row.get(2)?;
            Ok((id, payload_blob, nonce_blob))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (id, payload_blob, nonce_blob) = row?;
            let nonce = aes_gcm::Nonce::from_slice(&nonce_blob);
            match self.cipher.decrypt(nonce, payload_blob.as_ref()) {
                Ok(decrypted) => {
                    results.push((id, decrypted));
                }
                Err(_) => {
                    // Skip decryption failures
                }
            }
        }
        Ok(results)
    }

    pub fn delete_batch(&self, ids: &[i64]) -> Result<(), anyhow::Error> {
        if ids.is_empty() {
            return Ok(());
        }
        let conn = self.conn.lock().unwrap();
        let id_list: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let query = format!("DELETE FROM events WHERE id IN ({})", id_list.join(","));
        conn.execute(&query, [])?;
        Ok(())
    }

    pub fn peek_recent(&self, limit: usize) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT payload, nonce FROM events ORDER BY id DESC LIMIT ?1")?;

        let rows = stmt.query_map([limit as i64], |row| {
            let payload_blob: Vec<u8> = row.get(0)?;
            let nonce_blob: Vec<u8> = row.get(1)?;
            Ok((payload_blob, nonce_blob))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (payload_blob, nonce_blob) = row?;
            let nonce = aes_gcm::Nonce::from_slice(&nonce_blob);
            if let Ok(decrypted) = self.cipher.decrypt(nonce, payload_blob.as_ref()) {
                results.push(decrypted);
            }
        }
        Ok(results)
    }
}
