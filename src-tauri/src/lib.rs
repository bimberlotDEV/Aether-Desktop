mod ai;
mod commands;
mod db;

use db::Database;
use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Resolve app data directory
            let app_data_dir: PathBuf = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let db_path = app_data_dir.join("aether.db");

            // Open database (creates and migrates if needed)
            let database = Database::open(db_path).map_err(|e| {
                log::error!("Database initialization failed: {}", e);
                e
            })?;

            log::info!("Database initialized at: {}", app_data_dir.display());

            // Ensure secrets table exists for credential storage
            {
                let conn = database.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
                crate::ai::credentials::ensure_table(&conn)?;
            }

            // Store database in Tauri state
            app.manage(database);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_setting,
            commands::set_setting,
            commands::delete_setting,
            commands::list_settings,
            commands::get_profile,
            commands::create_profile,
            commands::update_profile,
            commands::create_space,
            commands::create_space_with_modules,
            commands::create_school_space,
            commands::get_space,
            commands::list_spaces,
            commands::list_top_level_spaces,
            commands::list_child_spaces,
            commands::update_space,
            commands::set_space_modules,
            commands::get_space_modules,
            commands::archive_space,
            commands::restore_space,
            commands::delete_space,
            commands::favourite_space,
            commands::reorder_spaces,
            commands::duplicate_space,
            commands::create_note,
            commands::get_note,
            commands::update_note,
            commands::list_notes_by_space,
            commands::list_recent_notes,
            commands::list_pinned_notes,
            commands::list_archived_notes,
            commands::search_notes,
            commands::pin_note,
            commands::archive_note,
            commands::restore_note,
            commands::delete_note,
            commands::move_note,
            commands::duplicate_note,
            commands::record_activity,
            commands::list_activity,
            // AI
            commands::ai_get_key_status,
            commands::ai_set_api_key,
            commands::ai_remove_api_key,
            commands::ai_test_connection,
            commands::ai_create_conversation,
            commands::ai_get_conversation,
            commands::ai_list_conversations,
            commands::ai_update_conversation,
            commands::ai_delete_conversation,
            commands::ai_list_messages,
            commands::ai_send_message,
            commands::ai_add_context,
            commands::ai_list_context,
            commands::ai_remove_context,
            commands::ai_clear_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
