use chrono::Utc;
use rusqlite::{backup::Backup, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub size_bytes: u64,
    pub created_at: String,
}

pub fn export(source: &Connection, destination: &Path) -> Result<BackupResult, String> {
    validate_destination(source, destination)?;

    let parent = destination
        .parent()
        .ok_or_else(|| "Backup destination has no parent directory".to_string())?;
    if !parent.is_dir() {
        return Err("Backup destination directory does not exist".to_string());
    }

    let partial = parent.join(format!(".aether-{}.partial", Uuid::now_v7()));
    let result = export_to_partial(source, &partial).and_then(|()| {
        finalize_with_rollback(&partial, destination)?;
        let size_bytes = std::fs::metadata(destination)
            .map_err(|error| format!("Could not inspect completed backup: {error}"))?
            .len();
        Ok(BackupResult {
            size_bytes,
            created_at: Utc::now().to_rfc3339(),
        })
    });

    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}

fn finalize_with_rollback(partial: &Path, destination: &Path) -> Result<(), String> {
    let previous = destination.with_file_name(format!(".aether-{}.previous", Uuid::now_v7()));
    let had_previous = destination.exists();
    if had_previous {
        std::fs::rename(destination, &previous).map_err(|error| {
            format!("Could not prepare existing backup for replacement: {error}")
        })?;
    }

    if let Err(error) = std::fs::rename(partial, destination) {
        if had_previous {
            std::fs::rename(&previous, destination).map_err(|restore_error| {
                format!(
                    "Could not finalize backup ({error}) or restore the previous backup ({restore_error})"
                )
            })?;
        }
        return Err(format!("Could not finalize backup: {error}"));
    }

    if had_previous {
        let _ = std::fs::remove_file(previous);
    }
    Ok(())
}

fn export_to_partial(source: &Connection, partial: &Path) -> Result<(), String> {
    let mut target = Connection::open(partial)
        .map_err(|error| format!("Could not create backup database: {error}"))?;
    {
        let backup = Backup::new(source, &mut target)
            .map_err(|error| format!("Could not start database backup: {error}"))?;
        backup
            .run_to_completion(128, Duration::from_millis(1), None)
            .map_err(|error| format!("Could not copy workspace database: {error}"))?;
    }

    target
        .execute_batch("DROP TABLE IF EXISTS secrets;")
        .map_err(|error| format!("Could not sanitize backup: {error}"))?;
    let integrity: String = target
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("Could not verify backup integrity: {error}"))?;
    if integrity != "ok" {
        return Err(format!("Backup integrity check failed: {integrity}"));
    }
    target
        .close()
        .map_err(|(_, error)| format!("Could not close backup database: {error}"))?;
    Ok(())
}

fn validate_destination(source: &Connection, destination: &Path) -> Result<(), String> {
    if !destination.is_absolute() {
        return Err("Backup destination must be an absolute path".to_string());
    }
    if destination.extension().and_then(|value| value.to_str()) != Some("db")
        || !destination
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(".aether-backup.db"))
    {
        return Err("Backup filename must end with .aether-backup.db".to_string());
    }

    let source_path: Option<PathBuf> = source
        .query_row(
            "SELECT file FROM pragma_database_list WHERE name = 'main'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    if source_path
        .as_deref()
        .is_some_and(|path| same_path(path, destination))
    {
        return Err("Backup destination cannot overwrite the live database".to_string());
    }
    Ok(())
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ai::credentials, db::migrations};
    use tempfile::tempdir;

    #[test]
    fn exports_workspace_data_without_credentials() {
        let directory = tempdir().unwrap();
        let source_path = directory.path().join("aether.db");
        let source = Connection::open(&source_path).unwrap();
        migrations::run(&source).unwrap();
        credentials::ensure_table(&source).unwrap();
        source
            .execute(
                "INSERT INTO spaces (id, name) VALUES ('space-1', 'Research')",
                [],
            )
            .unwrap();
        source
            .execute(
                "INSERT INTO secrets (key, value) VALUES ('ai_api_key', 'encrypted')",
                [],
            )
            .unwrap();

        let destination = directory.path().join("workspace.aether-backup.db");
        let result = export(&source, &destination).unwrap();
        assert!(result.size_bytes > 0);

        let backup = Connection::open(destination).unwrap();
        let space_name: String = backup
            .query_row("SELECT name FROM spaces WHERE id = 'space-1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(space_name, "Research");
        let secrets_count: i64 = backup
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'secrets'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(secrets_count, 0);
        let integrity: String = backup
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");

        source
            .execute(
                "UPDATE spaces SET name = 'Locked update' WHERE id = 'space-1'",
                [],
            )
            .unwrap();
        assert!(export(
            &source,
            &directory.path().join("workspace.aether-backup.db")
        )
        .is_err());
        let preserved_name: String = backup
            .query_row("SELECT name FROM spaces WHERE id = 'space-1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(preserved_name, "Research");
        drop(backup);

        source
            .execute(
                "UPDATE spaces SET name = 'Updated' WHERE id = 'space-1'",
                [],
            )
            .unwrap();
        export(
            &source,
            &directory.path().join("workspace.aether-backup.db"),
        )
        .unwrap();
        let replaced =
            Connection::open(directory.path().join("workspace.aether-backup.db")).unwrap();
        let updated_name: String = replaced
            .query_row("SELECT name FROM spaces WHERE id = 'space-1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(updated_name, "Updated");
    }

    #[test]
    fn rejects_unsafe_destinations() {
        let directory = tempdir().unwrap();
        let source_path = directory.path().join("aether.db");
        let source = Connection::open(&source_path).unwrap();

        assert!(export(&source, Path::new("relative.aether-backup.db")).is_err());
        assert!(export(&source, &directory.path().join("wrong.db")).is_err());
        assert!(export(&source, &source_path).is_err());
    }
}
