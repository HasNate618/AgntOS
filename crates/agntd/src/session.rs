//! SQLite FTS5 session store for the AgntOS agent.
//!
//! Stores every conversation turn (user, assistant, tool) in an append-only
//! table backed by a FTS5 full-text search index.  Supports `history <query>`
//! lookups that need to span sessions without adding to the LLM context window.
//!
//! ## Schema
//!
//! ```sql
//! CREATE TABLE session_turns (id, session_id, role, content, tool_name, created_at);
//! CREATE VIRTUAL TABLE session_turns_fts USING fts5(content=session_turns, content_rowid=id);
//! ```
//!
//! Triggers keep the FTS index in sync on INSERT / DELETE / UPDATE.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct SessionStore {
    db_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SessionHit {
    pub row_id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

impl SessionStore {
    pub fn from_config_dir(config_dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = config_dir.as_ref().join("memory");
        std::fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "Failed to create session store dir {}: {}",
                dir.display(),
                e
            )
        })?;
        let db_path = dir.join("sessions.db");
        let store = Self { db_path };
        store.migrate()?;
        Ok(store)
    }

    pub fn new_session_id() -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("s-{:x}", ts)
    }

    pub fn append_turn(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        tool_name: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.open()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO session_turns (session_id, role, content, tool_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, role, content, tool_name, now],
        )
        .map_err(|e| format!("Failed to append session turn: {}", e))?;
        Ok(())
    }

    /// Returns the most recent turns (up to `limit`) across all sessions,
    /// ordered newest-first.
    pub fn recent_turns(&self, limit: usize) -> Result<Vec<SessionHit>, String> {
        let conn = self.open()?;
        let mut stmt = conn
            .prepare(
                "SELECT rowid, session_id, role, content, created_at
                 FROM session_turns
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to query recent turns: {}", e))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let ts: String = row.get(4)?;
                let parsed = DateTime::parse_from_rfc3339(&ts)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(SessionHit {
                    row_id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: parsed,
                })
            })
            .map_err(|e| format!("Failed to query recent turns: {}", e))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| format!("Failed to read recent turn: {}", e))?);
        }
        Ok(out)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SessionHit>, String> {
        let conn = self.open()?;
        let mut stmt = conn
            .prepare(
                "SELECT rowid, session_id, role, content, created_at
                 FROM session_turns_fts
                 WHERE session_turns_fts MATCH ?1
                 ORDER BY bm25(session_turns_fts)
                 LIMIT ?2",
            )
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        let rows = stmt
            .query_map(params![query, limit as i64], |row| {
                let ts: String = row.get(4)?;
                let parsed = DateTime::parse_from_rfc3339(&ts)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(SessionHit {
                    row_id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: parsed,
                })
            })
            .map_err(|e| format!("Failed to run session search: {}", e))?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| format!("Failed to read search result: {}", e))?);
        }
        Ok(out)
    }

    fn migrate(&self) -> Result<(), String> {
        let conn = self.open()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS session_turns (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              session_id TEXT NOT NULL,
              role TEXT NOT NULL,
              content TEXT NOT NULL,
              tool_name TEXT,
              created_at TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS session_turns_fts USING fts5(
              session_id,
              role,
              content,
              created_at,
              content='session_turns',
              content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS session_turns_ai AFTER INSERT ON session_turns BEGIN
              INSERT INTO session_turns_fts(rowid, session_id, role, content, created_at)
              VALUES (new.id, new.session_id, new.role, new.content, new.created_at);
            END;

            CREATE TRIGGER IF NOT EXISTS session_turns_ad AFTER DELETE ON session_turns BEGIN
              INSERT INTO session_turns_fts(session_turns_fts, rowid, session_id, role, content, created_at)
              VALUES ('delete', old.id, old.session_id, old.role, old.content, old.created_at);
            END;

            CREATE TRIGGER IF NOT EXISTS session_turns_au AFTER UPDATE ON session_turns BEGIN
              INSERT INTO session_turns_fts(session_turns_fts, rowid, session_id, role, content, created_at)
              VALUES ('delete', old.id, old.session_id, old.role, old.content, old.created_at);
              INSERT INTO session_turns_fts(rowid, session_id, role, content, created_at)
              VALUES (new.id, new.session_id, new.role, new.content, new.created_at);
            END;
            ",
        )
        .map_err(|e| format!("Failed to migrate session DB: {}", e))?;
        Ok(())
    }

    fn open(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open {}: {}", self.db_path.display(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_search() {
        let dir = std::env::temp_dir().join("agntos-session-store-test");
        let _ = std::fs::remove_dir_all(&dir);

        let store = SessionStore::from_config_dir(&dir).unwrap();
        let sid = SessionStore::new_session_id();
        store
            .append_turn(&sid, "user", "install hello package", None)
            .unwrap();
        store
            .append_turn(&sid, "assistant", "I can propose hello", None)
            .unwrap();

        let hits = store.search("hello", 10).unwrap();
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|h| h.content.contains("hello")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
