//! Persistent settings storage backed by SQLite key-value table.
//!
//! All settings are stored as strings. Consumers parse values as needed.
//! Defaults are managed by the frontend — the backend is a pure key-value store.

use crate::engine::db::Database;
use std::sync::Arc;

/// Manages persistent settings in SQLite.
pub struct SettingsManager {
    db: Arc<Database>,
}

impl SettingsManager {
    /// Create a new SettingsManager backed by the given database.
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Get a setting value by key. Returns `None` if the key doesn't exist.
    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| format!("Query error: {e}"))?;

        let value = stmt
            .query_row(rusqlite::params![key], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;

        Ok(value)
    }

    /// Get a setting value, returning the provided default if the key doesn't exist.
    pub fn get_or_default(&self, key: &str, default: &str) -> Result<String, String> {
        Ok(self.get(key)?.unwrap_or_else(|| default.to_string()))
    }

    /// Set a setting value. Creates or updates (upsert).
    pub fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )
        .map_err(|e| format!("Failed to set setting: {e}"))?;
        Ok(())
    }
}

/// Extension trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::db::Database;
    use rusqlite::Connection;

    fn test_db() -> Arc<Database> {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::from_conn(conn);
        db.migrate().unwrap();
        Arc::new(db)
    }

    #[test]
    fn set_and_get_setting() {
        let db = test_db();
        let sm = SettingsManager::new(db);

        sm.set("theme", "dark").unwrap();
        let val = sm.get("theme").unwrap();
        assert_eq!(val, Some("dark".to_string()));
    }

    #[test]
    fn get_missing_returns_none() {
        let db = test_db();
        let sm = SettingsManager::new(db);

        let val = sm.get("nonexistent").unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn get_or_default_uses_default() {
        let db = test_db();
        let sm = SettingsManager::new(db);

        let val = sm.get_or_default("missing", "fallback").unwrap();
        assert_eq!(val, "fallback");

        sm.set("missing", "actual").unwrap();
        let val = sm.get_or_default("missing", "fallback").unwrap();
        assert_eq!(val, "actual");
    }

    #[test]
    fn set_overwrites_existing() {
        let db = test_db();
        let sm = SettingsManager::new(db);

        sm.set("key", "first").unwrap();
        sm.set("key", "second").unwrap();
        let val = sm.get("key").unwrap();
        assert_eq!(val, Some("second".to_string()));
    }

    #[test]
    fn multiple_keys_independent() {
        let db = test_db();
        let sm = SettingsManager::new(db);

        sm.set("alpha", "1").unwrap();
        sm.set("beta", "2").unwrap();

        assert_eq!(sm.get("alpha").unwrap(), Some("1".to_string()));
        assert_eq!(sm.get("beta").unwrap(), Some("2".to_string()));
    }
}
