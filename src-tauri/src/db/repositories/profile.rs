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

/// Initialize the singleton profile without mistaking an upgraded workspace for a
/// brand-new installation. Earlier Aether versions shipped the profile table but never
/// created a row, so persisted domain data is the authoritative legacy signal.
pub fn initialize(conn: &Connection, id: &str) -> Result<UserProfile, String> {
    if let Some(profile) = get(conn)? {
        return Ok(profile);
    }

    let has_workspace_data = has_meaningful_workspace_data(conn)?;
    conn.execute(
        "INSERT INTO user_profile (id, onboarding_completed) VALUES (?1, ?2)",
        params![id, has_workspace_data as i64],
    )
    .map_err(|e| format!("Profile initialization error: {e}"))?;

    get(conn)?.ok_or_else(|| "Profile not found after initialization".to_string())
}

fn has_meaningful_workspace_data(conn: &Connection) -> Result<bool, String> {
    conn.query_row(
        "SELECT
            EXISTS(SELECT 1 FROM spaces LIMIT 1)
            OR EXISTS(SELECT 1 FROM notes LIMIT 1)
            OR EXISTS(SELECT 1 FROM tasks LIMIT 1)
            OR EXISTS(SELECT 1 FROM vault_items LIMIT 1)
            OR EXISTS(SELECT 1 FROM memory_items LIMIT 1)
            OR EXISTS(SELECT 1 FROM ai_conversations LIMIT 1)
            OR EXISTS(SELECT 1 FROM sources LIMIT 1)",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value != 0)
    .map_err(|e| format!("Profile workspace detection error: {e}"))
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

    #[test]
    fn initialization_requires_onboarding_only_for_an_empty_workspace() {
        let conn = setup();

        let profile = initialize(&conn, "fresh-profile").unwrap();
        assert!(!profile.onboarding_completed);

        let repeated = initialize(&conn, "ignored-id").unwrap();
        assert_eq!(repeated.id, "fresh-profile");
        assert!(!repeated.onboarding_completed);
    }

    #[test]
    fn initialization_bypasses_onboarding_for_legacy_workspace_data() {
        let conn = setup();
        conn.execute(
            "INSERT INTO spaces (id, name, sort_order) VALUES ('legacy-space', 'Existing work', 0)",
            [],
        )
        .unwrap();

        let profile = initialize(&conn, "legacy-profile").unwrap();
        assert!(profile.onboarding_completed);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM spaces", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
}
