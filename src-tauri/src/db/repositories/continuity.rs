use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::activity::{self, ActivityItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityItem {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub updated_at: String,
    pub destination: String,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuitySuggestion {
    pub title: String,
    pub detail: String,
    pub destination: String,
    pub source_type: String,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceContinuity {
    pub space_id: String,
    pub space_name: String,
    pub last_worked_at: String,
    pub recent_notes: Vec<ContinuityItem>,
    pub open_tasks: Vec<ContinuityItem>,
    pub recent_files: Vec<ContinuityItem>,
    pub latest_conversation: Option<ContinuityItem>,
    pub recent_activity: Vec<ActivityItem>,
    pub suggested_next_step: ContinuitySuggestion,
}

pub fn get(conn: &Connection, space_id: &str) -> Result<SpaceContinuity, String> {
    let space: Option<(String, String)> = conn
        .query_row(
            "SELECT name, updated_at FROM spaces WHERE id = ?1 AND archived_at IS NULL",
            [space_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| format!("Continuity Space lookup error: {e}"))?;
    let Some((space_name, space_updated_at)) = space else {
        return Err("Active Space not found".to_string());
    };

    let root = format!("/spaces/{space_id}");
    let note_destination = module_destination(conn, space_id, "notes", &root)?;
    let task_destination = module_destination(conn, space_id, "tasks", "/tasks")?;
    let ai_destination = module_destination(conn, space_id, "ai", "/ai")?;
    let recent_notes = list_notes(conn, space_id, &note_destination)?;
    let open_tasks = list_tasks(conn, space_id, &task_destination)?;
    let recent_files = list_files(conn, space_id)?;
    let latest_conversation = latest_conversation(conn, space_id, &ai_destination)?;
    let recent_activity = activity::list_items(conn, Some(space_id), Some(6))?;
    let last_worked_at = conn
        .query_row(
            "SELECT COALESCE(MAX(worked_at), ?2) FROM (
                SELECT updated_at AS worked_at FROM notes WHERE space_id = ?1 AND archived_at IS NULL
                UNION ALL SELECT updated_at FROM tasks WHERE space_id = ?1 AND archived_at IS NULL
                UNION ALL SELECT f.updated_at FROM indexed_files f JOIN sources s ON s.id = f.source_id WHERE s.space_id = ?1 AND f.state = 'present'
                UNION ALL SELECT updated_at FROM ai_conversations WHERE space_id = ?1 AND archived_at IS NULL
                UNION ALL SELECT created_at FROM activity_events WHERE space_id = ?1 AND event_type <> 'space_opened'
            )",
            params![space_id, space_updated_at],
            |row| row.get(0),
        )
        .map_err(|e| format!("Continuity timestamp error: {e}"))?;
    let suggested_next_step = suggestion(
        &open_tasks,
        &recent_notes,
        latest_conversation.as_ref(),
        &recent_files,
    );

    Ok(SpaceContinuity {
        space_id: space_id.to_string(),
        space_name,
        last_worked_at,
        recent_notes,
        open_tasks,
        recent_files,
        latest_conversation,
        recent_activity,
        suggested_next_step,
    })
}

fn module_destination(
    conn: &Connection,
    space_id: &str,
    module: &str,
    fallback: &str,
) -> Result<String, String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM module_instances WHERE space_id = ?1 AND module_type = ?2)",
            params![space_id, module],
            |row| row.get(0),
        )
        .map_err(|e| format!("Continuity module lookup error: {e}"))?;
    Ok(if exists {
        format!("/spaces/{space_id}/{module}")
    } else {
        fallback.to_string()
    })
}

fn list_notes(
    conn: &Connection,
    space_id: &str,
    destination: &str,
) -> Result<Vec<ContinuityItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, title, excerpt, updated_at FROM notes
             WHERE space_id = ?1 AND archived_at IS NULL
             ORDER BY COALESCE(last_opened_at, updated_at) DESC, updated_at DESC, id DESC LIMIT 3",
        )
        .map_err(|e| format!("Continuity Note query error: {e}"))?;
    let rows = statement
        .query_map([space_id], |row| {
            let id: String = row.get(0)?;
            Ok(ContinuityItem {
                id,
                title: row.get(1)?,
                detail: row.get::<_, String>(2)?,
                updated_at: row.get(3)?,
                destination: destination.to_string(),
                provenance: "Note".to_string(),
            })
        })
        .map_err(|e| format!("Continuity Note query error: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Continuity Note row error: {e}"))
}

fn list_tasks(
    conn: &Connection,
    space_id: &str,
    destination: &str,
) -> Result<Vec<ContinuityItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, title, priority, due_date, updated_at FROM tasks
             WHERE space_id = ?1 AND archived_at IS NULL AND status <> 'done'
             ORDER BY CASE priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,
                      CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date, updated_at DESC, id DESC LIMIT 4",
        )
        .map_err(|e| format!("Continuity Task query error: {e}"))?;
    let rows = statement
        .query_map([space_id], |row| {
            let priority: String = row.get(2)?;
            let due_date: Option<String> = row.get(3)?;
            let detail = due_date
                .map(|date| format!("{priority} priority · due {date}"))
                .unwrap_or_else(|| format!("{priority} priority"));
            Ok(ContinuityItem {
                id: row.get(0)?,
                title: row.get(1)?,
                detail,
                updated_at: row.get(4)?,
                destination: destination.to_string(),
                provenance: "Task".to_string(),
            })
        })
        .map_err(|e| format!("Continuity Task query error: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Continuity Task row error: {e}"))
}

fn list_files(conn: &Connection, space_id: &str) -> Result<Vec<ContinuityItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT f.id, f.filename, f.relative_path, s.display_name, f.updated_at
             FROM indexed_files f JOIN sources s ON s.id = f.source_id
             WHERE s.space_id = ?1 AND f.state = 'present'
             ORDER BY COALESCE(f.modified_at_fs, 0) DESC, f.updated_at DESC, f.id DESC LIMIT 4",
        )
        .map_err(|e| format!("Continuity Source-file query error: {e}"))?;
    let rows = statement
        .query_map([space_id], |row| {
            let relative_path: String = row.get(2)?;
            let source_name: String = row.get(3)?;
            Ok(ContinuityItem {
                id: row.get(0)?,
                title: row.get(1)?,
                detail: format!("{source_name} · {relative_path}"),
                updated_at: row.get(4)?,
                destination: "/sources".to_string(),
                provenance: format!("Source · {source_name}"),
            })
        })
        .map_err(|e| format!("Continuity Source-file query error: {e}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Continuity Source-file row error: {e}"))
}

fn latest_conversation(
    conn: &Connection,
    space_id: &str,
    destination: &str,
) -> Result<Option<ContinuityItem>, String> {
    conn.query_row(
        "SELECT id, title, updated_at FROM ai_conversations
         WHERE space_id = ?1 AND archived_at IS NULL
         ORDER BY COALESCE(last_opened_at, updated_at) DESC, updated_at DESC, id DESC LIMIT 1",
        [space_id],
        |row| {
            Ok(ContinuityItem {
                id: row.get(0)?,
                title: row.get(1)?,
                detail: "Most recently used conversation".to_string(),
                updated_at: row.get(2)?,
                destination: destination.to_string(),
                provenance: "AI conversation".to_string(),
            })
        },
    )
    .optional()
    .map_err(|e| format!("Continuity AI query error: {e}"))
}

fn suggestion(
    tasks: &[ContinuityItem],
    notes: &[ContinuityItem],
    conversation: Option<&ContinuityItem>,
    files: &[ContinuityItem],
) -> ContinuitySuggestion {
    if let Some(task) = tasks.first() {
        return ContinuitySuggestion {
            title: format!("Continue task: {}", task.title),
            detail: task.detail.clone(),
            destination: task.destination.clone(),
            source_type: "task".to_string(),
            source_id: Some(task.id.clone()),
        };
    }
    if let Some(note) = notes.first() {
        return ContinuitySuggestion {
            title: format!("Return to note: {}", note.title),
            detail: "Your most recently active Note in this Space".to_string(),
            destination: note.destination.clone(),
            source_type: "note".to_string(),
            source_id: Some(note.id.clone()),
        };
    }
    if let Some(conversation) = conversation {
        return ContinuitySuggestion {
            title: format!("Continue conversation: {}", conversation.title),
            detail: conversation.detail.clone(),
            destination: conversation.destination.clone(),
            source_type: "conversation".to_string(),
            source_id: Some(conversation.id.clone()),
        };
    }
    if let Some(file) = files.first() {
        return ContinuitySuggestion {
            title: format!("Review recent file: {}", file.title),
            detail: file.detail.clone(),
            destination: file.destination.clone(),
            source_type: "file".to_string(),
            source_id: Some(file.id.clone()),
        };
    }
    ContinuitySuggestion {
        title: "Capture the next step".to_string(),
        detail: "Create a Task or Note to give this Space a clear continuation point.".to_string(),
        destination: "/tasks".to_string(),
        source_type: "empty".to_string(),
        source_id: None,
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
        conn.execute(
            "INSERT INTO spaces (id,name) VALUES ('s1','Alpha'),('s2','Private')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn isolates_every_section_to_active_space_and_excludes_inactive_rows() {
        let conn = setup();
        conn.execute("INSERT INTO notes (id,space_id,title,content,excerpt) VALUES ('n1','s1','Alpha note','x','safe'),('n2','s2','Private note','secret','secret'),('n3','s1','Archived','x','x')", []).unwrap();
        conn.execute(
            "UPDATE notes SET archived_at=datetime('now') WHERE id='n3'",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO tasks (id,space_id,title,status,priority,completed_at) VALUES ('t1','s1','Do alpha','in_progress','high',NULL),('t2','s2','Private task','inbox','high',NULL),('t3','s1','Done','done','high',datetime('now'))", []).unwrap();
        conn.execute("INSERT INTO sources (id,root_path,display_name,space_id) VALUES ('src1','C:\\safe','Safe Source','s1'),('src2','C:\\private','Private Source','s2')", []).unwrap();
        conn.execute("INSERT INTO indexed_files (id,source_id,relative_path,filename,size_bytes,state) VALUES ('f1','src1','docs\\safe.txt','safe.txt',1,'present'),('f2','src2','private.txt','private.txt',1,'present'),('f3','src1','gone.txt','gone.txt',1,'removed')", []).unwrap();
        conn.execute("INSERT INTO ai_conversations (id,space_id,title) VALUES ('c1','s1','Alpha chat'),('c2','s2','Private chat')", []).unwrap();

        let snapshot = get(&conn, "s1").unwrap();
        assert_eq!(
            snapshot
                .recent_notes
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["n1"]
        );
        assert_eq!(
            snapshot
                .open_tasks
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["t1"]
        );
        assert_eq!(
            snapshot
                .recent_files
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["f1"]
        );
        assert_eq!(snapshot.latest_conversation.as_ref().unwrap().id, "c1");
        assert_eq!(
            snapshot.suggested_next_step.source_id.as_deref(),
            Some("t1")
        );
        assert!(!format!("{snapshot:?}").contains("Private"));
        assert!(!format!("{snapshot:?}").contains("C:\\safe"));
    }

    #[test]
    fn rejects_archived_or_missing_spaces() {
        let conn = setup();
        conn.execute(
            "UPDATE spaces SET archived_at=datetime('now') WHERE id='s1'",
            [],
        )
        .unwrap();
        assert!(get(&conn, "s1").is_err());
        assert!(get(&conn, "missing").is_err());
    }
}
