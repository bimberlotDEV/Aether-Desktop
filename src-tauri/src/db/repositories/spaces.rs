use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub accent: Option<String>,
    pub template_type: Option<String>,
    pub favourite: bool,
    pub archived_at: Option<String>,
    pub sort_order: i64,
    pub settings_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn row_to_space(row: &rusqlite::Row) -> rusqlite::Result<Space> {
    Ok(Space {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        icon: row.get(3)?,
        accent: row.get(4)?,
        template_type: row.get(5)?,
        favourite: row.get::<_, i64>(6)? != 0,
        archived_at: row.get(7)?,
        sort_order: row.get(8)?,
        settings_json: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

pub fn create(conn: &Connection, space: &Space) -> Result<Space, String> {
    conn.execute(
        "INSERT INTO spaces (id, name, description, icon, accent, template_type, sort_order, settings_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            space.id,
            space.name,
            space.description,
            space.icon,
            space.accent,
            space.template_type,
            space.sort_order,
            space.settings_json,
        ],
    )
    .map_err(|e| format!("Space create error: {}", e))?;

    get_by_id(conn, &space.id)?
        .ok_or_else(|| "Space not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Space>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, icon, accent, template_type,
                    favourite, archived_at, sort_order, settings_json, created_at, updated_at
             FROM spaces WHERE id = ?1",
        )
        .map_err(|e| format!("Space get error: {}", e))?;

    let result = stmt.query_row(params![id], row_to_space);

    match result {
        Ok(space) => Ok(Some(space)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Space get error: {}", e)),
    }
}

pub fn list(conn: &Connection, include_archived: bool) -> Result<Vec<Space>, String> {
    let sql = if include_archived {
        "SELECT id, name, description, icon, accent, template_type,
                favourite, archived_at, sort_order, settings_json, created_at, updated_at
         FROM spaces ORDER BY favourite DESC, sort_order ASC, created_at DESC"
    } else {
        "SELECT id, name, description, icon, accent, template_type,
                favourite, archived_at, sort_order, settings_json, created_at, updated_at
         FROM spaces WHERE archived_at IS NULL
         ORDER BY favourite DESC, sort_order ASC, created_at DESC"
    };

    let mut stmt = conn.prepare(sql).map_err(|e| format!("Space list error: {}", e))?;
    let rows = stmt
        .query_map([], row_to_space)
        .map_err(|e| format!("Space list error: {}", e))?;

    let mut spaces = Vec::new();
    for row in rows {
        spaces.push(row.map_err(|e| format!("Space row error: {}", e))?);
    }
    Ok(spaces)
}

pub fn update(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    description: Option<&str>,
    icon: Option<&str>,
    accent: Option<&str>,
    settings_json: Option<&str>,
) -> Result<Option<Space>, String> {
    let mut sets = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = name {
        sets.push("name = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = description {
        sets.push("description = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = icon {
        sets.push("icon = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = accent {
        sets.push("accent = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = settings_json {
        sets.push("settings_json = ?");
        values.push(Box::new(v.to_string()));
    }

    if sets.is_empty() {
        return get_by_id(conn, id);
    }

    sets.push("updated_at = datetime('now')");
    values.push(Box::new(id.to_string()));

    let sql = format!(
        "UPDATE spaces SET {} WHERE id = ?{}",
        sets.join(", "),
        values.len()
    );
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();

    let affected = conn
        .execute(&sql, params_refs.as_slice())
        .map_err(|e| format!("Space update error: {}", e))?;

    if affected == 0 {
        Ok(None)
    } else {
        get_by_id(conn, id)
    }
}

pub fn archive(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute(
            "UPDATE spaces SET archived_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1 AND archived_at IS NULL",
            params![id],
        )
        .map_err(|e| format!("Space archive error: {}", e))?;
    Ok(affected > 0)
}

pub fn restore(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute(
            "UPDATE spaces SET archived_at = NULL, updated_at = datetime('now') WHERE id = ?1 AND archived_at IS NOT NULL",
            params![id],
        )
        .map_err(|e| format!("Space restore error: {}", e))?;
    Ok(affected > 0)
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute("DELETE FROM spaces WHERE id = ?1", params![id])
        .map_err(|e| format!("Space delete error: {}", e))?;
    Ok(affected > 0)
}

pub fn set_favourite(conn: &Connection, id: &str, fav: bool) -> Result<bool, String> {
    let affected = conn
        .execute(
            "UPDATE spaces SET favourite = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![fav as i64, id],
        )
        .map_err(|e| format!("Space favourite error: {}", e))?;
    Ok(affected > 0)
}

pub fn reorder(conn: &Connection, ordered_ids: &[String]) -> Result<(), String> {
    for (i, id) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE spaces SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![i as i64, id],
        )
        .map_err(|e| format!("Space reorder error: {}", e))?;
    }
    Ok(())
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

    fn test_space(id: &str, name: &str) -> Space {
        Space {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            icon: None,
            accent: None,
            template_type: Some("blank".to_string()),
            favourite: false,
            archived_at: None,
            sort_order: 0,
            settings_json: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_space_crud() {
        let conn = setup();
        let s = create(&conn, &test_space("s1", "Test")).unwrap();
        assert_eq!(s.name, "Test");
        assert!(!s.favourite);

        let s = get_by_id(&conn, "s1").unwrap().unwrap();
        assert_eq!(s.template_type.as_deref(), Some("blank"));

        let s = update(&conn, "s1", Some("Renamed"), None, None, None, None).unwrap().unwrap();
        assert_eq!(s.name, "Renamed");

        set_favourite(&conn, "s1", true).unwrap();
        let s = get_by_id(&conn, "s1").unwrap().unwrap();
        assert!(s.favourite);
    }

    #[test]
    fn test_archive_restore() {
        let conn = setup();
        create(&conn, &test_space("s1", "Test")).unwrap();

        assert!(archive(&conn, "s1").unwrap());
        let s = get_by_id(&conn, "s1").unwrap().unwrap();
        assert!(s.archived_at.is_some());

        // Not in unarchived list
        let active = list(&conn, false).unwrap();
        assert!(active.is_empty());

        // In full list
        let all = list(&conn, true).unwrap();
        assert_eq!(all.len(), 1);

        assert!(restore(&conn, "s1").unwrap());
        let s = get_by_id(&conn, "s1").unwrap().unwrap();
        assert!(s.archived_at.is_none());
    }

    #[test]
    fn test_delete() {
        let conn = setup();
        create(&conn, &test_space("s1", "Test")).unwrap();
        assert!(delete(&conn, "s1").unwrap());
        assert!(get_by_id(&conn, "s1").unwrap().is_none());
        assert!(!delete(&conn, "nope").unwrap());
    }

    #[test]
    fn test_reorder() {
        let conn = setup();
        create(&conn, &test_space("a", "A")).unwrap();
        create(&conn, &test_space("b", "B")).unwrap();
        create(&conn, &test_space("c", "C")).unwrap();

        reorder(&conn, &["c".into(), "a".into(), "b".into()]).unwrap();

        let spaces = list(&conn, false).unwrap();
        assert_eq!(spaces[0].id, "c");
        assert_eq!(spaces[1].id, "a");
        assert_eq!(spaces[2].id, "b");
    }
}
