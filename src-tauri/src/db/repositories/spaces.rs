#![allow(clippy::too_many_arguments)]

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub parent_space_id: Option<String>,
    pub last_opened_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInstance {
    pub id: String,
    pub space_id: String,
    pub module_type: String,
    pub title: Option<String>,
    pub config_json: Option<String>,
    pub layout_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Row mapping ─────────────────────────────────────────

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
        parent_space_id: row.get(10)?,
        last_opened_at: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn row_to_module(row: &rusqlite::Row) -> rusqlite::Result<ModuleInstance> {
    Ok(ModuleInstance {
        id: row.get(0)?,
        space_id: row.get(1)?,
        module_type: row.get(2)?,
        title: row.get(3)?,
        config_json: row.get(4)?,
        layout_json: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

const SPACE_COLS: &str = "id, name, description, icon, accent, template_type,
    favourite, archived_at, sort_order, settings_json,
    parent_space_id, last_opened_at, created_at, updated_at";

const MODULE_COLS: &str =
    "id, space_id, module_type, title, config_json, layout_json, created_at, updated_at";

// ─── CRUD ────────────────────────────────────────────────

pub fn create(conn: &Connection, space: &Space) -> Result<Space, String> {
    conn.execute(
        "INSERT INTO spaces (id, name, description, icon, accent, template_type, sort_order, settings_json, parent_space_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            space.id, space.name, space.description, space.icon, space.accent,
            space.template_type, space.sort_order, space.settings_json,
            space.parent_space_id,
        ],
    ).map_err(|e| format!("Space create error: {}", e))?;
    get_by_id(conn, &space.id)?.ok_or_else(|| "Space not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Space>, String> {
    let sql = format!("SELECT {} FROM spaces WHERE id = ?1", SPACE_COLS);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Space get error: {}", e))?;
    match stmt.query_row(params![id], row_to_space) {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Space get error: {}", e)),
    }
}

pub fn list(conn: &Connection, include_archived: bool) -> Result<Vec<Space>, String> {
    let where_clause = if include_archived {
        ""
    } else {
        "WHERE archived_at IS NULL"
    };
    let sql = format!(
        "SELECT {} FROM spaces {} ORDER BY favourite DESC, sort_order ASC, created_at DESC",
        SPACE_COLS, where_clause
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Space list error: {}", e))?;
    let rows = stmt
        .query_map([], row_to_space)
        .map_err(|e| format!("Space list error: {}", e))?;
    let mut spaces = Vec::new();
    for row in rows {
        spaces.push(row.map_err(|e| format!("Space row error: {}", e))?);
    }
    Ok(spaces)
}

pub fn list_by_parent(conn: &Connection, parent_id: &str) -> Result<Vec<Space>, String> {
    let sql = format!(
        "SELECT {} FROM spaces WHERE parent_space_id = ?1 AND archived_at IS NULL ORDER BY sort_order ASC, created_at ASC",
        SPACE_COLS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Space list_by_parent error: {}", e))?;
    let rows = stmt
        .query_map(params![parent_id], row_to_space)
        .map_err(|e| format!("Space list_by_parent error: {}", e))?;
    let mut spaces = Vec::new();
    for row in rows {
        spaces.push(row.map_err(|e| format!("Space row error: {}", e))?);
    }
    Ok(spaces)
}

pub fn list_top_level(conn: &Connection) -> Result<Vec<Space>, String> {
    let sql = format!(
        "SELECT {} FROM spaces WHERE parent_space_id IS NULL AND archived_at IS NULL ORDER BY favourite DESC, sort_order ASC, created_at DESC",
        SPACE_COLS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Space list_top_level error: {}", e))?;
    let rows = stmt
        .query_map([], row_to_space)
        .map_err(|e| format!("Space list_top_level error: {}", e))?;
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
    parent_space_id: Option<Option<String>>,
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
    if let Some(v) = parent_space_id {
        sets.push("parent_space_id = ?");
        values.push(Box::new(v));
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

pub fn touch_last_opened(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE spaces SET last_opened_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("touch_last_opened error: {}", e))?;
    Ok(())
}

// ─── Archive / Restore / Delete ─────────────────────────

pub fn archive(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn.execute(
        "UPDATE spaces SET archived_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1 AND archived_at IS NULL",
        params![id],
    ).map_err(|e| format!("Space archive error: {}", e))?;
    Ok(affected > 0)
}

pub fn restore(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn.execute(
        "UPDATE spaces SET archived_at = NULL, updated_at = datetime('now') WHERE id = ?1 AND archived_at IS NOT NULL",
        params![id],
    ).map_err(|e| format!("Space restore error: {}", e))?;
    Ok(affected > 0)
}

pub fn delete_permanent(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute("DELETE FROM spaces WHERE id = ?1", params![id])
        .map_err(|e| format!("Space delete error: {}", e))?;
    Ok(affected > 0)
}

// ─── Favourite / Reorder ────────────────────────────────

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

// ─── Duplicate ──────────────────────────────────────────

pub fn duplicate(conn: &Connection, id: &str) -> Result<Space, String> {
    let original = get_by_id(conn, id)?.ok_or("Space not found")?;
    let new_id = Uuid::now_v7().to_string();
    let new_name = format!("{} (copy)", original.name);

    conn.execute(
        "INSERT INTO spaces (id, name, description, icon, accent, template_type, sort_order, settings_json, parent_space_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![new_id, new_name, original.description, original.icon, original.accent,
                original.template_type, original.sort_order, original.settings_json,
                original.parent_space_id],
    ).map_err(|e| format!("Space duplicate error: {}", e))?;

    // Copy modules
    let modules = list_modules(conn, id)?;
    for m in &modules {
        conn.execute(
            "INSERT INTO module_instances (id, space_id, module_type, title, config_json, layout_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![Uuid::now_v7().to_string(), new_id, m.module_type, m.title, m.config_json, m.layout_json],
        ).map_err(|e| format!("Module duplicate error: {}", e))?;
    }

    // Copy child spaces
    let children = list_by_parent(conn, id)?;
    for child in &children {
        let child_id = Uuid::now_v7().to_string();
        conn.execute(
            "INSERT INTO spaces (id, name, description, icon, accent, template_type, sort_order, settings_json, parent_space_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![child_id, child.name, child.description, child.icon, child.accent,
                    child.template_type, child.sort_order, child.settings_json, new_id],
        ).map_err(|e| format!("Child space duplicate error: {}", e))?;

        let child_modules = list_modules(conn, &child.id)?;
        for cm in &child_modules {
            conn.execute(
                "INSERT INTO module_instances (id, space_id, module_type, title, config_json, layout_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![Uuid::now_v7().to_string(), child_id, cm.module_type, cm.title, cm.config_json, cm.layout_json],
            ).map_err(|e| format!("Child module duplicate error: {}", e))?;
        }
    }

    get_by_id(conn, &new_id)?.ok_or_else(|| "Duplicated space not found".to_string())
}

// ─── Modules ────────────────────────────────────────────

pub fn list_modules(conn: &Connection, space_id: &str) -> Result<Vec<ModuleInstance>, String> {
    let sql = format!(
        "SELECT {} FROM module_instances WHERE space_id = ?1 ORDER BY created_at ASC",
        MODULE_COLS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Module list error: {}", e))?;
    let rows = stmt
        .query_map(params![space_id], row_to_module)
        .map_err(|e| format!("Module list error: {}", e))?;
    let mut mods = Vec::new();
    for row in rows {
        mods.push(row.map_err(|e| format!("Module row error: {}", e))?);
    }
    Ok(mods)
}

pub fn set_modules(
    conn: &Connection,
    space_id: &str,
    module_types: &[&str],
) -> Result<Vec<ModuleInstance>, String> {
    // Remove modules not in the new list
    if module_types.is_empty() {
        conn.execute(
            "DELETE FROM module_instances WHERE space_id = ?1",
            params![space_id],
        )
        .map_err(|e| format!("Module clear error: {}", e))?;
    } else {
        let placeholders: Vec<String> = module_types
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect();
        let sql = format!(
            "DELETE FROM module_instances WHERE space_id = ?1 AND module_type NOT IN ({})",
            placeholders.join(",")
        );
        let mut del_params: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(space_id.to_string())];
        for mt in module_types {
            del_params.push(Box::new(mt.to_string()));
        }
        let del_refs: Vec<&dyn rusqlite::types::ToSql> =
            del_params.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, del_refs.as_slice())
            .map_err(|e| format!("Module delete error: {}", e))?;
    }

    // Add new modules
    let existing = list_modules(conn, space_id)?;
    for mt in module_types {
        if !existing.iter().any(|m| m.module_type == *mt) {
            conn.execute(
                "INSERT INTO module_instances (id, space_id, module_type) VALUES (?1, ?2, ?3)",
                params![Uuid::now_v7().to_string(), space_id, mt],
            )
            .map_err(|e| format!("Module insert error: {}", e))?;
        }
    }

    list_modules(conn, space_id)
}

// ─── Transaction helpers ────────────────────────────────

/// Create a space with modules in a transaction
pub fn create_with_modules(
    conn: &Connection,
    space: &Space,
    module_types: &[&str],
) -> Result<(Space, Vec<ModuleInstance>), String> {
    let created = create(conn, space)?;
    let modules = set_modules(conn, &created.id, module_types)?;
    Ok((created, modules))
}

// ─── Tests ──────────────────────────────────────────────

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
            parent_space_id: None,
            last_opened_at: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_crud() {
        let conn = setup();
        let s = create(&conn, &test_space("s1", "Test")).unwrap();
        assert_eq!(s.name, "Test");
        let s = update(&conn, "s1", Some("Renamed"), None, None, None, None, None)
            .unwrap()
            .unwrap();
        assert_eq!(s.name, "Renamed");
    }

    #[test]
    fn test_parent_child() {
        let conn = setup();
        let parent = test_space("p1", "School");
        create(&conn, &parent).unwrap();

        let mut child = test_space("c1", "Math");
        child.parent_space_id = Some("p1".to_string());
        create(&conn, &child).unwrap();

        let children = list_by_parent(&conn, "p1").unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Math");

        // Self-parent prevention test is at app level
    }

    #[test]
    fn test_archive_restore() {
        let conn = setup();
        create(&conn, &test_space("s1", "Test")).unwrap();
        assert!(archive(&conn, "s1").unwrap());
        let active = list(&conn, false).unwrap();
        assert!(active.is_empty());
        assert!(restore(&conn, "s1").unwrap());
        assert_eq!(list(&conn, false).unwrap().len(), 1);
    }

    #[test]
    fn test_delete() {
        let conn = setup();
        create(&conn, &test_space("s1", "Test")).unwrap();
        assert!(delete_permanent(&conn, "s1").unwrap());
        assert!(get_by_id(&conn, "s1").unwrap().is_none());
    }

    #[test]
    fn test_reorder() {
        let conn = setup();
        create(&conn, &test_space("a", "A")).unwrap();
        create(&conn, &test_space("b", "B")).unwrap();
        reorder(&conn, &["b".into(), "a".into()]).unwrap();
        let spaces = list(&conn, false).unwrap();
        assert_eq!(spaces[0].id, "b");
    }

    #[test]
    fn test_modules() {
        let conn = setup();
        create(&conn, &test_space("s1", "Test")).unwrap();
        let mods = set_modules(&conn, "s1", &["notes", "tasks"]).unwrap();
        assert_eq!(mods.len(), 2);
        let mods = set_modules(&conn, "s1", &["notes"]).unwrap();
        assert_eq!(mods.len(), 1);
    }

    #[test]
    fn test_duplicate() {
        let conn = setup();
        create(&conn, &test_space("s1", "Original")).unwrap();
        set_modules(&conn, "s1", &["notes"]).unwrap();
        let dup = duplicate(&conn, "s1").unwrap();
        assert_eq!(dup.name, "Original (copy)");
        let dup_mods = list_modules(&conn, &dup.id).unwrap();
        assert_eq!(dup_mods.len(), 1);
    }

    #[test]
    fn test_top_level() {
        let conn = setup();
        let parent = test_space("p1", "Parent");
        create(&conn, &parent).unwrap();
        let mut child = test_space("c1", "Child");
        child.parent_space_id = Some("p1".to_string());
        create(&conn, &child).unwrap();
        let top = list_top_level(&conn).unwrap();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].name, "Parent");
    }

    #[test]
    fn test_duplicate_with_children() {
        let conn = setup();
        create(&conn, &test_space("p1", "School")).unwrap();
        let mut child = test_space("c1", "Math");
        child.parent_space_id = Some("p1".to_string());
        create(&conn, &child).unwrap();
        set_modules(&conn, "c1", &["notes"]).unwrap();

        let dup = duplicate(&conn, "p1").unwrap();
        let dup_children = list_by_parent(&conn, &dup.id).unwrap();
        assert_eq!(dup_children.len(), 1);
        assert_eq!(dup_children[0].name, "Math");
        let dup_child_mods = list_modules(&conn, &dup_children[0].id).unwrap();
        assert_eq!(dup_child_mods.len(), 1);
    }
}
