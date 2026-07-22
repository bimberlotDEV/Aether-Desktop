use rusqlite::{Connection, Transaction};

const MIGRATIONS: &[(&str, &str)] = &[
    // Migration 001: Core tables
    (
        "001_init",
        "
        CREATE TABLE IF NOT EXISTS app_settings (
            key         TEXT PRIMARY KEY NOT NULL,
            value       TEXT NOT NULL,
            value_type  TEXT NOT NULL DEFAULT 'string',
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS user_profile (
            id                  TEXT PRIMARY KEY NOT NULL,
            display_name        TEXT,
            onboarding_completed INTEGER NOT NULL DEFAULT 0,
            created_at          TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS spaces (
            id              TEXT PRIMARY KEY NOT NULL,
            name            TEXT NOT NULL,
            description     TEXT,
            icon            TEXT,
            accent          TEXT,
            template_type   TEXT,
            favourite       INTEGER NOT NULL DEFAULT 0,
            archived_at     TEXT,
            sort_order      INTEGER NOT NULL DEFAULT 0,
            settings_json   TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_spaces_favourite ON spaces(favourite);
        CREATE INDEX IF NOT EXISTS idx_spaces_archived ON spaces(archived_at);
        CREATE INDEX IF NOT EXISTS idx_spaces_sort ON spaces(sort_order);

        CREATE TABLE IF NOT EXISTS module_instances (
            id              TEXT PRIMARY KEY NOT NULL,
            space_id        TEXT NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
            module_type     TEXT NOT NULL,
            title           TEXT,
            config_json     TEXT,
            layout_json     TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_module_instances_space ON module_instances(space_id);

        CREATE TABLE IF NOT EXISTS activity_events (
            id              TEXT PRIMARY KEY NOT NULL,
            event_type      TEXT NOT NULL,
            entity_type     TEXT,
            entity_id       TEXT,
            space_id        TEXT REFERENCES spaces(id) ON DELETE SET NULL,
            metadata_json   TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_activity_events_created ON activity_events(created_at);
        CREATE INDEX IF NOT EXISTS idx_activity_events_space ON activity_events(space_id);
        CREATE INDEX IF NOT EXISTS idx_activity_events_type ON activity_events(event_type);
        ",
    ),
    // Migration 002: Space hierarchy + tracking
    (
        "002_space_hierarchy",
        "
        ALTER TABLE spaces ADD COLUMN parent_space_id TEXT REFERENCES spaces(id) ON DELETE SET NULL;
        ALTER TABLE spaces ADD COLUMN last_opened_at TEXT;
        CREATE INDEX IF NOT EXISTS idx_spaces_parent ON spaces(parent_space_id);
        ",
    ),
    // Migration 003: Notes
    (
        "003_notes",
        "
        CREATE TABLE IF NOT EXISTS notes (
            id              TEXT PRIMARY KEY NOT NULL,
            space_id        TEXT NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
            title           TEXT NOT NULL DEFAULT 'Untitled note',
            content         TEXT NOT NULL DEFAULT '',
            content_format  TEXT NOT NULL DEFAULT 'markdown',
            excerpt         TEXT NOT NULL DEFAULT '',
            pinned          INTEGER NOT NULL DEFAULT 0,
            revision        INTEGER NOT NULL DEFAULT 1,
            archived_at     TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
            last_opened_at  TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_notes_space ON notes(space_id);
        CREATE INDEX IF NOT EXISTS idx_notes_archived ON notes(archived_at);
        CREATE INDEX IF NOT EXISTS idx_notes_pinned ON notes(pinned);
        CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at);
        CREATE INDEX IF NOT EXISTS idx_notes_last_opened ON notes(last_opened_at);

        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            title,
            content,
            excerpt,
            content='notes',
            content_rowid='rowid'
        );

        CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, title, content, excerpt)
            VALUES (new.rowid, new.title, new.content, new.excerpt);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content, excerpt)
            VALUES ('delete', old.rowid, old.title, old.content, old.excerpt);
        END;

        CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, title, content, excerpt)
            VALUES ('delete', old.rowid, old.title, old.content, old.excerpt);
            INSERT INTO notes_fts(rowid, title, content, excerpt)
            VALUES (new.rowid, new.title, new.content, new.excerpt);
        END;
        ",
    ),
];

fn ensure_migrations_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| format!("Failed to create migrations table: {}", e))?;
    Ok(())
}

fn is_applied(conn: &Connection, name: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM _migrations WHERE name = ?1")
        .map_err(|e| format!("Migration check error: {}", e))?;
    let count: i64 = stmt
        .query_row([name], |row| row.get(0))
        .map_err(|e| format!("Migration check error: {}", e))?;
    Ok(count > 0)
}

fn apply_migration(tx: &Transaction, name: &str, sql: &str) -> Result<(), String> {
    tx.execute_batch(sql)
        .map_err(|e| format!("Migration '{}' failed: {}", name, e))?;
    tx.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])
        .map_err(|e| format!("Failed to record migration '{}': {}", name, e))?;
    Ok(())
}

pub fn run(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Transaction error: {}", e))?;

    ensure_migrations_table(&tx)?;

    for (name, sql) in MIGRATIONS {
        if !is_applied(&tx, name)? {
            apply_migration(&tx, name, sql)?;
            log::info!("Applied migration: {}", name);
        }
    }

    tx.commit()
        .map_err(|e| format!("Migration commit error: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn
    }

    #[test]
    fn test_migrations_run() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"_migrations".to_string()));
        assert!(tables.contains(&"app_settings".to_string()));
        assert!(tables.contains(&"user_profile".to_string()));
        assert!(tables.contains(&"spaces".to_string()));
        assert!(tables.contains(&"module_instances".to_string()));
        assert!(tables.contains(&"activity_events".to_string()));
        assert!(tables.contains(&"notes".to_string()));
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        run(&conn).unwrap();
    }

    #[test]
    fn test_migration_records() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }

    #[test]
    fn test_notes_table_exists() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(notes)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(cols.contains(&"title".to_string()));
        assert!(cols.contains(&"content".to_string()));
        assert!(cols.contains(&"revision".to_string()));
    }
}
