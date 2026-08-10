use rusqlite::{params, params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MEMORY_COLS: &str =
    "id, space_id, title, content, reason, category, source, created_at, updated_at";
const CATEGORIES: &[&str] = &[
    "preference",
    "decision",
    "recurring_context",
    "terminology",
    "goal",
    "constraint",
];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MemoryItem {
    pub id: String,
    pub space_id: Option<String>,
    pub title: String,
    pub content: String,
    pub reason: String,
    pub category: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInput {
    pub space_id: Option<String>,
    pub title: String,
    pub content: String,
    pub reason: String,
    pub category: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFilter {
    pub space_id: Option<String>,
    #[serde(default)]
    pub global_only: bool,
    pub category: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u32>,
}

fn row_to_memory(row: &rusqlite::Row) -> rusqlite::Result<MemoryItem> {
    Ok(MemoryItem {
        id: row.get(0)?,
        space_id: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        reason: row.get(4)?,
        category: row.get(5)?,
        source: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn validate(conn: &Connection, input: &MemoryInput) -> Result<(String, String, String), String> {
    let title = input.title.trim();
    let content = input.content.trim();
    let reason = input.reason.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err("Memory title must contain 1 to 200 characters".to_string());
    }
    if content.is_empty() || content.chars().count() > 20_000 {
        return Err("Memory content must contain 1 to 20000 characters".to_string());
    }
    if reason.is_empty() || reason.chars().count() > 500 {
        return Err("Memory reason must contain 1 to 500 characters".to_string());
    }
    if !CATEGORIES.contains(&input.category.as_str()) {
        return Err("Invalid Memory category".to_string());
    }
    if let Some(space_id) = input.space_id.as_deref() {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM spaces WHERE id = ?1 AND archived_at IS NULL)",
                params![space_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("Memory Space validation error: {}", error))?;
        if !exists {
            return Err("Memory Space does not exist or is archived".to_string());
        }
    }
    Ok((title.to_string(), content.to_string(), reason.to_string()))
}

pub fn create(conn: &Connection, input: &MemoryInput) -> Result<MemoryItem, String> {
    let (title, content, reason) = validate(conn, input)?;
    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO memory_items (id, space_id, title, content, reason, category, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'user')",
        params![id, input.space_id, title, content, reason, input.category],
    )
    .map_err(|error| format!("Memory create error: {}", error))?;
    get_by_id(conn, &id)?.ok_or_else(|| "Memory not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<MemoryItem>, String> {
    let sql = format!("SELECT {} FROM memory_items WHERE id = ?1", MEMORY_COLS);
    match conn.query_row(&sql, params![id], row_to_memory) {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Memory get error: {}", error)),
    }
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &MemoryInput,
) -> Result<Option<MemoryItem>, String> {
    if get_by_id(conn, id)?.is_none() {
        return Ok(None);
    }
    let (title, content, reason) = validate(conn, input)?;
    conn.execute(
        "UPDATE memory_items SET space_id = ?1, title = ?2, content = ?3, reason = ?4,
         category = ?5, updated_at = datetime('now') WHERE id = ?6",
        params![input.space_id, title, content, reason, input.category, id],
    )
    .map_err(|error| format!("Memory update error: {}", error))?;
    get_by_id(conn, id)
}

pub fn list(conn: &Connection, filter: &MemoryFilter) -> Result<Vec<MemoryItem>, String> {
    if filter.global_only && filter.space_id.is_some() {
        return Err("Memory filter cannot combine a Space with global-only".to_string());
    }
    if let Some(category) = filter.category.as_deref() {
        if !CATEGORIES.contains(&category) {
            return Err("Invalid Memory category filter".to_string());
        }
    }
    let mut conditions = Vec::new();
    let mut values = Vec::<Value>::new();
    if let Some(space_id) = filter.space_id.as_deref() {
        values.push(Value::Text(space_id.to_string()));
        conditions.push(format!("space_id = ?{}", values.len()));
    } else if filter.global_only {
        conditions.push("space_id IS NULL".to_string());
    }
    if let Some(category) = filter.category.as_deref() {
        values.push(Value::Text(category.to_string()));
        conditions.push(format!("category = ?{}", values.len()));
    }
    if let Some(search) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        values.push(Value::Text(format!("%{}%", search)));
        let index = values.len();
        conditions.push(format!(
            "(title LIKE ?{0} OR content LIKE ?{0} OR reason LIKE ?{0})",
            index
        ));
    }
    values.push(Value::Integer(i64::from(
        filter.limit.unwrap_or(500).clamp(1, 500),
    )));
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT {} FROM memory_items{} ORDER BY updated_at DESC LIMIT ?{}",
        MEMORY_COLS,
        where_clause,
        values.len()
    );
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Memory list error: {}", error))?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), row_to_memory)
        .map_err(|error| format!("Memory list error: {}", error))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Memory row error: {}", error))
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.execute("DELETE FROM memory_items WHERE id = ?1", params![id])
        .map(|count| count > 0)
        .map_err(|error| format!("Memory delete error: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        conn.execute("INSERT INTO spaces (id, name) VALUES ('space-a', 'A')", [])
            .unwrap();
        conn
    }

    fn input(space_id: Option<&str>) -> MemoryInput {
        MemoryInput {
            space_id: space_id.map(str::to_string),
            title: "Writing style".into(),
            content: "Prefer concise answers".into(),
            reason: "Keeps collaboration efficient".into(),
            category: "preference".into(),
        }
    }

    #[test]
    fn crud_filter_and_search() {
        let conn = setup();
        let global = create(&conn, &input(None)).unwrap();
        let scoped = create(&conn, &input(Some("space-a"))).unwrap();
        assert_eq!(
            list(
                &conn,
                &MemoryFilter {
                    global_only: true,
                    ..Default::default()
                }
            )
            .unwrap(),
            vec![global.clone()]
        );
        assert_eq!(
            list(
                &conn,
                &MemoryFilter {
                    space_id: Some("space-a".into()),
                    search: Some("concise".into()),
                    ..Default::default()
                }
            )
            .unwrap(),
            vec![scoped.clone()]
        );
        let mut changed = input(Some("space-a"));
        changed.title = "Updated".into();
        assert_eq!(
            update(&conn, &scoped.id, &changed).unwrap().unwrap().title,
            "Updated"
        );
        assert!(delete(&conn, &global.id).unwrap());
    }

    #[test]
    fn validates_and_cascades_space_scope() {
        let conn = setup();
        let item = create(&conn, &input(Some("space-a"))).unwrap();
        conn.execute(
            "INSERT INTO ai_conversations (id, space_id) VALUES ('chat-a', 'space-a')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO ai_context_items (id, conversation_id, entity_type, entity_id) VALUES ('context-a', 'chat-a', 'memory', ?1)", params![item.id]).unwrap();
        let mut invalid = input(None);
        invalid.category = "secret".into();
        assert!(create(&conn, &invalid).is_err());
        conn.execute("DELETE FROM spaces WHERE id = 'space-a'", [])
            .unwrap();
        assert!(get_by_id(&conn, &item.id).unwrap().is_none());
        let attachments: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ai_context_items WHERE entity_type = 'memory'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(attachments, 0);
    }
}
