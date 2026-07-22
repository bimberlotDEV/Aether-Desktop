#![allow(clippy::too_many_arguments)]

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const NOTE_COLS: &str = "id, space_id, title, content, content_format, excerpt, pinned, revision, archived_at, created_at, updated_at, last_opened_at";

pub const NOTE_LIST_COLS: &str = "id, space_id, title, excerpt, content_format, pinned, revision, archived_at, created_at, updated_at, last_opened_at";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub space_id: String,
    pub title: String,
    pub content: String,
    pub content_format: String,
    pub excerpt: String,
    pub pinned: bool,
    pub revision: i64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteListItem {
    pub id: String,
    pub space_id: String,
    pub title: String,
    pub excerpt: String,
    pub content_format: String,
    pub pinned: bool,
    pub revision: i64,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_opened_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSearchResult {
    pub id: String,
    pub space_id: String,
    pub title: String,
    pub excerpt: String,
    pub pinned: bool,
    pub archived_at: Option<String>,
    pub updated_at: String,
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        space_id: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        content_format: row.get(4)?,
        excerpt: row.get(5)?,
        pinned: row.get::<_, i64>(6)? != 0,
        revision: row.get(7)?,
        archived_at: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        last_opened_at: row.get(11)?,
    })
}

fn row_to_list_item(row: &rusqlite::Row) -> rusqlite::Result<NoteListItem> {
    Ok(NoteListItem {
        id: row.get(0)?,
        space_id: row.get(1)?,
        title: row.get(2)?,
        excerpt: row.get(3)?,
        content_format: row.get(4)?,
        pinned: row.get::<_, i64>(5)? != 0,
        revision: row.get(6)?,
        archived_at: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        last_opened_at: row.get(10)?,
    })
}

// ─── CRUD ────────────────────────────────────────────────

pub fn create(conn: &Connection, space_id: &str) -> Result<Note, String> {
    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO notes (id, space_id) VALUES (?1, ?2)",
        params![id, space_id],
    )
    .map_err(|e| format!("Note create error: {}", e))?;
    get_by_id(conn, &id)?.ok_or_else(|| "Note not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Note>, String> {
    let sql = format!("SELECT {} FROM notes WHERE id = ?1", NOTE_COLS);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Note get error: {}", e))?;
    let result = stmt.query_row(params![id], row_to_note);
    match result {
        Ok(note) => Ok(Some(note)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Note get error: {}", e)),
    }
}

pub fn update(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
    excerpt: Option<&str>,
    expected_revision: Option<i64>,
) -> Result<Option<Note>, String> {
    if let Some(expected) = expected_revision {
        let current: i64 = conn
            .query_row(
                "SELECT revision FROM notes WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Note revision check error: {}", e))?;
        if current != expected {
            return Err(format!(
                "Stale update: expected revision {} but current is {}",
                expected, current
            ));
        }
    }

    let mut sets = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(v) = title {
        sets.push("title = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = content {
        sets.push("content = ?");
        values.push(Box::new(v.to_string()));
    }
    if let Some(v) = excerpt {
        sets.push("excerpt = ?");
        values.push(Box::new(v.to_string()));
    }

    if sets.is_empty() {
        return get_by_id(conn, id);
    }

    sets.push("revision = revision + 1");
    sets.push("updated_at = datetime('now')");
    values.push(Box::new(id.to_string()));

    let sql = format!(
        "UPDATE notes SET {} WHERE id = ?{}",
        sets.join(", "),
        values.len()
    );
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let affected = conn
        .execute(&sql, params_refs.as_slice())
        .map_err(|e| format!("Note update error: {}", e))?;

    if affected == 0 {
        Ok(None)
    } else {
        get_by_id(conn, id)
    }
}

pub fn touch_last_opened(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE notes SET last_opened_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Note touch error: {}", e))?;
    Ok(())
}

pub fn list_by_space(conn: &Connection, space_id: &str) -> Result<Vec<NoteListItem>, String> {
    let sql = format!(
        "SELECT {} FROM notes WHERE space_id = ?1 AND archived_at IS NULL ORDER BY pinned DESC, updated_at DESC",
        NOTE_LIST_COLS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Note list error: {}", e))?;
    let rows = stmt
        .query_map(params![space_id], row_to_list_item)
        .map_err(|e| format!("Note list error: {}", e))?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row.map_err(|e| format!("Note row error: {}", e))?);
    }
    Ok(notes)
}

pub fn list_recent(
    conn: &Connection,
    space_id: Option<&str>,
    limit: u32,
) -> Result<Vec<NoteListItem>, String> {
    let sql: String;
    let params_vec: Vec<Box<dyn rusqlite::types::ToSql>>;
    if let Some(sid) = space_id {
        sql = format!(
            "SELECT {} FROM notes WHERE space_id = ?1 AND archived_at IS NULL ORDER BY last_opened_at DESC, updated_at DESC LIMIT ?2",
            NOTE_LIST_COLS
        );
        params_vec = vec![Box::new(sid.to_string()), Box::new(limit)];
    } else {
        sql = format!(
            "SELECT {} FROM notes WHERE archived_at IS NULL ORDER BY last_opened_at DESC, updated_at DESC LIMIT ?1",
            NOTE_LIST_COLS
        );
        params_vec = vec![Box::new(limit)];
    };
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Note recent error: {}", e))?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_vec.iter().map(|v| v.as_ref()).collect();
    let rows = stmt
        .query_map(params_refs.as_slice(), row_to_list_item)
        .map_err(|e| format!("Note recent error: {}", e))?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row.map_err(|e| format!("Note row error: {}", e))?);
    }
    Ok(notes)
}

pub fn list_pinned(conn: &Connection, space_id: Option<&str>) -> Result<Vec<NoteListItem>, String> {
    let sql: String;
    let params_vec: Vec<Box<dyn rusqlite::types::ToSql>>;
    if let Some(sid) = space_id {
        sql = format!(
            "SELECT {} FROM notes WHERE space_id = ?1 AND pinned = 1 AND archived_at IS NULL ORDER BY updated_at DESC",
            NOTE_LIST_COLS
        );
        params_vec = vec![Box::new(sid.to_string())];
    } else {
        sql = format!(
            "SELECT {} FROM notes WHERE pinned = 1 AND archived_at IS NULL ORDER BY updated_at DESC",
            NOTE_LIST_COLS
        );
        params_vec = vec![];
    };
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Note pinned error: {}", e))?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_vec.iter().map(|v| v.as_ref()).collect();
    let rows = stmt
        .query_map(params_refs.as_slice(), row_to_list_item)
        .map_err(|e| format!("Note pinned error: {}", e))?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row.map_err(|e| format!("Note row error: {}", e))?);
    }
    Ok(notes)
}

pub fn list_archived(
    conn: &Connection,
    space_id: Option<&str>,
) -> Result<Vec<NoteListItem>, String> {
    let sql = if let Some(_sid) = space_id {
        format!(
            "SELECT {} FROM notes WHERE space_id = ?1 AND archived_at IS NOT NULL ORDER BY archived_at DESC",
            NOTE_LIST_COLS
        )
    } else {
        format!(
            "SELECT {} FROM notes WHERE archived_at IS NOT NULL ORDER BY archived_at DESC",
            NOTE_LIST_COLS
        )
    };
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Note archived error: {}", e))?;
    let rows = if space_id.is_some() {
        stmt.query_map(params![space_id.unwrap()], row_to_list_item)
    } else {
        stmt.query_map([], row_to_list_item)
    }
    .map_err(|e| format!("Note archived error: {}", e))?;
    let mut notes = Vec::new();
    for row in rows {
        notes.push(row.map_err(|e| format!("Note row error: {}", e))?);
    }
    Ok(notes)
}

pub fn search(
    conn: &Connection,
    query: &str,
    space_id: Option<&str>,
    limit: u32,
) -> Result<Vec<NoteSearchResult>, String> {
    let sql = if space_id.is_some() {
        "SELECT n.id, n.space_id, n.title, n.excerpt, n.pinned, n.archived_at, n.updated_at FROM notes n JOIN notes_fts fts ON n.rowid = fts.rowid WHERE notes_fts MATCH ?1 AND n.space_id = ?2 AND n.archived_at IS NULL ORDER BY rank LIMIT ?3"
    } else {
        "SELECT n.id, n.space_id, n.title, n.excerpt, n.pinned, n.archived_at, n.updated_at FROM notes n JOIN notes_fts fts ON n.rowid = fts.rowid WHERE notes_fts MATCH ?1 AND n.archived_at IS NULL ORDER BY rank LIMIT ?2"
    };
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Note search error: {}", e))?;

    // Build params based on whether space_id is present
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(query.to_string())];
    if let Some(ref sid) = space_id {
        param_values.push(Box::new(sid.to_string()));
    }
    param_values.push(Box::new(limit));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|v| v.as_ref()).collect();

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(NoteSearchResult {
                id: row.get(0)?,
                space_id: row.get(1)?,
                title: row.get(2)?,
                excerpt: row.get(3)?,
                pinned: row.get::<_, i64>(4)? != 0,
                archived_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Note search error: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Note search row error: {}", e))?);
    }
    Ok(results)
}

pub fn set_pinned(conn: &Connection, id: &str, pinned: bool) -> Result<bool, String> {
    let affected = conn
        .execute(
            "UPDATE notes SET pinned = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![pinned as i64, id],
        )
        .map_err(|e| format!("Note pin error: {}", e))?;
    Ok(affected > 0)
}

pub fn archive(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn.execute(
        "UPDATE notes SET archived_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1 AND archived_at IS NULL",
        params![id],
    ).map_err(|e| format!("Note archive error: {}", e))?;
    Ok(affected > 0)
}

pub fn restore(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn.execute(
        "UPDATE notes SET archived_at = NULL, updated_at = datetime('now') WHERE id = ?1 AND archived_at IS NOT NULL",
        params![id],
    ).map_err(|e| format!("Note restore error: {}", e))?;
    Ok(affected > 0)
}

pub fn delete_permanent(conn: &Connection, id: &str) -> Result<bool, String> {
    let affected = conn
        .execute("DELETE FROM notes WHERE id = ?1", params![id])
        .map_err(|e| format!("Note delete error: {}", e))?;
    Ok(affected > 0)
}

pub fn move_to_space(conn: &Connection, id: &str, new_space_id: &str) -> Result<bool, String> {
    let affected = conn
        .execute(
            "UPDATE notes SET space_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![new_space_id, id],
        )
        .map_err(|e| format!("Note move error: {}", e))?;
    Ok(affected > 0)
}

pub fn duplicate(conn: &Connection, id: &str) -> Result<Note, String> {
    let original = get_by_id(conn, id)?.ok_or("Note not found")?;
    let new_id = Uuid::now_v7().to_string();
    let new_title = format!("{} (copy)", original.title);
    conn.execute(
        "INSERT INTO notes (id, space_id, title, content, content_format, excerpt) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![new_id, original.space_id, new_title, original.content, original.content_format, original.excerpt],
    ).map_err(|e| format!("Note duplicate error: {}", e))?;
    get_by_id(conn, &new_id)?.ok_or_else(|| "Duplicated note not found".to_string())
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
            "INSERT INTO spaces (id, name) VALUES ('sp1', 'Test Space')",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_create_and_get() {
        let conn = setup();
        let note = create(&conn, "sp1").unwrap();
        assert_eq!(note.title, "Untitled note");
        assert_eq!(note.space_id, "sp1");
        assert!(!note.pinned);
    }

    #[test]
    fn test_update_with_revision() {
        let conn = setup();
        let note = create(&conn, "sp1").unwrap();
        assert_eq!(note.revision, 1);
        let updated = update(
            &conn,
            &note.id,
            Some("Hello"),
            Some("# Content"),
            Some("Content"),
            Some(1),
        )
        .unwrap()
        .unwrap();
        assert_eq!(updated.title, "Hello");
        assert_eq!(updated.revision, 2);
        let result = update(&conn, &note.id, Some("Stale"), None, None, Some(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_pin_and_unpin() {
        let conn = setup();
        let note = create(&conn, "sp1").unwrap();
        assert!(set_pinned(&conn, &note.id, true).unwrap());
        assert_eq!(list_pinned(&conn, Some("sp1")).unwrap().len(), 1);
        assert!(set_pinned(&conn, &note.id, false).unwrap());
        assert_eq!(list_pinned(&conn, Some("sp1")).unwrap().len(), 0);
    }

    #[test]
    fn test_archive_restore_delete() {
        let conn = setup();
        let note = create(&conn, "sp1").unwrap();
        assert!(archive(&conn, &note.id).unwrap());
        assert_eq!(list_by_space(&conn, "sp1").unwrap().len(), 0);
        assert!(restore(&conn, &note.id).unwrap());
        assert_eq!(list_by_space(&conn, "sp1").unwrap().len(), 1);
        assert!(delete_permanent(&conn, &note.id).unwrap());
        assert!(get_by_id(&conn, &note.id).unwrap().is_none());
    }

    #[test]
    fn test_move_and_duplicate() {
        let conn = setup();
        conn.execute(
            "INSERT INTO spaces (id, name) VALUES ('sp2', 'Space 2')",
            [],
        )
        .unwrap();
        let note = create(&conn, "sp1").unwrap();
        assert!(move_to_space(&conn, &note.id, "sp2").unwrap());
        let moved = get_by_id(&conn, &note.id).unwrap().unwrap();
        assert_eq!(moved.space_id, "sp2");
        let dup = duplicate(&conn, &note.id).unwrap();
        assert_eq!(dup.space_id, "sp2");
    }

    #[test]
    fn test_search() {
        let conn = setup();
        let n1 = create(&conn, "sp1").unwrap();
        update(
            &conn,
            &n1.id,
            Some("Recipe"),
            Some("Pasta carbonara with eggs"),
            Some("Pasta"),
            None,
        )
        .unwrap();
        let n2 = create(&conn, "sp1").unwrap();
        update(
            &conn,
            &n2.id,
            Some("Shopping"),
            Some("Buy eggs and milk"),
            Some("Buy eggs"),
            None,
        )
        .unwrap();
        let results = search(&conn, "eggs", None, 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_list_recent() {
        let conn = setup();
        create(&conn, "sp1").unwrap();
        create(&conn, "sp1").unwrap();
        assert_eq!(list_recent(&conn, None, 5).unwrap().len(), 2);
    }
}
