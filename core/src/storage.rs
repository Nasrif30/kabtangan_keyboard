//! SQLite storage layer — v1 (local-only).
//!
//! Stores only user-mutable data:
//! - Personal dictionary (user-added words)
//! - User settings (key-value)
//! - Clipboard history
//! - Word usage statistics (for local prediction tuning)
//!
//! The main dictionary is NOT stored here.
//! It is loaded from a bundled static JSON file (see `dictionary::StaticDictionary`).
//!
//! Everything is local. No network. No sync. No telemetry.

use rusqlite::Connection;
use crate::error::CoreResult;

/// Schema version. Bump on breaking schema changes; add a migration arm.
const SCHEMA_VERSION: u32 = 1;

/// Encapsulates the SQLite connection and its lifecycle.
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open (or create) the database at `path`.
    ///
    /// Applies schema migrations automatically.
    pub fn open(path: &str) -> CoreResult<Self> {
        let conn = Connection::open(path)?;
        let mut storage = Self { conn };
        storage.configure()?;
        storage.migrate()?;
        Ok(storage)
    }

    /// Open an in-memory database — for tests only.
    pub fn open_in_memory() -> CoreResult<Self> {
        let conn = Connection::open_in_memory()?;
        let mut storage = Self { conn };
        storage.configure()?;
        storage.migrate()?;
        Ok(storage)
    }

    /// Apply PRAGMA performance settings.
    fn configure(&self) -> CoreResult<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous   = NORMAL;
            PRAGMA foreign_keys  = ON;
            PRAGMA temp_store     = MEMORY;
            PRAGMA cache_size     = -4000;  -- 4 MB
        ",
        )?;
        Ok(())
    }

    fn migrate(&mut self) -> CoreResult<()> {
        let version = self.schema_version()?;
        if version < 1 {
            self.apply_v1()?;
        }
        Ok(())
    }

    fn schema_version(&self) -> CoreResult<u32> {
        Ok(self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap_or(0))
    }

    fn set_schema_version(&self, v: u32) -> CoreResult<()> {
        self.conn
            .execute_batch(&format!("PRAGMA user_version = {v}"))?;
        Ok(())
    }

    /// v1 schema: personal dictionary, settings, clipboard, word usage.
    fn apply_v1(&mut self) -> CoreResult<()> {
        self.conn.execute_batch(
            "
            -- Words the user has added (names, places, abbreviations)
            CREATE TABLE IF NOT EXISTS personal_dictionary (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                word       TEXT NOT NULL UNIQUE COLLATE NOCASE,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Key-value store for all user settings
            CREATE TABLE IF NOT EXISTS settings (
                key        TEXT PRIMARY KEY,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Clipboard history (local only)
            CREATE TABLE IF NOT EXISTS clipboard (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                content    TEXT NOT NULL,
                is_pinned  INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_clipboard_pinned  ON clipboard(is_pinned DESC);
            CREATE INDEX IF NOT EXISTS idx_clipboard_created ON clipboard(created_at DESC);

            -- Local word usage for personalised prediction ranking
            CREATE TABLE IF NOT EXISTS word_usage (
                word      TEXT PRIMARY KEY,
                count     INTEGER NOT NULL DEFAULT 1,
                last_used TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_usage_last ON word_usage(last_used DESC);
        ",
        )?;
        self.set_schema_version(SCHEMA_VERSION)?;
        Ok(())
    }

    /// Access the raw connection — used by repository types.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_in_memory() {
        assert!(Storage::open_in_memory().is_ok());
    }

    #[test]
    fn correct_schema_version() {
        let s = Storage::open_in_memory().unwrap();
        assert_eq!(s.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn expected_tables_exist() {
        let s = Storage::open_in_memory().unwrap();
        let conn = s.connection();
        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        assert!(tables.contains(&"personal_dictionary".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"clipboard".to_string()));
        assert!(tables.contains(&"word_usage".to_string()));
        // Main dictionary is NOT in SQLite — it's a static JSON file.
        assert!(!tables.contains(&"dictionary".to_string()));
    }
}
