#![allow(clippy::too_many_arguments)]

use crate::ai::credentials;
use crate::ai::provider::{
    self, ChatCompletionRequest, ChatMessage, ProviderConfig, ThinkingConfig,
};
use crate::db::repositories::{self, with_conn};
use crate::db::Database;
use crate::vault as vault_storage;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

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
            repositories::spaces::touch_last_opened(conn, &id)?;
            let modules = repositories::spaces::list_modules(conn, &id)?;
            let children = repositories::spaces::list_by_parent(conn, &id)?;
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
pub fn record_activity(
    db: State<Database>,
    event_type: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    space_id: Option<String>,
    metadata_json: Option<String>,
) -> Result<serde_json::Value, String> {
    let event = repositories::activity::ActivityEvent {
        id: Uuid::now_v7().to_string(),
        event_type,
        entity_type,
        entity_id,
        space_id,
        metadata_json,
        created_at: String::new(),
    };
    with_conn(&db.conn, |conn| {
        Ok(json_activity(&repositories::activity::record(
            conn, &event,
        )?))
    })
}

#[tauri::command]
pub fn list_activity(
    db: State<Database>,
    space_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let events = if let Some(sid) = space_id {
            repositories::activity::list_by_space(conn, &sid, limit)?
        } else {
            repositories::activity::list_recent(conn, limit)?
        };
        Ok(events.iter().map(json_activity).collect())
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

// ─── Notes ───────────────────────────────────────────────

#[tauri::command]
pub fn create_note(db: State<Database>, space_id: String) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        Ok(json_note(&repositories::notes::create(conn, &space_id)?))
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
        Ok(repositories::notes::update(
            conn,
            &id,
            title.as_deref(),
            content.as_deref(),
            excerpt.as_deref(),
            expected_revision,
        )?
        .map(|n| json_note(&n)))
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

fn json_activity(a: &repositories::activity::ActivityEvent) -> serde_json::Value {
    serde_json::json!({"id": a.id, "event_type": a.event_type, "entity_type": a.entity_type, "entity_id": a.entity_id, "space_id": a.space_id, "metadata_json": a.metadata_json, "created_at": a.created_at})
}

// ─── AI Credentials ─────────────────────────────────────

#[derive(serde::Serialize)]
pub struct KeyStatus {
    pub configured: bool,
    pub status: String, // "configured", "missing", "unavailable"
}

#[tauri::command]
pub fn ai_get_key_status(db: State<Database>) -> Result<KeyStatus, String> {
    match credentials::get(&db, credentials::AI_API_KEY) {
        Ok(Some(_)) => Ok(KeyStatus {
            configured: true,
            status: "configured".to_string(),
        }),
        Ok(None) => Ok(KeyStatus {
            configured: false,
            status: "missing".to_string(),
        }),
        Err(_) => Ok(KeyStatus {
            configured: false,
            status: "unavailable".to_string(),
        }),
    }
}

#[tauri::command]
pub fn ai_set_api_key(db: State<Database>, api_key: String) -> Result<(), String> {
    credentials::store(&db, credentials::AI_API_KEY, &api_key)
}

#[tauri::command]
pub fn ai_remove_api_key(db: State<Database>) -> Result<bool, String> {
    credentials::remove(&db, credentials::AI_API_KEY)
}

#[tauri::command]
pub fn ai_test_connection(db: State<Database>) -> Result<String, String> {
    let key = credentials::get(&db, credentials::AI_API_KEY)?
        .ok_or_else(|| "No API key configured".to_string())?;

    let config = ProviderConfig::default_deepseek(key);
    let provider = provider::create_provider(config)?;

    provider.test_connection()?;
    Ok("Connection successful".to_string())
}

// ─── AI Conversations ────────────────────────────────────

#[tauri::command]
pub fn ai_create_conversation(
    db: State<Database>,
    space_id: Option<String>,
    title: Option<String>,
) -> Result<serde_json::Value, String> {
    with_conn(&db.conn, |conn| {
        let conv = repositories::conversations::create_conversation(
            conn,
            space_id.as_deref(),
            title.as_deref().unwrap_or("New conversation"),
            "deepseek",
            "deepseek-chat",
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

// ─── AI Chat (non-streaming) ─────────────────────────────

#[derive(serde::Serialize)]
pub struct AiChatResponse {
    pub content: String,
    pub usage: Option<serde_json::Value>,
}

#[tauri::command]
pub fn ai_send_message(
    db: State<Database>,
    conversation_id: String,
    content: String,
) -> Result<AiChatResponse, String> {
    // Get API key
    let key = credentials::get(&db, credentials::AI_API_KEY)?.ok_or_else(|| {
        "No API key configured. Please configure an AI provider in Settings.".to_string()
    })?;

    // Save user message
    with_conn(&db.conn, |conn| {
        repositories::conversations::add_message(
            conn,
            &conversation_id,
            "user",
            &content,
            "complete",
        )?;
        Ok::<_, String>(())
    })?;

    // Get conversation history
    let messages = with_conn(&db.conn, |conn| {
        repositories::conversations::list_messages(conn, &conversation_id, Some(50))
    })?;

    // Build provider request
    let chat_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let request = ChatCompletionRequest {
        model: "deepseek-chat".to_string(),
        messages: chat_messages,
        temperature: Some(0.1),
        max_tokens: Some(4096),
        top_p: Some(1.0),
        stream: None,
        thinking: Some(ThinkingConfig {
            thinking_type: "enabled".to_string(),
        }),
    };

    // Create provider and send
    let config = ProviderConfig::default_deepseek(key);
    let provider = provider::create_provider(config)?;

    let response = provider
        .chat_completion(&request)
        .map_err(|e| format!("AI request failed: {}", e))?;

    let assistant_content = response
        .choices
        .first()
        .and_then(|c| c.message.as_ref())
        .map(|m| m.content.clone())
        .unwrap_or_default();

    // Save assistant message
    with_conn(&db.conn, |conn| {
        repositories::conversations::add_message(
            conn,
            &conversation_id,
            "assistant",
            &assistant_content,
            "complete",
        )?;
        Ok::<_, String>(())
    })?;

    Ok(AiChatResponse {
        content: assistant_content,
        usage: response.usage.map(|u| serde_json::to_value(u).unwrap()),
    })
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
