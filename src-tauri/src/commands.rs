#![allow(clippy::too_many_arguments)]

use crate::actions::{self, ActionRuntime, OpenTarget};
use crate::ai::context;
use crate::ai::credentials;
use crate::ai::proposals;
use crate::ai::provider::{self, ChatCompletionRequest, ChatMessage, ProviderConfig};
use crate::ai::routing;
use crate::ai::runtime::AiRuntime;
use crate::backup;
use crate::context::{self as local_context, ContextRuntime};
use crate::db::repositories::{self, with_conn};
use crate::db::Database;
use crate::diagnostics::{self, BetaDiagnosticReport};
use crate::native::NativeStatus;
use crate::updater::{self, UpdateProgressEvent, UpdateRuntime};
use crate::vault as vault_storage;
use std::path::PathBuf;
use tauri::{ipc::Channel, AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

struct AiRequestGuard<'a> {
    runtime: &'a AiRuntime,
    request_id: String,
}

impl Drop for AiRequestGuard<'_> {
    fn drop(&mut self) {
        self.runtime.finish(&self.request_id);
    }
}

// ─── Native desktop ─────────────────────────────────────

#[tauri::command]
pub fn native_get_status(status: State<NativeStatus>) -> NativeStatus {
    status.inner().clone()
}

#[tauri::command]
pub fn native_get_beta_diagnostics(
    db: State<Database>,
    status: State<NativeStatus>,
) -> Result<BetaDiagnosticReport, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|error| format!("Lock error: {error}"))?;
    diagnostics::generate(&conn, status.inner())
}

#[tauri::command]
pub fn native_test_notification(app: AppHandle) -> Result<(), String> {
    app.notification()
        .builder()
        .title("Aether notifications are ready")
        .body("You can close Aether to the tray and reopen it with Ctrl+Shift+Space.")
        .show()
        .map_err(|error| format!("Native notification error: {}", error))
}

#[tauri::command]
pub fn native_get_update_status(
    app: AppHandle,
    runtime: State<UpdateRuntime>,
) -> updater::UpdateStatus {
    updater::status(&app, &runtime)
}

#[tauri::command]
pub async fn native_check_for_update(
    app: AppHandle,
    runtime: State<'_, UpdateRuntime>,
) -> Result<Option<updater::UpdatePreview>, String> {
    updater::check(&app, &runtime).await
}

#[tauri::command]
pub fn native_cancel_update(runtime: State<UpdateRuntime>, token: String) -> Result<bool, String> {
    updater::cancel(&runtime, &token)
}

#[tauri::command]
pub async fn native_install_update(
    runtime: State<'_, UpdateRuntime>,
    token: String,
    on_event: Channel<UpdateProgressEvent>,
) -> Result<(), String> {
    updater::install(&runtime, &token, on_event).await
}

#[tauri::command]
pub fn export_workspace_backup(
    db: State<Database>,
    destination: String,
) -> Result<backup::BackupResult, String> {
    let destination = PathBuf::from(destination);
    let conn = db
        .conn
        .lock()
        .map_err(|error| format!("Lock error: {error}"))?;
    backup::export(&conn, &destination)
}

#[tauri::command]
pub async fn export_workspace_archive(
    app: AppHandle,
    destination: String,
) -> Result<backup::ArchiveResult, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve Aether data directory: {error}"))?;
    let database = app_data.join("aether.db");
    let vault = app_data.join("vault");
    tauri::async_runtime::spawn_blocking(move || {
        let conn = rusqlite::Connection::open(database)
            .map_err(|error| format!("Could not open workspace for backup: {error}"))?;
        backup::export_archive(&conn, &vault, &app_data, &PathBuf::from(destination))
    })
    .await
    .map_err(|error| format!("Backup worker failed: {error}"))?
}

#[tauri::command]
pub async fn preview_workspace_restore(
    app: AppHandle,
    runtime: State<'_, backup::RestoreRuntime>,
    source: String,
) -> Result<backup::RestorePreview, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve Aether data directory: {error}"))?;
    let runtime = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        backup::preview_restore(&PathBuf::from(source), &app_data, &runtime)
    })
    .await
    .map_err(|error| format!("Restore preview worker failed: {error}"))?
}

#[tauri::command]
pub fn cancel_workspace_restore(
    runtime: State<backup::RestoreRuntime>,
    token: String,
) -> Result<bool, String> {
    backup::cancel_restore(&runtime, &token)
}

#[tauri::command]
pub async fn approve_workspace_restore(
    app: AppHandle,
    runtime: State<'_, backup::RestoreRuntime>,
    token: String,
) -> Result<(), String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve Aether data directory: {error}"))?;
    let database = app_data.join("aether.db");
    let vault = app_data.join("vault");
    let runtime = runtime.inner().clone();
    let worker_data = app_data.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = rusqlite::Connection::open(database)
            .map_err(|error| format!("Could not open current workspace for restore: {error}"))?;
        backup::stage_restore(&conn, &vault, &worker_data, &runtime, &token)
    })
    .await
    .map_err(|error| format!("Restore preparation worker failed: {error}"))??;
    app.request_restart();
    Ok(())
}

// ─── Safe Actions ──────────────────────────────────────

#[tauri::command]
pub fn preview_action(
    db: State<Database>,
    runtime: State<ActionRuntime>,
    request: actions::ActionRequest,
) -> Result<actions::ActionPreview, String> {
    with_conn(&db.conn, |conn| actions::preview(conn, &runtime, request))
}

#[tauri::command]
pub fn cancel_action(runtime: State<ActionRuntime>, token: String) -> Result<bool, String> {
    actions::cancel(&runtime, &token)
}

#[tauri::command]
pub fn execute_action(
    app: AppHandle,
    db: State<Database>,
    runtime: State<ActionRuntime>,
    token: String,
) -> Result<actions::ActionResult, String> {
    let mut conn = db
        .conn
        .lock()
        .map_err(|error| format!("Database lock error: {error}"))?;
    actions::execute(&mut conn, &runtime, &token, |path, target| {
        app.opener()
            .open_path(path.to_string_lossy(), None::<&str>)
            .map_err(|error| match target {
                OpenTarget::File => format!("Failed to open approved Source file: {error}"),
                OpenTarget::Folder => format!("Failed to open approved Source folder: {error}"),
            })
    })
}

#[tauri::command]
pub fn get_pulse(db: State<Database>) -> Result<repositories::pulse::PulseSnapshot, String> {
    with_conn(&db.conn, repositories::pulse::get)
}

// ─── Context Sources ────────────────────────────────────

#[tauri::command]
pub fn create_source(
    app: AppHandle,
    db: State<Database>,
    input: repositories::sources::CreateSourceInput,
) -> Result<repositories::sources::Source, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve Aether data directory: {e}"))?;
    let root = local_context::authorize_directory(&PathBuf::from(&input.root_path), &app_data)?;
    let validated = repositories::sources::CreateSourceInput {
        root_path: local_context::canonical_text(&root)?,
        display_name: input.display_name,
        space_id: input.space_id,
    };
    with_conn(&db.conn, |conn| {
        repositories::sources::create(conn, &validated)
    })
}

#[tauri::command]
pub fn list_sources(db: State<Database>) -> Result<Vec<repositories::sources::Source>, String> {
    with_conn(&db.conn, repositories::sources::list)
}

#[tauri::command]
pub fn universal_search(
    db: State<Database>,
    query: String,
    current_space_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<repositories::search::SearchResult>, String> {
    with_conn(&db.conn, |conn| {
        repositories::search::search(conn, &query, current_space_id.as_deref(), limit)
    })
}

#[tauri::command]
pub fn update_source_space(
    db: State<Database>,
    id: String,
    space_id: Option<String>,
) -> Result<Option<repositories::sources::Source>, String> {
    with_conn(&db.conn, |conn| {
        repositories::sources::update_space(conn, &id, space_id.as_deref())
    })
}

#[tauri::command]
pub fn revoke_source(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::sources::delete(conn, &id))
}

#[tauri::command]
pub fn list_indexed_files(
    db: State<Database>,
    source_id: String,
    include_removed: Option<bool>,
) -> Result<Vec<repositories::sources::IndexedFile>, String> {
    with_conn(&db.conn, |conn| {
        repositories::sources::list_files(conn, &source_id, include_removed.unwrap_or(false))
    })
}

#[tauri::command]
pub async fn scan_source(
    db: State<'_, Database>,
    runtime: State<'_, ContextRuntime>,
    id: String,
) -> Result<repositories::sources::ScanResult, String> {
    runtime.begin(&id)?;
    let result: Result<repositories::sources::ScanResult, String> = async {
        let source = with_conn(&db.conn, |conn| repositories::sources::get(conn, &id))?
            .ok_or_else(|| "Source does not exist".to_string())?;
        with_conn(&db.conn, |conn| {
            repositories::sources::mark_scanning(conn, &id)
        })?;
        let root = PathBuf::from(&source.root_path);
        let snapshot =
            tauri::async_runtime::spawn_blocking(move || local_context::scan_directory(&root))
                .await
                .map_err(|e| format!("Source scan worker failed: {e}"))??;
        let scan = with_conn(&db.conn, |conn| {
            repositories::sources::apply_snapshot(conn, &id, snapshot)
        })?;
        if scan.added + scan.changed + scan.renamed + scan.removed > 0 {
            let event = repositories::activity::ActivityEvent {
                id: Uuid::now_v7().to_string(),
                event_type: "source_scanned".to_string(),
                entity_type: Some("source".to_string()),
                entity_id: Some(source.id.clone()),
                space_id: source.space_id.clone(),
                metadata_json: None,
                created_at: String::new(),
            };
            if let Err(error) = with_conn(&db.conn, |conn| {
                repositories::activity::record_deduped(conn, &event, 10).map(|_| ())
            }) {
                log::warn!("Source scan Activity could not be recorded: {error}");
            }
        }
        Ok(scan)
    }
    .await;
    if let Err(error) = &result {
        let _ = with_conn(&db.conn, |conn| {
            repositories::sources::mark_error(conn, &id, error)
        });
    }
    runtime.finish(&id);
    result
}

// ─── Settings ───────────────────────────────────────────

#[tauri::command]
pub fn get_setting(db: State<Database>, key: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let s = repositories::settings::get(conn, &key)?;
        Ok(s.map(|s| json_setting(&s)))
    })
}

#[tauri::command]
pub fn set_setting(
    db: State<Database>,
    key: String,
    value: String,
    value_type: Option<String>,
) -> Result<(), String> {
    with_conn(&db.conn, |conn| {
        repositories::settings::set(
            conn,
            &key,
            &value,
            &value_type.unwrap_or_else(|| "string".into()),
        )
    })
}

#[tauri::command]
pub fn delete_setting(db: State<Database>, key: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::settings::delete(conn, &key))
}

#[tauri::command]
pub fn list_settings(db: State<Database>) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::settings::list(conn)?
            .iter()
            .map(json_setting)
            .collect())
    })
}

// ─── User Profile ───────────────────────────────────────

#[tauri::command]
pub fn get_profile(db: State<Database>) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::profile::get(conn)?.map(|p| json_profile(&p)))
    })
}

#[tauri::command]
pub fn create_profile(db: State<Database>) -> Result<serde_json::Value, String> {
    let id = Uuid::now_v7().to_string();
    with_conn(&db.conn, |conn| {
        Ok(json_profile(&repositories::profile::create(conn, &id)?))
    })
}

#[tauri::command]
pub fn initialize_profile(db: State<Database>) -> Result<serde_json::Value, String> {
    let id = Uuid::now_v7().to_string();
    with_conn(&db.conn, |conn| {
        Ok(json_profile(&repositories::profile::initialize(conn, &id)?))
    })
}

#[tauri::command]
pub fn update_profile(
    db: State<Database>,
    id: String,
    display_name: Option<String>,
    onboarding_completed: Option<bool>,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(
            repositories::profile::update(
                conn,
                &id,
                display_name.as_deref(),
                onboarding_completed,
            )?
            .map(|p| json_profile(&p)),
        )
    })
}

// ─── Spaces ─────────────────────────────────────────────

#[tauri::command]
pub fn create_space(
    db: State<Database>,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    accent: Option<String>,
    template_type: Option<String>,
    parent_space_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let space = repositories::spaces::Space {
        id: Uuid::now_v7().to_string(),
        name,
        description,
        icon,
        accent,
        template_type,
        favourite: false,
        archived_at: None,
        sort_order: 0,
        settings_json: None,
        parent_space_id,
        last_opened_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    with_conn(&db.conn, |conn| {
        Ok(json_space(&repositories::spaces::create(conn, &space)?))
    })
}

#[tauri::command]
pub fn create_space_with_modules(
    db: State<Database>,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    accent: Option<String>,
    template_type: Option<String>,
    parent_space_id: Option<String>,
    module_types: Vec<String>,
) -> Result<serde_json::Value, String> {
    let space = repositories::spaces::Space {
        id: Uuid::now_v7().to_string(),
        name,
        description,
        icon,
        accent,
        template_type,
        favourite: false,
        archived_at: None,
        sort_order: 0,
        settings_json: None,
        parent_space_id,
        last_opened_at: None,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let module_refs: Vec<&str> = module_types.iter().map(|s| s.as_str()).collect();
    with_conn(&db.conn, |conn| {
        let (space, modules) =
            repositories::spaces::create_with_modules(conn, &space, &module_refs)?;
        Ok(serde_json::json!({
            "space": json_space(&space),
            "modules": modules.iter().map(json_module).collect::<Vec<_>>(),
        }))
    })
}

#[tauri::command]
pub fn create_school_space(
    db: State<Database>,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    accent: Option<String>,
    module_types: Vec<String>,
    subjects: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // subjects is Vec<{ name, icon?, accent? }>
    with_conn(&db.conn, |conn| {
        // Create parent School space
        let school = repositories::spaces::Space {
            id: Uuid::now_v7().to_string(),
            name,
            description,
            icon,
            accent,
            template_type: Some("school".to_string()),
            favourite: false,
            archived_at: None,
            sort_order: 0,
            settings_json: None,
            parent_space_id: None,
            last_opened_at: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        let school = repositories::spaces::create(conn, &school)?;
        let module_refs: Vec<&str> = module_types.iter().map(|s| s.as_str()).collect();
        repositories::spaces::set_modules(conn, &school.id, &module_refs)?;

        // Create subject child spaces
        let mut child_spaces = Vec::new();
        for (idx, subj) in subjects.iter().enumerate() {
            let order = idx as i64;
            let name = subj["name"].as_str().unwrap_or("Untitled");
            let icon = subj["icon"].as_str().map(|s| s.to_string());
            let accent = subj["accent"].as_str().map(|s| s.to_string());
            let child = repositories::spaces::Space {
                id: Uuid::now_v7().to_string(),
                name: name.to_string(),
                description: None,
                icon,
                accent,
                template_type: Some("subject".to_string()),
                favourite: false,
                archived_at: None,
                sort_order: order,
                settings_json: None,
                parent_space_id: Some(school.id.clone()),
                last_opened_at: None,
                created_at: String::new(),
                updated_at: String::new(),
            };
            let child = repositories::spaces::create(conn, &child)?;
            // Default modules for subject: notes, tasks, files, ai
            repositories::spaces::set_modules(conn, &child.id, &["notes", "tasks", "files", "ai"])?;
            child_spaces.push(child);
        }

        Ok(serde_json::json!({
            "school": json_space(&school),
            "subjects": child_spaces.iter().map(json_space).collect::<Vec<_>>(),
        }))
    })
}

#[tauri::command]
pub fn get_space(db: State<Database>, id: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        if let Some(space) = repositories::spaces::get_by_id(conn, &id)? {
            let transaction = conn
                .unchecked_transaction()
                .map_err(|error| format!("Space open transaction error: {error}"))?;
            repositories::spaces::touch_last_opened(&transaction, &id)?;
            if space.archived_at.is_none() {
                let event = repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "space_opened".to_string(),
                    entity_type: Some("space".to_string()),
                    entity_id: Some(id.clone()),
                    space_id: Some(id.clone()),
                    metadata_json: None,
                    created_at: String::new(),
                };
                repositories::activity::record_deduped(&transaction, &event, 30)?;
            }
            let modules = repositories::spaces::list_modules(&transaction, &id)?;
            let children = repositories::spaces::list_by_parent(&transaction, &id)?;
            transaction
                .commit()
                .map_err(|error| format!("Space open transaction commit error: {error}"))?;
            Ok(Some(serde_json::json!({
                "space": json_space(&space),
                "modules": modules.iter().map(json_module).collect::<Vec<_>>(),
                "children": children.iter().map(json_space).collect::<Vec<_>>(),
            })))
        } else {
            Ok(None)
        }
    })
}

#[tauri::command]
pub fn list_spaces(
    db: State<Database>,
    include_archived: Option<bool>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(
            repositories::spaces::list(conn, include_archived.unwrap_or(false))?
                .iter()
                .map(json_space)
                .collect(),
        )
    })
}

#[tauri::command]
pub fn list_top_level_spaces(db: State<Database>) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::spaces::list_top_level(conn)?
            .iter()
            .map(json_space)
            .collect())
    })
}

#[tauri::command]
pub fn list_child_spaces(
    db: State<Database>,
    parent_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::spaces::list_by_parent(conn, &parent_id)?
            .iter()
            .map(json_space)
            .collect())
    })
}

#[tauri::command]
pub fn update_space(
    db: State<Database>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    icon: Option<String>,
    accent: Option<String>,
    settings_json: Option<String>,
    parent_space_id: Option<Option<String>>,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::spaces::update(
            conn,
            &id,
            name.as_deref(),
            description.as_deref(),
            icon.as_deref(),
            accent.as_deref(),
            settings_json.as_deref(),
            parent_space_id,
        )?
        .map(|s| json_space(&s)))
    })
}

#[tauri::command]
pub fn set_space_modules(
    db: State<Database>,
    space_id: String,
    module_types: Vec<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let refs: Vec<&str> = module_types.iter().map(|s| s.as_str()).collect();
    with_conn(&db.conn, |conn| {
        Ok(repositories::spaces::set_modules(conn, &space_id, &refs)?
            .iter()
            .map(json_module)
            .collect())
    })
}

#[tauri::command]
pub fn get_space_modules(
    db: State<Database>,
    space_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::spaces::list_modules(conn, &space_id)?
            .iter()
            .map(json_module)
            .collect())
    })
}

#[tauri::command]
pub fn archive_space(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::spaces::archive(conn, &id))
}

#[tauri::command]
pub fn restore_space(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::spaces::restore(conn, &id))
}

#[tauri::command]
pub fn delete_space(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::spaces::delete_permanent(conn, &id)
    })
}

#[tauri::command]
pub fn favourite_space(db: State<Database>, id: String, fav: bool) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::spaces::set_favourite(conn, &id, fav)
    })
}

#[tauri::command]
pub fn reorder_spaces(db: State<Database>, ids: Vec<String>) -> Result<(), String> {
    with_conn(&db.conn, |conn| repositories::spaces::reorder(conn, &ids))
}

#[tauri::command]
pub fn duplicate_space(db: State<Database>, id: String) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        Ok(json_space(&repositories::spaces::duplicate(conn, &id)?))
    })
}

// ─── Vault ───────────────────────────────────────────────

fn vault_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("vault"))
        .map_err(|error| format!("Failed to resolve Vault storage: {}", error))
}

fn json_vault_item(item: &repositories::vault::VaultItem) -> serde_json::Value {
    serde_json::json!({
        "id": item.id,
        "space_id": item.space_id,
        "storage_mode": item.storage_mode,
        "display_title": item.display_title,
        "original_name": item.original_name,
        "media_type": item.media_type,
        "size_bytes": item.size_bytes,
        "tags": item.tags,
        "created_at": item.created_at,
        "updated_at": item.updated_at,
    })
}

#[tauri::command]
pub async fn import_vault_item(
    app: AppHandle,
    db: State<'_, Database>,
    path: String,
    storage_mode: String,
    space_id: Option<String>,
    display_title: Option<String>,
    tags: Vec<String>,
) -> Result<serde_json::Value, String> {
    let id = Uuid::now_v7().to_string();
    let root = vault_root(&app)?;
    let worker_root = root.clone();
    let worker_id = id.clone();
    let worker_mode = storage_mode.clone();
    let prepared = tauri::async_runtime::spawn_blocking(move || {
        vault_storage::prepare_import(&worker_root, &worker_id, &PathBuf::from(path), &worker_mode)
    })
    .await
    .map_err(|error| format!("Vault import worker failed: {}", error))??;

    let title = display_title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&prepared.original_name)
        .to_string();
    let new_item = repositories::vault::NewVaultItem {
        id,
        space_id,
        storage_mode,
        display_title: title,
        original_name: prepared.original_name,
        stored_path: prepared.stored_path,
        media_type: prepared.media_type,
        size_bytes: prepared.size_bytes,
        tags,
    };
    let result = with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Vault transaction error: {}", error))?;
        let item = repositories::vault::create(&transaction, &new_item)?;
        repositories::activity::record(
            &transaction,
            &repositories::activity::ActivityEvent {
                id: Uuid::now_v7().to_string(),
                event_type: "vault_imported".to_string(),
                entity_type: Some("vault_item".to_string()),
                entity_id: Some(item.id.clone()),
                space_id: item.space_id.clone(),
                metadata_json: Some(
                    serde_json::json!({ "storageMode": item.storage_mode }).to_string(),
                ),
                created_at: String::new(),
            },
        )?;
        transaction
            .commit()
            .map_err(|error| format!("Vault transaction commit error: {}", error))?;
        Ok(json_vault_item(&item))
    });
    if result.is_err() {
        vault_storage::discard_import(prepared.managed_directory.as_deref());
    }
    result
}

#[tauri::command]
pub fn get_vault_item(
    db: State<Database>,
    id: String,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::vault::get_by_id(conn, &id)?.map(|item| json_vault_item(&item)))
    })
}

#[tauri::command]
pub fn list_vault_items(
    db: State<Database>,
    filter: Option<repositories::vault::VaultFilter>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(
            repositories::vault::list(conn, &filter.unwrap_or_default())?
                .iter()
                .map(json_vault_item)
                .collect(),
        )
    })
}

#[tauri::command]
pub fn update_vault_item(
    db: State<Database>,
    id: String,
    input: repositories::vault::VaultUpdateInput,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Vault transaction error: {}", error))?;
        let item = repositories::vault::update(&transaction, &id, &input)?;
        if let Some(item) = item.as_ref() {
            repositories::activity::record(
                &transaction,
                &repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "vault_updated".to_string(),
                    entity_type: Some("vault_item".to_string()),
                    entity_id: Some(item.id.clone()),
                    space_id: item.space_id.clone(),
                    metadata_json: None,
                    created_at: String::new(),
                },
            )?;
        }
        transaction
            .commit()
            .map_err(|error| format!("Vault transaction commit error: {}", error))?;
        Ok(item.as_ref().map(json_vault_item))
    })
}

#[tauri::command]
pub async fn remove_vault_item(
    app: AppHandle,
    db: State<'_, Database>,
    id: String,
) -> Result<bool, String> {
    let item = with_conn(&db.conn, |conn| repositories::vault::get_by_id(conn, &id))?;
    let Some(item) = item else {
        return Ok(false);
    };
    let root = vault_root(&app)?;
    let worker_root = root.clone();
    let worker_item = item.clone();
    let quarantine = tauri::async_runtime::spawn_blocking(move || {
        vault_storage::quarantine_managed(&worker_root, &worker_item)
    })
    .await
    .map_err(|error| format!("Vault removal worker failed: {}", error))??;

    let deletion = with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Vault transaction error: {}", error))?;
        let removed = repositories::vault::delete(&transaction, &id)?;
        if removed {
            repositories::activity::record(
                &transaction,
                &repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "vault_removed".to_string(),
                    entity_type: Some("vault_item".to_string()),
                    entity_id: Some(id),
                    space_id: item.space_id.clone(),
                    metadata_json: Some(
                        serde_json::json!({ "storageMode": item.storage_mode }).to_string(),
                    ),
                    created_at: String::new(),
                },
            )?;
        }
        transaction
            .commit()
            .map_err(|error| format!("Vault transaction commit error: {}", error))?;
        Ok(removed)
    });

    if let Err(error) = deletion {
        if let Some((quarantined, original)) = quarantine.as_ref() {
            vault_storage::restore_quarantine(quarantined, original)?;
        }
        return Err(error);
    }
    if let Some((quarantined, _)) = quarantine {
        let cleanup_root = root;
        if let Err(error) = tauri::async_runtime::spawn_blocking(move || {
            vault_storage::finalize_quarantine(&cleanup_root, &quarantined)
        })
        .await
        .map_err(|error| format!("Vault cleanup worker failed: {}", error))?
        {
            log::warn!("{}", error);
        }
    }
    deletion
}

fn resolve_vault_item_path(app: &AppHandle, db: &Database, id: &str) -> Result<PathBuf, String> {
    let item = with_conn(&db.conn, |conn| repositories::vault::get_by_id(conn, id))?
        .ok_or_else(|| "Vault item does not exist".to_string())?;
    vault_storage::resolve_item_path(&vault_root(app)?, &item)
}

#[tauri::command]
pub fn open_vault_item(app: AppHandle, db: State<Database>, id: String) -> Result<(), String> {
    let path = resolve_vault_item_path(&app, &db, &id)?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|error| format!("Failed to open Vault file: {}", error))
}

#[tauri::command]
pub fn reveal_vault_item(app: AppHandle, db: State<Database>, id: String) -> Result<(), String> {
    let path = resolve_vault_item_path(&app, &db, &id)?;
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|error| format!("Failed to reveal Vault file: {}", error))
}

// ─── Activity ───────────────────────────────────────────

#[tauri::command]
pub fn list_activity(
    db: State<Database>,
    space_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<repositories::activity::ActivityItem>, String> {
    with_conn(&db.conn, |conn| {
        repositories::activity::list_items(conn, space_id.as_deref(), limit)
    })
}

#[tauri::command]
pub fn get_space_continuity(
    db: State<Database>,
    space_id: String,
) -> Result<repositories::continuity::SpaceContinuity, String> {
    with_conn(&db.conn, |conn| {
        repositories::continuity::get(conn, &space_id)
    })
}

// ─── Tasks ──────────────────────────────────────────────

#[tauri::command]
pub fn create_task(
    db: State<Database>,
    input: repositories::tasks::TaskInput,
) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Task transaction error: {}", error))?;
        let task = repositories::tasks::create(&transaction, &input)?;
        repositories::activity::record(
            &transaction,
            &repositories::activity::ActivityEvent {
                id: Uuid::now_v7().to_string(),
                event_type: "task_created".to_string(),
                entity_type: Some("task".to_string()),
                entity_id: Some(task.id.clone()),
                space_id: task.space_id.clone(),
                metadata_json: None,
                created_at: String::new(),
            },
        )?;
        let value = serde_json::to_value(task)
            .map_err(|error| format!("Task serialization error: {}", error))?;
        transaction
            .commit()
            .map_err(|error| format!("Task transaction commit error: {}", error))?;
        Ok(value)
    })
}

#[tauri::command]
pub fn get_task(db: State<Database>, id: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        repositories::tasks::get_by_id(conn, &id)?
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| format!("Task serialization error: {}", error))
    })
}

#[tauri::command]
pub fn list_tasks(
    db: State<Database>,
    filter: Option<repositories::tasks::TaskFilter>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        repositories::tasks::list(conn, &filter.unwrap_or_default())?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Task serialization error: {}", error))
    })
}

#[tauri::command]
pub fn update_task(
    db: State<Database>,
    id: String,
    input: repositories::tasks::TaskInput,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Task transaction error: {}", error))?;
        let previous = repositories::tasks::get_by_id(&transaction, &id)?;
        let updated = repositories::tasks::update(&transaction, &id, &input)?;
        if previous.as_ref().is_some_and(|task| task.status != "done")
            && updated.as_ref().is_some_and(|task| task.status == "done")
        {
            if let Some(task) = updated.as_ref() {
                repositories::activity::record(
                    &transaction,
                    &repositories::activity::ActivityEvent {
                        id: Uuid::now_v7().to_string(),
                        event_type: "task_completed".to_string(),
                        entity_type: Some("task".to_string()),
                        entity_id: Some(task.id.clone()),
                        space_id: task.space_id.clone(),
                        metadata_json: None,
                        created_at: String::new(),
                    },
                )?;
            }
        }
        let value = updated
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| format!("Task serialization error: {}", error))?;
        transaction
            .commit()
            .map_err(|error| format!("Task transaction commit error: {}", error))?;
        Ok(value)
    })
}

#[tauri::command]
pub fn list_task_attention(
    db: State<Database>,
    today: String,
    horizon: String,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        repositories::tasks::list_attention(conn, &today, &horizon, limit.unwrap_or(20))?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Task serialization error: {}", error))
    })
}

#[tauri::command]
pub fn archive_task(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Task transaction error: {}", error))?;
        let task = repositories::tasks::get_by_id(&transaction, &id)?;
        let archived = repositories::tasks::archive(&transaction, &id)?;
        if archived {
            repositories::activity::record(
                &transaction,
                &repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "task_archived".to_string(),
                    entity_type: Some("task".to_string()),
                    entity_id: Some(id),
                    space_id: task.and_then(|value| value.space_id),
                    metadata_json: None,
                    created_at: String::new(),
                },
            )?;
        }
        transaction
            .commit()
            .map_err(|error| format!("Task transaction commit error: {}", error))?;
        Ok(archived)
    })
}

#[tauri::command]
pub fn restore_task(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::tasks::restore(conn, &id))
}

#[tauri::command]
pub fn delete_task(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::tasks::delete_permanent(conn, &id)
    })
}

// ─── Memory ─────────────────────────────────────────────

#[tauri::command]
pub fn create_memory(
    db: State<Database>,
    input: repositories::memory::MemoryInput,
) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Memory transaction error: {}", error))?;
        let item = repositories::memory::create(&transaction, &input)?;
        repositories::activity::record(
            &transaction,
            &repositories::activity::ActivityEvent {
                id: Uuid::now_v7().to_string(),
                event_type: "memory_created".to_string(),
                entity_type: Some("memory".to_string()),
                entity_id: Some(item.id.clone()),
                space_id: item.space_id.clone(),
                metadata_json: Some(format!(r#"{{"category":"{}"}}"#, item.category)),
                created_at: String::new(),
            },
        )?;
        let value = serde_json::to_value(item)
            .map_err(|error| format!("Memory serialization error: {}", error))?;
        transaction
            .commit()
            .map_err(|error| format!("Memory transaction commit error: {}", error))?;
        Ok(value)
    })
}

#[tauri::command]
pub fn get_memory(db: State<Database>, id: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        repositories::memory::get_by_id(conn, &id)?
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| format!("Memory serialization error: {}", error))
    })
}

#[tauri::command]
pub fn list_memory(
    db: State<Database>,
    filter: Option<repositories::memory::MemoryFilter>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        repositories::memory::list(conn, &filter.unwrap_or_default())?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("Memory serialization error: {}", error))
    })
}

#[tauri::command]
pub fn update_memory(
    db: State<Database>,
    id: String,
    input: repositories::memory::MemoryInput,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Memory transaction error: {}", error))?;
        let item = repositories::memory::update(&transaction, &id, &input)?;
        if let Some(item) = item.as_ref() {
            repositories::activity::record(
                &transaction,
                &repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "memory_updated".to_string(),
                    entity_type: Some("memory".to_string()),
                    entity_id: Some(item.id.clone()),
                    space_id: item.space_id.clone(),
                    metadata_json: Some(format!(r#"{{"category":"{}"}}"#, item.category)),
                    created_at: String::new(),
                },
            )?;
        }
        let value = item
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| format!("Memory serialization error: {}", error))?;
        transaction
            .commit()
            .map_err(|error| format!("Memory transaction commit error: {}", error))?;
        Ok(value)
    })
}

#[tauri::command]
pub fn delete_memory(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Memory transaction error: {}", error))?;
        let item = repositories::memory::get_by_id(&transaction, &id)?;
        let deleted = repositories::memory::delete(&transaction, &id)?;
        if let Some(item) = item {
            repositories::activity::record(
                &transaction,
                &repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "memory_deleted".to_string(),
                    entity_type: Some("memory".to_string()),
                    entity_id: Some(item.id),
                    space_id: item.space_id,
                    metadata_json: Some(format!(r#"{{"category":"{}"}}"#, item.category)),
                    created_at: String::new(),
                },
            )?;
        }
        transaction
            .commit()
            .map_err(|error| format!("Memory transaction commit error: {}", error))?;
        Ok(deleted)
    })
}

// ─── Notes ───────────────────────────────────────────────

#[tauri::command]
pub fn create_note(db: State<Database>, space_id: String) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Note transaction error: {error}"))?;
        let note = repositories::notes::create(&transaction, &space_id)?;
        repositories::activity::record(
            &transaction,
            &repositories::activity::ActivityEvent {
                id: Uuid::now_v7().to_string(),
                event_type: "note_created".to_string(),
                entity_type: Some("note".to_string()),
                entity_id: Some(note.id.clone()),
                space_id: Some(note.space_id.clone()),
                metadata_json: None,
                created_at: String::new(),
            },
        )?;
        let value = json_note(&note);
        transaction
            .commit()
            .map_err(|error| format!("Note transaction commit error: {error}"))?;
        Ok(value)
    })
}

#[tauri::command]
pub fn get_note(db: State<Database>, id: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        if let Some(note) = repositories::notes::get_by_id(conn, &id)? {
            repositories::notes::touch_last_opened(conn, &id)?;
            Ok(Some(json_note(&note)))
        } else {
            Ok(None)
        }
    })
}

#[tauri::command]
pub fn update_note(
    db: State<Database>,
    id: String,
    title: Option<String>,
    content: Option<String>,
    excerpt: Option<String>,
    expected_revision: Option<i64>,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let transaction = conn
            .unchecked_transaction()
            .map_err(|error| format!("Note transaction error: {error}"))?;
        let updated = repositories::notes::update(
            &transaction,
            &id,
            title.as_deref(),
            content.as_deref(),
            excerpt.as_deref(),
            expected_revision,
        )?;
        if let Some(note) = updated.as_ref() {
            let event = repositories::activity::ActivityEvent {
                id: Uuid::now_v7().to_string(),
                event_type: "note_edited".to_string(),
                entity_type: Some("note".to_string()),
                entity_id: Some(note.id.clone()),
                space_id: Some(note.space_id.clone()),
                metadata_json: None,
                created_at: String::new(),
            };
            repositories::activity::record_deduped(&transaction, &event, 10)?;
        }
        let value = updated.map(|note| json_note(&note));
        transaction
            .commit()
            .map_err(|error| format!("Note transaction commit error: {error}"))?;
        Ok(value)
    })
}

#[tauri::command]
pub fn list_notes_by_space(
    db: State<Database>,
    space_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::notes::list_by_space(conn, &space_id)?
            .iter()
            .map(json_note_list_item)
            .collect())
    })
}

#[tauri::command]
pub fn list_recent_notes(
    db: State<Database>,
    space_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(
            repositories::notes::list_recent(conn, space_id.as_deref(), limit.unwrap_or(10))?
                .iter()
                .map(json_note_list_item)
                .collect(),
        )
    })
}

#[tauri::command]
pub fn list_pinned_notes(
    db: State<Database>,
    space_id: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(repositories::notes::list_pinned(conn, space_id.as_deref())?
            .iter()
            .map(json_note_list_item)
            .collect())
    })
}

#[tauri::command]
pub fn list_archived_notes(
    db: State<Database>,
    space_id: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(
            repositories::notes::list_archived(conn, space_id.as_deref())?
                .iter()
                .map(json_note_list_item)
                .collect(),
        )
    })
}

#[tauri::command]
pub fn search_notes(
    db: State<Database>,
    query: String,
    space_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        Ok(
            repositories::notes::search(conn, &query, space_id.as_deref(), limit.unwrap_or(20))?
                .iter()
                .map(json_note_search_result)
                .collect(),
        )
    })
}

#[tauri::command]
pub fn pin_note(db: State<Database>, id: String, pinned: bool) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::notes::set_pinned(conn, &id, pinned)
    })
}

#[tauri::command]
pub fn archive_note(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::notes::archive(conn, &id))
}

#[tauri::command]
pub fn restore_note(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::notes::restore(conn, &id))
}

#[tauri::command]
pub fn delete_note(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::notes::delete_permanent(conn, &id)
    })
}

#[tauri::command]
pub fn move_note(db: State<Database>, id: String, new_space_id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::notes::move_to_space(conn, &id, &new_space_id)
    })
}

#[tauri::command]
pub fn duplicate_note(db: State<Database>, id: String) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        Ok(json_note(&repositories::notes::duplicate(conn, &id)?))
    })
}

// ─── JSON helpers ───────────────────────────────────────

fn json_note(n: &repositories::notes::Note) -> serde_json::Value {
    serde_json::json!({
        "id": n.id, "space_id": n.space_id, "title": n.title,
        "content": n.content, "content_format": n.content_format,
        "excerpt": n.excerpt, "pinned": n.pinned, "revision": n.revision,
        "archived_at": n.archived_at, "created_at": n.created_at,
        "updated_at": n.updated_at, "last_opened_at": n.last_opened_at,
    })
}

fn json_note_list_item(n: &repositories::notes::NoteListItem) -> serde_json::Value {
    serde_json::json!({
        "id": n.id, "space_id": n.space_id, "title": n.title,
        "excerpt": n.excerpt, "content_format": n.content_format,
        "pinned": n.pinned, "revision": n.revision,
        "archived_at": n.archived_at, "created_at": n.created_at,
        "updated_at": n.updated_at, "last_opened_at": n.last_opened_at,
    })
}

fn json_note_search_result(n: &repositories::notes::NoteSearchResult) -> serde_json::Value {
    serde_json::json!({
        "id": n.id, "space_id": n.space_id, "title": n.title,
        "excerpt": n.excerpt, "pinned": n.pinned,
        "archived_at": n.archived_at, "updated_at": n.updated_at,
    })
}

fn json_space(s: &repositories::spaces::Space) -> serde_json::Value {
    serde_json::json!({
        "id": s.id, "name": s.name, "description": s.description,
        "icon": s.icon, "accent": s.accent, "template_type": s.template_type,
        "favourite": s.favourite, "archived_at": s.archived_at,
        "sort_order": s.sort_order, "settings_json": s.settings_json,
        "parent_space_id": s.parent_space_id, "last_opened_at": s.last_opened_at,
        "created_at": s.created_at, "updated_at": s.updated_at,
    })
}

fn json_module(m: &repositories::spaces::ModuleInstance) -> serde_json::Value {
    serde_json::json!({
        "id": m.id, "space_id": m.space_id, "module_type": m.module_type,
        "title": m.title, "config_json": m.config_json, "layout_json": m.layout_json,
        "created_at": m.created_at, "updated_at": m.updated_at,
    })
}

fn json_setting(s: &repositories::settings::AppSetting) -> serde_json::Value {
    serde_json::json!({"key": s.key, "value": s.value, "value_type": s.value_type, "created_at": s.created_at, "updated_at": s.updated_at})
}

fn json_profile(p: &repositories::profile::UserProfile) -> serde_json::Value {
    serde_json::json!({"id": p.id, "display_name": p.display_name, "onboarding_completed": p.onboarding_completed, "created_at": p.created_at, "updated_at": p.updated_at})
}

// ─── AI Credentials ─────────────────────────────────────

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub provider: String,
    pub configured: bool,
    pub status: String,
}

fn provider_status(db: &Database, provider_name: &str) -> ProviderStatus {
    match credentials::get_provider_key(db, provider_name) {
        Ok(Some(_)) => ProviderStatus {
            provider: provider_name.to_string(),
            configured: true,
            status: "configured".to_string(),
        },
        Ok(None) => ProviderStatus {
            provider: provider_name.to_string(),
            configured: false,
            status: "missing".to_string(),
        },
        Err(_) => ProviderStatus {
            provider: provider_name.to_string(),
            configured: false,
            status: "unavailable".to_string(),
        },
    }
}

#[tauri::command]
pub fn ai_list_models() -> Result<Vec<provider::ModelInfo>, String> {
    Ok(provider::model_catalog())
}

#[tauri::command]
pub fn ai_parse_action_proposals(
    db: State<Database>,
    conversation_id: String,
    message_id: String,
) -> Result<Vec<proposals::ActionDraft>, String> {
    with_conn(&db.conn, |conn| {
        let actions = stored_ai_proposals(conn, &conversation_id, &message_id)?;
        Ok(proposals::describe(&actions))
    })
}

#[tauri::command]
pub fn ai_preview_action_proposal(
    db: State<Database>,
    runtime: State<ActionRuntime>,
    conversation_id: String,
    message_id: String,
    index: usize,
) -> Result<actions::ActionPreview, String> {
    with_conn(&db.conn, |conn| {
        let mut proposals = stored_ai_proposals(conn, &conversation_id, &message_id)?;
        if index >= proposals.len() {
            return Err("AI Action proposal index is out of range.".to_string());
        }
        let request = proposals.remove(index);
        actions::preview_ai(
            conn,
            &runtime,
            request,
            actions::ActionOrigin {
                conversation_id,
                message_id,
            },
        )
    })
}

fn stored_ai_proposals(
    conn: &rusqlite::Connection,
    conversation_id: &str,
    message_id: &str,
) -> Result<Vec<actions::ActionRequest>, String> {
    let conversation = repositories::conversations::get_conversation(conn, conversation_id)?
        .ok_or_else(|| "Conversation not found.".to_string())?;
    let message = repositories::conversations::get_message(conn, message_id)?
        .ok_or_else(|| "AI proposal message not found.".to_string())?;
    if message.conversation_id != conversation.id
        || message.role != "assistant"
        || message.status != "complete"
    {
        return Err(
            "Only a completed assistant message from this conversation can be reviewed."
                .to_string(),
        );
    }
    proposals::parse(&message.content, conversation.space_id.as_deref())
}

#[tauri::command]
pub fn ai_list_providers() -> Result<Vec<provider::ProviderInfo>, String> {
    Ok(provider::provider_catalog())
}

#[tauri::command]
pub fn ai_list_provider_statuses(db: State<Database>) -> Result<Vec<ProviderStatus>, String> {
    Ok(provider::provider_catalog()
        .iter()
        .map(|item| provider_status(&db, &item.id))
        .collect())
}

#[tauri::command]
pub fn ai_set_provider_api_key(
    db: State<Database>,
    provider: String,
    api_key: String,
) -> Result<(), String> {
    if !provider::provider_catalog()
        .iter()
        .any(|item| item.id == provider)
    {
        return Err("Unknown AI provider.".to_string());
    }
    let api_key = api_key.trim();
    if api_key.is_empty() || api_key.len() > 512 {
        return Err("Enter a valid API key.".to_string());
    }
    credentials::store_provider_key(&db, &provider, api_key)
}

#[tauri::command]
pub fn ai_remove_provider_api_key(db: State<Database>, provider: String) -> Result<bool, String> {
    credentials::remove_provider_key(&db, &provider)
}

#[tauri::command]
pub async fn ai_test_provider_connection(
    db: State<'_, Database>,
    provider: String,
    model: String,
) -> Result<String, String> {
    provider::validate_provider_model(&provider, &model).map_err(|error| error.message)?;
    let key = credentials::get_provider_key(&db, &provider)?
        .ok_or_else(|| "No API key configured for this provider.".to_string())?;
    let config =
        ProviderConfig::for_route(&provider, key, &model).map_err(|error| error.message)?;
    provider::create_provider(config)
        .map_err(|error| error.message)?
        .test_connection()
        .await
        .map_err(|error| error.message)?;
    Ok("Connection successful".to_string())
}

// ─── AI Conversations ────────────────────────────────────

#[tauri::command]
pub fn ai_create_conversation(
    db: State<Database>,
    space_id: Option<String>,
    title: Option<String>,
    provider: Option<String>,
    model: Option<String>,
) -> Result<serde_json::Value, String> {
    let provider = provider.unwrap_or_else(|| {
        if model
            .as_deref()
            .is_some_and(|value| value.starts_with("deepseek-"))
        {
            "deepseek".to_string()
        } else if model
            .as_deref()
            .is_some_and(|value| value.starts_with("gpt-"))
        {
            "openai".to_string()
        } else {
            "auto".to_string()
        }
    });
    let model = model.unwrap_or_else(|| {
        if provider == "auto" {
            "auto"
        } else if provider == "openai" {
            "gpt-5-mini"
        } else {
            "deepseek-v4-flash"
        }
        .to_string()
    });
    with_conn(&db.conn, |conn| {
        let conv = repositories::conversations::create_conversation(
            conn,
            space_id.as_deref(),
            title.as_deref().unwrap_or("New conversation"),
            &provider,
            &model,
        )?;
        serde_json::to_value(conv).map_err(|e| format!("Serialize error: {}", e))
    })
}

#[tauri::command]
pub fn ai_get_conversation(
    db: State<Database>,
    id: String,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let conv = repositories::conversations::get_conversation(conn, &id)?;
        Ok(conv.map(|c| serde_json::to_value(c).unwrap()))
    })
}

#[tauri::command]
pub fn ai_list_conversations(
    db: State<Database>,
    space_id: Option<String>,
    include_archived: Option<bool>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let convs = repositories::conversations::list_conversations(
            conn,
            space_id.as_deref(),
            include_archived.unwrap_or(false),
        )?;
        Ok(convs
            .iter()
            .map(|c| serde_json::to_value(c).unwrap())
            .collect())
    })
}

#[tauri::command]
pub fn ai_update_conversation(
    db: State<Database>,
    id: String,
    title: Option<String>,
    archived: Option<bool>,
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let conv = repositories::conversations::update_conversation(
            conn,
            &id,
            title.as_deref(),
            archived,
        )?;
        Ok(conv.map(|c| serde_json::to_value(c).unwrap()))
    })
}

#[tauri::command]
pub fn ai_delete_conversation(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::conversations::delete_conversation(conn, &id)
    })
}

// ─── AI Messages ─────────────────────────────────────────

#[tauri::command]
pub fn ai_list_messages(
    db: State<Database>,
    conversation_id: String,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let msgs = repositories::conversations::list_messages(conn, &conversation_id, limit)?;
        Ok(msgs
            .iter()
            .map(|m| serde_json::to_value(m).unwrap())
            .collect())
    })
}

// ─── AI Chat streaming ───────────────────────────────────

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum AiStreamEvent {
    Started {
        request_id: String,
        user_message: Box<repositories::conversations::AiMessage>,
        assistant_message: Box<repositories::conversations::AiMessage>,
    },
    Delta {
        content: String,
    },
    Complete {
        message: Box<repositories::conversations::AiMessage>,
    },
    Cancelled {
        message: Box<repositories::conversations::AiMessage>,
    },
    Failed {
        code: String,
        message: String,
        assistant_message: Box<repositories::conversations::AiMessage>,
    },
}

#[tauri::command]
pub async fn ai_stream_message(
    db: State<'_, Database>,
    runtime: State<'_, AiRuntime>,
    request_id: String,
    conversation_id: String,
    content: String,
    retry_user_message_id: Option<String>,
    mode: Option<String>,
    on_event: Channel<AiStreamEvent>,
) -> Result<(), String> {
    let mut content = content.trim().to_string();
    if retry_user_message_id.is_none() && (content.is_empty() || content.chars().count() > 32_000) {
        return Err("Message must contain 1 to 32,000 characters.".to_string());
    }
    let mode = mode.unwrap_or_else(|| "ask".to_string());
    let mode_instruction = match mode.as_str() {
        "ask" => None,
        "summarize" => Some("Summarize the attached context or the user's text concisely. Preserve important facts and uncertainties."),
        "explain" => Some("Explain the topic clearly in plain language. Use the attached context only when relevant."),
        "plan" => Some("Create a practical ordered plan with clear next actions. Do not claim to execute actions."),
        "rewrite" => Some("Rewrite the supplied text while preserving its meaning. Return the rewritten text without unnecessary preamble."),
        "create_tasks" | "propose_actions" => Some("Propose Tasks or Notes but do not execute them. Return only valid JSON shaped as {\"actions\":[{\"type\":\"createTask\",\"title\":\"...\",\"description\":\"...\",\"dueDate\":null},{\"type\":\"createNote\",\"title\":\"...\",\"content\":\"...\"}]} with 1 to 20 actions. Use only createTask and createNote. Do not include space IDs or any other fields."),
        _ => return Err("Unsupported AI response mode.".to_string()),
    };
    let conversation = with_conn(&db.conn, |conn| {
        repositories::conversations::get_conversation(conn, &conversation_id)?
            .ok_or_else(|| "Conversation not found.".to_string())
    })?;
    if conversation.archived_at.is_some() {
        return Err("Restore this conversation before sending a message.".to_string());
    }
    let route = if conversation.provider == "auto" {
        let deepseek_configured = credentials::get_provider_key(&db, "deepseek")?.is_some();
        let openai_configured = credentials::get_provider_key(&db, "openai")?.is_some();
        routing::select_route(
            "auto",
            "auto",
            &mode,
            deepseek_configured,
            openai_configured,
        )?
    } else {
        routing::select_route(
            &conversation.provider,
            &conversation.model,
            &mode,
            false,
            false,
        )?
    };
    let key = credentials::get_provider_key(&db, &route.provider)?.ok_or_else(|| {
        format!(
            "No API key configured for {}. Configure it in Settings first.",
            route.provider
        )
    })?;
    let retry_user = if let Some(message_id) = retry_user_message_id.as_deref() {
        let message = with_conn(&db.conn, |conn| {
            repositories::conversations::get_message(conn, message_id)?
                .ok_or_else(|| "Retry source message not found.".to_string())
        })?;
        if message.conversation_id != conversation_id || message.role != "user" {
            return Err("Retry source must be a user message from this conversation.".to_string());
        }
        content.clone_from(&message.content);
        Some(message)
    } else {
        None
    };
    let config = ProviderConfig::for_route(&route.provider, key, &route.model)
        .map_err(|error| error.message)?;
    let provider = provider::create_provider(config).map_err(|error| error.message)?;
    let cancellation = runtime.start(&request_id)?;
    let _request_guard = AiRequestGuard {
        runtime: &runtime,
        request_id: request_id.clone(),
    };

    let (user_message, assistant_message, mut chat_messages, context_count) =
        with_conn(&db.conn, |conn| {
            let transaction = conn
                .unchecked_transaction()
                .map_err(|error| format!("AI transaction error: {}", error))?;
            let user_message = if let Some(message) = retry_user.clone() {
                message
            } else {
                repositories::conversations::add_message(
                    &transaction,
                    &conversation_id,
                    "user",
                    &content,
                    "complete",
                )?
            };
            let assistant_message = repositories::conversations::add_message(
                &transaction,
                &conversation_id,
                "assistant",
                "",
                "streaming",
            )?;
            if retry_user.is_none() && conversation.title == "New conversation" {
                let title: String = content.chars().take(60).collect();
                repositories::conversations::update_conversation(
                    &transaction,
                    &conversation_id,
                    Some(&title),
                    None,
                )?;
            }
            let history = repositories::conversations::list_messages(
                &transaction,
                &conversation_id,
                Some(50),
            )?;
            let attachments =
                repositories::conversations::list_context_items(&transaction, &conversation_id)?;
            let resolved =
                context::resolve_all(&transaction, conversation.space_id.as_deref(), &attachments)?;
            let context_count = resolved.len();
            let mut chat_messages = Vec::new();
            if let Some(system) = context::system_message(&resolved)? {
                chat_messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: system,
                });
            }
            if let Some(instruction) = mode_instruction {
                chat_messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: instruction.to_string(),
                });
            }
            chat_messages.extend(
                history
                    .iter()
                    .filter(|message| {
                        message.status == "complete"
                            && ["user", "assistant"].contains(&message.role.as_str())
                    })
                    .map(|message| ChatMessage {
                        role: message.role.clone(),
                        content: message.content.clone(),
                    }),
            );
            transaction
                .commit()
                .map_err(|error| format!("AI transaction commit error: {}", error))?;
            Ok((
                user_message,
                assistant_message,
                chat_messages,
                context_count,
            ))
        })?;
    if on_event
        .send(AiStreamEvent::Started {
            request_id: request_id.clone(),
            user_message: Box::new(user_message),
            assistant_message: Box::new(assistant_message.clone()),
        })
        .is_err()
    {
        cancellation.cancel();
    }

    if chat_messages.is_empty() {
        chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content,
        });
    }
    let request = ChatCompletionRequest {
        model: route.model.clone(),
        messages: chat_messages,
        temperature: None,
        max_tokens: None,
        top_p: None,
        stream: Some(true),
        thinking: None,
    };
    let collected = std::sync::Mutex::new(String::new());
    let result = provider
        .stream_chat(&request, cancellation, &|delta| {
            collected
                .lock()
                .map_err(|_| "AI response buffer is unavailable.".to_string())?
                .push_str(&delta);
            on_event
                .send(AiStreamEvent::Delta { content: delta })
                .map_err(|_| "AI response listener closed.".to_string())
        })
        .await;
    let final_content = collected
        .into_inner()
        .map_err(|_| "AI response buffer is unavailable.".to_string())?;
    let provenance = repositories::conversations::AiRouteProvenance {
        provider: &route.provider,
        model: &route.model,
        routing_mode: &route.routing_mode,
        route_reason: &route.reason,
    };

    let terminal = match result {
        Ok(()) => {
            let metadata =
                serde_json::json!({ "mode": mode, "contextCount": context_count }).to_string();
            let message = with_conn(&db.conn, |conn| {
                let transaction = conn
                    .unchecked_transaction()
                    .map_err(|error| format!("AI completion transaction error: {error}"))?;
                let message = repositories::conversations::finish_message(
                    &transaction,
                    &assistant_message.id,
                    &final_content,
                    "complete",
                    None,
                    Some(&metadata),
                    Some(&provenance),
                )?
                .ok_or_else(|| "Assistant message no longer exists.".to_string())?;
                let event = repositories::activity::ActivityEvent {
                    id: Uuid::now_v7().to_string(),
                    event_type: "ai_conversation_used".to_string(),
                    entity_type: Some("conversation".to_string()),
                    entity_id: Some(conversation.id.clone()),
                    space_id: conversation.space_id.clone(),
                    metadata_json: None,
                    created_at: String::new(),
                };
                repositories::activity::record_deduped(&transaction, &event, 10)?;
                transaction
                    .commit()
                    .map_err(|error| format!("AI completion commit error: {error}"))?;
                Ok(message)
            })?;
            AiStreamEvent::Complete {
                message: Box::new(message),
            }
        }
        Err(error) if error.code == "cancelled" => {
            let metadata =
                serde_json::json!({ "mode": mode, "contextCount": context_count }).to_string();
            let message = with_conn(&db.conn, |conn| {
                repositories::conversations::finish_message(
                    conn,
                    &assistant_message.id,
                    &final_content,
                    "cancelled",
                    Some(error.code),
                    Some(&metadata),
                    Some(&provenance),
                )?
                .ok_or_else(|| "Assistant message no longer exists.".to_string())
            })?;
            AiStreamEvent::Cancelled {
                message: Box::new(message),
            }
        }
        Err(error) => {
            let metadata =
                serde_json::json!({ "mode": mode, "contextCount": context_count }).to_string();
            let assistant_message = with_conn(&db.conn, |conn| {
                repositories::conversations::finish_message(
                    conn,
                    &assistant_message.id,
                    &final_content,
                    "error",
                    Some(error.code),
                    Some(&metadata),
                    Some(&provenance),
                )?
                .ok_or_else(|| "Assistant message no longer exists.".to_string())
            })?;
            AiStreamEvent::Failed {
                code: error.code.to_string(),
                message: error.message,
                assistant_message: Box::new(assistant_message),
            }
        }
    };
    let _ = on_event.send(terminal);
    Ok(())
}

#[tauri::command]
pub fn ai_cancel_request(runtime: State<AiRuntime>, request_id: String) -> Result<bool, String> {
    runtime.cancel(&request_id)
}

// ─── AI Context Items ────────────────────────────────────

#[tauri::command]
pub fn ai_add_context(
    db: State<Database>,
    conversation_id: String,
    entity_type: String,
    entity_id: String,
) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        let conversation = repositories::conversations::get_conversation(conn, &conversation_id)?
            .ok_or_else(|| "Conversation not found.".to_string())?;
        let candidate = repositories::conversations::AiContextItem {
            id: String::new(),
            conversation_id: conversation_id.clone(),
            entity_type: entity_type.clone(),
            entity_id: entity_id.clone(),
            context_mode: "attached".to_string(),
            added_at: String::new(),
        };
        context::resolve_one(conn, conversation.space_id.as_deref(), &candidate)?;
        let item = repositories::conversations::add_context_item(
            conn,
            &conversation_id,
            &entity_type,
            &entity_id,
            "attached",
        )?;
        serde_json::to_value(item).map_err(|e| format!("Serialize error: {}", e))
    })
}

#[tauri::command]
pub fn ai_resolve_context(
    db: State<Database>,
    conversation_id: String,
) -> Result<Vec<context::ResolvedContextItem>, String> {
    with_conn(&db.conn, |conn| {
        let conversation = repositories::conversations::get_conversation(conn, &conversation_id)?
            .ok_or_else(|| "Conversation not found.".to_string())?;
        let items = repositories::conversations::list_context_items(conn, &conversation_id)?;
        context::resolve_all(conn, conversation.space_id.as_deref(), &items)
    })
}

#[tauri::command]
pub fn ai_list_context(
    db: State<Database>,
    conversation_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let items = repositories::conversations::list_context_items(conn, &conversation_id)?;
        Ok(items
            .iter()
            .map(|i| serde_json::to_value(i).unwrap())
            .collect())
    })
}

#[tauri::command]
pub fn ai_remove_context(db: State<Database>, id: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| {
        repositories::conversations::remove_context_item(conn, &id)
    })
}

#[tauri::command]
pub fn ai_clear_context(db: State<Database>, conversation_id: String) -> Result<usize, String> {
    with_conn(&db.conn, |conn| {
        repositories::conversations::clear_context(conn, &conversation_id)
    })
}

#[cfg(test)]
mod vault_command_tests {
    use super::*;

    #[test]
    fn vault_json_keeps_storage_paths_inside_rust() {
        let item = repositories::vault::VaultItem {
            id: "vault-1".to_string(),
            space_id: None,
            storage_mode: "linked".to_string(),
            display_title: "Private document".to_string(),
            original_name: "document.md".to_string(),
            stored_path: "C:\\Users\\Private\\document.md".to_string(),
            media_type: "text/markdown".to_string(),
            size_bytes: 10,
            tags: vec![],
            created_at: String::new(),
            updated_at: String::new(),
        };
        let value = json_vault_item(&item);
        assert_eq!(value["id"], "vault-1");
        assert!(value.get("stored_path").is_none());
    }
}
