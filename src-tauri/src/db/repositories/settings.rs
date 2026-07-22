use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn get(conn: &Connection, key: &str) -> Result<Option<AppSetting>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value, value_type, created_at, updated_at FROM app_settings WHERE key = ?1")
        .map_err(|e| format!("Settings get error: {}", e))?;

    let result = stmt.query_row(params![key], |row| {
        Ok(AppSetting {
            key: row.get(0)?,
            value: row.get(1)?,
            value_type: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    });

    match result {
        Ok(setting) => Ok(Some(setting)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Settings get error: {}", e)),
    }
}

pub fn set(conn: &Connection, key: &str, value: &str, value_type: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_settings (key, value, value_type) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, value_type = excluded.value_type, updated_at = datetime('now')",
        params![key, value, value_type],
    )
    .map_err(|e| format!("Settings set error: {}", e))?;
    Ok(())
}

pub fn delete(conn: &Connection, key: &str) -> Result<bool, String> {
    let count = conn
        .execute("DELETE FROM app_settings WHERE key = ?1", params![key])
        .map_err(|e| format!("Settings delete error: {}", e))?;
    Ok(count > 0)
}

pub fn list(conn: &Connection) -> Result<Vec<AppSetting>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value, value_type, created_at, updated_at FROM app_settings ORDER BY key")
        .map_err(|e| format!("Settings list error: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
                value_type: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Settings list error: {}", e))?;

    let mut settings = Vec::new();
    for row in rows {
        settings.push(row.map_err(|e| format!("Settings row error: {}", e))?);
    }
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        conn
    }

    #[test]
    fn test_crud() {
        let conn = setup();

        // Set
        set(&conn, "theme", "dark", "string").unwrap();
        let s = get(&conn, "theme").unwrap().unwrap();
        assert_eq!(s.value, "dark");

        // Update
        set(&conn, "theme", "light", "string").unwrap();
        let s = get(&conn, "theme").unwrap().unwrap();
        assert_eq!(s.value, "light");

        // List
        set(&conn, "onboarding", "true", "bool").unwrap();
        let list = list(&conn).unwrap();
        assert_eq!(list.len(), 2);

        // Delete
        assert!(delete(&conn, "theme").unwrap());
        assert!(get(&conn, "theme").unwrap().is_none());

        // Delete non-existent
        assert!(!delete(&conn, "nope").unwrap());
    }
}
