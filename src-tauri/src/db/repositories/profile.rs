use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub onboarding_completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub fn get(conn: &Connection) -> Result<Option<UserProfile>, String> {
    let mut stmt = conn
        .prepare("SELECT id, display_name, onboarding_completed, created_at, updated_at FROM user_profile LIMIT 1")
        .map_err(|e| format!("Profile get error: {}", e))?;

    let result = stmt.query_row([], |row| {
        Ok(UserProfile {
            id: row.get(0)?,
            display_name: row.get(1)?,
            onboarding_completed: row.get::<_, i64>(2)? != 0,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    });

    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Profile get error: {}", e)),
    }
}

pub fn create(conn: &Connection, id: &str) -> Result<UserProfile, String> {
    // Only one local profile is allowed
    if get(conn)?.is_some() {
        return Err("A local profile already exists".to_string());
    }

    conn.execute("INSERT INTO user_profile (id) VALUES (?1)", params![id])
        .map_err(|e| format!("Profile create error: {}", e))?;

    get(conn)?.ok_or_else(|| "Profile not found after create".to_string())
}

pub fn update(
    conn: &Connection,
    id: &str,
    display_name: Option<&str>,
    onboarding_completed: Option<bool>,
) -> Result<Option<UserProfile>, String> {
    let mut sets = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(name) = display_name {
        sets.push("display_name = ?");
        values.push(Box::new(name.to_string()));
    }
    if let Some(completed) = onboarding_completed {
        sets.push("onboarding_completed = ?");
        values.push(Box::new(completed as i64));
    }

    if sets.is_empty() {
        return get(conn);
    }

    sets.push("updated_at = datetime('now')");
    values.push(Box::new(id.to_string()));

    let sql = format!(
        "UPDATE user_profile SET {} WHERE id = ?{}",
        sets.join(", "),
        values.len()
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();

    let affected = conn
        .execute(&sql, params_refs.as_slice())
        .map_err(|e| format!("Profile update error: {}", e))?;

    if affected == 0 {
        Ok(None)
    } else {
        get(conn)
    }
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
    fn test_profile_lifecycle() {
        let conn = setup();

        // No profile initially
        assert!(get(&conn).unwrap().is_none());

        // Create
        let p = create(&conn, "test-uuid-1").unwrap();
        assert_eq!(p.id, "test-uuid-1");
        assert!(!p.onboarding_completed);
        assert!(p.display_name.is_none());

        // Update
        let p = update(&conn, "test-uuid-1", Some("Bim"), Some(true))
            .unwrap()
            .unwrap();
        assert_eq!(p.display_name.as_deref(), Some("Bim"));
        assert!(p.onboarding_completed);

        // Only one profile exists
        assert!(create(&conn, "test-uuid-2").is_err());
    }
}
