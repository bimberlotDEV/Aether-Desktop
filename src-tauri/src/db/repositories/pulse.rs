use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::activity::{self, ActivityItem};

const SECTION_LIMIT: i64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulseTask {
    pub id: String,
    pub title: String,
    pub space_id: Option<String>,
    pub space_name: Option<String>,
    pub due_date: String,
    pub priority: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulseSpace {
    pub id: String,
    pub name: String,
    pub reason: String,
    pub last_worked_at: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulseFile {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub space_id: Option<String>,
    pub space_name: Option<String>,
    pub detected_at: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulseSuggestion {
    pub title: String,
    pub detail: String,
    pub destination: String,
    pub source_type: String,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PulseSnapshot {
    pub today: String,
    pub overdue: Vec<PulseTask>,
    pub due_today: Vec<PulseTask>,
    pub upcoming: Vec<PulseTask>,
    pub continue_spaces: Vec<PulseSpace>,
    pub new_files: Vec<PulseFile>,
    pub recent_activity: Vec<ActivityItem>,
    pub suggested_next_step: PulseSuggestion,
}

pub fn get(conn: &Connection) -> Result<PulseSnapshot, String> {
    let today: String = conn
        .query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))
        .map_err(|error| format!("Pulse local date error: {error}"))?;
    get_for_date(conn, &today)
}

fn get_for_date(conn: &Connection, today: &str) -> Result<PulseSnapshot, String> {
    let overdue = list_tasks(conn, today, "overdue")?;
    let due_today = list_tasks(conn, today, "today")?;
    let upcoming = list_tasks(conn, today, "upcoming")?;
    let continue_spaces = list_continue_spaces(conn)?;
    let new_files = list_new_files(conn)?;
    let mut recent_activity = Vec::new();
    for item in activity::list_items(conn, None, Some(20))? {
        if active_scope(conn, item.space_id.as_deref())? {
            recent_activity.push(item);
            if recent_activity.len() == SECTION_LIMIT as usize {
                break;
            }
        }
    }
    let suggested_next_step = suggestion(
        &overdue,
        &due_today,
        &upcoming,
        &continue_spaces,
        &new_files,
    );

    Ok(PulseSnapshot {
        today: today.to_string(),
        overdue,
        due_today,
        upcoming,
        continue_spaces,
        new_files,
        recent_activity,
        suggested_next_step,
    })
}

fn active_scope(conn: &Connection, space_id: Option<&str>) -> Result<bool, String> {
    let Some(space_id) = space_id else {
        return Ok(true);
    };
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM spaces WHERE id = ?1 AND archived_at IS NULL)",
        [space_id],
        |row| row.get(0),
    )
    .map_err(|error| format!("Pulse Space scope error: {error}"))
}

fn list_tasks(conn: &Connection, today: &str, group: &str) -> Result<Vec<PulseTask>, String> {
    let predicate = match group {
        "overdue" => "t.due_date < ?1",
        "today" => "t.due_date = ?1",
        "upcoming" => "t.due_date > ?1 AND t.due_date <= date(?1, '+7 days')",
        _ => return Err("Unsupported Pulse task group".to_string()),
    };
    let sql = format!(
        "SELECT t.id, t.title, t.space_id, s.name, t.due_date, t.priority
         FROM tasks t LEFT JOIN spaces s ON s.id = t.space_id
         WHERE t.archived_at IS NULL AND t.status <> 'done' AND t.due_date IS NOT NULL
           AND (t.space_id IS NULL OR s.archived_at IS NULL) AND {predicate}
         ORDER BY t.due_date ASC,
           CASE t.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,
           t.updated_at DESC, t.id ASC LIMIT ?2"
    );
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Pulse Task query error: {error}"))?;
    let rows = statement
        .query_map(params![today, SECTION_LIMIT], |row| {
            let space_id: Option<String> = row.get(2)?;
            Ok(PulseTask {
                id: row.get(0)?,
                title: row.get(1)?,
                destination: "/tasks".to_string(),
                space_id,
                space_name: row.get(3)?,
                due_date: row.get(4)?,
                priority: row.get(5)?,
            })
        })
        .map_err(|error| format!("Pulse Task query error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Pulse Task row error: {error}"))
}

fn list_continue_spaces(conn: &Connection) -> Result<Vec<PulseSpace>, String> {
    let mut statement = conn
        .prepare(
            "WITH work(space_id, worked_at, reason) AS (
                SELECT space_id, updated_at, 'Recent Note work' FROM notes WHERE archived_at IS NULL
                UNION ALL SELECT space_id, updated_at, 'Open Task activity' FROM tasks WHERE archived_at IS NULL AND status <> 'done' AND space_id IS NOT NULL
                UNION ALL SELECT so.space_id, f.updated_at, 'New Source metadata' FROM indexed_files f JOIN sources so ON so.id = f.source_id WHERE f.state = 'present' AND so.space_id IS NOT NULL
                UNION ALL SELECT space_id, updated_at, 'Recent AI conversation' FROM ai_conversations WHERE archived_at IS NULL AND space_id IS NOT NULL
                UNION ALL SELECT space_id, created_at, 'Meaningful recent activity' FROM activity_events
                  WHERE space_id IS NOT NULL AND event_type IN ('note_created','note_edited','task_created','task_created_from_ai_proposal','task_completed','task_archived','vault_imported','vault_updated','vault_removed','memory_created','memory_updated','memory_deleted','source_scanned','ai_conversation_used','action_executed')
                UNION ALL SELECT id, last_opened_at, 'Recently opened' FROM spaces WHERE last_opened_at IS NOT NULL
                UNION ALL SELECT id, updated_at, 'Space recently changed' FROM spaces
             ), ranked AS (
                SELECT space_id, worked_at, reason,
                  ROW_NUMBER() OVER (PARTITION BY space_id ORDER BY worked_at DESC, reason ASC) AS position
                FROM work
             )
             SELECT s.id, s.name, ranked.reason, ranked.worked_at
             FROM ranked JOIN spaces s ON s.id = ranked.space_id
             WHERE ranked.position = 1 AND s.archived_at IS NULL
             ORDER BY ranked.worked_at DESC, s.name ASC LIMIT ?1",
        )
        .map_err(|error| format!("Pulse Continue query error: {error}"))?;
    let rows = statement
        .query_map([SECTION_LIMIT], |row| {
            let id: String = row.get(0)?;
            Ok(PulseSpace {
                destination: format!("/spaces/{id}"),
                id,
                name: row.get(1)?,
                reason: row.get(2)?,
                last_worked_at: row.get(3)?,
            })
        })
        .map_err(|error| format!("Pulse Continue query error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Pulse Continue row error: {error}"))
}

fn list_new_files(conn: &Connection) -> Result<Vec<PulseFile>, String> {
    let mut statement = conn
        .prepare(
            "SELECT f.id, f.filename, so.display_name || ' · ' || f.relative_path,
                    so.space_id, s.name, f.first_seen_at
             FROM indexed_files f
             JOIN sources so ON so.id = f.source_id
             LEFT JOIN spaces s ON s.id = so.space_id
             WHERE f.state = 'present' AND f.first_seen_at >= datetime('now', '-7 days')
               AND (so.space_id IS NULL OR s.archived_at IS NULL)
             ORDER BY f.first_seen_at DESC, f.id ASC LIMIT ?1",
        )
        .map_err(|error| format!("Pulse New query error: {error}"))?;
    let rows = statement
        .query_map([SECTION_LIMIT], |row| {
            Ok(PulseFile {
                id: row.get(0)?,
                title: row.get(1)?,
                detail: row.get(2)?,
                space_id: row.get(3)?,
                space_name: row.get(4)?,
                detected_at: row.get(5)?,
                destination: "/sources".to_string(),
            })
        })
        .map_err(|error| format!("Pulse New query error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Pulse New row error: {error}"))
}

fn suggestion(
    overdue: &[PulseTask],
    due_today: &[PulseTask],
    upcoming: &[PulseTask],
    spaces: &[PulseSpace],
    files: &[PulseFile],
) -> PulseSuggestion {
    if let Some(task) = overdue.first() {
        return task_suggestion(task, "This open Task is overdue");
    }
    if let Some(task) = due_today.first() {
        return task_suggestion(task, "This open Task is due today");
    }
    if let Some(task) = upcoming.first() {
        return task_suggestion(task, &format!("This open Task is due {}", task.due_date));
    }
    if let Some(space) = spaces.first() {
        return PulseSuggestion {
            title: format!("Continue {}", space.name),
            detail: space.reason.clone(),
            destination: space.destination.clone(),
            source_type: "space".to_string(),
            source_id: Some(space.id.clone()),
        };
    }
    if let Some(file) = files.first() {
        return PulseSuggestion {
            title: format!("Review {}", file.title),
            detail: "This file was newly detected in an authorized Source".to_string(),
            destination: file.destination.clone(),
            source_type: "file".to_string(),
            source_id: Some(file.id.clone()),
        };
    }
    PulseSuggestion {
        title: "Shape your next focus".to_string(),
        detail: "No dated Tasks or recent workspace changes need attention".to_string(),
        destination: "/tasks".to_string(),
        source_type: "empty".to_string(),
        source_id: None,
    }
}

fn task_suggestion(task: &PulseTask, detail: &str) -> PulseSuggestion {
    PulseSuggestion {
        title: format!("Continue {}", task.title),
        detail: detail.to_string(),
        destination: task.destination.clone(),
        source_type: "task".to_string(),
        source_id: Some(task.id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn database() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        migrations::run(&connection).unwrap();
        connection
    }

    #[test]
    fn groups_real_attention_and_excludes_inactive_records() {
        let connection = database();
        connection.execute("INSERT INTO spaces (id,name,archived_at) VALUES ('s1','Current',NULL),('s2','Private',NULL),('arch','Archived',datetime('now'))", []).unwrap();
        connection.execute("INSERT INTO tasks (id,space_id,title,status,priority,due_date,completed_at,archived_at) VALUES ('late','s1','Late','planned','high','2026-08-26',NULL,NULL),('today','s1','Today','planned','medium','2026-08-27',NULL,NULL),('soon',NULL,'Soon','inbox','none','2026-08-29',NULL,NULL),('other','s2','Other','planned','high','2026-08-27',NULL,NULL),('done','s1','Done','done','high','2026-08-26',datetime('now'),NULL),('hidden','arch','Hidden','planned','high','2026-08-26',NULL,NULL)", []).unwrap();
        connection.execute("INSERT INTO notes (id,space_id,title,updated_at) VALUES ('n1','s1','Work','2026-08-27 12:00:00'),('n2','s2','Other work','2026-08-27 11:00:00'),('n3','arch','Hidden work','2026-08-27 13:00:00')", []).unwrap();
        connection.execute("INSERT INTO sources (id,root_path,display_name,space_id) VALUES ('src','C:/safe','Safe Source','s1'),('hidden-src','C:/hidden','Hidden Source','arch')", []).unwrap();
        connection.execute("INSERT INTO indexed_files (id,source_id,relative_path,filename,size_bytes,state) VALUES ('f1','src','folder/report.pdf','report.pdf',1,'present'),('removed','src','old.pdf','old.pdf',1,'removed'),('hidden-file','hidden-src','secret.pdf','secret.pdf',1,'present')", []).unwrap();

        let pulse = get_for_date(&connection, "2026-08-27").unwrap();
        assert_eq!(
            pulse
                .overdue
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["late"]
        );
        assert_eq!(
            pulse
                .due_today
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["other", "today"]
        );
        assert_eq!(pulse.upcoming[0].id, "soon");
        assert!(pulse.continue_spaces.iter().all(|space| space.id != "arch"));
        assert_eq!(
            pulse
                .new_files
                .iter()
                .map(|file| file.id.as_str())
                .collect::<Vec<_>>(),
            vec!["f1"]
        );
        assert!(!pulse.new_files[0].detail.contains("C:/safe"));
        assert_eq!(pulse.suggested_next_step.source_id.as_deref(), Some("late"));
        assert_eq!(
            pulse.suggested_next_step.detail,
            "This open Task is overdue"
        );
    }

    #[test]
    fn suggestion_priority_is_deterministic_and_empty_is_honest() {
        let task = PulseTask {
            id: "t".into(),
            title: "Task".into(),
            space_id: None,
            space_name: None,
            due_date: "2026-08-27".into(),
            priority: "none".into(),
            destination: "/tasks".into(),
        };
        let space = PulseSpace {
            id: "s".into(),
            name: "Space".into(),
            reason: "Recent Note work".into(),
            last_worked_at: "2026-08-27".into(),
            destination: "/spaces/s".into(),
        };
        assert_eq!(
            suggestion(&[], &[task], &[], &[space], &[]).source_type,
            "task"
        );
        assert_eq!(suggestion(&[], &[], &[], &[], &[]).source_type, "empty");
    }
}
