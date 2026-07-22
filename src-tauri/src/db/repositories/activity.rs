use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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

pub fn record(conn: &Connection, event: &ActivityEvent) -> Result<ActivityEvent, String> {
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
    .map_err(|e| format!("Activity record error: {}", e))?;

    // Return the event with created_at populated
    list_recent(conn, Some(1))?
        .into_iter()
        .next()
        .ok_or_else(|| "Activity not found after insert".to_string())
}

pub fn list_recent(conn: &Connection, limit: Option<u32>) -> Result<Vec<ActivityEvent>, String> {
    let limit = limit.unwrap_or(50);
    let mut stmt = conn
        .prepare(
            "SELECT id, event_type, entity_type, entity_id, space_id, metadata_json, created_at
             FROM activity_events
             ORDER BY created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("Activity list error: {}", e))?;

    let rows = stmt
        .query_map([limit], |row| {
            Ok(ActivityEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                space_id: row.get(4)?,
                metadata_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Activity list error: {}", e))?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| format!("Activity row error: {}", e))?);
    }
    Ok(events)
}

pub fn list_by_space(
    conn: &Connection,
    space_id: &str,
    limit: Option<u32>,
) -> Result<Vec<ActivityEvent>, String> {
    let limit = limit.unwrap_or(50);
    let mut stmt = conn
        .prepare(
            "SELECT id, event_type, entity_type, entity_id, space_id, metadata_json, created_at
             FROM activity_events
             WHERE space_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("Activity list_by_space error: {}", e))?;

    let rows = stmt
        .query_map(params![space_id, limit], |row| {
            Ok(ActivityEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                space_id: row.get(4)?,
                metadata_json: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Activity list_by_space error: {}", e))?;

    let mut events = Vec::new();
    for row in rows {
        events.push(row.map_err(|e| format!("Activity row error: {}", e))?);
    }
    Ok(events)
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
    fn test_record_and_list() {
        let conn = setup();

        record(&conn, &test_event("space_created")).unwrap();
        record(&conn, &test_event("setting_changed")).unwrap();
        record(&conn, &test_event("space_created")).unwrap();

        let events = list_recent(&conn, None).unwrap();
        assert_eq!(events.len(), 3);

        let limited = list_recent(&conn, Some(2)).unwrap();
        assert_eq!(limited.len(), 2);
    }
}
