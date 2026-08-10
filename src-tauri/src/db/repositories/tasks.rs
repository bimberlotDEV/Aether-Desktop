use chrono::{NaiveDate, Utc};
use rusqlite::{params, params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

const TASK_COLS: &str = "id, space_id, parent_task_id, title, description, status, priority, due_date, tags_json, completed_at, archived_at, created_at, updated_at";
const STATUSES: &[&str] = &["inbox", "planned", "in_progress", "done"];
const PRIORITIES: &[&str] = &["none", "low", "medium", "high"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub space_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub completed_at: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub space_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    pub due_date: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFilter {
    pub space_id: Option<String>,
    #[serde(default)]
    pub unassigned_only: bool,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
    pub limit: Option<u32>,
}

fn default_status() -> String {
    "inbox".to_string()
}

fn default_priority() -> String {
    "none".to_string()
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let tags_json: String = row.get(8)?;
    let tags = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Task {
        id: row.get(0)?,
        space_id: row.get(1)?,
        parent_task_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        status: row.get(5)?,
        priority: row.get(6)?,
        due_date: row.get(7)?,
        tags,
        completed_at: row.get(9)?,
        archived_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn validate_date(value: &str, field: &str) -> Result<(), String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| format!("{} must be a valid date in YYYY-MM-DD format", field))
}

fn normalize_tags(tags: &[String]) -> Result<Vec<String>, String> {
    if tags.len() > 20 {
        return Err("A Task can have at most 20 tags".to_string());
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for tag in tags {
        let value = tag.trim();
        if value.is_empty() {
            return Err("Task tags cannot be empty".to_string());
        }
        if value.chars().count() > 40 {
            return Err("Task tags cannot exceed 40 characters".to_string());
        }
        let key = value.to_lowercase();
        if seen.insert(key) {
            normalized.push(value.to_string());
        }
    }
    Ok(normalized)
}

fn validate_input(
    conn: &Connection,
    id: Option<&str>,
    input: &TaskInput,
) -> Result<(String, Vec<String>), String> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err("Task title is required".to_string());
    }
    if title.chars().count() > 200 {
        return Err("Task title cannot exceed 200 characters".to_string());
    }
    if input.description.chars().count() > 10_000 {
        return Err("Task description cannot exceed 10000 characters".to_string());
    }
    if !STATUSES.contains(&input.status.as_str()) {
        return Err(format!("Invalid Task status: {}", input.status));
    }
    if !PRIORITIES.contains(&input.priority.as_str()) {
        return Err(format!("Invalid Task priority: {}", input.priority));
    }
    if let Some(due_date) = input.due_date.as_deref() {
        validate_date(due_date, "Task due date")?;
    }

    if let Some(space_id) = input.space_id.as_deref() {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM spaces WHERE id = ?1)",
                params![space_id],
                |row| row.get(0),
            )
            .map_err(|error| format!("Task Space validation error: {}", error))?;
        if !exists {
            return Err("Task Space does not exist".to_string());
        }
    }

    if let Some(parent_id) = input.parent_task_id.as_deref() {
        if id == Some(parent_id) {
            return Err("A Task cannot be its own parent".to_string());
        }
        let parent_space: Option<Option<String>> = match conn.query_row(
            "SELECT space_id FROM tasks WHERE id = ?1 AND archived_at IS NULL",
            params![parent_id],
            |row| row.get(0),
        ) {
            Ok(space) => Some(space),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(error) => return Err(format!("Task parent validation error: {}", error)),
        };
        let Some(parent_space) = parent_space else {
            return Err("Parent Task does not exist or is archived".to_string());
        };
        if parent_space != input.space_id {
            return Err("A subtask must belong to the same Space as its parent".to_string());
        }

        if let Some(task_id) = id {
            let creates_cycle: bool = conn
                .query_row(
                    "WITH RECURSIVE descendants(id) AS (
                        SELECT id FROM tasks WHERE parent_task_id = ?1
                        UNION ALL
                        SELECT tasks.id FROM tasks JOIN descendants ON tasks.parent_task_id = descendants.id
                     )
                     SELECT EXISTS(SELECT 1 FROM descendants WHERE id = ?2)",
                    params![task_id, parent_id],
                    |row| row.get(0),
                )
                .map_err(|error| format!("Task cycle validation error: {}", error))?;
            if creates_cycle {
                return Err("Task hierarchy cannot contain a cycle".to_string());
            }
        }
    }

    Ok((title.to_string(), normalize_tags(&input.tags)?))
}

pub fn create(conn: &Connection, input: &TaskInput) -> Result<Task, String> {
    let (title, tags) = validate_input(conn, None, input)?;
    let id = Uuid::now_v7().to_string();
    let tags_json = serde_json::to_string(&tags)
        .map_err(|error| format!("Task tags serialization error: {}", error))?;
    let completed_at = (input.status == "done").then(|| Utc::now().to_rfc3339());
    conn.execute(
        "INSERT INTO tasks (
            id, space_id, parent_task_id, title, description, status, priority,
            due_date, tags_json, completed_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            input.space_id,
            input.parent_task_id,
            title,
            input.description,
            input.status,
            input.priority,
            input.due_date,
            tags_json,
            completed_at,
        ],
    )
    .map_err(|error| format!("Task create error: {}", error))?;
    get_by_id(conn, &id)?.ok_or_else(|| "Task not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Task>, String> {
    let sql = format!("SELECT {} FROM tasks WHERE id = ?1", TASK_COLS);
    match conn.query_row(&sql, params![id], row_to_task) {
        Ok(task) => Ok(Some(task)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Task get error: {}", error)),
    }
}

pub fn update(conn: &Connection, id: &str, input: &TaskInput) -> Result<Option<Task>, String> {
    if get_by_id(conn, id)?.is_none() {
        return Ok(None);
    }
    let (title, tags) = validate_input(conn, Some(id), input)?;
    let tags_json = serde_json::to_string(&tags)
        .map_err(|error| format!("Task tags serialization error: {}", error))?;
    let completed_at = if input.status == "done" {
        conn.query_row(
            "SELECT COALESCE(completed_at, ?2) FROM tasks WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
            |row| row.get::<_, String>(0),
        )
        .map(Some)
        .map_err(|error| format!("Task completion error: {}", error))?
    } else {
        None
    };
    conn.execute(
        "UPDATE tasks SET
            space_id = ?1, parent_task_id = ?2, title = ?3, description = ?4,
            status = ?5, priority = ?6, due_date = ?7, tags_json = ?8,
            completed_at = ?9, updated_at = datetime('now')
         WHERE id = ?10",
        params![
            input.space_id,
            input.parent_task_id,
            title,
            input.description,
            input.status,
            input.priority,
            input.due_date,
            tags_json,
            completed_at,
            id,
        ],
    )
    .map_err(|error| format!("Task update error: {}", error))?;
    conn.execute(
        "WITH RECURSIVE descendants(id) AS (
            SELECT id FROM tasks WHERE parent_task_id = ?1
            UNION ALL
            SELECT tasks.id FROM tasks JOIN descendants ON tasks.parent_task_id = descendants.id
         )
         UPDATE tasks SET space_id = ?2, updated_at = datetime('now')
         WHERE id IN (SELECT id FROM descendants) AND space_id IS NOT ?2",
        params![id, input.space_id],
    )
    .map_err(|error| format!("Task descendant Space update error: {}", error))?;
    get_by_id(conn, id)
}

pub fn list(conn: &Connection, filter: &TaskFilter) -> Result<Vec<Task>, String> {
    if filter.unassigned_only && filter.space_id.is_some() {
        return Err("Task filter cannot combine a Space with unassigned-only".to_string());
    }
    if let Some(status) = filter.status.as_deref() {
        if !STATUSES.contains(&status) {
            return Err(format!("Invalid Task status filter: {}", status));
        }
    }
    if let Some(priority) = filter.priority.as_deref() {
        if !PRIORITIES.contains(&priority) {
            return Err(format!("Invalid Task priority filter: {}", priority));
        }
    }

    let mut conditions = Vec::new();
    let mut values = Vec::<Value>::new();
    if !filter.include_archived {
        conditions.push("archived_at IS NULL".to_string());
    }
    if let Some(space_id) = filter.space_id.as_deref() {
        values.push(Value::Text(space_id.to_string()));
        conditions.push(format!("space_id = ?{}", values.len()));
    } else if filter.unassigned_only {
        conditions.push("space_id IS NULL".to_string());
    }
    if let Some(status) = filter.status.as_deref() {
        values.push(Value::Text(status.to_string()));
        conditions.push(format!("status = ?{}", values.len()));
    }
    if let Some(priority) = filter.priority.as_deref() {
        values.push(Value::Text(priority.to_string()));
        conditions.push(format!("priority = ?{}", values.len()));
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
            "(title LIKE ?{0} OR description LIKE ?{0} OR tags_json LIKE ?{0})",
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
        "SELECT {} FROM tasks{} ORDER BY
         CASE WHEN due_date IS NULL THEN 1 ELSE 0 END,
         due_date ASC, created_at DESC LIMIT ?{}",
        TASK_COLS,
        where_clause,
        values.len()
    );
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Task list error: {}", error))?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), row_to_task)
        .map_err(|error| format!("Task list error: {}", error))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Task row error: {}", error))
}

pub fn list_attention(
    conn: &Connection,
    today: &str,
    horizon: &str,
    limit: u32,
) -> Result<Vec<Task>, String> {
    validate_date(today, "Attention start date")?;
    validate_date(horizon, "Attention horizon date")?;
    if horizon < today {
        return Err("Attention horizon cannot be before the start date".to_string());
    }
    let sql = format!(
        "SELECT {} FROM tasks
         WHERE archived_at IS NULL AND parent_task_id IS NULL AND status <> 'done'
           AND due_date IS NOT NULL AND due_date <= ?1
         ORDER BY due_date ASC, CASE priority
           WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,
           created_at ASC
         LIMIT ?2",
        TASK_COLS
    );
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Task attention error: {}", error))?;
    let rows = statement
        .query_map(params![horizon, limit.clamp(1, 100)], row_to_task)
        .map_err(|error| format!("Task attention error: {}", error))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Task attention row error: {}", error))
}

pub fn archive(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute(
            "WITH RECURSIVE tree(id) AS (
                SELECT id FROM tasks WHERE id = ?1
                UNION ALL
                SELECT tasks.id FROM tasks JOIN tree ON tasks.parent_task_id = tree.id
             )
             UPDATE tasks SET archived_at = datetime('now'), updated_at = datetime('now')
             WHERE id IN (SELECT id FROM tree) AND archived_at IS NULL",
            params![id],
        )
        .map_err(|error| format!("Task archive error: {}", error))?;
    Ok(affected > 0)
}

pub fn restore(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute(
            "WITH RECURSIVE tree(id) AS (
                SELECT id FROM tasks WHERE id = ?1
                UNION ALL
                SELECT tasks.id FROM tasks JOIN tree ON tasks.parent_task_id = tree.id
             )
             UPDATE tasks SET archived_at = NULL, updated_at = datetime('now')
             WHERE id IN (SELECT id FROM tree) AND archived_at IS NOT NULL",
            params![id],
        )
        .map_err(|error| format!("Task restore error: {}", error))?;
    Ok(affected > 0)
}

pub fn delete_permanent(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
        .map(|affected| affected > 0)
        .map_err(|error| format!("Task delete error: {}", error))
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
            "INSERT INTO spaces (id, name) VALUES ('space-1', 'Test Space')",
            [],
        )
        .unwrap();
        conn
    }

    fn input(title: &str) -> TaskInput {
        TaskInput {
            space_id: Some("space-1".to_string()),
            parent_task_id: None,
            title: title.to_string(),
            description: String::new(),
            status: "inbox".to_string(),
            priority: "none".to_string(),
            due_date: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn test_create_get_and_update_completion() {
        let conn = setup();
        let mut draft = input("Write report");
        draft.tags = vec!["Work".to_string(), "work".to_string()];
        let created = create(&conn, &draft).unwrap();
        assert_eq!(created.title, "Write report");
        assert_eq!(created.tags, vec!["Work"]);
        assert!(created.completed_at.is_none());

        draft.status = "done".to_string();
        draft.priority = "high".to_string();
        draft.due_date = Some("2026-08-12".to_string());
        let updated = update(&conn, &created.id, &draft).unwrap().unwrap();
        assert_eq!(updated.status, "done");
        assert!(updated.completed_at.is_some());

        draft.status = "planned".to_string();
        let reopened = update(&conn, &created.id, &draft).unwrap().unwrap();
        assert!(reopened.completed_at.is_none());
    }

    #[test]
    fn test_validation() {
        let conn = setup();
        let mut draft = input(" ");
        assert!(create(&conn, &draft)
            .unwrap_err()
            .contains("title is required"));
        draft.title = "Valid".to_string();
        draft.status = "blocked".to_string();
        assert!(create(&conn, &draft)
            .unwrap_err()
            .contains("Invalid Task status"));
        draft.status = "inbox".to_string();
        draft.priority = "critical".to_string();
        assert!(create(&conn, &draft)
            .unwrap_err()
            .contains("Invalid Task priority"));
        draft.priority = "none".to_string();
        draft.due_date = Some("2026-02-30".to_string());
        assert!(create(&conn, &draft).unwrap_err().contains("valid date"));
        draft.due_date = None;
        draft.tags = vec!["".to_string()];
        assert!(create(&conn, &draft)
            .unwrap_err()
            .contains("cannot be empty"));
        draft.tags = Vec::new();
        draft.space_id = Some("missing-space".to_string());
        assert!(create(&conn, &draft)
            .unwrap_err()
            .contains("does not exist"));
        draft.space_id = Some("space-1".to_string());
        draft.parent_task_id = Some("missing-parent".to_string());
        assert!(create(&conn, &draft)
            .unwrap_err()
            .contains("Parent Task does not exist"));
    }

    #[test]
    fn test_subtasks_cycle_and_delete_cascade() {
        let conn = setup();
        let parent = create(&conn, &input("Parent")).unwrap();
        let mut child_input = input("Child");
        child_input.parent_task_id = Some(parent.id.clone());
        let child = create(&conn, &child_input).unwrap();

        let mut self_parent = input("Parent");
        self_parent.parent_task_id = Some(parent.id.clone());
        assert!(update(&conn, &parent.id, &self_parent)
            .unwrap_err()
            .contains("own parent"));

        let mut parent_update = input("Parent");
        parent_update.parent_task_id = Some(child.id.clone());
        assert!(update(&conn, &parent.id, &parent_update)
            .unwrap_err()
            .contains("cycle"));

        let mut moved_parent = input("Parent");
        moved_parent.space_id = None;
        update(&conn, &parent.id, &moved_parent).unwrap();
        assert!(get_by_id(&conn, &child.id)
            .unwrap()
            .unwrap()
            .space_id
            .is_none());

        assert!(archive(&conn, &parent.id).unwrap());
        assert!(get_by_id(&conn, &child.id)
            .unwrap()
            .unwrap()
            .archived_at
            .is_some());
        assert!(restore(&conn, &parent.id).unwrap());
        assert!(get_by_id(&conn, &child.id)
            .unwrap()
            .unwrap()
            .archived_at
            .is_none());
        assert!(delete_permanent(&conn, &parent.id).unwrap());
        assert!(get_by_id(&conn, &child.id).unwrap().is_none());
    }

    #[test]
    fn test_list_filters_search_and_archive() {
        let conn = setup();
        let mut first = input("Write release notes");
        first.description = "Document changes".to_string();
        first.status = "planned".to_string();
        first.priority = "high".to_string();
        let first = create(&conn, &first).unwrap();
        create(&conn, &input("Buy groceries")).unwrap();

        let filtered = list(
            &conn,
            &TaskFilter {
                status: Some("planned".to_string()),
                priority: Some("high".to_string()),
                search: Some("release".to_string()),
                ..TaskFilter::default()
            },
        )
        .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, first.id);

        let mut unassigned = input("Inbox item");
        unassigned.space_id = None;
        let unassigned = create(&conn, &unassigned).unwrap();
        let inbox = list(
            &conn,
            &TaskFilter {
                unassigned_only: true,
                ..TaskFilter::default()
            },
        )
        .unwrap();
        assert_eq!(
            inbox.iter().map(|task| &task.id).collect::<Vec<_>>(),
            vec![&unassigned.id]
        );

        assert!(archive(&conn, &first.id).unwrap());
        assert!(list(&conn, &TaskFilter::default())
            .unwrap()
            .iter()
            .all(|task| task.id != first.id));
        assert!(restore(&conn, &first.id).unwrap());
        assert!(get_by_id(&conn, &first.id)
            .unwrap()
            .unwrap()
            .archived_at
            .is_none());
    }

    #[test]
    fn test_attention_query() {
        let conn = setup();
        for (title, due_date) in [
            ("Overdue", "2026-08-09"),
            ("Today", "2026-08-10"),
            ("Upcoming", "2026-08-12"),
            ("Later", "2026-09-01"),
        ] {
            let mut draft = input(title);
            draft.due_date = Some(due_date.to_string());
            create(&conn, &draft).unwrap();
        }
        let tasks = list_attention(&conn, "2026-08-10", "2026-08-17", 10).unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].title, "Overdue");
        assert!(list_attention(&conn, "2026-08-10", "2026-08-01", 10).is_err());
    }
}
