#![allow(clippy::too_many_arguments)]

use crate::db::repositories::{self, with_conn};
use crate::db::Database;
use tauri::State;
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

// ─── JSON helpers ───────────────────────────────────────

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
