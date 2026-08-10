use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConversation {
    pub id: String,
    pub space_id: Option<String>,
    pub title: String,
    pub provider: String,
    pub model: String,
    pub system_context_version: i64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub provider_message_id: Option<String>,
    pub error_code: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContextItem {
    pub id: String,
    pub conversation_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub context_mode: String,
    pub added_at: String,
}

// ─── Conversations ───────────────────────────────────────

pub fn create_conversation(
    conn: &Connection,
    space_id: Option<&str>,
    title: &str,
    provider: &str,
    model: &str,
) -> Result<AiConversation, String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err("Conversation title must contain 1 to 200 characters.".to_string());
    }
    if provider != "deepseek" {
        return Err("Unsupported AI provider.".to_string());
    }
    if !["deepseek-v4-flash", "deepseek-v4-pro"].contains(&model) {
        return Err("Unsupported DeepSeek model.".to_string());
    }
    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO ai_conversations (id, space_id, title, provider, model, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, space_id, title, provider, model, now],
    )
    .map_err(|e| format!("Create conversation error: {}", e))?;

    get_conversation(conn, &id)?.ok_or("Conversation not found after insert".to_string())
}

pub fn get_conversation(conn: &Connection, id: &str) -> Result<Option<AiConversation>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, space_id, title, provider, model, system_context_version,
                    archived_at, created_at, updated_at, last_opened_at
             FROM ai_conversations WHERE id = ?1",
        )
        .map_err(|e| format!("Query error: {}", e))?;

    let result = stmt.query_row(params![id], |row| {
        Ok(AiConversation {
            id: row.get(0)?,
            space_id: row.get(1)?,
            title: row.get(2)?,
            provider: row.get(3)?,
            model: row.get(4)?,
            system_context_version: row.get(5)?,
            archived_at: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_opened_at: row.get(9)?,
        })
    });

    match result {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Get conversation error: {}", e)),
    }
}

pub fn list_conversations(
    conn: &Connection,
    space_id: Option<&str>,
    include_archived: bool,
) -> Result<Vec<AiConversation>, String> {
    let has_space = space_id.is_some();
    let sql = match (has_space, include_archived) {
        (true, true) =>
            "SELECT id, space_id, title, provider, model, system_context_version,
                    archived_at, created_at, updated_at, last_opened_at
             FROM ai_conversations WHERE space_id = ?1 ORDER BY updated_at DESC",
        (false, true) =>
            "SELECT id, space_id, title, provider, model, system_context_version,
                    archived_at, created_at, updated_at, last_opened_at
             FROM ai_conversations ORDER BY updated_at DESC",
        (true, false) =>
            "SELECT id, space_id, title, provider, model, system_context_version,
                    archived_at, created_at, updated_at, last_opened_at
             FROM ai_conversations WHERE space_id = ?1 AND archived_at IS NULL ORDER BY updated_at DESC",
        (false, false) =>
            "SELECT id, space_id, title, provider, model, system_context_version,
                    archived_at, created_at, updated_at, last_opened_at
             FROM ai_conversations WHERE archived_at IS NULL ORDER BY updated_at DESC",
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Query error: {}", e))?;

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<AiConversation> {
        Ok(AiConversation {
            id: row.get(0)?,
            space_id: row.get(1)?,
            title: row.get(2)?,
            provider: row.get(3)?,
            model: row.get(4)?,
            system_context_version: row.get(5)?,
            archived_at: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_opened_at: row.get(9)?,
        })
    };

    let rows: Vec<AiConversation> = if let Some(sid) = space_id {
        stmt.query_map(params![sid], map_row)
            .map_err(|e| format!("Query error: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {}", e))?
    } else {
        stmt.query_map([], map_row)
            .map_err(|e| format!("Query error: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {}", e))?
    };

    Ok(rows)
}

pub fn update_conversation(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    archived: Option<bool>,
) -> Result<Option<AiConversation>, String> {
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(title) = title {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 200 {
            return Err("Conversation title must contain 1 to 200 characters.".to_string());
        }
        conn.execute(
            "UPDATE ai_conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )
        .map_err(|e| format!("Update error: {}", e))?;
    }

    if let Some(archive) = archived {
        if archive {
            conn.execute(
                "UPDATE ai_conversations SET archived_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| format!("Archive error: {}", e))?;
        } else {
            conn.execute(
                "UPDATE ai_conversations SET archived_at = NULL, updated_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| format!("Restore error: {}", e))?;
        }
    }

    // Touch last_opened_at
    conn.execute(
        "UPDATE ai_conversations SET last_opened_at = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| format!("Touch error: {}", e))?;

    get_conversation(conn, id)
}

pub fn delete_conversation(conn: &Connection, id: &str) -> Result<bool, String> {
    let deleted = conn
        .execute("DELETE FROM ai_conversations WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete error: {}", e))?;
    Ok(deleted > 0)
}

// ─── Messages ────────────────────────────────────────────

pub fn add_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
    status: &str,
) -> Result<AiMessage, String> {
    if !["user", "assistant", "system"].contains(&role) {
        return Err("Invalid AI message role.".to_string());
    }
    if !["pending", "streaming", "complete", "error", "cancelled"].contains(&status) {
        return Err("Invalid AI message status.".to_string());
    }
    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO ai_messages (id, conversation_id, role, content, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![id, conversation_id, role, content, status, now],
    )
    .map_err(|e| format!("Add message error: {}", e))?;

    // Update conversation timestamp
    conn.execute(
        "UPDATE ai_conversations SET updated_at = ?1 WHERE id = ?2",
        params![now, conversation_id],
    )
    .map_err(|e| format!("Touch conversation error: {}", e))?;

    get_message(conn, &id)?.ok_or("Message not found after insert".to_string())
}

pub fn get_message(conn: &Connection, id: &str) -> Result<Option<AiMessage>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, status, provider_message_id,
                    error_code, metadata_json, created_at, updated_at
             FROM ai_messages WHERE id = ?1",
        )
        .map_err(|e| format!("Query error: {}", e))?;

    let result = stmt.query_row(params![id], |row| {
        Ok(AiMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            status: row.get(4)?,
            provider_message_id: row.get(5)?,
            error_code: row.get(6)?,
            metadata_json: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    });

    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Get message error: {}", e)),
    }
}

pub fn list_messages(
    conn: &Connection,
    conversation_id: &str,
    limit: Option<i64>,
) -> Result<Vec<AiMessage>, String> {
    let sql = "SELECT id, conversation_id, role, content, status, provider_message_id,
                      error_code, metadata_json, created_at, updated_at
               FROM (
                 SELECT id, conversation_id, role, content, status, provider_message_id,
                        error_code, metadata_json, created_at, updated_at
                 FROM ai_messages WHERE conversation_id = ?1
                 ORDER BY created_at DESC LIMIT ?2
               ) ORDER BY created_at ASC";
    let limit = limit.unwrap_or(200).clamp(1, 500);

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Query error: {}", e))?;

    let rows = stmt
        .query_map(params![conversation_id, limit], |row| {
            Ok(AiMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                status: row.get(4)?,
                provider_message_id: row.get(5)?,
                error_code: row.get(6)?,
                metadata_json: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {}", e))
}

pub fn finish_message(
    conn: &Connection,
    id: &str,
    content: &str,
    status: &str,
    error_code: Option<&str>,
    metadata_json: Option<&str>,
) -> Result<Option<AiMessage>, String> {
    if !["complete", "error", "cancelled"].contains(&status) {
        return Err("Invalid terminal AI message status.".to_string());
    }
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE ai_messages SET content = ?1, status = ?2, error_code = ?3,
         metadata_json = ?4, updated_at = ?5 WHERE id = ?6",
        params![content, status, error_code, metadata_json, now, id],
    )
    .map_err(|e| format!("Finish message error: {}", e))?;
    get_message(conn, id)
}

#[allow(dead_code)] // Reserved for explicit message deletion in the Phase 7 AI UI.
pub fn delete_message(conn: &Connection, id: &str) -> Result<bool, String> {
    let deleted = conn
        .execute("DELETE FROM ai_messages WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete message error: {}", e))?;
    Ok(deleted > 0)
}

// ─── Context Items ───────────────────────────────────────

pub fn add_context_item(
    conn: &Connection,
    conversation_id: &str,
    entity_type: &str,
    entity_id: &str,
    context_mode: &str,
) -> Result<AiContextItem, String> {
    if let Some(existing) = list_context_items(conn, conversation_id)?
        .into_iter()
        .find(|item| item.entity_type == entity_type && item.entity_id == entity_id)
    {
        return Ok(existing);
    }
    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO ai_context_items (id, conversation_id, entity_type, entity_id, context_mode, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, conversation_id, entity_type, entity_id, context_mode, now],
    )
    .map_err(|e| format!("Add context error: {}", e))?;

    Ok(AiContextItem {
        id,
        conversation_id: conversation_id.to_string(),
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        context_mode: context_mode.to_string(),
        added_at: now,
    })
}

pub fn list_context_items(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<AiContextItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, entity_type, entity_id, context_mode, added_at
             FROM ai_context_items
             WHERE conversation_id = ?1
             ORDER BY added_at ASC",
        )
        .map_err(|e| format!("Query error: {}", e))?;

    let rows = stmt
        .query_map(params![conversation_id], |row| {
            Ok(AiContextItem {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                context_mode: row.get(4)?,
                added_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Row error: {}", e))
}

pub fn remove_context_item(conn: &Connection, id: &str) -> Result<bool, String> {
    let deleted = conn
        .execute("DELETE FROM ai_context_items WHERE id = ?1", params![id])
        .map_err(|e| format!("Remove context error: {}", e))?;
    Ok(deleted > 0)
}

pub fn clear_context(conn: &Connection, conversation_id: &str) -> Result<usize, String> {
    conn.execute(
        "DELETE FROM ai_context_items WHERE conversation_id = ?1",
        params![conversation_id],
    )
    .map_err(|e| format!("Clear context error: {}", e))
}

// ─── Tests ───────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS spaces (
                id TEXT PRIMARY KEY, name TEXT NOT NULL,
                favourite INTEGER DEFAULT 0, archived_at TEXT,
                sort_order INTEGER DEFAULT 0, created_at TEXT, updated_at TEXT
            );
            CREATE TABLE ai_conversations (
                id TEXT PRIMARY KEY, space_id TEXT REFERENCES spaces(id) ON DELETE SET NULL,
                title TEXT DEFAULT 'New conversation', provider TEXT DEFAULT 'deepseek',
                model TEXT DEFAULT 'deepseek-chat', system_context_version INTEGER DEFAULT 1,
                archived_at TEXT, created_at TEXT, updated_at TEXT, last_opened_at TEXT
            );
            CREATE TABLE ai_messages (
                id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL, content TEXT DEFAULT '', status TEXT DEFAULT 'complete',
                provider_message_id TEXT, error_code TEXT, metadata_json TEXT,
                created_at TEXT, updated_at TEXT
            );
            CREATE TABLE ai_context_items (
                id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
                entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
                context_mode TEXT DEFAULT 'attached', added_at TEXT
            );
        ").unwrap();
        conn.execute("INSERT INTO spaces (id, name, created_at, updated_at) VALUES ('space1', 'Test', datetime('now'), datetime('now'))", []).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_conversation() {
        let conn = setup_db();
        let conv = create_conversation(
            &conn,
            Some("space1"),
            "Test Chat",
            "deepseek",
            "deepseek-v4-flash",
        )
        .unwrap();
        assert_eq!(conv.title, "Test Chat");
        assert_eq!(conv.space_id, Some("space1".to_string()));

        let fetched = get_conversation(&conn, &conv.id).unwrap().unwrap();
        assert_eq!(fetched.id, conv.id);
        assert!(create_conversation(&conn, None, " ", "deepseek", "deepseek-v4-flash").is_err());
        assert!(create_conversation(&conn, None, "Chat", "deepseek", "deepseek-chat").is_err());
    }

    #[test]
    fn test_list_conversations() {
        let conn = setup_db();
        create_conversation(&conn, None, "Global Chat", "deepseek", "deepseek-v4-flash").unwrap();
        create_conversation(
            &conn,
            Some("space1"),
            "Space Chat",
            "deepseek",
            "deepseek-v4-flash",
        )
        .unwrap();

        let all = list_conversations(&conn, None, false).unwrap();
        assert_eq!(all.len(), 2);

        let space = list_conversations(&conn, Some("space1"), false).unwrap();
        assert_eq!(space.len(), 1);
    }

    #[test]
    fn test_messages() {
        let conn = setup_db();
        let conv =
            create_conversation(&conn, None, "Chat", "deepseek", "deepseek-v4-flash").unwrap();

        let msg1 = add_message(&conn, &conv.id, "user", "Hello", "complete").unwrap();
        assert_eq!(msg1.content, "Hello");

        add_message(&conn, &conv.id, "assistant", "Hi there!", "complete").unwrap();

        let messages = list_messages(&conn, &conv.id, None).unwrap();
        assert_eq!(messages.len(), 2);

        let updated = finish_message(&conn, &msg1.id, "Hello world", "complete", None, None)
            .unwrap()
            .unwrap();
        assert_eq!(updated.content, "Hello world");

        add_message(&conn, &conv.id, "user", "Last", "complete").unwrap();
        let latest = list_messages(&conn, &conv.id, Some(2)).unwrap();
        assert_eq!(latest.len(), 2);
        assert_eq!(latest[1].content, "Last");

        let failed = finish_message(
            &conn,
            &latest[1].id,
            "partial",
            "error",
            Some("network"),
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(failed.status, "error");
        assert_eq!(failed.error_code.as_deref(), Some("network"));
    }

    #[test]
    fn test_archive() {
        let conn = setup_db();
        let conv =
            create_conversation(&conn, None, "Chat", "deepseek", "deepseek-v4-flash").unwrap();

        update_conversation(&conn, &conv.id, None, Some(true)).unwrap();
        let archived = get_conversation(&conn, &conv.id).unwrap().unwrap();
        assert!(archived.archived_at.is_some());

        let active = list_conversations(&conn, None, false).unwrap();
        assert!(active.is_empty());
    }

    #[test]
    fn test_context_items() {
        let conn = setup_db();
        let conv =
            create_conversation(&conn, None, "Chat", "deepseek", "deepseek-v4-flash").unwrap();

        let first = add_context_item(&conn, &conv.id, "note", "note1", "attached").unwrap();
        let duplicate = add_context_item(&conn, &conv.id, "note", "note1", "attached").unwrap();
        assert_eq!(first.id, duplicate.id);
        add_context_item(&conn, &conv.id, "task", "task1", "attached").unwrap();

        let items = list_context_items(&conn, &conv.id).unwrap();
        assert_eq!(items.len(), 2);

        remove_context_item(&conn, &items[0].id).unwrap();
        let remaining = list_context_items(&conn, &conv.id).unwrap();
        assert_eq!(remaining.len(), 1);

        clear_context(&conn, &conv.id).unwrap();
        let empty = list_context_items(&conn, &conv.id).unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_delete_conversation_cascades() {
        let conn = setup_db();
        let conv =
            create_conversation(&conn, None, "Chat", "deepseek", "deepseek-v4-flash").unwrap();
        add_message(&conn, &conv.id, "user", "msg", "complete").unwrap();
        add_context_item(&conn, &conv.id, "note", "note1", "attached").unwrap();

        delete_conversation(&conn, &conv.id).unwrap();

        let msgs = list_messages(&conn, &conv.id, None).unwrap();
        let ctx = list_context_items(&conn, &conv.id).unwrap();
        assert!(msgs.is_empty());
        assert!(ctx.is_empty());
    }
}
