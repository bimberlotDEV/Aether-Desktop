use crate::ai::credentials;
use crate::db::migrations;
use crate::db::repositories::vault::VaultItem;
use crate::vault;
use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::{backup::Backup, params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const ARCHIVE_FORMAT: u32 = 1;
const MANIFEST_ENTRY: &str = "manifest.json";
const DATABASE_ENTRY: &str = "workspace.db";
const ARCHIVE_SUFFIX: &str = ".aether-backup";
const MAX_ENTRIES: usize = 10_002;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_PAYLOAD_BYTES: u64 = 20 * 1024 * 1024 * 1024;
const PREVIEW_TTL_MINUTES: i64 = 10;
const PENDING_FILE: &str = "pending-restore.json";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub size_bytes: u64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResult {
    pub size_bytes: u64,
    pub created_at: String,
    pub managed_file_count: usize,
    pub linked_file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCounts {
    pub spaces: u64,
    pub notes: u64,
    pub tasks: u64,
    pub memories: u64,
    pub conversations: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub token: String,
    pub created_at: String,
    pub app_version: String,
    pub archive_size_bytes: u64,
    pub managed_file_count: usize,
    pub linked_file_count: usize,
    pub expires_at: String,
    pub counts: RestoreCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Payload {
    path: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ManagedPayload {
    item_id: String,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArchiveManifest {
    format_version: u32,
    app_version: String,
    created_at: String,
    database: Payload,
    managed_files: Vec<ManagedPayload>,
    linked_file_count: usize,
    migrations: Vec<String>,
}

#[derive(Debug, Clone)]
struct ArchiveInspection {
    manifest: ArchiveManifest,
    counts: RestoreCounts,
}

#[derive(Debug, Clone)]
struct ArchiveFingerprint {
    path: PathBuf,
    size_bytes: u64,
    modified: SystemTime,
    sha256: String,
}

#[derive(Debug, Clone)]
struct RestoreAuthorization {
    fingerprint: ArchiveFingerprint,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Default)]
struct RestoreState {
    previews: HashMap<String, RestoreAuthorization>,
    executing: bool,
}

#[derive(Clone, Default)]
pub struct RestoreRuntime {
    state: Arc<Mutex<RestoreState>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PendingRestore {
    id: String,
    database_sha256: String,
    managed_files: Vec<ManagedPayload>,
    recovery_file: String,
}

/// Legacy database-only export retained for compatibility with Alpha 0.3/0.4.
pub fn export(source: &Connection, destination: &Path) -> Result<BackupResult, String> {
    validate_legacy_destination(source, destination)?;
    let parent = existing_parent(destination)?;
    let partial = parent.join(format!(".aether-{}.partial", Uuid::now_v7()));
    let result = export_database(source, &partial).and_then(|()| {
        finalize_with_rollback(&partial, destination)?;
        Ok(BackupResult {
            size_bytes: fs::metadata(destination)
                .map_err(|error| format!("Could not inspect completed backup: {error}"))?
                .len(),
            created_at: Utc::now().to_rfc3339(),
        })
    });
    if result.is_err() {
        let _ = fs::remove_file(&partial);
    }
    result
}

pub fn export_archive(
    source: &Connection,
    vault_root: &Path,
    app_data_dir: &Path,
    destination: &Path,
) -> Result<ArchiveResult, String> {
    validate_archive_destination(source, app_data_dir, destination)?;
    write_archive(source, vault_root, destination)
}

pub fn preview_restore(
    source_path: &Path,
    app_data_dir: &Path,
    runtime: &RestoreRuntime,
) -> Result<RestorePreview, String> {
    let fingerprint = validate_restore_source(source_path, app_data_dir)?;
    let preview_root = controlled_directory(app_data_dir, ".backup-preview")?;
    let temporary = preview_root.join(Uuid::now_v7().to_string());
    fs::create_dir(&temporary)
        .map_err(|error| format!("Could not create restore preview workspace: {error}"))?;
    let db_path = temporary.join(DATABASE_ENTRY);
    let inspected = inspect_archive(&fingerprint.path, &db_path, None);
    let _ = fs::remove_dir_all(&temporary);
    let inspected = inspected?;

    let token = Uuid::now_v7().to_string();
    let expires_at = Utc::now() + ChronoDuration::minutes(PREVIEW_TTL_MINUTES);
    let mut state = runtime
        .state
        .lock()
        .map_err(|error| format!("Restore preview lock error: {error}"))?;
    if state.executing {
        return Err("A workspace restore is already being prepared".to_string());
    }
    state
        .previews
        .retain(|_, value| value.expires_at > Utc::now());
    state.previews.insert(
        token.clone(),
        RestoreAuthorization {
            fingerprint: fingerprint.clone(),
            expires_at,
        },
    );

    Ok(RestorePreview {
        token,
        created_at: inspected.manifest.created_at,
        app_version: inspected.manifest.app_version,
        archive_size_bytes: fingerprint.size_bytes,
        managed_file_count: inspected.manifest.managed_files.len(),
        linked_file_count: inspected.manifest.linked_file_count,
        expires_at: expires_at.to_rfc3339(),
        counts: inspected.counts,
    })
}

pub fn stage_restore(
    source: &Connection,
    current_vault_root: &Path,
    app_data_dir: &Path,
    runtime: &RestoreRuntime,
    token: &str,
) -> Result<(), String> {
    let authorization = {
        let mut state = runtime
            .state
            .lock()
            .map_err(|error| format!("Restore approval lock error: {error}"))?;
        if state.executing {
            return Err("A workspace restore is already being prepared".to_string());
        }
        let authorization = state
            .previews
            .remove(token)
            .ok_or_else(|| "Restore preview expired or was already used".to_string())?;
        state.executing = true;
        authorization
    };
    let mut stage_to_clean = None;
    let staged = (|| {
        if authorization.expires_at <= Utc::now() {
            return Err("Restore preview expired; inspect the archive again".to_string());
        }
        let current_fingerprint = fingerprint(&authorization.fingerprint.path)?;
        if current_fingerprint.size_bytes != authorization.fingerprint.size_bytes
            || current_fingerprint.modified != authorization.fingerprint.modified
            || current_fingerprint.sha256 != authorization.fingerprint.sha256
        {
            return Err("The selected backup changed after preview; inspect it again".to_string());
        }

        let staging_root = controlled_directory(app_data_dir, "restore-staging")?;
        let id = Uuid::now_v7().to_string();
        let stage = staging_root.join(&id);
        fs::create_dir(&stage)
            .map_err(|error| format!("Could not create restore staging directory: {error}"))?;
        stage_to_clean = Some(stage.clone());
        let staged_db = stage.join(DATABASE_ENTRY);
        let staged_vault = stage.join("vault");
        let inspection = inspect_archive(
            &authorization.fingerprint.path,
            &staged_db,
            Some(&staged_vault),
        )?;
        let staged_conn = Connection::open(&staged_db)
            .map_err(|error| format!("Could not open staged restore database: {error}"))?;
        migrations::run(&staged_conn)?;
        preserve_current_secrets(source, &staged_conn)?;
        verify_database(&staged_conn, None, true)?;
        staged_conn
            .close()
            .map_err(|(_, error)| format!("Could not close staged database: {error}"))?;

        let recovery_root = controlled_directory(app_data_dir, "restore-recovery")?;
        let recovery_name = format!(
            "Aether-before-restore-{}-{}.aether-backup",
            Utc::now().format("%Y%m%d-%H%M%S"),
            Uuid::now_v7()
        );
        let recovery_path = recovery_root.join(&recovery_name);
        write_archive(source, current_vault_root, &recovery_path)?;

        let pending = PendingRestore {
            id: id.clone(),
            database_sha256: hash_file(&staged_db)?.1,
            managed_files: inspection.manifest.managed_files,
            recovery_file: recovery_name,
        };
        write_pending(app_data_dir, &pending)?;
        Ok(())
    })();
    if staged.is_err() {
        if let Some(stage) = stage_to_clean {
            let _ = fs::remove_dir_all(stage);
        }
        if let Ok(mut state) = runtime.state.lock() {
            state.executing = false;
        }
    }
    staged
}

pub fn cancel_restore(runtime: &RestoreRuntime, token: &str) -> Result<bool, String> {
    let mut state = runtime
        .state
        .lock()
        .map_err(|error| format!("Restore preview lock error: {error}"))?;
    if state.executing {
        return Ok(false);
    }
    Ok(state.previews.remove(token).is_some())
}

pub fn apply_pending_restore(app_data_dir: &Path) -> Result<bool, String> {
    let marker = app_data_dir.join(PENDING_FILE);
    if !marker.exists() {
        return Ok(false);
    }
    let bytes = fs::read(&marker)
        .map_err(|error| format!("Could not read pending restore marker: {error}"))?;
    if bytes.len() > MAX_MANIFEST_BYTES as usize {
        return Err("Pending restore marker is unexpectedly large".to_string());
    }
    let pending: PendingRestore = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Pending restore marker is invalid: {error}"))?;
    Uuid::parse_str(&pending.id).map_err(|_| "Pending restore id is invalid".to_string())?;
    if Path::new(&pending.recovery_file).components().count() != 1
        || !pending.recovery_file.ends_with(ARCHIVE_SUFFIX)
    {
        return Err("Pending restore recovery reference is invalid".to_string());
    }
    let recovery = app_data_dir
        .join("restore-recovery")
        .join(&pending.recovery_file);
    if !recovery.is_file() {
        return Err("Pending restore recovery archive is missing".to_string());
    }

    let staging_root = controlled_directory(app_data_dir, "restore-staging")?;
    let stage = staging_root.join(&pending.id);
    ensure_direct_child(&staging_root, &stage, "restore staging")?;
    let staged_db = stage.join(DATABASE_ENTRY);
    let staged_vault = stage.join("vault");
    let live_db = app_data_dir.join("aether.db");
    let live_vault = app_data_dir.join("vault");
    fs::create_dir_all(&live_vault)
        .map_err(|error| format!("Could not prepare live Vault directory: {error}"))?;
    let live_items = live_vault.join("items");
    let staged_items = staged_vault.join("items");
    let rollback = stage.join("rollback");

    if recover_interrupted_swap(
        &live_db,
        &live_items,
        &staged_db,
        &staged_items,
        &rollback,
        &pending,
    )? {
        if let Err(error) = fs::remove_file(&marker) {
            log::warn!("Could not remove completed restore marker: {error}");
        }
        if let Err(error) = fs::remove_dir_all(&stage) {
            log::warn!("Could not clean completed restore staging directory: {error}");
        }
        return Ok(true);
    }
    if hash_file(&staged_db)?.1 != pending.database_sha256 {
        return Err("Staged restore database changed before restart".to_string());
    }
    verify_staged_managed(&staged_vault, &pending.managed_files)?;
    let staged_conn = Connection::open(&staged_db)
        .map_err(|error| format!("Could not inspect staged restore database: {error}"))?;
    verify_database(&staged_conn, Some(&pending.managed_files), true)?;
    staged_conn
        .close()
        .map_err(|(_, error)| format!("Could not close staged restore database: {error}"))?;

    if !staged_items.exists() {
        fs::create_dir_all(&staged_items)
            .map_err(|error| format!("Could not prepare empty staged Vault: {error}"))?;
    }
    fs::create_dir(&rollback)
        .map_err(|error| format!("Could not create restore rollback directory: {error}"))?;

    let result = swap_live_paths(&live_db, &live_items, &staged_db, &staged_items, &rollback);
    if result.is_err() {
        return result.map(|()| false);
    }
    if let Err(error) = fs::remove_file(&marker) {
        log::warn!("Restore applied but pending marker cleanup failed: {error}");
    }
    if let Err(error) = fs::remove_dir_all(&stage) {
        log::warn!("Could not clean restore staging directory: {error}");
    }
    Ok(true)
}

fn recover_interrupted_swap(
    live_db: &Path,
    live_items: &Path,
    staged_db: &Path,
    staged_items: &Path,
    rollback: &Path,
    pending: &PendingRestore,
) -> Result<bool, String> {
    if live_db.is_file()
        && hash_file(live_db)?.1 == pending.database_sha256
        && verify_staged_managed(
            live_items
                .parent()
                .ok_or_else(|| "Live Vault path has no parent".to_string())?,
            &pending.managed_files,
        )
        .is_ok()
    {
        let conn = Connection::open(live_db)
            .map_err(|error| format!("Could not inspect completed restored database: {error}"))?;
        let verified = verify_database(&conn, Some(&pending.managed_files), true).is_ok();
        let _ = conn.close();
        if verified {
            return Ok(true);
        }
    }

    if !rollback.exists() {
        return Ok(false);
    }
    let old_db = rollback.join("aether.db");
    let old_wal = rollback.join("aether.db-wal");
    let old_shm = rollback.join("aether.db-shm");
    let old_items = rollback.join("items");
    let live_wal = live_db.with_file_name("aether.db-wal");
    let live_shm = live_db.with_file_name("aether.db-shm");

    if !staged_db.exists() && live_db.exists() {
        fs::rename(live_db, staged_db)
            .map_err(|error| format!("Could not recover interrupted staged database: {error}"))?;
    }
    if !staged_items.exists() && live_items.exists() {
        if let Some(parent) = staged_items.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("Could not recover interrupted staged Vault directory: {error}")
            })?;
        }
        fs::rename(live_items, staged_items)
            .map_err(|error| format!("Could not recover interrupted staged Vault: {error}"))?;
    }
    if old_db.exists() {
        fs::rename(&old_db, live_db)
            .map_err(|error| format!("Could not restore interrupted current database: {error}"))?;
    }
    if old_wal.exists() {
        fs::rename(&old_wal, &live_wal)
            .map_err(|error| format!("Could not restore interrupted database WAL: {error}"))?;
    }
    if old_shm.exists() {
        fs::rename(&old_shm, &live_shm)
            .map_err(|error| format!("Could not restore interrupted database SHM: {error}"))?;
    }
    if old_items.exists() {
        fs::rename(&old_items, live_items)
            .map_err(|error| format!("Could not restore interrupted managed Vault: {error}"))?;
    }
    fs::remove_dir(rollback)
        .map_err(|error| format!("Could not clear interrupted restore journal: {error}"))?;
    Ok(false)
}

fn swap_live_paths(
    live_db: &Path,
    live_items: &Path,
    staged_db: &Path,
    staged_items: &Path,
    rollback: &Path,
) -> Result<(), String> {
    let old_db = rollback.join("aether.db");
    let old_wal = rollback.join("aether.db-wal");
    let old_shm = rollback.join("aether.db-shm");
    let old_items = rollback.join("items");
    let live_wal = live_db.with_file_name("aether.db-wal");
    let live_shm = live_db.with_file_name("aether.db-shm");
    let mut moved_old_db = false;
    let mut moved_old_wal = false;
    let mut moved_old_shm = false;
    let mut moved_old_items = false;
    let mut moved_new_db = false;
    let mut moved_new_items = false;

    let operation: Result<(), String> = (|| {
        if live_db.exists() {
            fs::rename(live_db, &old_db)
                .map_err(|error| format!("Could not protect current database: {error}"))?;
            moved_old_db = true;
        }
        if live_wal.exists() {
            fs::rename(&live_wal, &old_wal)
                .map_err(|error| format!("Could not protect current database WAL: {error}"))?;
            moved_old_wal = true;
        }
        if live_shm.exists() {
            fs::rename(&live_shm, &old_shm)
                .map_err(|error| format!("Could not protect current database SHM: {error}"))?;
            moved_old_shm = true;
        }
        if live_items.exists() {
            fs::rename(live_items, &old_items)
                .map_err(|error| format!("Could not protect current managed Vault: {error}"))?;
            moved_old_items = true;
        }
        fs::rename(staged_db, live_db)
            .map_err(|error| format!("Could not activate restored database: {error}"))?;
        moved_new_db = true;
        fs::rename(staged_items, live_items)
            .map_err(|error| format!("Could not activate restored managed Vault: {error}"))?;
        moved_new_items = true;
        Ok(())
    })();

    if let Err(error) = operation {
        if moved_new_items {
            let _ = fs::rename(live_items, staged_items);
        }
        if moved_new_db {
            let _ = fs::rename(live_db, staged_db);
        }
        if moved_old_items {
            let _ = fs::rename(&old_items, live_items);
        }
        if moved_old_shm {
            let _ = fs::rename(&old_shm, &live_shm);
        }
        if moved_old_wal {
            let _ = fs::rename(&old_wal, &live_wal);
        }
        if moved_old_db {
            let _ = fs::rename(&old_db, live_db);
        }
        return Err(format!("{error}. The previous workspace was restored."));
    }
    Ok(())
}

fn write_archive(
    source: &Connection,
    vault_root: &Path,
    destination: &Path,
) -> Result<ArchiveResult, String> {
    let parent = existing_parent(destination)?;
    let id = Uuid::now_v7();
    let database_partial = parent.join(format!(".aether-{id}.db.partial"));
    let archive_partial = parent.join(format!(".aether-{id}.archive.partial"));
    let result = (|| {
        export_database(source, &database_partial)?;
        let database_conn = Connection::open(&database_partial)
            .map_err(|error| format!("Could not inspect archive database: {error}"))?;
        let migrations = applied_migrations(&database_conn)?;
        let (managed_items, linked_file_count) = archive_vault_items(&database_conn)?;
        database_conn
            .close()
            .map_err(|(_, error)| format!("Could not close archive database: {error}"))?;

        let (database_size, database_sha) = hash_file(&database_partial)?;
        let mut managed = Vec::with_capacity(managed_items.len());
        let mut sources = Vec::with_capacity(managed_items.len());
        for item in managed_items {
            let source_path = vault::resolve_item_path(vault_root, &item)?;
            let (size, sha256) = hash_file(&source_path)?;
            let size_i64 = i64::try_from(size)
                .map_err(|_| "Managed Vault file is too large to archive".to_string())?;
            if size_i64 != item.size_bytes {
                return Err(format!(
                    "Managed Vault item '{}' changed size; update or re-import it before backup",
                    item.display_title
                ));
            }
            let archive_path = managed_archive_path(&item)?;
            managed.push(ManagedPayload {
                item_id: item.id,
                payload: Payload {
                    path: archive_path,
                    size_bytes: size,
                    sha256,
                },
            });
            sources.push(source_path);
        }
        let manifest = ArchiveManifest {
            format_version: ARCHIVE_FORMAT,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now().to_rfc3339(),
            database: Payload {
                path: DATABASE_ENTRY.to_string(),
                size_bytes: database_size,
                sha256: database_sha,
            },
            managed_files: managed,
            linked_file_count,
            migrations,
        };
        create_zip(&archive_partial, &database_partial, &sources, &manifest)?;
        finalize_with_rollback(&archive_partial, destination)?;
        Ok(ArchiveResult {
            size_bytes: fs::metadata(destination)
                .map_err(|error| format!("Could not inspect completed archive: {error}"))?
                .len(),
            created_at: manifest.created_at,
            managed_file_count: manifest.managed_files.len(),
            linked_file_count,
        })
    })();
    let _ = fs::remove_file(&database_partial);
    if result.is_err() {
        let _ = fs::remove_file(&archive_partial);
    }
    result
}

fn create_zip(
    archive_path: &Path,
    database_path: &Path,
    managed_sources: &[PathBuf],
    manifest: &ArchiveManifest,
) -> Result<(), String> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(archive_path)
        .map_err(|error| format!("Could not create backup archive: {error}"))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    writer
        .start_file(MANIFEST_ENTRY, options)
        .map_err(|error| format!("Could not write archive manifest: {error}"))?;
    let manifest_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("Could not serialize archive manifest: {error}"))?;
    writer
        .write_all(&manifest_bytes)
        .map_err(|error| format!("Could not write archive manifest: {error}"))?;
    write_zip_payload(&mut writer, database_path, &manifest.database, options)?;
    for (payload, source) in manifest.managed_files.iter().zip(managed_sources) {
        write_zip_payload(&mut writer, source, &payload.payload, options)?;
    }
    let file = writer
        .finish()
        .map_err(|error| format!("Could not finalize backup archive: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("Could not flush backup archive: {error}"))
}

fn write_zip_payload(
    writer: &mut ZipWriter<File>,
    source: &Path,
    payload: &Payload,
    options: SimpleFileOptions,
) -> Result<(), String> {
    writer
        .start_file(&payload.path, options)
        .map_err(|error| format!("Could not add '{}' to archive: {error}", payload.path))?;
    let mut input = File::open(source)
        .map_err(|error| format!("Could not open archive source '{}': {error}", payload.path))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input.read(&mut buffer).map_err(|error| {
            format!("Could not read archive source '{}': {error}", payload.path)
        })?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or_else(|| "Archive payload size overflow".to_string())?;
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read]).map_err(|error| {
            format!(
                "Could not write archive payload '{}': {error}",
                payload.path
            )
        })?;
    }
    let digest = hex_digest(hasher.finalize());
    if size != payload.size_bytes || digest != payload.sha256 {
        return Err(format!(
            "Archive source '{}' changed while the backup was being created",
            payload.path
        ));
    }
    Ok(())
}

fn inspect_archive(
    archive_path: &Path,
    database_output: &Path,
    vault_output: Option<&Path>,
) -> Result<ArchiveInspection, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("Could not open backup archive: {error}"))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| format!("Selected file is not a readable Aether archive: {error}"))?;
    if archive.len() < 2 || archive.len() > MAX_ENTRIES {
        return Err("Backup archive contains an unsafe number of entries".to_string());
    }
    let mut names = HashSet::new();
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| format!("Could not inspect archive entry: {error}"))?;
        validate_entry_name(entry.name())?;
        if entry.is_dir() || !names.insert(entry.name().to_string()) {
            return Err("Backup archive contains a directory or duplicate entry".to_string());
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("Backup archive contains a symbolic link".to_string());
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| "Backup archive size overflow".to_string())?;
        if total > MAX_PAYLOAD_BYTES {
            return Err("Backup archive expands beyond Aether's safety limit".to_string());
        }
    }
    let manifest: ArchiveManifest = {
        let mut entry = archive
            .by_name(MANIFEST_ENTRY)
            .map_err(|_| "Backup archive manifest is missing".to_string())?;
        if entry.size() > MAX_MANIFEST_BYTES {
            return Err("Backup archive manifest is too large".to_string());
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Could not read archive manifest: {error}"))?;
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("Backup archive manifest is invalid: {error}"))?
    };
    validate_manifest(&manifest)?;
    let expected: HashSet<&str> = std::iter::once(MANIFEST_ENTRY)
        .chain(std::iter::once(manifest.database.path.as_str()))
        .chain(
            manifest
                .managed_files
                .iter()
                .map(|entry| entry.payload.path.as_str()),
        )
        .collect();
    if names.len() != expected.len() || !names.iter().all(|name| expected.contains(name.as_str())) {
        return Err("Backup archive contains missing or unexpected entries".to_string());
    }
    extract_verified_entry(&mut archive, &manifest.database, Some(database_output))?;
    for managed in &manifest.managed_files {
        let destination = if let Some(root) = vault_output {
            let (_, _, relative) = parse_managed_archive_path(&managed.payload.path)?;
            let target = root.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!("Could not create staged managed Vault directory: {error}")
                })?;
            }
            Some(target)
        } else {
            None
        };
        extract_verified_entry(&mut archive, &managed.payload, destination.as_deref())?;
    }
    let conn = Connection::open(database_output)
        .map_err(|error| format!("Could not inspect restored database: {error}"))?;
    let counts = verify_database(&conn, Some(&manifest.managed_files), false)?;
    let actual_migrations = applied_migrations(&conn)?;
    if actual_migrations != manifest.migrations {
        return Err("Backup migration metadata does not match its database".to_string());
    }
    let linked_count = count_rows_where(&conn, "vault_items", Some("storage_mode = 'linked'"))?;
    if linked_count as usize != manifest.linked_file_count {
        return Err("Backup linked Vault count does not match its database".to_string());
    }
    conn.close()
        .map_err(|(_, error)| format!("Could not close inspected database: {error}"))?;
    Ok(ArchiveInspection { manifest, counts })
}

fn extract_verified_entry(
    archive: &mut ZipArchive<File>,
    payload: &Payload,
    destination: Option<&Path>,
) -> Result<(), String> {
    let mut entry = archive
        .by_name(&payload.path)
        .map_err(|_| format!("Archive payload '{}' is missing", payload.path))?;
    if entry.size() != payload.size_bytes {
        return Err(format!(
            "Archive payload '{}' has the wrong size",
            payload.path
        ));
    }
    let mut output = match destination {
        Some(path) => Some(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|error| format!("Could not stage '{}': {error}", payload.path))?,
        ),
        None => None,
    };
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = entry
            .read(&mut buffer)
            .map_err(|error| format!("Could not read '{}': {error}", payload.path))?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or_else(|| "Archive payload size overflow".to_string())?;
        if size > payload.size_bytes {
            return Err(format!(
                "Archive payload '{}' exceeds its declared size",
                payload.path
            ));
        }
        hasher.update(&buffer[..read]);
        if let Some(file) = output.as_mut() {
            file.write_all(&buffer[..read])
                .map_err(|error| format!("Could not stage '{}': {error}", payload.path))?;
        }
    }
    if let Some(file) = output {
        file.sync_all()
            .map_err(|error| format!("Could not flush '{}': {error}", payload.path))?;
    }
    if size != payload.size_bytes || hex_digest(hasher.finalize()) != payload.sha256 {
        if let Some(path) = destination {
            let _ = fs::remove_file(path);
        }
        return Err(format!(
            "Archive payload '{}' failed verification",
            payload.path
        ));
    }
    Ok(())
}

fn validate_manifest(manifest: &ArchiveManifest) -> Result<(), String> {
    if manifest.format_version != ARCHIVE_FORMAT {
        return Err(format!(
            "Unsupported Aether backup format {}",
            manifest.format_version
        ));
    }
    if manifest.database.path != DATABASE_ENTRY {
        return Err("Backup database entry has an invalid path".to_string());
    }
    if manifest.app_version.is_empty()
        || manifest.app_version.len() > 64
        || chrono::DateTime::parse_from_rfc3339(&manifest.created_at).is_err()
    {
        return Err("Backup manifest contains invalid release metadata".to_string());
    }
    validate_sha(&manifest.database.sha256)?;
    let known: HashSet<&str> = migrations::known_names().collect();
    let mut seen_migrations = HashSet::new();
    if manifest
        .migrations
        .iter()
        .any(|name| !known.contains(name.as_str()) || !seen_migrations.insert(name.as_str()))
    {
        return Err("Backup was created by an unsupported or newer Aether schema".to_string());
    }
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    let mut total = manifest.database.size_bytes;
    for managed in &manifest.managed_files {
        Uuid::parse_str(&managed.item_id)
            .map_err(|_| "Backup contains an invalid managed Vault item id".to_string())?;
        let (id, _, _) = parse_managed_archive_path(&managed.payload.path)?;
        if id != managed.item_id
            || !ids.insert(managed.item_id.as_str())
            || !paths.insert(managed.payload.path.as_str())
        {
            return Err(
                "Backup contains duplicate or mismatched managed Vault entries".to_string(),
            );
        }
        validate_sha(&managed.payload.sha256)?;
        total = total
            .checked_add(managed.payload.size_bytes)
            .ok_or_else(|| "Backup manifest size overflow".to_string())?;
    }
    if total > MAX_PAYLOAD_BYTES || manifest.managed_files.len() + 2 > MAX_ENTRIES {
        return Err("Backup manifest exceeds Aether's safety limits".to_string());
    }
    Ok(())
}

fn verify_database(
    conn: &Connection,
    expected_managed: Option<&[ManagedPayload]>,
    allow_secrets: bool,
) -> Result<RestoreCounts, String> {
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("Could not verify backup database integrity: {error}"))?;
    if integrity != "ok" {
        return Err(format!(
            "Backup database integrity check failed: {integrity}"
        ));
    }
    let foreign_key_error: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(|error| format!("Could not verify backup relationships: {error}"))?;
    if foreign_key_error != 0 {
        return Err("Backup database contains broken relationships".to_string());
    }
    let secrets = table_exists(conn, "secrets")?;
    if secrets && !allow_secrets {
        return Err("Backup archive illegally contains credentials".to_string());
    }
    if let Some(expected) = expected_managed {
        let mut statement = conn
            .prepare("SELECT id, stored_path, size_bytes FROM vault_items WHERE storage_mode = 'managed' ORDER BY id")
            .map_err(|error| format!("Could not inspect managed Vault metadata: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|error| format!("Could not inspect managed Vault metadata: {error}"))?;
        let actual: BTreeMap<String, (String, i64)> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Could not read managed Vault metadata: {error}"))?
            .into_iter()
            .map(|(id, path, size)| (id, (path, size)))
            .collect();
        if actual.len() != expected.len() {
            return Err("Backup managed Vault metadata and file count differ".to_string());
        }
        for item in expected {
            let (_, filename, _) = parse_managed_archive_path(&item.payload.path)?;
            let expected_path = Path::new("items").join(&item.item_id).join(filename);
            let expected_path = expected_path
                .to_str()
                .ok_or_else(|| "Managed Vault path contains unsupported Unicode".to_string())?;
            let Some((stored_path, size_bytes)) = actual.get(&item.item_id) else {
                return Err("Backup is missing managed Vault metadata".to_string());
            };
            if Path::new(stored_path) != Path::new(expected_path)
                || u64::try_from(*size_bytes).ok() != Some(item.payload.size_bytes)
            {
                return Err("Backup managed Vault ownership metadata is inconsistent".to_string());
            }
        }
    }
    Ok(RestoreCounts {
        spaces: count_rows_where(conn, "spaces", None)?,
        notes: count_rows_where(conn, "notes", None)?,
        tasks: count_rows_where(conn, "tasks", None)?,
        memories: count_rows_where(conn, "memory_items", None)?,
        conversations: count_rows_where(conn, "ai_conversations", None)?,
    })
}

fn verify_staged_managed(root: &Path, managed: &[ManagedPayload]) -> Result<(), String> {
    let expected: HashSet<PathBuf> = managed
        .iter()
        .map(|item| parse_managed_archive_path(&item.payload.path).map(|(_, _, relative)| relative))
        .collect::<Result<_, _>>()?;
    let items_root = root.join("items");
    let mut actual = HashSet::new();
    if items_root.exists() {
        collect_regular_files(&items_root, &items_root, &mut actual)?;
    }
    if actual != expected {
        return Err("Staged managed Vault contains missing or unexpected files".to_string());
    }
    for item in managed {
        let (_, _, relative) = parse_managed_archive_path(&item.payload.path)?;
        let path = root.join(relative);
        let (size, sha) = hash_file(&path)?;
        if size != item.payload.size_bytes || sha != item.payload.sha256 {
            return Err("Staged managed Vault content changed before restart".to_string());
        }
    }
    Ok(())
}

fn collect_regular_files(
    root: &Path,
    directory: &Path,
    output: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("Could not inspect staged managed Vault: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("Could not inspect staged managed Vault: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Could not inspect staged Vault entry type: {error}"))?;
        if file_type.is_symlink() {
            return Err("Staged managed Vault contains a symbolic link".to_string());
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_regular_files(root, &path, output)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "Staged managed Vault path escaped its root".to_string())?;
            output.insert(Path::new("items").join(relative));
        } else {
            return Err("Staged managed Vault contains an unsupported entry".to_string());
        }
    }
    Ok(())
}

fn preserve_current_secrets(source: &Connection, staged: &Connection) -> Result<(), String> {
    credentials::ensure_table(staged)?;
    if !table_exists(source, "secrets")? {
        return Ok(());
    }
    let mut statement = source
        .prepare("SELECT key, value, created_at, updated_at FROM secrets")
        .map_err(|error| format!("Could not prepare local credential preservation: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| format!("Could not read local credentials for preservation: {error}"))?;
    for row in rows {
        let (key, value, created_at, updated_at) =
            row.map_err(|error| format!("Could not read local credential row: {error}"))?;
        staged
            .execute(
                "INSERT OR REPLACE INTO secrets (key, value, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![key, value, created_at, updated_at],
            )
            .map_err(|error| format!("Could not preserve local credential: {error}"))?;
    }
    Ok(())
}

fn archive_vault_items(conn: &Connection) -> Result<(Vec<VaultItem>, usize), String> {
    let mut statement = conn
        .prepare("SELECT id, space_id, storage_mode, display_title, original_name, stored_path, media_type, size_bytes, tags_json, created_at, updated_at FROM vault_items ORDER BY id")
        .map_err(|error| format!("Could not inspect Vault items for backup: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            let tags_json: String = row.get(8)?;
            let tags = serde_json::from_str(&tags_json).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            Ok(VaultItem {
                id: row.get(0)?,
                space_id: row.get(1)?,
                storage_mode: row.get(2)?,
                display_title: row.get(3)?,
                original_name: row.get(4)?,
                stored_path: row.get(5)?,
                media_type: row.get(6)?,
                size_bytes: row.get(7)?,
                tags,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|error| format!("Could not inspect Vault items for backup: {error}"))?;
    let mut managed = Vec::new();
    let mut linked = 0;
    for row in rows {
        let item = row.map_err(|error| format!("Could not read Vault backup row: {error}"))?;
        match item.storage_mode.as_str() {
            "managed" => managed.push(item),
            "linked" => linked += 1,
            _ => return Err("Vault contains an invalid storage mode".to_string()),
        }
    }
    Ok((managed, linked))
}

fn managed_archive_path(item: &VaultItem) -> Result<String, String> {
    let relative = Path::new(&item.stored_path);
    let mut components = relative.components();
    let items = matches!(components.next(), Some(Component::Normal(value)) if value == "items");
    let id =
        matches!(components.next(), Some(Component::Normal(value)) if value == item.id.as_str());
    let Some(Component::Normal(filename)) = components.next() else {
        return Err("Managed Vault item has an invalid stored path".to_string());
    };
    if !items || !id || components.next().is_some() {
        return Err("Managed Vault item escaped its ownership path".to_string());
    }
    let filename = filename
        .to_str()
        .ok_or_else(|| "Managed Vault filename contains unsupported Unicode".to_string())?;
    Ok(format!("vault/items/{}/{filename}", item.id))
}

fn parse_managed_archive_path(path: &str) -> Result<(String, String, PathBuf), String> {
    if path.contains('\\') {
        return Err("Backup entry uses an invalid path separator".to_string());
    }
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 4
        || parts[0] != "vault"
        || parts[1] != "items"
        || parts[2].is_empty()
        || parts[3].is_empty()
        || matches!(parts[3], "." | "..")
    {
        return Err("Backup managed Vault entry has an invalid path".to_string());
    }
    let mut filename_components = Path::new(parts[3]).components();
    if !matches!(filename_components.next(), Some(Component::Normal(_)))
        || filename_components.next().is_some()
        || parts[3].chars().count() > 255
        || parts[3].ends_with('.')
        || parts[3].ends_with(' ')
        || parts[3].chars().any(char::is_control)
    {
        return Err("Backup managed Vault filename is unsafe".to_string());
    }
    let relative = Path::new("items").join(parts[2]).join(parts[3]);
    Ok((parts[2].to_string(), parts[3].to_string(), relative))
}

fn export_database(source: &Connection, destination: &Path) -> Result<(), String> {
    let mut target = Connection::open(destination)
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
    verify_database(&target, None, false)?;
    target
        .close()
        .map_err(|(_, error)| format!("Could not close backup database: {error}"))
}

fn finalize_with_rollback(partial: &Path, destination: &Path) -> Result<(), String> {
    let previous = destination.with_file_name(format!(".aether-{}.previous", Uuid::now_v7()));
    let had_previous = destination.exists();
    if had_previous {
        let metadata = fs::symlink_metadata(destination)
            .map_err(|error| format!("Could not inspect existing backup: {error}"))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err("Existing backup destination is not a regular file".to_string());
        }
        fs::rename(destination, &previous).map_err(|error| {
            format!("Could not prepare existing backup for replacement: {error}")
        })?;
    }
    if let Err(error) = fs::rename(partial, destination) {
        if had_previous {
            fs::rename(&previous, destination).map_err(|restore_error| {
                format!(
                    "Could not finalize backup ({error}) or restore the previous backup ({restore_error})"
                )
            })?;
        }
        return Err(format!("Could not finalize backup: {error}"));
    }
    if had_previous {
        let _ = fs::remove_file(previous);
    }
    Ok(())
}

fn validate_legacy_destination(source: &Connection, destination: &Path) -> Result<(), String> {
    if !destination.is_absolute() {
        return Err("Backup destination must be an absolute path".to_string());
    }
    if !destination
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.ends_with(".aether-backup.db"))
    {
        return Err("Backup filename must end with .aether-backup.db".to_string());
    }
    if source_path(source)
        .as_deref()
        .is_some_and(|path| same_path(path, destination))
    {
        return Err("Backup destination cannot overwrite the live database".to_string());
    }
    Ok(())
}

fn validate_archive_destination(
    source: &Connection,
    app_data_dir: &Path,
    destination: &Path,
) -> Result<(), String> {
    if !destination.is_absolute()
        || !destination
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(ARCHIVE_SUFFIX))
    {
        return Err("Backup filename must end with .aether-backup".to_string());
    }
    if source_path(source)
        .as_deref()
        .is_some_and(|path| same_path(path, destination))
    {
        return Err("Backup destination cannot overwrite the live database".to_string());
    }
    let parent = existing_parent(destination)?;
    let app_data = fs::canonicalize(app_data_dir)
        .map_err(|error| format!("Could not resolve Aether data directory: {error}"))?;
    if parent.starts_with(app_data) {
        return Err("Choose a backup location outside Aether's live data directory".to_string());
    }
    Ok(())
}

fn validate_restore_source(path: &Path, app_data_dir: &Path) -> Result<ArchiveFingerprint, String> {
    if !path.is_absolute()
        || !path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(ARCHIVE_SUFFIX))
    {
        return Err("Restore requires an absolute .aether-backup file".to_string());
    }
    let fingerprint = fingerprint(path)?;
    let app_data = fs::canonicalize(app_data_dir)
        .map_err(|error| format!("Could not resolve Aether data directory: {error}"))?;
    if fingerprint.path.starts_with(app_data) {
        return Err("Choose a backup outside Aether's live data directory".to_string());
    }
    Ok(fingerprint)
}

fn fingerprint(path: &Path) -> Result<ArchiveFingerprint, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("Selected backup is unavailable: {error}"))?;
    let metadata = fs::metadata(&canonical)
        .map_err(|error| format!("Could not inspect selected backup: {error}"))?;
    if !metadata.is_file() {
        return Err("Selected backup is not a regular file".to_string());
    }
    let (size_bytes, sha256) = hash_file(&canonical)?;
    Ok(ArchiveFingerprint {
        path: canonical,
        size_bytes,
        modified: metadata
            .modified()
            .map_err(|error| format!("Could not read backup modification time: {error}"))?,
        sha256,
    })
}

fn hash_file(path: &Path) -> Result<(u64, String), String> {
    let mut file = File::open(path)
        .map_err(|error| format!("Could not open file for verification: {error}"))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read file for verification: {error}"))?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or_else(|| "File size overflow".to_string())?;
        hasher.update(&buffer[..read]);
    }
    Ok((size, hex_digest(hasher.finalize())))
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn validate_sha(value: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Backup manifest contains an invalid SHA-256 digest".to_string());
    }
    Ok(())
}

fn validate_entry_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('\\')
        || Path::new(name).is_absolute()
        || Path::new(name).components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("Backup archive contains an unsafe entry path".to_string());
    }
    Ok(())
}

fn write_pending(app_data_dir: &Path, pending: &PendingRestore) -> Result<(), String> {
    let marker = app_data_dir.join(PENDING_FILE);
    if marker.exists() {
        return Err("Another restore is already pending".to_string());
    }
    let partial = app_data_dir.join(format!(".{PENDING_FILE}.{}.partial", Uuid::now_v7()));
    let bytes = serde_json::to_vec_pretty(pending)
        .map_err(|error| format!("Could not serialize pending restore: {error}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)
        .map_err(|error| format!("Could not create pending restore marker: {error}"))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("Could not persist pending restore marker: {error}"))?;
    fs::rename(&partial, marker)
        .map_err(|error| format!("Could not activate pending restore marker: {error}"))
}

fn existing_parent(path: &Path) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Backup path has no parent directory".to_string())?;
    if !parent.is_dir() {
        return Err("Backup directory does not exist".to_string());
    }
    fs::canonicalize(parent).map_err(|error| format!("Could not resolve backup directory: {error}"))
}

fn controlled_directory(root: &Path, name: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(root)
        .map_err(|error| format!("Could not create Aether data directory: {error}"))?;
    let root = fs::canonicalize(root)
        .map_err(|error| format!("Could not resolve Aether data directory: {error}"))?;
    let directory = root.join(name);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create Aether {name} directory: {error}"))?;
    let directory = fs::canonicalize(&directory)
        .map_err(|error| format!("Could not resolve Aether {name} directory: {error}"))?;
    ensure_direct_child(&root, &directory, name)?;
    Ok(directory)
}

fn ensure_direct_child(parent: &Path, child: &Path, label: &str) -> Result<(), String> {
    let child_parent = child
        .parent()
        .ok_or_else(|| format!("{label} has no parent"))?;
    if child_parent != parent {
        return Err(format!("{label} escaped Aether storage"));
    }
    Ok(())
}

fn source_path(source: &Connection) -> Option<PathBuf> {
    source
        .query_row(
            "SELECT file FROM pragma_database_list WHERE name = 'main'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

fn same_path(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn applied_migrations(conn: &Connection) -> Result<Vec<String>, String> {
    if !table_exists(conn, "_migrations")? {
        return Err("Backup database has no migration history".to_string());
    }
    let mut statement = conn
        .prepare("SELECT name FROM _migrations ORDER BY rowid")
        .map_err(|error| format!("Could not inspect backup migrations: {error}"))?;
    let names = statement
        .query_map([], |row| row.get(0))
        .map_err(|error| format!("Could not inspect backup migrations: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not read backup migrations: {error}"))?;
    Ok(names)
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [name],
        |row| row.get(0),
    )
    .map_err(|error| format!("Could not inspect backup table '{name}': {error}"))
}

fn count_rows_where(
    conn: &Connection,
    table: &str,
    condition: Option<&str>,
) -> Result<u64, String> {
    let allowed = [
        "spaces",
        "notes",
        "tasks",
        "memory_items",
        "ai_conversations",
        "vault_items",
    ];
    if !allowed.contains(&table) {
        return Err("Unsupported backup count table".to_string());
    }
    let sql = match condition {
        Some("storage_mode = 'linked'") if table == "vault_items" => {
            "SELECT COUNT(*) FROM vault_items WHERE storage_mode = 'linked'".to_string()
        }
        None => format!("SELECT COUNT(*) FROM {table}"),
        _ => return Err("Unsupported backup count condition".to_string()),
    };
    let count: i64 = conn
        .query_row(&sql, [], |row| row.get(0))
        .map_err(|error| format!("Could not count backup {table}: {error}"))?;
    u64::try_from(count).map_err(|_| "Backup contains an invalid row count".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Connection, PathBuf) {
        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        fs::create_dir_all(&app_data).unwrap();
        let db_path = app_data.join("aether.db");
        let conn = Connection::open(db_path).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        credentials::ensure_table(&conn).unwrap();
        conn.execute(
            "INSERT INTO spaces (id, name) VALUES ('space-1', 'Research')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notes (id, space_id, title) VALUES ('note-1', 'space-1', 'Plan')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO secrets (key, value) VALUES ('ai_api_key', 'encrypted')",
            [],
        )
        .unwrap();
        let vault_root = app_data.join("vault");
        let source = temp.path().join("managed.txt");
        fs::write(&source, "managed bytes").unwrap();
        let prepared = vault::prepare_import(
            &vault_root,
            "018f47f4-7b7c-7d00-8000-000000000001",
            &source,
            "managed",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO vault_items (id, storage_mode, display_title, original_name, stored_path, media_type, size_bytes) VALUES (?1, 'managed', 'Managed', 'managed.txt', ?2, 'text/plain', ?3)",
            params!["018f47f4-7b7c-7d00-8000-000000000001", prepared.stored_path, prepared.size_bytes],
        ).unwrap();
        let linked = temp.path().join("linked.txt");
        fs::write(&linked, "linked bytes").unwrap();
        conn.execute(
            "INSERT INTO vault_items (id, storage_mode, display_title, original_name, stored_path, media_type, size_bytes) VALUES (?1, 'linked', 'Linked', 'linked.txt', ?2, 'text/plain', 12)",
            params!["018f47f4-7b7c-7d00-8000-000000000002", linked.to_string_lossy()],
        ).unwrap();
        (temp, conn, vault_root)
    }

    #[test]
    fn legacy_export_preserves_workspace_without_credentials() {
        let (temp, conn, _) = setup();
        let destination = temp.path().join("workspace.aether-backup.db");
        export(&conn, &destination).unwrap();
        let backup = Connection::open(destination).unwrap();
        assert!(!table_exists(&backup, "secrets").unwrap());
        assert_eq!(count_rows_where(&backup, "spaces", None).unwrap(), 1);
    }

    #[test]
    fn complete_archive_round_trips_managed_bytes_and_excludes_secrets_and_links() {
        let (temp, conn, vault_root) = setup();
        let destination = temp.path().join("workspace.aether-backup");
        let result = export_archive(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &destination,
        )
        .unwrap();
        assert_eq!(result.managed_file_count, 1);
        assert_eq!(result.linked_file_count, 1);

        let output = temp.path().join("inspect");
        fs::create_dir(&output).unwrap();
        let db = output.join(DATABASE_ENTRY);
        let vault = output.join("vault");
        let inspected = inspect_archive(&destination, &db, Some(&vault)).unwrap();
        assert_eq!(inspected.counts.spaces, 1);
        assert!(!table_exists(&Connection::open(&db).unwrap(), "secrets").unwrap());
        assert_eq!(
            fs::read_to_string(
                vault.join("items/018f47f4-7b7c-7d00-8000-000000000001/managed.txt")
            )
            .unwrap(),
            "managed bytes"
        );
        assert!(!vault
            .join("items/018f47f4-7b7c-7d00-8000-000000000002")
            .exists());
    }

    #[test]
    fn preview_token_is_one_time_and_stage_preserves_local_credentials() {
        let (temp, conn, vault_root) = setup();
        let destination = temp.path().join("workspace.aether-backup");
        export_archive(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &destination,
        )
        .unwrap();
        let runtime = RestoreRuntime::default();
        let preview =
            preview_restore(&destination, &temp.path().join("app-data"), &runtime).unwrap();
        stage_restore(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &runtime,
            &preview.token,
        )
        .unwrap();
        assert!(stage_restore(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &runtime,
            &preview.token
        )
        .is_err());
        drop(conn);
        assert!(apply_pending_restore(&temp.path().join("app-data")).unwrap());
        let restored = Connection::open(temp.path().join("app-data/aether.db")).unwrap();
        let secret: String = restored
            .query_row(
                "SELECT value FROM secrets WHERE key = 'ai_api_key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(secret, "encrypted");
        assert_eq!(count_rows_where(&restored, "notes", None).unwrap(), 1);
        assert_eq!(
            fs::read_to_string(
                temp.path()
                    .join("app-data/vault/items/018f47f4-7b7c-7d00-8000-000000000001/managed.txt")
            )
            .unwrap(),
            "managed bytes"
        );
    }

    #[test]
    fn rejects_unsafe_destinations_and_tampered_archive() {
        let (temp, conn, vault_root) = setup();
        assert!(export_archive(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            Path::new("relative.aether-backup")
        )
        .is_err());
        assert!(export_archive(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &temp.path().join("wrong.zip")
        )
        .is_err());
        assert!(export_archive(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &temp.path().join("app-data/inside.aether-backup")
        )
        .is_err());

        let destination = temp.path().join("workspace.aether-backup");
        export_archive(
            &conn,
            &vault_root,
            &temp.path().join("app-data"),
            &destination,
        )
        .unwrap();
        let mut bytes = fs::read(&destination).unwrap();
        let index = bytes.len() / 2;
        bytes[index] ^= 0x5a;
        fs::write(&destination, bytes).unwrap();
        let output = temp.path().join("tampered.db");
        assert!(inspect_archive(&destination, &output, None).is_err());
    }

    #[test]
    fn rejects_traversal_duplicates_and_newer_schema_metadata() {
        let temp = TempDir::new().unwrap();
        assert!(parse_managed_archive_path(
            "vault/items/018f47f4-7b7c-7d00-8000-000000000001/C:alternate"
        )
        .is_err());
        let path = temp.path().join("traversal.aether-backup");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        writer.start_file("../escape", options).unwrap();
        writer.write_all(b"{}").unwrap();
        writer.start_file(MANIFEST_ENTRY, options).unwrap();
        writer.write_all(b"{}").unwrap();
        writer.finish().unwrap();
        assert!(inspect_archive(&path, &temp.path().join("traversal.db"), None).is_err());

        let duplicate_path = temp.path().join("duplicate.aether-backup");
        let file = File::create(duplicate_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer.start_file(MANIFEST_ENTRY, options).unwrap();
        writer.write_all(b"{}").unwrap();
        assert!(writer.start_file(MANIFEST_ENTRY, options).is_err());

        let (workspace, conn, _) = setup();
        let db = workspace.path().join("newer.db");
        export_database(&conn, &db).unwrap();
        let (size, sha256) = hash_file(&db).unwrap();
        let manifest = ArchiveManifest {
            format_version: ARCHIVE_FORMAT,
            app_version: "99.0.0".to_string(),
            created_at: Utc::now().to_rfc3339(),
            database: Payload {
                path: DATABASE_ENTRY.to_string(),
                size_bytes: size,
                sha256,
            },
            managed_files: vec![],
            linked_file_count: 0,
            migrations: vec!["999_future".to_string()],
        };
        let archive = workspace.path().join("newer.aether-backup");
        create_zip(&archive, &db, &[], &manifest).unwrap();
        assert!(
            inspect_archive(&archive, &workspace.path().join("newer-output.db"), None)
                .unwrap_err()
                .contains("newer Aether schema")
        );
    }

    #[test]
    fn startup_recovers_an_interrupted_swap_before_retrying() {
        let (temp, conn, vault_root) = setup();
        let app_data = temp.path().join("app-data");
        let destination = temp.path().join("workspace.aether-backup");
        export_archive(&conn, &vault_root, &app_data, &destination).unwrap();
        let runtime = RestoreRuntime::default();
        let preview = preview_restore(&destination, &app_data, &runtime).unwrap();
        stage_restore(&conn, &vault_root, &app_data, &runtime, &preview.token).unwrap();
        drop(conn);

        let pending: PendingRestore =
            serde_json::from_slice(&fs::read(app_data.join(PENDING_FILE)).unwrap()).unwrap();
        let stage = app_data.join("restore-staging").join(&pending.id);
        let rollback = stage.join("rollback");
        fs::create_dir(&rollback).unwrap();
        fs::rename(app_data.join("aether.db"), rollback.join("aether.db")).unwrap();
        fs::rename(app_data.join("vault/items"), rollback.join("items")).unwrap();
        fs::rename(stage.join(DATABASE_ENTRY), app_data.join("aether.db")).unwrap();

        assert!(apply_pending_restore(&app_data).unwrap());
        let restored = Connection::open(app_data.join("aether.db")).unwrap();
        assert_eq!(count_rows_where(&restored, "spaces", None).unwrap(), 1);
        assert_eq!(
            fs::read_to_string(
                app_data.join("vault/items/018f47f4-7b7c-7d00-8000-000000000001/managed.txt")
            )
            .unwrap(),
            "managed bytes"
        );
    }
}
