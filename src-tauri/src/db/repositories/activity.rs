use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const MEANINGFUL_EVENT_TYPES: &[&str] = &[
    "space_opened",
    "note_created",
    "note_edited",
    "task_created",
    "task_created_from_ai_proposal",
    "task_completed",
    "task_archived",
    "vault_imported",
    "vault_updated",
    "vault_removed",
    "memory_created",
    "memory_updated",
    "memory_deleted",
    "source_scanned",
    "ai_conversation_used",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub space_id: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub id: String,
    pub event_type: String,
    pub title: String,
    pub detail: Option<String>,
    pub space_id: Option<String>,
    pub space_name: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub destination: String,
    pub created_at: String,
}

pub fn is_meaningful(event_type: &str) -> bool {
    MEANINGFUL_EVENT_TYPES.contains(&event_type)
}

pub fn record(conn: &Connection, event: &ActivityEvent) -> Result<ActivityEvent, String> {
    if !is_meaningful(&event.event_type) {
        return Err(format!("Unsupported Activity event: {}", event.event_type));
    }
    conn.execute(
        "INSERT INTO activity_events (id, event_type, entity_type, entity_id, space_id, metadata_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            event.id,
            event.event_type,
            event.entity_type,
            event.entity_id,
            event.space_id,
            event.metadata_json,
        ],
    )
    .map_err(|e| format!("Activity record error: {e}"))?;

    conn.query_row(
        "SELECT id, event_type, entity_type, entity_id, space_id, metadata_json, created_at
         FROM activity_events WHERE id = ?1",
        [&event.id],
        map_event,
    )
    .map_err(|e| format!("Activity lookup error: {e}"))
}

pub fn record_deduped(
    conn: &Connection,
    event: &ActivityEvent,
    quiet_minutes: u32,
) -> Result<Option<ActivityEvent>, String> {
    if !is_meaningful(&event.event_type) {
        return Err(format!("Unsupported Activity event: {}", event.event_type));
    }
    let duplicate: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM activity_events
                WHERE event_type = ?1
                  AND entity_type IS ?2
                  AND entity_id IS ?3
                  AND space_id IS ?4
                  AND created_at >= datetime('now', '-' || ?5 || ' minutes')
            )",
            params![
                event.event_type,
                event.entity_type,
                event.entity_id,
                event.space_id,
                quiet_minutes,
            ],
            |row| row.get(0),
        )
        .map_err(|e| format!("Activity deduplication error: {e}"))?;
    if duplicate {
        return Ok(None);
    }
    record(conn, event).map(Some)
}

fn map_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<ActivityEvent> {
    Ok(ActivityEvent {
        id: row.get(0)?,
        event_type: row.get(1)?,
        entity_type: row.get(2)?,
        entity_id: row.get(3)?,
        space_id: row.get(4)?,
        metadata_json: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn list_events(
    conn: &Connection,
    space_id: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<ActivityEvent>, String> {
    let limit = limit.unwrap_or(50).clamp(1, 100);
    let allowed = MEANINGFUL_EVENT_TYPES
        .iter()
        .map(|value| format!("'{value}'"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = if space_id.is_some() {
        format!(
            "SELECT id, event_type, entity_type, entity_id, space_id, metadata_json, created_at
             FROM activity_events WHERE space_id = ?1 AND event_type IN ({allowed})
             ORDER BY created_at DESC, id DESC LIMIT ?2"
        )
    } else {
        format!(
            "SELECT id, event_type, entity_type, entity_id, space_id, metadata_json, created_at
             FROM activity_events WHERE event_type IN ({allowed})
             ORDER BY created_at DESC, id DESC LIMIT ?1"
        )
    };
    let mut statement = conn
        .prepare(&sql)
        .map_err(|e| format!("Activity list error: {e}"))?;
    let rows = if let Some(space_id) = space_id {
        statement.query_map(params![space_id, limit], map_event)
    } else {
        statement.query_map([limit], map_event)
    }
    .map_err(|e| format!("Activity list error: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Activity row error: {e}"))
}

pub fn list_items(
    conn: &Connection,
    space_id: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<ActivityItem>, String> {
    let events = list_events(conn, space_id, limit)?;
    events
        .into_iter()
        .map(|event| to_item(conn, event))
        .collect()
}

fn entity_title(
    conn: &Connection,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
) -> Result<Option<String>, String> {
    let Some(id) = entity_id else { return Ok(None) };
    let sql = match entity_type {
        Some("space") => "SELECT name FROM spaces WHERE id = ?1",
        Some("note") => "SELECT title FROM notes WHERE id = ?1",
        Some("task") => "SELECT title FROM tasks WHERE id = ?1",
        Some("vault") => "SELECT display_title FROM vault_items WHERE id = ?1",
        Some("memory") => "SELECT title FROM memory_items WHERE id = ?1",
        Some("source") => "SELECT display_name FROM sources WHERE id = ?1",
        Some("conversation") => "SELECT title FROM ai_conversations WHERE id = ?1",
        _ => return Ok(None),
    };
    conn.query_row(sql, [id], |row| row.get(0))
        .optional()
        .map_err(|e| format!("Activity entity lookup error: {e}"))
}

fn destination(conn: &Connection, event: &ActivityEvent) -> Result<String, String> {
    let Some(space_id) = event.space_id.as_deref() else {
        return Ok(match event.entity_type.as_deref() {
            Some("task") => "/tasks",
            Some("vault") => "/vault",
            Some("memory") => "/memory",
            Some("conversation") => "/ai",
            Some("source") => "/sources",
            Some("space") | Some("note") => "/spaces",
            _ => "/activity",
        }
        .to_string());
    };
    let root = format!("/spaces/{space_id}");
    let (module, suffix, fallback) = match event.entity_type.as_deref() {
        Some("note") => (Some("notes"), "/notes", root.clone()),
        Some("task") => (Some("tasks"), "/tasks", "/tasks".to_string()),
        Some("vault") => (Some("files"), "/files", "/vault".to_string()),
        Some("memory") => (Some("memory"), "/memory", "/memory".to_string()),
        Some("conversation") => (Some("ai"), "/ai", "/ai".to_string()),
        Some("source") => return Ok("/sources".to_string()),
        _ => return Ok(root),
    };
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM module_instances WHERE space_id = ?1 AND module_type = ?2)",
            params![space_id, module],
            |row| row.get(0),
        )
        .map_err(|e| format!("Activity module lookup error: {e}"))?;
    Ok(if exists {
        format!("{root}{suffix}")
    } else {
        fallback
    })
}

fn to_item(conn: &Connection, event: ActivityEvent) -> Result<ActivityItem, String> {
    let entity = entity_title(
        conn,
        event.entity_type.as_deref(),
        event.entity_id.as_deref(),
    )?;
    let space_name = event
        .space_id
        .as_deref()
        .map(|id| {
            conn.query_row("SELECT name FROM spaces WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| format!("Activity Space lookup error: {e}"))
        })
        .transpose()?
        .flatten();
    let noun = entity.as_deref().unwrap_or("an item");
    let title = match event.event_type.as_str() {
        "space_opened" => format!("Opened {}", entity.as_deref().unwrap_or("a Space")),
        "note_created" => format!("Created note {noun}"),
        "note_edited" => format!("Edited note {noun}"),
        "task_created" => format!("Created task {noun}"),
        "task_created_from_ai_proposal" => format!("Created confirmed AI task {noun}"),
        "task_completed" => format!("Completed task {noun}"),
        "task_archived" => format!("Archived task {noun}"),
        "vault_imported" => format!("Added Vault file {noun}"),
        "vault_updated" => format!("Updated Vault file {noun}"),
        "vault_removed" => "Removed a Vault file".to_string(),
        "memory_created" => format!("Added Memory {noun}"),
        "memory_updated" => format!("Updated Memory {noun}"),
        "memory_deleted" => "Deleted a Memory item".to_string(),
        "source_scanned" => format!("Indexed changes from {noun}"),
        "ai_conversation_used" => format!("Used AI conversation {noun}"),
        _ => "Updated the workspace".to_string(),
    };
    let detail = match event.event_type.as_str() {
        "source_scanned" => Some("Local file metadata changed".to_string()),
        "task_created_from_ai_proposal" => Some("Created after your confirmation".to_string()),
        _ => None,
    };
    Ok(ActivityItem {
        id: event.id.clone(),
        event_type: event.event_type.clone(),
        title,
        detail,
        space_id: event.space_id.clone(),
        space_name,
        entity_type: event.entity_type.clone(),
        entity_id: event.entity_id.clone(),
        destination: destination(conn, &event)?,
        created_at: event.created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use uuid::Uuid;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        conn
    }

    fn test_event(event_type: &str) -> ActivityEvent {
        ActivityEvent {
            id: Uuid::now_v7().to_string(),
            event_type: event_type.to_string(),
            entity_type: None,
            entity_id: None,
            space_id: None,
            metadata_json: None,
            created_at: String::new(),
        }
    }

    #[test]
    fn accepts_only_curated_events_and_applies_limits() {
        let conn = setup();
        record(&conn, &test_event("task_created")).unwrap();
        assert!(record(&conn, &test_event("setting_changed")).is_err());
        record(&conn, &test_event("task_completed")).unwrap();
        assert_eq!(list_events(&conn, None, Some(1)).unwrap().len(), 1);
    }

    #[test]
    fn deduplicates_quiet_events_by_entity_and_space() {
        let conn = setup();
        conn.execute("INSERT INTO spaces (id, name) VALUES ('s1', 'Alpha')", [])
            .unwrap();
        let mut event = test_event("space_opened");
        event.entity_type = Some("space".to_string());
        event.entity_id = Some("s1".to_string());
        event.space_id = Some("s1".to_string());
        assert!(record_deduped(&conn, &event, 30).unwrap().is_some());
        event.id = Uuid::now_v7().to_string();
        assert!(record_deduped(&conn, &event, 30).unwrap().is_none());
    }

    #[test]
    fn destinations_fall_back_when_a_space_module_is_unavailable() {
        let conn = setup();
        conn.execute("INSERT INTO spaces (id, name) VALUES ('s1', 'Alpha')", [])
            .unwrap();
        let mut event = test_event("note_edited");
        event.entity_type = Some("note".to_string());
        event.entity_id = Some("n1".to_string());
        event.space_id = Some("s1".to_string());
        record(&conn, &event).unwrap();
        assert_eq!(
            list_items(&conn, Some("s1"), Some(1)).unwrap()[0].destination,
            "/spaces/s1"
        );
        conn.execute(
            "INSERT INTO module_instances (id,space_id,module_type) VALUES ('m1','s1','notes')",
            [],
        )
        .unwrap();
        assert_eq!(
            list_items(&conn, Some("s1"), Some(1)).unwrap()[0].destination,
            "/spaces/s1/notes"
        );
    }
}
