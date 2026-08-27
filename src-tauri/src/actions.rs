use crate::db::repositories::{activity, notes, sources, tasks};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs, io,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};
use uuid::Uuid;

const PROPOSAL_TTL_MINUTES: i64 = 10;
const MAX_PENDING_PROPOSALS: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ActionRequest {
    CreateTask {
        title: String,
        description: String,
        due_date: Option<String>,
        space_id: Option<String>,
    },
    CreateNote {
        title: String,
        content: String,
        space_id: String,
    },
    CreateFolder {
        source_id: String,
        relative_path: String,
    },
    CopyFile {
        source_id: String,
        from_relative_path: String,
        to_relative_path: String,
    },
    MoveFile {
        source_id: String,
        from_relative_path: String,
        to_relative_path: String,
    },
    RenameFile {
        source_id: String,
        from_relative_path: String,
        new_name: String,
    },
    OpenFile {
        source_id: String,
        relative_path: String,
    },
    OpenFolder {
        source_id: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPreview {
    pub token: String,
    pub action_type: String,
    pub title: String,
    pub summary: String,
    pub consequence: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub action_type: String,
    pub title: String,
    pub detail: String,
    pub destination: String,
    pub executed_at: String,
}

#[derive(Debug, Clone, Copy)]
pub enum OpenTarget {
    File,
    Folder,
}

#[derive(Debug, Clone)]
struct PendingAction {
    request: ActionRequest,
    origin: Option<ActionOrigin>,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ActionOrigin {
    pub conversation_id: String,
    pub message_id: String,
}

#[derive(Default)]
pub struct ActionRuntime {
    pending: Mutex<HashMap<String, PendingAction>>,
}

struct Descriptor {
    action_type: String,
    title: String,
    summary: String,
    consequence: String,
}

struct SourcePaths {
    source: sources::Source,
    root: PathBuf,
    from: Option<PathBuf>,
    destination: Option<PathBuf>,
}

pub fn preview(
    conn: &Connection,
    runtime: &ActionRuntime,
    request: ActionRequest,
) -> Result<ActionPreview, String> {
    preview_with_origin(conn, runtime, request, None)
}

pub fn preview_ai(
    conn: &Connection,
    runtime: &ActionRuntime,
    request: ActionRequest,
    origin: ActionOrigin,
) -> Result<ActionPreview, String> {
    preview_with_origin(conn, runtime, request, Some(origin))
}

fn preview_with_origin(
    conn: &Connection,
    runtime: &ActionRuntime,
    request: ActionRequest,
    origin: Option<ActionOrigin>,
) -> Result<ActionPreview, String> {
    let descriptor = validate(conn, &request)?;
    let token = Uuid::now_v7().to_string();
    let expires_at = Utc::now() + Duration::minutes(PROPOSAL_TTL_MINUTES);
    let mut pending = runtime
        .pending
        .lock()
        .map_err(|error| format!("Action proposal lock error: {error}"))?;
    pending.retain(|_, proposal| proposal.expires_at >= Utc::now());
    if pending.len() >= MAX_PENDING_PROPOSALS {
        return Err("Too many pending Action proposals; approve or cancel one first".into());
    }
    pending.insert(
        token.clone(),
        PendingAction {
            request,
            origin,
            expires_at,
        },
    );
    Ok(ActionPreview {
        token,
        action_type: descriptor.action_type,
        title: descriptor.title,
        summary: descriptor.summary,
        consequence: descriptor.consequence,
        expires_at: expires_at.to_rfc3339(),
    })
}

pub fn cancel(runtime: &ActionRuntime, token: &str) -> Result<bool, String> {
    runtime
        .pending
        .lock()
        .map_err(|error| format!("Action proposal lock error: {error}"))?
        .remove(token)
        .map(|_| true)
        .ok_or_else(|| "Action proposal is missing, expired, or already used".to_string())
}

pub fn execute<F>(
    conn: &mut Connection,
    runtime: &ActionRuntime,
    token: &str,
    open: F,
) -> Result<ActionResult, String>
where
    F: FnOnce(&Path, OpenTarget) -> Result<(), String>,
{
    let pending = runtime
        .pending
        .lock()
        .map_err(|error| format!("Action proposal lock error: {error}"))?
        .remove(token)
        .ok_or_else(|| "Action proposal is missing, expired, or already used".to_string())?;
    if pending.expires_at < Utc::now() {
        return Err("Action proposal expired; review it again before executing".to_string());
    }
    let descriptor = validate(conn, &pending.request)?;
    let result = match &pending.request {
        ActionRequest::CreateTask {
            title,
            description,
            due_date,
            space_id,
        } => execute_task(
            conn,
            title,
            description,
            due_date.as_deref(),
            space_id.as_deref(),
            pending.origin.as_ref(),
        )?,
        ActionRequest::CreateNote {
            title,
            content,
            space_id,
        } => execute_note(conn, title, content, space_id, pending.origin.as_ref())?,
        ActionRequest::CreateFolder { .. } => execute_filesystem(conn, &pending.request)?,
        ActionRequest::CopyFile { .. } => execute_filesystem(conn, &pending.request)?,
        ActionRequest::MoveFile { .. } => execute_filesystem(conn, &pending.request)?,
        ActionRequest::RenameFile { .. } => execute_filesystem(conn, &pending.request)?,
        ActionRequest::OpenFile { .. } | ActionRequest::OpenFolder { .. } => {
            let paths = source_paths(conn, &pending.request)?;
            let (path, kind) = match pending.request {
                ActionRequest::OpenFile { .. } => (
                    paths.from.ok_or("Approved file is unavailable")?,
                    OpenTarget::File,
                ),
                _ => (paths.root, OpenTarget::Folder),
            };
            open(&path, kind)?;
            record_action(
                conn,
                &pending.request,
                Some("source"),
                Some(&paths.source.id),
                None,
            )?;
            ActionResult {
                action_type: descriptor.action_type.clone(),
                title: descriptor.title.clone(),
                detail: "Opened through the approved native action".to_string(),
                destination: "/actions".to_string(),
                executed_at: Utc::now().to_rfc3339(),
            }
        }
    };
    Ok(result)
}

fn validate(conn: &Connection, request: &ActionRequest) -> Result<Descriptor, String> {
    match request {
        ActionRequest::CreateTask {
            title,
            description,
            due_date,
            space_id,
        } => {
            validate_space(conn, space_id.as_deref(), false)?;
            let input = tasks::TaskInput {
                space_id: space_id.clone(),
                parent_task_id: None,
                title: title.clone(),
                description: description.clone(),
                status: "inbox".to_string(),
                priority: "none".to_string(),
                due_date: due_date.clone(),
                tags: Vec::new(),
            };
            let transaction = conn.unchecked_transaction().map_err(db_error)?;
            tasks::create(&transaction, &input)?;
            transaction.rollback().map_err(db_error)?;
            Ok(Descriptor {
                action_type: "createTask".into(),
                title: "Create Task".into(),
                summary: format!("Create Task “{}”", title.trim()),
                consequence: scope_consequence(conn, space_id.as_deref(), "Adds one open Task")?,
            })
        }
        ActionRequest::CreateNote {
            title,
            content,
            space_id,
        } => {
            validate_space(conn, Some(space_id), true)?;
            validate_note(title, content)?;
            Ok(Descriptor {
                action_type: "createNote".into(),
                title: "Create Note".into(),
                summary: format!("Create Note “{}”", title.trim()),
                consequence: scope_consequence(conn, Some(space_id), "Adds one Markdown Note")?,
            })
        }
        ActionRequest::CreateFolder { relative_path, .. } => {
            let paths = source_paths(conn, request)?;
            Ok(file_descriptor(
                "createFolder",
                "Create folder",
                relative_path,
                &paths.source.display_name,
                "Creates one folder; existing items are never overwritten",
            ))
        }
        ActionRequest::CopyFile {
            to_relative_path, ..
        } => {
            let paths = source_paths(conn, request)?;
            Ok(file_descriptor(
                "copyFile",
                "Copy file",
                to_relative_path,
                &paths.source.display_name,
                "Creates one copy; the original remains and existing items are never overwritten",
            ))
        }
        ActionRequest::MoveFile {
            to_relative_path, ..
        } => {
            let paths = source_paths(conn, request)?;
            Ok(file_descriptor(
                "moveFile",
                "Move file",
                to_relative_path,
                &paths.source.display_name,
                "Moves one file inside this Source without overwriting",
            ))
        }
        ActionRequest::RenameFile { new_name, .. } => {
            let paths = source_paths(conn, request)?;
            Ok(file_descriptor(
                "renameFile",
                "Rename file",
                new_name,
                &paths.source.display_name,
                "Renames one file in place without overwriting",
            ))
        }
        ActionRequest::OpenFile { relative_path, .. } => {
            let paths = source_paths(conn, request)?;
            Ok(file_descriptor(
                "openFile",
                "Open file",
                relative_path,
                &paths.source.display_name,
                "Opens this indexed file with its configured Windows application",
            ))
        }
        ActionRequest::OpenFolder { .. } => {
            let paths = source_paths(conn, request)?;
            Ok(file_descriptor(
                "openFolder",
                "Open folder",
                &paths.source.display_name,
                &paths.source.display_name,
                "Opens the authorized Source root in Windows Explorer",
            ))
        }
    }
}

fn execute_task(
    conn: &mut Connection,
    title: &str,
    description: &str,
    due_date: Option<&str>,
    space_id: Option<&str>,
    origin: Option<&ActionOrigin>,
) -> Result<ActionResult, String> {
    let transaction = conn.transaction().map_err(db_error)?;
    let task = tasks::create(
        &transaction,
        &tasks::TaskInput {
            space_id: space_id.map(str::to_string),
            parent_task_id: None,
            title: title.to_string(),
            description: description.to_string(),
            status: "inbox".into(),
            priority: "none".into(),
            due_date: due_date.map(str::to_string),
            tags: Vec::new(),
        },
    )?;
    record_action(
        &transaction,
        &ActionRequest::CreateTask {
            title: title.into(),
            description: description.into(),
            due_date: due_date.map(str::to_string),
            space_id: space_id.map(str::to_string),
        },
        Some("task"),
        Some(&task.id),
        origin,
    )?;
    transaction.commit().map_err(db_error)?;
    Ok(ActionResult {
        action_type: "createTask".into(),
        title: format!("Created {}", task.title),
        detail: "The approved Task was added to Aether".into(),
        destination: "/tasks".into(),
        executed_at: Utc::now().to_rfc3339(),
    })
}

fn execute_note(
    conn: &mut Connection,
    title: &str,
    content: &str,
    space_id: &str,
    origin: Option<&ActionOrigin>,
) -> Result<ActionResult, String> {
    let transaction = conn.transaction().map_err(db_error)?;
    validate_space(&transaction, Some(space_id), true)?;
    validate_note(title, content)?;
    let note = notes::create(&transaction, space_id)?;
    let excerpt: String = content.chars().take(240).collect();
    let note = notes::update(
        &transaction,
        &note.id,
        Some(title.trim()),
        Some(content),
        Some(&excerpt),
        Some(note.revision),
    )?
    .ok_or("Note disappeared during approved action")?;
    record_action(
        &transaction,
        &ActionRequest::CreateNote {
            title: title.into(),
            content: content.into(),
            space_id: space_id.into(),
        },
        Some("note"),
        Some(&note.id),
        origin,
    )?;
    transaction.commit().map_err(db_error)?;
    Ok(ActionResult {
        action_type: "createNote".into(),
        title: format!("Created {}", note.title),
        detail: "The approved Note was added to its Space".into(),
        destination: format!("/spaces/{space_id}/notes"),
        executed_at: Utc::now().to_rfc3339(),
    })
}

fn execute_filesystem(conn: &Connection, request: &ActionRequest) -> Result<ActionResult, String> {
    let paths = source_paths(conn, request)?;
    let source_id = paths.source.id.clone();
    let (action_type, title, destination_path, rollback) = match request {
        ActionRequest::CreateFolder { .. } => {
            let destination = paths
                .destination
                .ok_or("Approved destination is unavailable")?;
            fs::create_dir(&destination)
                .map_err(|error| format!("Could not create approved folder: {error}"))?;
            (
                "createFolder",
                "Folder created",
                destination.clone(),
                Rollback::RemoveDirectory(destination),
            )
        }
        ActionRequest::CopyFile { .. } => {
            let from = paths.from.ok_or("Approved source file is unavailable")?;
            let destination = paths
                .destination
                .ok_or("Approved destination is unavailable")?;
            copy_without_overwrite(&from, &destination)?;
            (
                "copyFile",
                "File copied",
                destination.clone(),
                Rollback::RemoveFile(destination),
            )
        }
        ActionRequest::MoveFile { .. } | ActionRequest::RenameFile { .. } => {
            let from = paths.from.ok_or("Approved source file is unavailable")?;
            let destination = paths
                .destination
                .ok_or("Approved destination is unavailable")?;
            fs::rename(&from, &destination)
                .map_err(|error| format!("Could not move approved file: {error}"))?;
            let action_type = if matches!(request, ActionRequest::RenameFile { .. }) {
                "renameFile"
            } else {
                "moveFile"
            };
            let title = if action_type == "renameFile" {
                "File renamed"
            } else {
                "File moved"
            };
            (
                action_type,
                title,
                destination.clone(),
                Rollback::MoveBack {
                    from: destination,
                    to: from,
                },
            )
        }
        _ => return Err("Unsupported filesystem Action".into()),
    };
    if let Err(error) = record_action(conn, request, Some("source"), Some(&source_id), None) {
        rollback.apply()?;
        return Err(format!(
            "Action audit failed and the file change was rolled back: {error}"
        ));
    }
    let display = destination_path
        .strip_prefix(&paths.root)
        .ok()
        .map(safe_path_text)
        .transpose()?
        .unwrap_or_else(|| "approved destination".into());
    Ok(ActionResult {
        action_type: action_type.into(),
        title: title.into(),
        detail: format!(
            "Completed inside {} · {}",
            paths.source.display_name, display
        ),
        destination: "/sources".into(),
        executed_at: Utc::now().to_rfc3339(),
    })
}

fn copy_without_overwrite(from: &Path, destination: &Path) -> Result<(), String> {
    let mut source = fs::File::open(from)
        .map_err(|error| format!("Could not open approved source file: {error}"))?;
    let mut target = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| format!("Could not create approved copy without overwrite: {error}"))?;
    if let Err(error) = io::copy(&mut source, &mut target) {
        drop(target);
        let _ = fs::remove_file(destination);
        return Err(format!("Could not copy approved file: {error}"));
    }
    Ok(())
}

enum Rollback {
    RemoveDirectory(PathBuf),
    RemoveFile(PathBuf),
    MoveBack { from: PathBuf, to: PathBuf },
}

impl Rollback {
    fn apply(self) -> Result<(), String> {
        match self {
            Self::RemoveDirectory(path) => fs::remove_dir(path),
            Self::RemoveFile(path) => fs::remove_file(path),
            Self::MoveBack { from, to } => fs::rename(from, to),
        }
        .map_err(|error| format!("Action rollback failed: {error}"))
    }
}

fn source_paths(conn: &Connection, request: &ActionRequest) -> Result<SourcePaths, String> {
    let source_id = match request {
        ActionRequest::CreateFolder { source_id, .. }
        | ActionRequest::CopyFile { source_id, .. }
        | ActionRequest::MoveFile { source_id, .. }
        | ActionRequest::RenameFile { source_id, .. }
        | ActionRequest::OpenFile { source_id, .. }
        | ActionRequest::OpenFolder { source_id } => source_id,
        _ => return Err("This Action does not use a Source".into()),
    };
    let source = sources::get(conn, source_id)?.ok_or("Authorized Source does not exist")?;
    let root = PathBuf::from(&source.root_path)
        .canonicalize()
        .map_err(|error| format!("Authorized Source is unavailable: {error}"))?;
    if !root.is_dir() {
        return Err("Authorized Source is not a directory".into());
    }
    let mut from = None;
    let mut destination = None;
    match request {
        ActionRequest::OpenFolder { .. } => {}
        ActionRequest::CreateFolder { relative_path, .. } => {
            destination = Some(resolve_destination(&root, relative_path)?);
        }
        ActionRequest::OpenFile { relative_path, .. } => {
            ensure_indexed(conn, source_id, relative_path)?;
            from = Some(resolve_existing_file(&root, relative_path)?);
        }
        ActionRequest::CopyFile {
            from_relative_path,
            to_relative_path,
            ..
        }
        | ActionRequest::MoveFile {
            from_relative_path,
            to_relative_path,
            ..
        } => {
            ensure_indexed(conn, source_id, from_relative_path)?;
            from = Some(resolve_existing_file(&root, from_relative_path)?);
            destination = Some(resolve_destination(&root, to_relative_path)?);
        }
        ActionRequest::RenameFile {
            from_relative_path,
            new_name,
            ..
        } => {
            ensure_indexed(conn, source_id, from_relative_path)?;
            let existing = resolve_existing_file(&root, from_relative_path)?;
            let name = safe_relative(new_name)?;
            if name.components().count() != 1 {
                return Err("A renamed file must use one filename without folders".into());
            }
            let parent = existing
                .parent()
                .ok_or("Approved file has no parent folder")?;
            destination = Some(resolve_destination(parent, new_name)?);
            from = Some(existing);
        }
        _ => unreachable!(),
    }
    Ok(SourcePaths {
        source,
        root,
        from,
        destination,
    })
}

fn ensure_indexed(conn: &Connection, source_id: &str, relative: &str) -> Result<(), String> {
    let relative = safe_path_text(&safe_relative(relative)?)?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM indexed_files WHERE source_id = ?1 AND relative_path = ?2 COLLATE NOCASE AND state = 'present')",
        params![source_id, relative], |row| row.get(0),
    ).map_err(db_error)?;
    if !exists {
        return Err("Action file is not a present indexed file in this Source".into());
    }
    Ok(())
}

fn safe_relative(value: &str) -> Result<PathBuf, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 500 {
        return Err("Action path must contain 1 to 500 characters".into());
    }
    let path = Path::new(value);
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            _ => return Err("Action paths must be relative and cannot contain traversal".into()),
        }
    }
    if safe.as_os_str().is_empty() {
        return Err("Action path is empty".into());
    }
    Ok(safe)
}

fn resolve_existing_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = root
        .join(safe_relative(relative)?)
        .canonicalize()
        .map_err(|error| format!("Approved file is unavailable: {error}"))?;
    if !path.starts_with(root) || !path.is_file() {
        return Err("Approved path is not a contained regular file".into());
    }
    Ok(path)
}

fn resolve_destination(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let destination = root.join(safe_relative(relative)?);
    if destination.exists() {
        return Err("Action destination already exists; overwrite is not allowed".into());
    }
    let parent = destination
        .parent()
        .ok_or("Action destination has no parent")?
        .canonicalize()
        .map_err(|error| format!("Action destination parent is unavailable: {error}"))?;
    if !parent.starts_with(root) || !parent.is_dir() {
        return Err("Action destination escaped the authorized Source".into());
    }
    let name = destination
        .file_name()
        .ok_or("Action destination has no name")?;
    Ok(parent.join(name))
}

fn safe_path_text(path: &Path) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(
                part.to_str()
                    .ok_or("Action path contains unsupported Unicode")?,
            ),
            _ => return Err("Action path is not presentation-safe".into()),
        }
    }
    Ok(parts.join("/"))
}

fn validate_space(conn: &Connection, space_id: Option<&str>, required: bool) -> Result<(), String> {
    if required && space_id.is_none() {
        return Err("This Action requires an active Space".into());
    }
    if let Some(id) = space_id {
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM spaces WHERE id = ?1 AND archived_at IS NULL)",
                [id],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        if !exists {
            return Err("Action Space does not exist or is archived".into());
        }
    }
    Ok(())
}

fn validate_note(title: &str, content: &str) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err("Note title must contain 1 to 200 characters".into());
    }
    if content.chars().count() > 20_000 {
        return Err("Note content cannot exceed 20000 characters".into());
    }
    Ok(())
}

fn scope_consequence(
    conn: &Connection,
    space_id: Option<&str>,
    lead: &str,
) -> Result<String, String> {
    let scope = match space_id {
        Some(id) => conn
            .query_row(
                "SELECT name FROM spaces WHERE id = ?1 AND archived_at IS NULL",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(db_error)?
            .ok_or("Action Space does not exist or is archived")?,
        None => "Task Inbox".into(),
    };
    Ok(format!("{lead} in {scope}"))
}

fn file_descriptor(
    action_type: &str,
    title: &str,
    relative: &str,
    source: &str,
    consequence: &str,
) -> Descriptor {
    Descriptor {
        action_type: action_type.into(),
        title: title.into(),
        summary: format!("{title}: {relative}"),
        consequence: format!("{consequence} inside {source}"),
    }
}

fn record_action(
    conn: &Connection,
    request: &ActionRequest,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    origin: Option<&ActionOrigin>,
) -> Result<(), String> {
    let space_id = match request {
        ActionRequest::CreateTask { space_id, .. } => space_id.clone(),
        ActionRequest::CreateNote { space_id, .. } => Some(space_id.clone()),
        ActionRequest::CreateFolder { source_id, .. }
        | ActionRequest::CopyFile { source_id, .. }
        | ActionRequest::MoveFile { source_id, .. }
        | ActionRequest::RenameFile { source_id, .. }
        | ActionRequest::OpenFile { source_id, .. }
        | ActionRequest::OpenFolder { source_id } => {
            sources::get(conn, source_id)?.and_then(|source| source.space_id)
        }
    };
    activity::record(
        conn,
        &activity::ActivityEvent {
            id: Uuid::now_v7().to_string(),
            event_type: "action_executed".into(),
            entity_type: entity_type.map(str::to_string),
            entity_id: entity_id.map(str::to_string),
            space_id,
            metadata_json: origin.map(|value| {
                serde_json::json!({
                    "origin": "ai_proposal",
                    "conversationId": value.conversation_id,
                    "messageId": value.message_id,
                })
                .to_string()
            }),
            created_at: String::new(),
        },
    )?;
    Ok(())
}

fn db_error(error: rusqlite::Error) -> String {
    format!("Action database error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use tempfile::tempdir;

    fn setup() -> (Connection, ActionRuntime, tempfile::TempDir) {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        migrations::run(&connection).unwrap();
        connection
            .execute("INSERT INTO spaces (id,name) VALUES ('s1','Work')", [])
            .unwrap();
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("one.txt"), "one").unwrap();
        connection.execute("INSERT INTO sources (id,root_path,display_name,space_id) VALUES ('src',?1,'Work files','s1')", [directory.path().to_string_lossy().as_ref()]).unwrap();
        connection.execute("INSERT INTO indexed_files (id,source_id,relative_path,filename,size_bytes) VALUES ('f','src','one.txt','one.txt',3)", []).unwrap();
        (connection, ActionRuntime::default(), directory)
    }

    fn approve(conn: &Connection, runtime: &ActionRuntime, request: ActionRequest) -> String {
        preview(conn, runtime, request).unwrap().token
    }

    #[test]
    fn creates_database_actions_once_and_audits_them_transactionally() {
        let (mut connection, runtime, _directory) = setup();
        let task_token = approve(
            &connection,
            &runtime,
            ActionRequest::CreateTask {
                title: "Ship".into(),
                description: "".into(),
                due_date: None,
                space_id: Some("s1".into()),
            },
        );
        let task = execute(&mut connection, &runtime, &task_token, |_, _| Ok(())).unwrap();
        assert_eq!(task.action_type, "createTask");
        assert!(execute(&mut connection, &runtime, &task_token, |_, _| Ok(())).is_err());
        let note_token = approve(
            &connection,
            &runtime,
            ActionRequest::CreateNote {
                title: "Plan".into(),
                content: "Body".into(),
                space_id: "s1".into(),
            },
        );
        execute(&mut connection, &runtime, &note_token, |_, _| Ok(())).unwrap();
        let counts: (i64, i64, i64) = connection.query_row("SELECT (SELECT COUNT(*) FROM tasks),(SELECT COUNT(*) FROM notes),(SELECT COUNT(*) FROM activity_events WHERE event_type='action_executed')", [], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        assert_eq!(counts, (1, 1, 2));
    }

    #[test]
    fn ai_preview_keeps_trusted_message_attribution_in_the_action_audit() {
        let (mut connection, runtime, _directory) = setup();
        let preview = preview_ai(
            &connection,
            &runtime,
            ActionRequest::CreateTask {
                title: "Attributed".into(),
                description: "".into(),
                due_date: None,
                space_id: Some("s1".into()),
            },
            ActionOrigin {
                conversation_id: "conversation-1".into(),
                message_id: "message-1".into(),
            },
        )
        .unwrap();
        execute(&mut connection, &runtime, &preview.token, |_, _| Ok(())).unwrap();
        let metadata: String = connection
            .query_row(
                "SELECT metadata_json FROM activity_events WHERE event_type = 'action_executed'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(metadata.contains("conversation-1"));
        assert!(metadata.contains("message-1"));
        assert!(metadata.contains("ai_proposal"));
    }

    #[test]
    fn performs_each_contained_file_action_without_overwrite() {
        let (mut connection, runtime, directory) = setup();
        fs::write(directory.path().join("occupied.txt"), "x").unwrap();
        let rejected = preview(
            &connection,
            &runtime,
            ActionRequest::CopyFile {
                source_id: "src".into(),
                from_relative_path: "one.txt".into(),
                to_relative_path: "occupied.txt".into(),
            },
        );
        assert!(rejected.unwrap_err().contains("overwrite"));
        fs::remove_file(directory.path().join("occupied.txt")).unwrap();

        for request in [
            ActionRequest::CreateFolder {
                source_id: "src".into(),
                relative_path: "new-folder".into(),
            },
            ActionRequest::CopyFile {
                source_id: "src".into(),
                from_relative_path: "one.txt".into(),
                to_relative_path: "copy.txt".into(),
            },
            ActionRequest::RenameFile {
                source_id: "src".into(),
                from_relative_path: "one.txt".into(),
                new_name: "renamed.txt".into(),
            },
        ] {
            let token = approve(&connection, &runtime, request);
            execute(&mut connection, &runtime, &token, |_, _| Ok(())).unwrap();
        }
        connection.execute("UPDATE indexed_files SET relative_path='renamed.txt',filename='renamed.txt' WHERE id='f'", []).unwrap();
        let move_token = approve(
            &connection,
            &runtime,
            ActionRequest::MoveFile {
                source_id: "src".into(),
                from_relative_path: "renamed.txt".into(),
                to_relative_path: "moved.txt".into(),
            },
        );
        execute(&mut connection, &runtime, &move_token, |_, _| Ok(())).unwrap();
        assert!(directory.path().join("new-folder").is_dir());
        assert!(directory.path().join("copy.txt").is_file());
        assert!(directory.path().join("moved.txt").is_file());
        assert!(!directory.path().join("one.txt").exists());
    }

    #[test]
    fn rolls_back_a_reversible_file_write_when_audit_persistence_fails() {
        let (mut connection, runtime, directory) = setup();
        let token = approve(
            &connection,
            &runtime,
            ActionRequest::CopyFile {
                source_id: "src".into(),
                from_relative_path: "one.txt".into(),
                to_relative_path: "copy.txt".into(),
            },
        );
        connection
            .execute_batch(
                "CREATE TRIGGER reject_action_audit BEFORE INSERT ON activity_events
                 WHEN NEW.event_type = 'action_executed'
                 BEGIN SELECT RAISE(FAIL, 'forced audit failure'); END;",
            )
            .unwrap();

        let error = execute(&mut connection, &runtime, &token, |_, _| Ok(())).unwrap_err();
        assert!(error.contains("rolled back"));
        assert!(!directory.path().join("copy.txt").exists());
        assert!(directory.path().join("one.txt").is_file());
    }

    #[test]
    fn rejects_traversal_unindexed_files_archived_spaces_and_expired_tokens() {
        let (mut connection, runtime, directory) = setup();
        fs::write(directory.path().join("hidden.txt"), "hidden").unwrap();
        for request in [
            ActionRequest::CreateFolder {
                source_id: "src".into(),
                relative_path: "../escape".into(),
            },
            ActionRequest::OpenFile {
                source_id: "src".into(),
                relative_path: "hidden.txt".into(),
            },
        ] {
            assert!(preview(&connection, &runtime, request).is_err());
        }
        connection
            .execute(
                "UPDATE spaces SET archived_at=datetime('now') WHERE id='s1'",
                [],
            )
            .unwrap();
        assert!(preview(
            &connection,
            &runtime,
            ActionRequest::CreateNote {
                title: "No".into(),
                content: "".into(),
                space_id: "s1".into()
            }
        )
        .is_err());
        connection
            .execute("UPDATE spaces SET archived_at=NULL WHERE id='s1'", [])
            .unwrap();
        let token = approve(
            &connection,
            &runtime,
            ActionRequest::OpenFolder {
                source_id: "src".into(),
            },
        );
        runtime
            .pending
            .lock()
            .unwrap()
            .get_mut(&token)
            .unwrap()
            .expires_at = Utc::now() - Duration::minutes(1);
        assert!(execute(&mut connection, &runtime, &token, |_, _| Ok(()))
            .unwrap_err()
            .contains("expired"));
    }

    #[cfg(windows)]
    #[test]
    fn rejects_an_indexed_symlink_that_resolves_outside_the_source() {
        use std::os::windows::fs::symlink_file;

        let (connection, runtime, directory) = setup();
        let outside = tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), "secret").unwrap();
        symlink_file(
            outside.path().join("secret.txt"),
            directory.path().join("escape.txt"),
        )
        .unwrap();
        connection.execute("INSERT INTO indexed_files (id,source_id,relative_path,filename,size_bytes) VALUES ('escape','src','escape.txt','escape.txt',6)", []).unwrap();

        let error = preview(
            &connection,
            &runtime,
            ActionRequest::OpenFile {
                source_id: "src".into(),
                relative_path: "escape.txt".into(),
            },
        )
        .unwrap_err();
        assert!(error.contains("contained regular file"));
    }

    #[test]
    fn opens_only_reviewed_source_targets_and_cancel_consumes_proposal() {
        let (mut connection, runtime, directory) = setup();
        let file_token = approve(
            &connection,
            &runtime,
            ActionRequest::OpenFile {
                source_id: "src".into(),
                relative_path: "one.txt".into(),
            },
        );
        let opened = std::cell::RefCell::new(None);
        execute(&mut connection, &runtime, &file_token, |path, kind| {
            assert!(matches!(kind, OpenTarget::File));
            *opened.borrow_mut() = Some(path.to_path_buf());
            Ok(())
        })
        .unwrap();
        assert_eq!(
            opened.into_inner().unwrap(),
            directory.path().join("one.txt").canonicalize().unwrap()
        );
        let folder_token = approve(
            &connection,
            &runtime,
            ActionRequest::OpenFolder {
                source_id: "src".into(),
            },
        );
        execute(&mut connection, &runtime, &folder_token, |path, kind| {
            assert!(matches!(kind, OpenTarget::Folder));
            assert_eq!(path, directory.path().canonicalize().unwrap());
            Ok(())
        })
        .unwrap();
        let cancelled_token = approve(
            &connection,
            &runtime,
            ActionRequest::OpenFolder {
                source_id: "src".into(),
            },
        );
        assert!(cancel(&runtime, &cancelled_token).unwrap());
        assert!(execute(&mut connection, &runtime, &cancelled_token, |_, _| Ok(())).is_err());
    }
}
