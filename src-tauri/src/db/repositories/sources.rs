use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub root_path: String,
    pub display_name: String,
    pub space_id: Option<String>,
    pub scan_status: String,
    pub last_scan_at: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSourceInput {
    pub root_path: String,
    pub display_name: String,
    pub space_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IndexedFile {
    pub id: String,
    pub source_id: String,
    pub relative_path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at_fs: Option<i64>,
    pub modified_at_fs: Option<i64>,
    pub state: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub relative_path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at_fs: Option<i64>,
    pub modified_at_fs: Option<i64>,
}

#[derive(Debug)]
pub struct ScanSnapshot {
    pub files: Vec<ScannedFile>,
    pub skipped: u32,
    pub errors: u32,
    pub truncated: bool,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub source_id: String,
    pub scanned: u32,
    pub added: u32,
    pub changed: u32,
    pub renamed: u32,
    pub removed: u32,
    pub unchanged: u32,
    pub skipped: u32,
    pub errors: u32,
    pub truncated: bool,
    pub completed_at: String,
}

fn source_from_row(row: &rusqlite::Row) -> rusqlite::Result<Source> {
    Ok(Source {
        id: row.get(0)?,
        root_path: row.get(1)?,
        display_name: row.get(2)?,
        space_id: row.get(3)?,
        scan_status: row.get(4)?,
        last_scan_at: row.get(5)?,
        last_error: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn file_from_row(row: &rusqlite::Row) -> rusqlite::Result<IndexedFile> {
    Ok(IndexedFile {
        id: row.get(0)?,
        source_id: row.get(1)?,
        relative_path: row.get(2)?,
        filename: row.get(3)?,
        extension: row.get(4)?,
        size_bytes: row.get(5)?,
        created_at_fs: row.get(6)?,
        modified_at_fs: row.get(7)?,
        state: row.get(8)?,
        first_seen_at: row.get(9)?,
        last_seen_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn validate_space(conn: &Connection, space_id: Option<&str>) -> Result<(), String> {
    if let Some(id) = space_id {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM spaces WHERE id = ?1 AND archived_at IS NULL)",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Source Space validation error: {e}"))?;
        if !exists {
            return Err("Source Space does not exist or is archived".into());
        }
    }
    Ok(())
}

pub fn create(conn: &Connection, input: &CreateSourceInput) -> Result<Source, String> {
    let name = input.display_name.trim();
    if name.is_empty() || name.chars().count() > 200 {
        return Err("Source name must contain 1 to 200 characters".into());
    }
    validate_space(conn, input.space_id.as_deref())?;
    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO sources (id, root_path, display_name, space_id) VALUES (?1, ?2, ?3, ?4)",
        params![id, input.root_path, name, input.space_id],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "That directory is already an Aether Source".into()
        } else {
            format!("Source create error: {e}")
        }
    })?;
    get(conn, &id)?.ok_or_else(|| "Source missing after create".into())
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Source>, String> {
    conn.query_row(
        "SELECT id, root_path, display_name, space_id, scan_status, last_scan_at, last_error, created_at, updated_at FROM sources WHERE id = ?1",
        [id], source_from_row,
    ).optional().map_err(|e| format!("Source get error: {e}"))
}

pub fn list(conn: &Connection) -> Result<Vec<Source>, String> {
    let mut statement = conn.prepare("SELECT id, root_path, display_name, space_id, scan_status, last_scan_at, last_error, created_at, updated_at FROM sources ORDER BY display_name COLLATE NOCASE")
        .map_err(|e| format!("Source list error: {e}"))?;
    let rows = statement
        .query_map([], source_from_row)
        .map_err(|e| format!("Source list error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Source row error: {e}"))?;
    Ok(rows)
}

pub fn update_space(
    conn: &Connection,
    id: &str,
    space_id: Option<&str>,
) -> Result<Option<Source>, String> {
    validate_space(conn, space_id)?;
    conn.execute(
        "UPDATE sources SET space_id = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![space_id, id],
    )
    .map_err(|e| format!("Source update error: {e}"))?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool, String> {
    conn.execute("DELETE FROM sources WHERE id = ?1", [id])
        .map(|n| n > 0)
        .map_err(|e| format!("Source revoke error: {e}"))
}

pub fn mark_scanning(conn: &Connection, id: &str) -> Result<(), String> {
    let count = conn.execute("UPDATE sources SET scan_status = 'scanning', last_error = NULL, updated_at = datetime('now') WHERE id = ?1", [id])
        .map_err(|e| format!("Source scan-state error: {e}"))?;
    if count == 0 {
        return Err("Source does not exist".into());
    }
    Ok(())
}

pub fn mark_error(conn: &Connection, id: &str, error: &str) -> Result<(), String> {
    conn.execute("UPDATE sources SET scan_status = 'error', last_error = ?1, updated_at = datetime('now') WHERE id = ?2", params![error, id])
        .map_err(|e| format!("Source error-state error: {e}"))?;
    Ok(())
}

pub fn list_files(
    conn: &Connection,
    source_id: &str,
    include_removed: bool,
) -> Result<Vec<IndexedFile>, String> {
    let sql = if include_removed {
        "SELECT id, source_id, relative_path, filename, extension, size_bytes, created_at_fs, modified_at_fs, state, first_seen_at, last_seen_at, updated_at FROM indexed_files WHERE source_id = ?1 ORDER BY state, relative_path COLLATE NOCASE LIMIT 2000"
    } else {
        "SELECT id, source_id, relative_path, filename, extension, size_bytes, created_at_fs, modified_at_fs, state, first_seen_at, last_seen_at, updated_at FROM indexed_files WHERE source_id = ?1 AND state = 'present' ORDER BY relative_path COLLATE NOCASE LIMIT 2000"
    };
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("Indexed file list error: {e}"))?;
    let rows = stmt
        .query_map([source_id], file_from_row)
        .map_err(|e| format!("Indexed file list error: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Indexed file row error: {e}"))?;
    Ok(rows)
}

type Signature = (i64, Option<i64>);

pub fn apply_snapshot(
    conn: &Connection,
    source_id: &str,
    snapshot: ScanSnapshot,
) -> Result<ScanResult, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Source snapshot transaction error: {e}"))?;
    let existing = list_files(&tx, source_id, true)?;
    let mut by_path: HashMap<String, IndexedFile> = existing
        .into_iter()
        .map(|f| (f.relative_path.to_lowercase(), f))
        .collect();
    let observed: HashSet<String> = snapshot
        .files
        .iter()
        .map(|f| f.relative_path.to_lowercase())
        .collect();
    let mut removed: Vec<IndexedFile> = if snapshot.truncated {
        vec![]
    } else {
        by_path
            .values()
            .filter(|f| f.state == "present" && !observed.contains(&f.relative_path.to_lowercase()))
            .cloned()
            .collect()
    };
    let new_indexes: Vec<usize> = snapshot
        .files
        .iter()
        .enumerate()
        .filter(|(_, f)| !by_path.contains_key(&f.relative_path.to_lowercase()))
        .map(|(i, _)| i)
        .collect();
    let mut old_sigs: HashMap<Signature, Vec<usize>> = HashMap::new();
    for (i, f) in removed.iter().enumerate() {
        old_sigs
            .entry((f.size_bytes, f.modified_at_fs))
            .or_default()
            .push(i);
    }
    let mut new_sigs: HashMap<Signature, Vec<usize>> = HashMap::new();
    for i in &new_indexes {
        let f = &snapshot.files[*i];
        new_sigs
            .entry((f.size_bytes, f.modified_at_fs))
            .or_default()
            .push(*i);
    }
    let mut renamed_new = HashSet::new();
    let mut renamed_old = HashSet::new();
    let now = Utc::now().to_rfc3339();
    let mut result = ScanResult {
        source_id: source_id.into(),
        scanned: snapshot.files.len() as u32,
        added: 0,
        changed: 0,
        renamed: 0,
        removed: 0,
        unchanged: 0,
        skipped: snapshot.skipped,
        errors: snapshot.errors,
        truncated: snapshot.truncated,
        completed_at: now.clone(),
    };
    for (sig, olds) in &old_sigs {
        if olds.len() == 1 {
            if let Some(news) = new_sigs.get(sig).filter(|v| v.len() == 1) {
                let old_index = olds[0];
                let new_index = news[0];
                let old = &removed[old_index];
                let new = &snapshot.files[new_index];
                tx.execute("UPDATE indexed_files SET relative_path=?1, filename=?2, extension=?3, created_at_fs=?4, modified_at_fs=?5, state='present', last_seen_at=?6, updated_at=?6 WHERE id=?7",
                    params![new.relative_path, new.filename, new.extension, new.created_at_fs, new.modified_at_fs, now, old.id]).map_err(|e| format!("Indexed rename error: {e}"))?;
                renamed_old.insert(old_index);
                renamed_new.insert(new_index);
                result.renamed += 1;
            }
        }
    }
    for (index, file) in snapshot.files.iter().enumerate() {
        if renamed_new.contains(&index) {
            continue;
        }
        if let Some(old) = by_path.remove(&file.relative_path.to_lowercase()) {
            let reappeared = old.state == "removed";
            let changed = old.size_bytes != file.size_bytes
                || old.modified_at_fs != file.modified_at_fs
                || old.created_at_fs != file.created_at_fs;
            tx.execute("UPDATE indexed_files SET filename=?1, extension=?2, size_bytes=?3, created_at_fs=?4, modified_at_fs=?5, state='present', last_seen_at=?6, updated_at=CASE WHEN ?7 THEN ?6 ELSE updated_at END WHERE id=?8",
                params![file.filename, file.extension, file.size_bytes, file.created_at_fs, file.modified_at_fs, now, changed, old.id]).map_err(|e| format!("Indexed update error: {e}"))?;
            if reappeared {
                result.added += 1;
            } else if changed {
                result.changed += 1;
            } else {
                result.unchanged += 1;
            }
        } else {
            tx.execute("INSERT INTO indexed_files (id, source_id, relative_path, filename, extension, size_bytes, created_at_fs, modified_at_fs, last_seen_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![Uuid::now_v7().to_string(), source_id, file.relative_path, file.filename, file.extension, file.size_bytes, file.created_at_fs, file.modified_at_fs, now]).map_err(|e| format!("Indexed insert error: {e}"))?;
            result.added += 1;
        }
    }
    for (index, old) in removed.drain(..).enumerate() {
        if renamed_old.contains(&index) {
            continue;
        }
        tx.execute(
            "UPDATE indexed_files SET state='removed', updated_at=?1 WHERE id=?2",
            params![now, old.id],
        )
        .map_err(|e| format!("Indexed removal error: {e}"))?;
        result.removed += 1;
    }
    let warning = if snapshot.truncated {
        Some("Scan limit reached; removals were not finalized")
    } else if snapshot.errors > 0 {
        Some("Some entries could not be inspected")
    } else {
        None
    };
    tx.execute("UPDATE sources SET scan_status='complete', last_scan_at=?1, last_error=?2, updated_at=?1 WHERE id=?3", params![now, warning, source_id]).map_err(|e| format!("Source completion error: {e}"))?;
    tx.commit()
        .map_err(|e| format!("Source snapshot commit error: {e}"))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    fn setup() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&c).unwrap();
        c
    }
    fn file(path: &str, size: i64, modified: i64) -> ScannedFile {
        ScannedFile {
            relative_path: path.into(),
            filename: path.rsplit('/').next().unwrap().into(),
            extension: None,
            size_bytes: size,
            created_at_fs: None,
            modified_at_fs: Some(modified),
        }
    }
    #[test]
    fn creates_updates_renames_removes_and_cascades_without_touching_files() {
        let c = setup();
        let s = create(
            &c,
            &CreateSourceInput {
                root_path: "C:\\Work".into(),
                display_name: "Work".into(),
                space_id: None,
            },
        )
        .unwrap();
        let first = apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![file("a.txt", 1, 1), file("gone.txt", 2, 2)],
                skipped: 0,
                errors: 0,
                truncated: false,
            },
        )
        .unwrap();
        assert_eq!(first.added, 2);
        let second = apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![file("a.txt", 3, 3), file("renamed.txt", 2, 2)],
                skipped: 0,
                errors: 0,
                truncated: false,
            },
        )
        .unwrap();
        assert_eq!((second.changed, second.renamed, second.removed), (1, 1, 0));
        assert_eq!(list_files(&c, &s.id, false).unwrap().len(), 2);
        assert!(delete(&c, &s.id).unwrap());
        assert!(list_files(&c, &s.id, true).unwrap().is_empty());
    }
    #[test]
    fn truncated_snapshot_does_not_remove_unseen_rows() {
        let c = setup();
        let s = create(
            &c,
            &CreateSourceInput {
                root_path: "C:\\Data".into(),
                display_name: "Data".into(),
                space_id: None,
            },
        )
        .unwrap();
        apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![file("keep.txt", 1, 1)],
                skipped: 0,
                errors: 0,
                truncated: false,
            },
        )
        .unwrap();
        let r = apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![],
                skipped: 0,
                errors: 0,
                truncated: true,
            },
        )
        .unwrap();
        assert_eq!(r.removed, 0);
        assert_eq!(list_files(&c, &s.id, false).unwrap().len(), 1);
    }

    #[test]
    fn removed_file_can_reappear_without_unique_path_failure() {
        let c = setup();
        let s = create(
            &c,
            &CreateSourceInput {
                root_path: "C:\\Return".into(),
                display_name: "Return".into(),
                space_id: None,
            },
        )
        .unwrap();
        apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![file("back.txt", 1, 1)],
                skipped: 0,
                errors: 0,
                truncated: false,
            },
        )
        .unwrap();
        apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![],
                skipped: 0,
                errors: 0,
                truncated: false,
            },
        )
        .unwrap();
        let result = apply_snapshot(
            &c,
            &s.id,
            ScanSnapshot {
                files: vec![file("back.txt", 2, 2)],
                skipped: 0,
                errors: 0,
                truncated: false,
            },
        )
        .unwrap();
        assert_eq!(result.added, 1);
        assert_eq!(list_files(&c, &s.id, false).unwrap().len(), 1);
    }

    #[test]
    fn rejects_duplicate_source_roots_case_insensitively() {
        let c = setup();
        create(
            &c,
            &CreateSourceInput {
                root_path: "C:\\Unique".into(),
                display_name: "One".into(),
                space_id: None,
            },
        )
        .unwrap();
        let duplicate = create(
            &c,
            &CreateSourceInput {
                root_path: "c:\\unique".into(),
                display_name: "Two".into(),
                space_id: None,
            },
        );
        assert_eq!(
            duplicate.unwrap_err(),
            "That directory is already an Aether Source"
        );
    }
}
