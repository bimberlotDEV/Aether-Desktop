use chrono::{NaiveDateTime, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;

const DEFAULT_LIMIT: u32 = 30;
const MAX_LIMIT: u32 = 50;
const DOMAIN_LIMIT: u32 = 40;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub kind: String,
    pub entity_id: String,
    pub space_id: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub provenance: String,
    pub score: i64,
    pub updated_at: String,
    pub source_id: Option<String>,
    pub relative_path: Option<String>,
}

#[derive(Debug)]
struct Candidate {
    kind: String,
    entity_id: String,
    space_id: Option<String>,
    title: String,
    detail: String,
    provenance: String,
    updated_at: String,
    favourite: bool,
    pinned: bool,
    open_task: bool,
    source_id: Option<String>,
    relative_path: Option<String>,
}

pub fn search(
    conn: &Connection,
    query: &str,
    current_space_id: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let query = query.trim();
    if !(2..=200).contains(&query.chars().count()) {
        return Err("Search query must contain between 2 and 200 characters".to_string());
    }
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let like = like_pattern(query);
    let mut candidates = Vec::new();

    metadata_candidates(
        conn,
        "SELECT 'space', s.id, s.id, s.name, COALESCE(s.description, ''), 'Space', s.updated_at,
                s.favourite, 0, 0, NULL, NULL
         FROM spaces s
         WHERE s.archived_at IS NULL
           AND (lower(s.name) LIKE ?1 ESCAPE '\\' OR lower(COALESCE(s.description, '')) LIKE ?1 ESCAPE '\\')
         ORDER BY s.updated_at DESC, s.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;
    note_candidates(conn, query, &mut candidates)?;
    metadata_candidates(
        conn,
        "SELECT 'task', t.id, t.space_id, t.title, t.description,
                'Task · ' || replace(t.status, '_', ' '), t.updated_at, 0, 0,
                CASE WHEN t.status <> 'done' THEN 1 ELSE 0 END, NULL, NULL
         FROM tasks t LEFT JOIN spaces s ON s.id = t.space_id
         WHERE t.archived_at IS NULL AND (t.space_id IS NULL OR s.archived_at IS NULL)
           AND (lower(t.title) LIKE ?1 ESCAPE '\\' OR lower(t.description) LIKE ?1 ESCAPE '\\' OR lower(t.tags_json) LIKE ?1 ESCAPE '\\')
         ORDER BY t.updated_at DESC, t.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;
    metadata_candidates(
        conn,
        "SELECT 'vault', v.id, v.space_id, v.display_title, v.original_name,
                'Vault · ' || v.storage_mode, v.updated_at, 0, 0, 0, NULL, NULL
         FROM vault_items v LEFT JOIN spaces s ON s.id = v.space_id
         WHERE (v.space_id IS NULL OR s.archived_at IS NULL)
           AND (lower(v.display_title) LIKE ?1 ESCAPE '\\' OR lower(v.original_name) LIKE ?1 ESCAPE '\\' OR lower(v.tags_json) LIKE ?1 ESCAPE '\\')
         ORDER BY v.updated_at DESC, v.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;
    metadata_candidates(
        conn,
        "SELECT 'memory', m.id, m.space_id, m.title, m.content,
                'Memory · ' || replace(m.category, '_', ' '), m.updated_at, 0, 0, 0, NULL, NULL
         FROM memory_items m LEFT JOIN spaces s ON s.id = m.space_id
         WHERE (m.space_id IS NULL OR s.archived_at IS NULL)
           AND (lower(m.title) LIKE ?1 ESCAPE '\\' OR lower(m.content) LIKE ?1 ESCAPE '\\' OR lower(m.reason) LIKE ?1 ESCAPE '\\')
         ORDER BY m.updated_at DESC, m.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;
    metadata_candidates(
        conn,
        "SELECT 'conversation', c.id, c.space_id, c.title, c.provider || ' · ' || c.model,
                'AI conversation', c.updated_at, 0, 0, 0, NULL, NULL
         FROM ai_conversations c LEFT JOIN spaces s ON s.id = c.space_id
         WHERE c.archived_at IS NULL AND (c.space_id IS NULL OR s.archived_at IS NULL)
           AND lower(c.title) LIKE ?1 ESCAPE '\\'
         ORDER BY c.updated_at DESC, c.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;
    metadata_candidates(
        conn,
        "SELECT 'activity', a.id, a.space_id, replace(a.event_type, '_', ' '),
                COALESCE(a.metadata_json, ''), 'Activity', a.created_at, 0, 0, 0, NULL, NULL
         FROM activity_events a LEFT JOIN spaces s ON s.id = a.space_id
         WHERE (a.space_id IS NULL OR s.archived_at IS NULL)
           AND a.event_type NOT IN ('setting_changed')
           AND (lower(replace(a.event_type, '_', ' ')) LIKE ?1 ESCAPE '\\' OR lower(COALESCE(a.metadata_json, '')) LIKE ?1 ESCAPE '\\')
         ORDER BY a.created_at DESC, a.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;
    metadata_candidates(
        conn,
        "SELECT 'file', f.id, so.space_id, f.filename, f.relative_path,
                'Source · ' || so.display_name, f.updated_at, 0, 0, 0, so.id, f.relative_path
         FROM indexed_files f
         JOIN sources so ON so.id = f.source_id
         LEFT JOIN spaces s ON s.id = so.space_id
         WHERE f.state = 'present' AND (so.space_id IS NULL OR s.archived_at IS NULL)
           AND (lower(f.filename) LIKE ?1 ESCAPE '\\' OR lower(f.relative_path) LIKE ?1 ESCAPE '\\')
         ORDER BY f.updated_at DESC, f.id
         LIMIT ?2",
        &like,
        &mut candidates,
    )?;

    let normalized = query.to_lowercase();
    let mut results: Vec<SearchResult> = candidates
        .into_iter()
        .map(|candidate| rank(candidate, &normalized, current_space_id))
        .collect();
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.updated_at.cmp(&left.updated_at))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.entity_id.cmp(&right.entity_id))
    });
    results.truncate(limit as usize);
    Ok(results)
}

fn metadata_candidates(
    conn: &Connection,
    sql: &str,
    like: &str,
    candidates: &mut Vec<Candidate>,
) -> Result<(), String> {
    let mut statement = conn
        .prepare(sql)
        .map_err(|error| format!("Universal Search prepare error: {error}"))?;
    let rows = statement
        .query_map(params![like, DOMAIN_LIMIT], candidate_from_row)
        .map_err(|error| format!("Universal Search query error: {error}"))?;
    for row in rows {
        candidates.push(row.map_err(|error| format!("Universal Search row error: {error}"))?);
    }
    Ok(())
}

fn note_candidates(
    conn: &Connection,
    query: &str,
    candidates: &mut Vec<Candidate>,
) -> Result<(), String> {
    let Some(fts_query) = fts_query(query) else {
        return Ok(());
    };
    let mut statement = conn
        .prepare(
            "SELECT 'note', n.id, n.space_id, n.title, n.excerpt, 'Note', n.updated_at,
                    0, n.pinned, 0, NULL, NULL
             FROM notes n
             JOIN notes_fts fts ON n.rowid = fts.rowid
             JOIN spaces s ON s.id = n.space_id
             WHERE notes_fts MATCH ?1 AND n.archived_at IS NULL AND s.archived_at IS NULL
             ORDER BY bm25(notes_fts, 8.0, 1.0, 2.0)
             LIMIT ?2",
        )
        .map_err(|error| format!("Universal Search note prepare error: {error}"))?;
    let rows = statement
        .query_map(params![fts_query, DOMAIN_LIMIT], candidate_from_row)
        .map_err(|error| format!("Universal Search note query error: {error}"))?;
    for row in rows {
        candidates.push(row.map_err(|error| format!("Universal Search note row error: {error}"))?);
    }
    Ok(())
}

fn candidate_from_row(row: &rusqlite::Row) -> rusqlite::Result<Candidate> {
    Ok(Candidate {
        kind: row.get(0)?,
        entity_id: row.get(1)?,
        space_id: row.get(2)?,
        title: row.get(3)?,
        detail: row.get(4)?,
        provenance: row.get(5)?,
        updated_at: row.get(6)?,
        favourite: row.get::<_, i64>(7)? != 0,
        pinned: row.get::<_, i64>(8)? != 0,
        open_task: row.get::<_, i64>(9)? != 0,
        source_id: row.get(10)?,
        relative_path: row.get(11)?,
    })
}

fn rank(candidate: Candidate, query: &str, current_space_id: Option<&str>) -> SearchResult {
    let title = candidate.title.to_lowercase();
    let detail = candidate.detail.to_lowercase();
    let mut score = if title == query {
        1_000
    } else if title.starts_with(query) {
        800
    } else if title.contains(query) {
        600
    } else if detail == query {
        500
    } else {
        300
    };
    if query
        .split_whitespace()
        .all(|token| title.contains(token) || detail.contains(token))
    {
        score += 100;
    }
    if current_space_id.is_some() && candidate.space_id.as_deref() == current_space_id {
        score += 120;
    }
    if candidate.favourite {
        score += 90;
    }
    if candidate.pinned {
        score += 80;
    }
    if candidate.open_task {
        score += 60;
    }
    score += recency_score(&candidate.updated_at);

    SearchResult {
        kind: candidate.kind,
        entity_id: candidate.entity_id,
        space_id: candidate.space_id,
        title: candidate.title,
        subtitle: candidate.detail,
        provenance: candidate.provenance,
        score,
        updated_at: candidate.updated_at,
        source_id: candidate.source_id,
        relative_path: candidate.relative_path,
    }
}

fn recency_score(value: &str) -> i64 {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|updated| {
            let days = (Utc::now().naive_utc() - updated).num_days().max(0);
            (60 - days).max(0)
        })
        .unwrap_or(0)
}

fn like_pattern(query: &str) -> String {
    let escaped = query
        .to_lowercase()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

fn fts_query(query: &str) -> Option<String> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .filter_map(|token| {
            let clean: String = token
                .chars()
                .filter(|character| {
                    character.is_alphanumeric() || *character == '_' || *character == '-'
                })
                .collect();
            (!clean.is_empty()).then(|| format!("\"{}\"*", clean.replace('"', "\"\"")))
        })
        .take(8)
        .collect();
    (!tokens.is_empty()).then(|| tokens.join(" AND "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        migrations::run(&connection).unwrap();
        connection
    }

    #[test]
    fn searches_every_permitted_domain_and_excludes_inactive_rows() {
        let connection = setup();
        connection.execute("INSERT INTO spaces (id,name,description,favourite) VALUES ('s1','Research','Gradient work',1),('arch','Archived gradient',NULL,0)", []).unwrap();
        connection
            .execute(
                "UPDATE spaces SET archived_at=datetime('now') WHERE id='arch'",
                [],
            )
            .unwrap();
        connection.execute("INSERT INTO notes (id,space_id,title,content,excerpt,pinned) VALUES ('n1','s1','Learning','gradient descent','Model notes',1),('n2','arch','Hidden','gradient','',0)", []).unwrap();
        connection.execute("INSERT INTO tasks (id,space_id,title,description,status) VALUES ('t1','s1','Gradient assignment','Finish it','in_progress'),('t2','s1','Archived gradient task','','inbox')", []).unwrap();
        connection
            .execute(
                "UPDATE tasks SET archived_at=datetime('now') WHERE id='t2'",
                [],
            )
            .unwrap();
        connection.execute("INSERT INTO vault_items (id,space_id,storage_mode,display_title,original_name,stored_path,media_type,size_bytes) VALUES ('v1','s1','linked','Gradient paper','paper.pdf','C:/paper.pdf','application/pdf',1)", []).unwrap();
        connection.execute("INSERT INTO memory_items (id,space_id,title,content,reason,category) VALUES ('m1','s1','Gradient convention','Use blue','Owner choice','decision')", []).unwrap();
        connection.execute("INSERT INTO ai_conversations (id,space_id,title) VALUES ('c1','s1','Gradient explanation')", []).unwrap();
        connection.execute("INSERT INTO activity_events (id,event_type,space_id,metadata_json) VALUES ('a1','note_edited','s1','{\"title\":\"Gradient log\"}')", []).unwrap();
        connection.execute("INSERT INTO sources (id,root_path,display_name,space_id) VALUES ('so1','C:/Research','Research files','s1')", []).unwrap();
        connection.execute("INSERT INTO indexed_files (id,source_id,relative_path,filename,size_bytes) VALUES ('f1','so1','math/gradient.pdf','gradient.pdf',1),('f2','so1','old-gradient.pdf','old-gradient.pdf',1)", []).unwrap();
        connection
            .execute("UPDATE indexed_files SET state='removed' WHERE id='f2'", [])
            .unwrap();

        let results = search(&connection, "gradient", Some("s1"), Some(50)).unwrap();
        let kinds: Vec<&str> = results.iter().map(|result| result.kind.as_str()).collect();
        for kind in [
            "space",
            "note",
            "task",
            "vault",
            "memory",
            "conversation",
            "activity",
            "file",
        ] {
            assert!(kinds.contains(&kind), "missing {kind}: {kinds:?}");
        }
        assert!(!results.iter().any(|result| result.entity_id == "n2"
            || result.entity_id == "t2"
            || result.entity_id == "f2"));
        let file = results.iter().find(|result| result.kind == "file").unwrap();
        assert_eq!(file.relative_path.as_deref(), Some("math/gradient.pdf"));
        assert!(!file.subtitle.contains("C:/Research"));
    }

    #[test]
    fn ranking_rewards_current_space_favourites_pins_and_open_tasks() {
        let connection = setup();
        connection.execute("INSERT INTO spaces (id,name,favourite) VALUES ('current','Alpha',0),('other','Alpha',1)", []).unwrap();
        let results = search(&connection, "alpha", Some("current"), Some(10)).unwrap();
        assert_eq!(results[0].space_id.as_deref(), Some("current"));

        connection.execute("INSERT INTO notes (id,space_id,title,content,excerpt,pinned) VALUES ('pinned','current','Alpha note','','',1)", []).unwrap();
        connection.execute("INSERT INTO tasks (id,space_id,title,description,status,completed_at) VALUES ('open','current','Alpha note','','inbox',NULL),('done','current','Alpha note','','done',datetime('now'))", []).unwrap();
        let results = search(&connection, "alpha note", Some("current"), Some(10)).unwrap();
        assert!(
            results
                .iter()
                .find(|r| r.entity_id == "pinned")
                .unwrap()
                .score
                > 1_000
        );
        assert!(
            results
                .iter()
                .find(|r| r.entity_id == "open")
                .unwrap()
                .score
                > results
                    .iter()
                    .find(|r| r.entity_id == "done")
                    .unwrap()
                    .score
        );
    }

    #[test]
    fn special_characters_and_limits_are_safe() {
        let connection = setup();
        connection
            .execute("INSERT INTO spaces (id,name) VALUES ('s1','100%_safe')", [])
            .unwrap();
        assert_eq!(search(&connection, "%_", None, Some(500)).unwrap().len(), 1);
        assert!(search(&connection, "' OR 1=1 --", None, Some(10))
            .unwrap()
            .is_empty());
        assert!(search(&connection, "x", None, None).is_err());
    }
}
