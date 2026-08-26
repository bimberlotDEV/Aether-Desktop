use rusqlite::{Connection, Transaction};
use std::{
    fs,
    io::{BufReader, Read},
    path::{Component, Path, PathBuf},
};

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
    // Migration 004: AI conversations
    (
        "004_ai",
        "
        CREATE TABLE IF NOT EXISTS ai_conversations (
            id                      TEXT PRIMARY KEY NOT NULL,
            space_id                TEXT REFERENCES spaces(id) ON DELETE SET NULL,
            title                   TEXT NOT NULL DEFAULT 'New conversation',
            provider                TEXT NOT NULL DEFAULT 'deepseek',
            model                   TEXT NOT NULL DEFAULT 'deepseek-chat',
            system_context_version  INTEGER NOT NULL DEFAULT 1,
            archived_at             TEXT,
            created_at              TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at              TEXT NOT NULL DEFAULT (datetime('now')),
            last_opened_at          TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_ai_conversations_space ON ai_conversations(space_id);
        CREATE INDEX IF NOT EXISTS idx_ai_conversations_archived ON ai_conversations(archived_at);
        CREATE INDEX IF NOT EXISTS idx_ai_conversations_updated ON ai_conversations(updated_at);

        CREATE TABLE IF NOT EXISTS ai_messages (
            id                  TEXT PRIMARY KEY NOT NULL,
            conversation_id     TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
            role                TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
            content             TEXT NOT NULL DEFAULT '',
            status              TEXT NOT NULL DEFAULT 'complete' CHECK(status IN ('pending', 'streaming', 'complete', 'error', 'cancelled')),
            provider_message_id TEXT,
            error_code          TEXT,
            metadata_json       TEXT,
            created_at          TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_ai_messages_conversation ON ai_messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_ai_messages_created ON ai_messages(created_at);

        CREATE TABLE IF NOT EXISTS ai_context_items (
            id              TEXT PRIMARY KEY NOT NULL,
            conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
            entity_type     TEXT NOT NULL,
            entity_id       TEXT NOT NULL,
            context_mode    TEXT NOT NULL DEFAULT 'attached',
            added_at        TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_ai_context_conversation ON ai_context_items(conversation_id);
        ",
    ),
    // Migration 005: Tasks
    (
        "005_tasks",
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id              TEXT PRIMARY KEY NOT NULL,
            space_id        TEXT REFERENCES spaces(id) ON DELETE SET NULL,
            parent_task_id  TEXT REFERENCES tasks(id) ON DELETE CASCADE,
            title           TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 200),
            description     TEXT NOT NULL DEFAULT '',
            status          TEXT NOT NULL DEFAULT 'inbox' CHECK(status IN ('inbox', 'planned', 'in_progress', 'done')),
            priority        TEXT NOT NULL DEFAULT 'none' CHECK(priority IN ('none', 'low', 'medium', 'high')),
            due_date        TEXT CHECK(due_date IS NULL OR (length(due_date) = 10 AND due_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]')),
            tags_json       TEXT NOT NULL DEFAULT '[]',
            completed_at    TEXT,
            archived_at     TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
            CHECK(parent_task_id IS NULL OR parent_task_id <> id),
            CHECK((status = 'done' AND completed_at IS NOT NULL) OR (status <> 'done' AND completed_at IS NULL))
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_space ON tasks(space_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_parent ON tasks(parent_task_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
        CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
        CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
        CREATE INDEX IF NOT EXISTS idx_tasks_archived ON tasks(archived_at);
        ",
    ),
    // Migration 006: Vault
    (
        "006_vault",
        "
        CREATE TABLE IF NOT EXISTS vault_items (
            id              TEXT PRIMARY KEY NOT NULL,
            space_id        TEXT REFERENCES spaces(id) ON DELETE SET NULL,
            storage_mode    TEXT NOT NULL CHECK(storage_mode IN ('linked', 'managed')),
            display_title   TEXT NOT NULL CHECK(length(trim(display_title)) BETWEEN 1 AND 200),
            original_name   TEXT NOT NULL CHECK(length(trim(original_name)) BETWEEN 1 AND 255),
            stored_path     TEXT NOT NULL UNIQUE CHECK(length(trim(stored_path)) > 0),
            media_type      TEXT NOT NULL DEFAULT 'application/octet-stream',
            size_bytes      INTEGER NOT NULL CHECK(size_bytes >= 0),
            tags_json       TEXT NOT NULL DEFAULT '[]',
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_vault_items_space ON vault_items(space_id);
        CREATE INDEX IF NOT EXISTS idx_vault_items_mode ON vault_items(storage_mode);
        CREATE INDEX IF NOT EXISTS idx_vault_items_title ON vault_items(display_title);
        CREATE INDEX IF NOT EXISTS idx_vault_items_updated ON vault_items(updated_at);
        ",
    ),
    // Migration 007: AI production hardening
    (
        "007_ai_hardening",
        "
        UPDATE ai_conversations
        SET model = 'deepseek-v4-flash'
        WHERE model IN ('deepseek-chat', 'deepseek-reasoner');

        CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_context_unique
        ON ai_context_items(conversation_id, entity_type, entity_id);
        ",
    ),
    // Migration 008: Explicit Memory
    (
        "008_memory",
        "
        CREATE TABLE IF NOT EXISTS memory_items (
            id          TEXT PRIMARY KEY NOT NULL,
            space_id    TEXT REFERENCES spaces(id) ON DELETE CASCADE,
            title       TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 200),
            content     TEXT NOT NULL CHECK(length(trim(content)) BETWEEN 1 AND 20000),
            reason      TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 500),
            category    TEXT NOT NULL CHECK(category IN ('preference', 'decision', 'recurring_context', 'terminology', 'goal', 'constraint')),
            source      TEXT NOT NULL DEFAULT 'user' CHECK(source = 'user'),
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_memory_items_space ON memory_items(space_id);
        CREATE INDEX IF NOT EXISTS idx_memory_items_category ON memory_items(category);
        CREATE INDEX IF NOT EXISTS idx_memory_items_updated ON memory_items(updated_at);

        CREATE TRIGGER IF NOT EXISTS memory_items_context_cleanup AFTER DELETE ON memory_items BEGIN
            DELETE FROM ai_context_items WHERE entity_type = 'memory' AND entity_id = old.id;
        END;
        ",
    ),
    // Migration 009: Explicit Context Sources and indexed file metadata
    (
        "009_context_sources",
        "
        CREATE TABLE IF NOT EXISTS sources (
            id               TEXT PRIMARY KEY NOT NULL,
            root_path        TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(root_path)) > 0),
            display_name     TEXT NOT NULL CHECK(length(trim(display_name)) BETWEEN 1 AND 200),
            space_id         TEXT REFERENCES spaces(id) ON DELETE SET NULL,
            scan_status      TEXT NOT NULL DEFAULT 'never' CHECK(scan_status IN ('never', 'scanning', 'complete', 'error')),
            last_scan_at     TEXT,
            last_error       TEXT,
            created_at       TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_sources_space ON sources(space_id);
        CREATE INDEX IF NOT EXISTS idx_sources_status ON sources(scan_status);

        CREATE TABLE IF NOT EXISTS indexed_files (
            id               TEXT PRIMARY KEY NOT NULL,
            source_id        TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
            relative_path    TEXT NOT NULL COLLATE NOCASE CHECK(length(trim(relative_path)) > 0),
            filename         TEXT NOT NULL CHECK(length(trim(filename)) > 0),
            extension        TEXT,
            size_bytes       INTEGER NOT NULL CHECK(size_bytes >= 0),
            created_at_fs    INTEGER,
            modified_at_fs   INTEGER,
            state            TEXT NOT NULL DEFAULT 'present' CHECK(state IN ('present', 'removed')),
            first_seen_at    TEXT NOT NULL DEFAULT (datetime('now')),
            last_seen_at     TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at       TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(source_id, relative_path)
        );

        CREATE INDEX IF NOT EXISTS idx_indexed_files_source_state ON indexed_files(source_id, state);
        CREATE INDEX IF NOT EXISTS idx_indexed_files_name ON indexed_files(filename);
        CREATE INDEX IF NOT EXISTS idx_indexed_files_modified ON indexed_files(modified_at_fs);
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

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let sql = format!("PRAGMA table_info({table})");
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Failed to inspect legacy table '{table}': {error}"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("Failed to inspect legacy table '{table}': {error}"))?;
    for value in columns {
        if value.map_err(|error| format!("Failed to inspect legacy table '{table}': {error}"))?
            == column
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Default)]
struct LegacyVaultFiles {
    created_files: Vec<PathBuf>,
    created_directories: Vec<PathBuf>,
}

impl LegacyVaultFiles {
    fn rollback(&self) {
        for file in self.created_files.iter().rev() {
            let _ = fs::remove_file(file);
        }
        for directory in self.created_directories.iter().rev() {
            let _ = fs::remove_dir(directory);
        }
    }
}

fn safe_path_component(value: &str) -> bool {
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn truncate_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn files_equal(left: &Path, right: &Path) -> Result<bool, String> {
    let left_file = fs::File::open(left)
        .map_err(|error| format!("Failed to inspect legacy Vault source: {error}"))?;
    let right_file = fs::File::open(right)
        .map_err(|error| format!("Failed to inspect migrated Vault target: {error}"))?;
    if left_file
        .metadata()
        .map_err(|error| format!("Failed to inspect legacy Vault source: {error}"))?
        .len()
        != right_file
            .metadata()
            .map_err(|error| format!("Failed to inspect migrated Vault target: {error}"))?
            .len()
    {
        return Ok(false);
    }

    let mut left = BufReader::new(left_file);
    let mut right = BufReader::new(right_file);
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    loop {
        let left_read = left
            .read(&mut left_buffer)
            .map_err(|error| format!("Failed to read legacy Vault source: {error}"))?;
        let right_read = right
            .read(&mut right_buffer)
            .map_err(|error| format!("Failed to read migrated Vault target: {error}"))?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn database_directory(conn: &Connection) -> Result<Option<PathBuf>, String> {
    let path: String = conn
        .query_row(
            "SELECT file FROM pragma_database_list WHERE name = 'main'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Failed to resolve legacy database path: {error}"))?;
    if path.is_empty() {
        return Ok(None);
    }
    Path::new(&path)
        .parent()
        .map(|parent| Some(parent.to_path_buf()))
        .ok_or_else(|| "Legacy database has no parent directory".to_string())
}

fn prepare_legacy_vault_files(tx: &Transaction) -> Result<LegacyVaultFiles, String> {
    if !table_has_column(tx, "vault_items", "stored_name")?
        || table_has_column(tx, "vault_items", "storage_mode")?
    {
        return Ok(LegacyVaultFiles::default());
    }

    let count: i64 = tx
        .query_row("SELECT COUNT(*) FROM vault_items", [], |row| row.get(0))
        .map_err(|error| format!("Failed to count legacy Vault items: {error}"))?;
    if count == 0 {
        return Ok(LegacyVaultFiles::default());
    }

    let data_directory = database_directory(tx)?
        .ok_or_else(|| "Legacy Vault files require a file-backed database".to_string())?
        .canonicalize()
        .map_err(|error| format!("Failed to resolve Aether data directory: {error}"))?;
    let vault_root = data_directory
        .join("vault")
        .canonicalize()
        .map_err(|error| format!("Legacy Vault storage is unavailable: {error}"))?;
    if vault_root.parent() != Some(data_directory.as_path()) {
        return Err("Legacy Vault storage escaped the Aether data directory".to_string());
    }

    let items_directory = vault_root.join("items");
    let items_existed = items_directory.exists();
    fs::create_dir_all(&items_directory)
        .map_err(|error| format!("Failed to create current Vault storage: {error}"))?;
    let items_directory = items_directory
        .canonicalize()
        .map_err(|error| format!("Failed to resolve current Vault storage: {error}"))?;
    if items_directory.parent() != Some(vault_root.as_path()) {
        return Err("Current Vault items directory escaped managed storage".to_string());
    }

    let mut rows = tx
        .prepare("SELECT id, original_name, stored_name, size_bytes FROM vault_items ORDER BY id")
        .map_err(|error| format!("Failed to inspect legacy Vault records: {error}"))?;
    let rows = rows
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|error| format!("Failed to inspect legacy Vault records: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Failed to read legacy Vault record: {error}"))?;

    let mut prepared = LegacyVaultFiles::default();
    if !items_existed {
        prepared.created_directories.push(items_directory.clone());
    }

    let result = (|| {
        for (id, original_name, stored_name, recorded_size) in rows {
            if !safe_path_component(&id) || !safe_path_component(&stored_name) {
                return Err(format!(
                    "Legacy Vault item '{id}' contains an unsafe storage name"
                ));
            }
            if recorded_size < 0 {
                return Err(format!("Legacy Vault item '{id}' has an invalid size"));
            }
            let original_name = truncate_chars(original_name.trim(), 255);
            if original_name.is_empty() || !safe_path_component(&original_name) {
                return Err(format!("Legacy Vault item '{id}' has an invalid filename"));
            }

            let source = vault_root
                .join(&stored_name)
                .canonicalize()
                .map_err(|error| format!("Legacy Vault item '{id}' is unavailable: {error}"))?;
            if source.parent() != Some(vault_root.as_path()) || !source.is_file() {
                return Err(format!("Legacy Vault item '{id}' escaped managed storage"));
            }
            let actual_size = source
                .metadata()
                .map_err(|error| format!("Failed to inspect legacy Vault item '{id}': {error}"))?
                .len();
            if actual_size != recorded_size as u64 {
                return Err(format!(
                    "Legacy Vault item '{id}' size does not match its stored metadata"
                ));
            }

            let item_directory = items_directory.join(&id);
            let item_directory_existed = item_directory.exists();
            fs::create_dir_all(&item_directory).map_err(|error| {
                format!("Failed to create storage for Vault item '{id}': {error}")
            })?;
            let item_directory = item_directory.canonicalize().map_err(|error| {
                format!("Failed to resolve storage for Vault item '{id}': {error}")
            })?;
            if item_directory.parent() != Some(items_directory.as_path()) {
                return Err(format!(
                    "Vault item '{id}' storage escaped the items directory"
                ));
            }
            if !item_directory_existed {
                prepared.created_directories.push(item_directory.clone());
            }

            let target = item_directory.join(&original_name);
            if target.exists() {
                let target = target.canonicalize().map_err(|error| {
                    format!("Failed to resolve migrated Vault item '{id}': {error}")
                })?;
                if target.parent() != Some(item_directory.as_path())
                    || !target.is_file()
                    || !files_equal(&source, &target)?
                {
                    return Err(format!(
                        "Vault item '{id}' already has a different migration target"
                    ));
                }
                continue;
            }

            let partial = item_directory.join(".legacy-migration.partial");
            if partial.exists() {
                fs::remove_file(&partial).map_err(|error| {
                    format!("Failed to clear an incomplete Vault migration for '{id}': {error}")
                })?;
            }
            let copy_result = (|| {
                let mut source_file = fs::File::open(&source)
                    .map_err(|error| format!("Failed to open legacy Vault item '{id}': {error}"))?;
                let mut partial_file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&partial)
                    .map_err(|error| {
                        format!("Failed to create migration copy for Vault item '{id}': {error}")
                    })?;
                std::io::copy(&mut source_file, &mut partial_file)
                    .map_err(|error| format!("Failed to copy legacy Vault item '{id}': {error}"))?;
                partial_file.sync_all().map_err(|error| {
                    format!("Failed to flush migrated Vault item '{id}': {error}")
                })?;
                Ok(())
            })();
            if let Err(error) = copy_result {
                let _ = fs::remove_file(&partial);
                return Err(error);
            }
            if let Err(error) = fs::rename(&partial, &target) {
                let _ = fs::remove_file(&partial);
                return Err(format!(
                    "Failed to finalize legacy Vault item '{id}': {error}"
                ));
            }
            prepared.created_files.push(target);
        }
        Ok(())
    })();

    if let Err(error) = result {
        prepared.rollback();
        return Err(error);
    }
    Ok(prepared)
}

/// Upgrade schemas shipped by the pre-0.3 personal-beta builds. Those builds used
/// different migration names, so `CREATE TABLE IF NOT EXISTS` cannot repair their
/// incompatible Tasks, Memory, and Vault tables by itself.
fn upgrade_legacy_personal_beta(tx: &Transaction) -> Result<LegacyVaultFiles, String> {
    if table_has_column(tx, "tasks", "due_at")? && !table_has_column(tx, "tasks", "parent_task_id")?
    {
        tx.execute_batch(
            "ALTER TABLE tasks RENAME TO tasks_legacy_pre_031;
             CREATE TABLE tasks (
                id              TEXT PRIMARY KEY NOT NULL,
                space_id        TEXT REFERENCES spaces(id) ON DELETE SET NULL,
                parent_task_id  TEXT REFERENCES tasks(id) ON DELETE CASCADE,
                title           TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 200),
                description     TEXT NOT NULL DEFAULT '',
                status          TEXT NOT NULL DEFAULT 'inbox' CHECK(status IN ('inbox', 'planned', 'in_progress', 'done')),
                priority        TEXT NOT NULL DEFAULT 'none' CHECK(priority IN ('none', 'low', 'medium', 'high')),
                due_date        TEXT CHECK(due_date IS NULL OR (length(due_date) = 10 AND due_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]')),
                tags_json       TEXT NOT NULL DEFAULT '[]',
                completed_at    TEXT,
                archived_at     TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
                CHECK(parent_task_id IS NULL OR parent_task_id <> id),
                CHECK((status = 'done' AND completed_at IS NOT NULL) OR (status <> 'done' AND completed_at IS NULL))
             );
             INSERT INTO tasks
                (id, space_id, title, description, status, priority, due_date, completed_at,
                 archived_at, created_at, updated_at)
             SELECT id, space_id,
                    CASE WHEN trim(title) = '' THEN 'Untitled task' ELSE substr(trim(title), 1, 200) END,
                    description,
                    CASE WHEN status = 'done' THEN 'done' ELSE 'inbox' END,
                    CASE priority WHEN 'low' THEN 'low' WHEN 'high' THEN 'high' WHEN 'normal' THEN 'medium' ELSE 'none' END,
                    CASE WHEN due_at IS NOT NULL AND substr(due_at, 1, 10) GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
                         THEN substr(due_at, 1, 10) ELSE NULL END,
                    CASE WHEN status = 'done' THEN coalesce(completed_at, updated_at, created_at, datetime('now')) ELSE NULL END,
                    archived_at, created_at, updated_at
             FROM tasks_legacy_pre_031;
             DROP TABLE tasks_legacy_pre_031;",
        )
        .map_err(|error| format!("Legacy Tasks upgrade failed: {error}"))?;
    }

    if table_has_column(tx, "memory_items", "scope")?
        && !table_has_column(tx, "memory_items", "title")?
    {
        tx.execute_batch(
            "ALTER TABLE memory_items RENAME TO memory_items_legacy_pre_031;
             CREATE TABLE memory_items (
                id          TEXT PRIMARY KEY NOT NULL,
                space_id    TEXT REFERENCES spaces(id) ON DELETE CASCADE,
                title       TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 200),
                content     TEXT NOT NULL CHECK(length(trim(content)) BETWEEN 1 AND 20000),
                reason      TEXT NOT NULL CHECK(length(trim(reason)) BETWEEN 1 AND 500),
                category    TEXT NOT NULL CHECK(category IN ('preference', 'decision', 'recurring_context', 'terminology', 'goal', 'constraint')),
                source      TEXT NOT NULL DEFAULT 'user' CHECK(source = 'user'),
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
             );
             INSERT INTO memory_items
                (id, space_id, title, content, reason, category, source, created_at, updated_at)
             SELECT id, space_id,
                    CASE WHEN active = 0 THEN 'Imported inactive memory' ELSE 'Imported memory' END,
                    trim(content), 'Migrated from an earlier Aether version',
                    'recurring_context', 'user', created_at, updated_at
             FROM memory_items_legacy_pre_031;
             DROP TABLE memory_items_legacy_pre_031;",
        )
        .map_err(|error| format!("Legacy Memory upgrade failed: {error}"))?;
    }

    if table_has_column(tx, "vault_items", "stored_name")?
        && !table_has_column(tx, "vault_items", "storage_mode")?
    {
        let prepared_files = prepare_legacy_vault_files(tx)?;
        tx.execute_batch(
            "ALTER TABLE vault_items RENAME TO vault_items_legacy_pre_031;
             CREATE TABLE vault_items (
                id              TEXT PRIMARY KEY NOT NULL,
                space_id        TEXT REFERENCES spaces(id) ON DELETE SET NULL,
                storage_mode    TEXT NOT NULL CHECK(storage_mode IN ('linked', 'managed')),
                display_title   TEXT NOT NULL CHECK(length(trim(display_title)) BETWEEN 1 AND 200),
                original_name   TEXT NOT NULL CHECK(length(trim(original_name)) BETWEEN 1 AND 255),
                stored_path     TEXT NOT NULL UNIQUE CHECK(length(trim(stored_path)) > 0),
                media_type      TEXT NOT NULL DEFAULT 'application/octet-stream',
                size_bytes      INTEGER NOT NULL CHECK(size_bytes >= 0),
                tags_json       TEXT NOT NULL DEFAULT '[]',
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
             );
             INSERT INTO vault_items
                (id, space_id, storage_mode, display_title, original_name, stored_path,
                 media_type, size_bytes, tags_json, created_at, updated_at)
             SELECT id, space_id, 'managed', substr(trim(original_name), 1, 200),
                    substr(trim(original_name), 1, 255),
                    'items/' || id || '/' || substr(trim(original_name), 1, 255),
                    'application/octet-stream', max(size_bytes, 0), '[]', created_at, created_at
             FROM vault_items_legacy_pre_031;
             DROP TABLE vault_items_legacy_pre_031;",
        )
        .map_err(|error| {
            prepared_files.rollback();
            format!("Legacy Vault upgrade failed: {error}")
        })?;
        return Ok(prepared_files);
    }

    Ok(LegacyVaultFiles::default())
}

pub fn run(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Transaction error: {}", e))?;

    ensure_migrations_table(&tx)?;
    let legacy_vault_files = upgrade_legacy_personal_beta(&tx)?;

    let apply_result = (|| {
        for (name, sql) in MIGRATIONS {
            if !is_applied(&tx, name)? {
                apply_migration(&tx, name, sql)?;
                log::info!("Applied migration: {}", name);
            }
        }
        Ok(())
    })();
    if let Err(error) = apply_result {
        legacy_vault_files.rollback();
        return Err(error);
    }

    if let Err(error) = tx.commit() {
        legacy_vault_files.rollback();
        return Err(format!("Migration commit error: {error}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repositories;
    use rusqlite::Connection;
    use tempfile::TempDir;

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
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"vault_items".to_string()));
        assert!(tables.contains(&"memory_items".to_string()));
        assert!(tables.contains(&"sources".to_string()));
        assert!(tables.contains(&"indexed_files".to_string()));
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        run(&conn).unwrap();
    }

    #[test]
    fn upgrades_current_workspace_with_context_tables() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        conn.execute_batch(
            "DROP TABLE indexed_files;
             DROP TABLE sources;
             DELETE FROM _migrations WHERE name = '009_context_sources';",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO spaces (id, name) VALUES ('kept', 'Kept workspace')",
            [],
        )
        .unwrap();
        run(&conn).unwrap();
        let kept: String = conn
            .query_row("SELECT name FROM spaces WHERE id='kept'", [], |row| {
                row.get(0)
            })
            .unwrap();
        let context_tables: i64 = conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('sources','indexed_files')", [], |row| row.get(0)).unwrap();
        assert_eq!(kept, "Kept workspace");
        assert_eq!(context_tables, 2);
    }

    #[test]
    fn test_upgrades_personal_beta_schema_without_losing_rows() {
        let temporary = TempDir::new().unwrap();
        let conn = Connection::open(temporary.path().join("aether.db")).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        run(&conn).unwrap();
        conn.execute_batch(
            "DROP TABLE tasks;
             DROP TABLE memory_items;
             DROP TABLE vault_items;
             DELETE FROM _migrations WHERE name IN ('005_tasks', '006_vault', '008_memory');
             INSERT INTO spaces (id, name) VALUES ('space-1', 'Legacy');
             CREATE TABLE tasks (
                id TEXT PRIMARY KEY NOT NULL, space_id TEXT NOT NULL, title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '', status TEXT NOT NULL, priority TEXT NOT NULL,
                due_at TEXT, sort_order INTEGER NOT NULL DEFAULT 0, archived_at TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT
             );
             CREATE TABLE memory_items (
                id TEXT PRIMARY KEY NOT NULL, scope TEXT NOT NULL, space_id TEXT,
                content TEXT NOT NULL, active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );
             CREATE TABLE vault_items (
                id TEXT PRIMARY KEY NOT NULL, space_id TEXT, original_name TEXT NOT NULL,
                stored_name TEXT NOT NULL UNIQUE, size_bytes INTEGER NOT NULL,
                created_at TEXT NOT NULL
             );
             INSERT INTO tasks VALUES
                ('task-1', 'space-1', 'Old task', '', 'done', 'normal', '2026-08-24T12:00:00Z', 0, NULL,
                 '2026-08-01 00:00:00', '2026-08-24 00:00:00', NULL);
             INSERT INTO tasks VALUES
                ('task-open', 'space-1', 'Open task', 'Keep working', 'open', 'low', 'not-a-date', 1,
                 '2026-08-20 00:00:00', '2026-08-01 00:00:00', '2026-08-24 00:00:00', NULL);
             INSERT INTO memory_items VALUES
                ('memory-1', 'space', 'space-1', 'Remember this', 1,
                 '2026-08-01 00:00:00', '2026-08-02 00:00:00');
             INSERT INTO memory_items VALUES
                ('memory-inactive', 'global', NULL, 'Keep this disabled memory', 0,
                 '2026-08-01 00:00:00', '2026-08-02 00:00:00');
             INSERT INTO vault_items VALUES
                ('vault-1', 'space-1', 'document.txt', 'legacy.bin', 12, '2026-08-01 00:00:00');",
        )
        .unwrap();
        let legacy_vault = temporary.path().join("vault");
        fs::create_dir(&legacy_vault).unwrap();
        fs::write(legacy_vault.join("legacy.bin"), b"legacy bytes").unwrap();

        run(&conn).unwrap();

        let task: (String, String, String, Option<String>) = conn
            .query_row(
                "SELECT status, priority, due_date, completed_at FROM tasks WHERE id = 'task-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(task.0, "done");
        assert_eq!(task.1, "medium");
        assert_eq!(task.2, "2026-08-24");
        assert!(task.3.is_some());
        let open_task = repositories::tasks::get_by_id(&conn, "task-open")
            .unwrap()
            .unwrap();
        assert_eq!(open_task.space_id.as_deref(), Some("space-1"));
        assert_eq!(open_task.status, "inbox");
        assert_eq!(open_task.priority, "low");
        assert_eq!(open_task.due_date, None);
        assert_eq!(
            open_task.archived_at.as_deref(),
            Some("2026-08-20 00:00:00")
        );
        assert_eq!(
            conn.query_row(
                "SELECT title FROM memory_items WHERE id = 'memory-1'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
            "Imported memory"
        );
        let inactive_memory: (Option<String>, String) = conn
            .query_row(
                "SELECT space_id, title FROM memory_items WHERE id = 'memory-inactive'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(inactive_memory, (None, "Imported inactive memory".into()));
        assert_eq!(
            conn.query_row(
                "SELECT stored_path FROM vault_items WHERE id = 'vault-1'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
            "items/vault-1/document.txt"
        );
        assert_eq!(
            fs::read(legacy_vault.join("items/vault-1/document.txt")).unwrap(),
            b"legacy bytes"
        );
        assert_eq!(
            fs::read(legacy_vault.join("legacy.bin")).unwrap(),
            b"legacy bytes"
        );
        assert!(repositories::vault::get_by_id(&conn, "vault-1")
            .unwrap()
            .is_some());
        assert!(repositories::tasks::get_by_id(&conn, "task-1")
            .unwrap()
            .is_some());
        assert!(repositories::memory::get_by_id(&conn, "memory-1")
            .unwrap()
            .is_some());
        assert_eq!(
            repositories::memory::get_by_id(&conn, "memory-1")
                .unwrap()
                .unwrap()
                .space_id
                .as_deref(),
            Some("space-1")
        );
        run(&conn).unwrap();
    }

    #[test]
    fn test_legacy_upgrade_rolls_back_schema_and_created_vault_copy() {
        let temporary = TempDir::new().unwrap();
        let conn = Connection::open(temporary.path().join("aether.db")).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        run(&conn).unwrap();
        conn.execute_batch(
            "DROP TABLE tasks;
             DROP TABLE memory_items;
             DROP TABLE vault_items;
             DELETE FROM _migrations WHERE name IN ('005_tasks', '006_vault', '008_memory');
             CREATE TABLE tasks (
                id TEXT PRIMARY KEY NOT NULL, space_id TEXT NOT NULL, title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '', status TEXT NOT NULL, priority TEXT NOT NULL,
                due_at TEXT, sort_order INTEGER NOT NULL DEFAULT 0, archived_at TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT
             );
             CREATE TABLE memory_items (
                id TEXT PRIMARY KEY NOT NULL, scope TEXT NOT NULL, space_id TEXT,
                content TEXT NOT NULL, active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
             );
             CREATE TABLE vault_items (
                id TEXT PRIMARY KEY NOT NULL, space_id TEXT, original_name TEXT NOT NULL,
                stored_name TEXT NOT NULL UNIQUE, size_bytes INTEGER NOT NULL,
                created_at TEXT NOT NULL
             );
             INSERT INTO vault_items VALUES
                ('vault-rollback', NULL, 'rollback.txt', 'rollback.bin', 8, '2026-08-01 00:00:00');
             CREATE TRIGGER block_current_migration BEFORE INSERT ON _migrations
             WHEN NEW.name = '005_tasks'
             BEGIN SELECT RAISE(ABORT, 'forced migration failure'); END;",
        )
        .unwrap();
        let legacy_vault = temporary.path().join("vault");
        fs::create_dir(&legacy_vault).unwrap();
        fs::write(legacy_vault.join("rollback.bin"), b"rollback").unwrap();

        assert!(run(&conn).is_err());
        assert!(table_has_column(&conn, "vault_items", "stored_name").unwrap());
        assert!(!table_has_column(&conn, "vault_items", "storage_mode").unwrap());
        assert!(!legacy_vault
            .join("items/vault-rollback/rollback.txt")
            .exists());
        assert_eq!(
            fs::read(legacy_vault.join("rollback.bin")).unwrap(),
            b"rollback"
        );

        conn.execute("DROP TRIGGER block_current_migration", [])
            .unwrap();
        let target_directory = legacy_vault.join("items/vault-rollback");
        fs::create_dir_all(&target_directory).unwrap();
        let target = target_directory.join("rollback.txt");
        fs::write(&target, b"different").unwrap();
        assert!(run(&conn).is_err());
        assert!(table_has_column(&conn, "vault_items", "stored_name").unwrap());
        assert_eq!(fs::read(&target).unwrap(), b"different");

        fs::write(&target, b"rollback").unwrap();
        run(&conn).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT stored_path FROM vault_items WHERE id = 'vault-rollback'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
            "items/vault-rollback/rollback.txt"
        );
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
    fn test_ai_hardening_index_exists() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        let index: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'index' AND name = 'idx_ai_context_unique'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index, "idx_ai_context_unique");
    }

    #[test]
    fn test_parent_column_exists() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        // Verify parent_space_id column exists
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(spaces)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(cols.contains(&"parent_space_id".to_string()));
        assert!(cols.contains(&"last_opened_at".to_string()));
    }

    #[test]
    fn test_task_columns_exist() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(tasks)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|result| result.ok())
            .collect();
        for expected in [
            "space_id",
            "parent_task_id",
            "status",
            "priority",
            "due_date",
            "tags_json",
            "completed_at",
            "archived_at",
        ] {
            assert!(cols.contains(&expected.to_string()));
        }
    }

    #[test]
    fn test_vault_columns_exist() {
        let conn = in_memory_db();
        run(&conn).unwrap();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(vault_items)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|result| result.ok())
            .collect();
        for expected in [
            "space_id",
            "storage_mode",
            "display_title",
            "original_name",
            "stored_path",
            "media_type",
            "size_bytes",
            "tags_json",
        ] {
            assert!(cols.contains(&expected.to_string()));
        }
    }
}
