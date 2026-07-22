use crate::db::repositories::{self, with_conn};
use crate::db::Database;
use tauri::State;
use uuid::Uuid;

// ─── Settings ───────────────────────────────────────────

#[tauri::command]
pub fn get_setting(db: State<Database>, key: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let setting = repositories::settings::get(conn, &key)?;
        Ok(setting.map(|s| {
            serde_json::json!({
                "key": s.key,
                "value": s.value,
                "value_type": s.value_type,
                "created_at": s.created_at,
                "updated_at": s.updated_at,
            })
        }))
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
        repositories::settings::set(conn, &key, &value, &value_type.unwrap_or_else(|| "string".into()))
    })
}

#[tauri::command]
pub fn delete_setting(db: State<Database>, key: String) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::settings::delete(conn, &key))
}

#[tauri::command]
pub fn list_settings(db: State<Database>) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let settings = repositories::settings::list(conn)?;
        Ok(settings
            .into_iter()
            .map(|s| {
                serde_json::json!({
                    "key": s.key,
                    "value": s.value,
                    "value_type": s.value_type,
                    "created_at": s.created_at,
                    "updated_at": s.updated_at,
                })
            })
            .collect())
    })
}

// ─── User Profile ───────────────────────────────────────

#[tauri::command]
pub fn get_profile(db: State<Database>) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let profile = repositories::profile::get(conn)?;
        Ok(profile.map(|p| {
            serde_json::json!({
                "id": p.id,
                "display_name": p.display_name,
                "onboarding_completed": p.onboarding_completed,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
            })
        }))
    })
}

#[tauri::command]
pub fn create_profile(db: State<Database>) -> Result<serde_json::Value, String> {
    let id = Uuid::now_v7().to_string();
    with_conn(&db.conn, |conn| {
        let profile = repositories::profile::create(conn, &id)?;
        Ok(serde_json::json!({
            "id": profile.id,
            "display_name": profile.display_name,
            "onboarding_completed": profile.onboarding_completed,
            "created_at": profile.created_at,
            "updated_at": profile.updated_at,
        }))
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
        let profile = repositories::profile::update(
            conn,
            &id,
            display_name.as_deref(),
            onboarding_completed,
        )?;
        Ok(profile.map(|p| {
            serde_json::json!({
                "id": p.id,
                "display_name": p.display_name,
                "onboarding_completed": p.onboarding_completed,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
            })
        }))
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
        created_at: String::new(),
        updated_at: String::new(),
    };
    with_conn(&db.conn, |conn| {
        let saved = repositories::spaces::create(conn, &space)?;
        Ok(serialize_space(&saved))
    })
}

#[tauri::command]
pub fn get_space(db: State<Database>, id: String) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let space = repositories::spaces::get_by_id(conn, &id)?;
        Ok(space.map(|s| serialize_space(&s)))
    })
}

#[tauri::command]
pub fn list_spaces(
    db: State<Database>,
    include_archived: Option<bool>,
) -> Result<Vec<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let spaces = repositories::spaces::list(conn, include_archived.unwrap_or(false))?;
        Ok(spaces.iter().map(|s| serialize_space(s)).collect())
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
) -> Result<Option<serde_json::Value>, String> {
    with_conn(&db.conn, |conn| {
        let space = repositories::spaces::update(
            conn, &id,
            name.as_deref(),
            description.as_deref(),
            icon.as_deref(),
            accent.as_deref(),
            settings_json.as_deref(),
        )?;
        Ok(space.map(|s| serialize_space(&s)))
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
    with_conn(&db.conn, |conn| repositories::spaces::delete(conn, &id))
}

#[tauri::command]
pub fn favourite_space(db: State<Database>, id: String, fav: bool) -> Result<bool, String> {
    with_conn(&db.conn, |conn| repositories::spaces::set_favourite(conn, &id, fav))
}

#[tauri::command]
pub fn reorder_spaces(db: State<Database>, ids: Vec<String>) -> Result<(), String> {
    with_conn(&db.conn, |conn| repositories::spaces::reorder(conn, &ids))
}

// ─── Activity ──────────────────────────────────────────

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
        let saved = repositories::activity::record(conn, &event)?;
        Ok(serde_json::json!({
            "id": saved.id,
            "event_type": saved.event_type,
            "entity_type": saved.entity_type,
            "entity_id": saved.entity_id,
            "space_id": saved.space_id,
            "metadata_json": saved.metadata_json,
            "created_at": saved.created_at,
        }))
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
        Ok(events
            .into_iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "event_type": e.event_type,
                    "entity_type": e.entity_type,
                    "entity_id": e.entity_id,
                    "space_id": e.space_id,
                    "metadata_json": e.metadata_json,
                    "created_at": e.created_at,
                })
            })
            .collect())
    })
}

// ─── Helpers ───────────────────────────────────────────

fn serialize_space(s: &repositories::spaces::Space) -> serde_json::Value {
    serde_json::json!({
        "id": s.id,
        "name": s.name,
        "description": s.description,
        "icon": s.icon,
        "accent": s.accent,
        "template_type": s.template_type,
        "favourite": s.favourite,
        "archived_at": s.archived_at,
        "sort_order": s.sort_order,
        "settings_json": s.settings_json,
        "created_at": s.created_at,
        "updated_at": s.updated_at,
    })
}
