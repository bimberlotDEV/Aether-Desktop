use rusqlite::{params, params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const VAULT_COLS: &str = "id, space_id, storage_mode, display_title, original_name, stored_path, media_type, size_bytes, tags_json, created_at, updated_at";
const STORAGE_MODES: &[&str] = &["linked", "managed"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultItem {
    pub id: String,
    pub space_id: Option<String>,
    pub storage_mode: String,
    pub display_title: String,
    pub original_name: String,
    pub stored_path: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct NewVaultItem {
    pub id: String,
    pub space_id: Option<String>,
    pub storage_mode: String,
    pub display_title: String,
    pub original_name: String,
    pub stored_path: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultUpdateInput {
    pub space_id: Option<String>,
    pub display_title: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultFilter {
    pub space_id: Option<String>,
    #[serde(default)]
    pub unassigned_only: bool,
    pub storage_mode: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u32>,
}

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<VaultItem> {
    let tags_json: String = row.get(8)?;
    let tags = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(VaultItem {
        id: row.get(0)?,
        space_id: row.get(1)?,
        storage_mode: row.get(2)?,
        display_title: row.get(3)?,
        original_name: row.get(4)?,
        stored_path: row.get(5)?,
        media_type: row.get(6)?,
        size_bytes: row.get(7)?,
        tags,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn normalize_tags(tags: &[String]) -> Result<Vec<String>, String> {
    if tags.len() > 20 {
        return Err("A Vault item can have at most 20 tags".to_string());
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for tag in tags {
        let value = tag.trim();
        if value.is_empty() {
            return Err("Vault tags cannot be empty".to_string());
        }
        if value.chars().count() > 40 {
            return Err("Vault tags cannot exceed 40 characters".to_string());
        }
        if seen.insert(value.to_lowercase()) {
            normalized.push(value.to_string());
        }
    }
    Ok(normalized)
}

fn validate_space(conn: &Connection, space_id: Option<&str>) -> Result<(), String> {
    if let Some(space_id) = space_id {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM spaces WHERE id = ?1 AND archived_at IS NULL)",
                [space_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("Vault Space validation error: {}", error))?;
        if !exists {
            return Err("Vault Space does not exist or is archived".to_string());
        }
    }
    Ok(())
}

fn validate_title(value: &str) -> Result<&str, String> {
    let title = value.trim();
    if title.is_empty() {
        return Err("Vault display title is required".to_string());
    }
    if title.chars().count() > 200 {
        return Err("Vault display title cannot exceed 200 characters".to_string());
    }
    Ok(title)
}

pub fn create(conn: &Connection, input: &NewVaultItem) -> Result<VaultItem, String> {
    if !STORAGE_MODES.contains(&input.storage_mode.as_str()) {
        return Err(format!(
            "Invalid Vault storage mode: {}",
            input.storage_mode
        ));
    }
    let title = validate_title(&input.display_title)?;
    let original_name = input.original_name.trim();
    if original_name.is_empty() || original_name.chars().count() > 255 {
        return Err("Vault original filename must contain 1 to 255 characters".to_string());
    }
    if input.stored_path.trim().is_empty() {
        return Err("Vault stored path is required".to_string());
    }
    if input.size_bytes < 0 {
        return Err("Vault file size cannot be negative".to_string());
    }
    validate_space(conn, input.space_id.as_deref())?;
    let tags_json = serde_json::to_string(&normalize_tags(&input.tags)?)
        .map_err(|error| format!("Vault tag serialization error: {}", error))?;
    conn.execute(
        "INSERT INTO vault_items
         (id, space_id, storage_mode, display_title, original_name, stored_path, media_type, size_bytes, tags_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            input.id,
            input.space_id,
            input.storage_mode,
            title,
            original_name,
            input.stored_path,
            input.media_type,
            input.size_bytes,
            tags_json,
        ],
    )
    .map_err(|error| format!("Vault create error: {}", error))?;
    get_by_id(conn, &input.id)?.ok_or_else(|| "Vault item not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<VaultItem>, String> {
    let sql = format!("SELECT {} FROM vault_items WHERE id = ?1", VAULT_COLS);
    match conn.query_row(&sql, [id], row_to_item) {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Vault get error: {}", error)),
    }
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &VaultUpdateInput,
) -> Result<Option<VaultItem>, String> {
    let title = validate_title(&input.display_title)?;
    validate_space(conn, input.space_id.as_deref())?;
    let tags_json = serde_json::to_string(&normalize_tags(&input.tags)?)
        .map_err(|error| format!("Vault tag serialization error: {}", error))?;
    conn.execute(
        "UPDATE vault_items SET space_id = ?1, display_title = ?2, tags_json = ?3,
         updated_at = datetime('now') WHERE id = ?4",
        params![input.space_id, title, tags_json, id],
    )
    .map_err(|error| format!("Vault update error: {}", error))?;
    get_by_id(conn, id)
}

pub fn list(conn: &Connection, filter: &VaultFilter) -> Result<Vec<VaultItem>, String> {
    if filter.unassigned_only && filter.space_id.is_some() {
        return Err("Vault filter cannot combine a Space with unassigned-only".to_string());
    }
    if let Some(mode) = filter.storage_mode.as_deref() {
        if !STORAGE_MODES.contains(&mode) {
            return Err(format!("Invalid Vault storage mode filter: {}", mode));
        }
    }
    let mut conditions = Vec::new();
    let mut values = Vec::<Value>::new();
    if let Some(space_id) = filter.space_id.as_deref() {
        values.push(Value::Text(space_id.to_string()));
        conditions.push(format!("space_id = ?{}", values.len()));
    } else if filter.unassigned_only {
        conditions.push("space_id IS NULL".to_string());
    }
    if let Some(mode) = filter.storage_mode.as_deref() {
        values.push(Value::Text(mode.to_string()));
        conditions.push(format!("storage_mode = ?{}", values.len()));
    }
    if let Some(search) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        values.push(Value::Text(format!("%{}%", search)));
        let index = values.len();
        conditions.push(format!(
            "(display_title LIKE ?{0} OR original_name LIKE ?{0} OR media_type LIKE ?{0} OR tags_json LIKE ?{0})",
            index
        ));
    }
    let limit = filter.limit.unwrap_or(500).clamp(1, 500);
    values.push(Value::Integer(i64::from(limit)));
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT {} FROM vault_items{} ORDER BY updated_at DESC, created_at DESC LIMIT ?{}",
        VAULT_COLS,
        where_clause,
        values.len()
    );
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Vault list error: {}", error))?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), row_to_item)
        .map_err(|error| format!("Vault list error: {}", error))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Vault row error: {}", error))
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.execute("DELETE FROM vault_items WHERE id = ?1", [id])
        .map(|affected| affected > 0)
        .map_err(|error| format!("Vault delete error: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        conn.execute(
            "INSERT INTO spaces (id, name) VALUES ('space-1', 'Research')",
            [],
        )
        .unwrap();
        conn
    }

    fn input(id: &str, mode: &str, title: &str) -> NewVaultItem {
        NewVaultItem {
            id: id.to_string(),
            space_id: None,
            storage_mode: mode.to_string(),
            display_title: title.to_string(),
            original_name: "source.md".to_string(),
            stored_path: format!("{}-source.md", id),
            media_type: "text/markdown".to_string(),
            size_bytes: 42,
            tags: vec!["Research".to_string(), "research".to_string()],
        }
    }

    #[test]
    fn create_update_and_delete() {
        let conn = setup();
        let item = create(&conn, &input("vault-1", "linked", "Source")).unwrap();
        assert_eq!(item.tags, vec!["Research"]);
        let updated = update(
            &conn,
            &item.id,
            &VaultUpdateInput {
                space_id: Some("space-1".to_string()),
                display_title: "Updated source".to_string(),
                tags: vec!["reading".to_string()],
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(updated.space_id.as_deref(), Some("space-1"));
        assert_eq!(updated.display_title, "Updated source");
        assert!(delete(&conn, &item.id).unwrap());
        assert!(get_by_id(&conn, &item.id).unwrap().is_none());
    }

    #[test]
    fn filters_and_search() {
        let conn = setup();
        let mut linked = input("linked", "linked", "Project brief");
        linked.tags = vec!["alpha".to_string()];
        create(&conn, &linked).unwrap();
        let mut managed = input("managed", "managed", "Reference image");
        managed.space_id = Some("space-1".to_string());
        managed.original_name = "diagram.png".to_string();
        managed.stored_path = "items/managed/diagram.png".to_string();
        managed.media_type = "image/png".to_string();
        create(&conn, &managed).unwrap();

        assert_eq!(
            list(
                &conn,
                &VaultFilter {
                    storage_mode: Some("managed".to_string()),
                    ..Default::default()
                }
            )
            .unwrap()
            .len(),
            1
        );
        assert_eq!(
            list(
                &conn,
                &VaultFilter {
                    search: Some("diagram".to_string()),
                    ..Default::default()
                }
            )
            .unwrap()[0]
                .id,
            "managed"
        );
        assert_eq!(
            list(
                &conn,
                &VaultFilter {
                    unassigned_only: true,
                    ..Default::default()
                }
            )
            .unwrap()[0]
                .id,
            "linked"
        );
    }

    #[test]
    fn rejects_invalid_metadata() {
        let conn = setup();
        assert!(create(&conn, &input("bad", "unknown", "Title")).is_err());
        assert!(create(&conn, &input("bad", "linked", "   ")).is_err());
        let mut missing_space = input("bad", "linked", "Title");
        missing_space.space_id = Some("missing".to_string());
        assert!(create(&conn, &missing_space).is_err());
    }
}
