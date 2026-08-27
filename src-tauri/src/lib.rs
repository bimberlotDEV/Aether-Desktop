mod ai;
mod backup;
mod commands;
mod context;
mod db;
mod native;
mod vault;

use db::Database;
use std::path::PathBuf;
use tauri::Manager;

use crate::ai::credentials::DpapiCrypto;
use crate::ai::runtime::AiRuntime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            native::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        native::show_main_window(app);
                    }
                })
                .build(),
        )
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
            let database = Database::open(db_path, Box::new(DpapiCrypto)).map_err(|e| {
                log::error!("Database initialization failed: {}", e);
                e
            })?;

            log::info!("Database initialized at: {}", app_data_dir.display());

            // Ensure secrets table exists for credential storage
            {
                let conn = database
                    .conn
                    .lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                crate::ai::credentials::ensure_table(&conn)?;
            }

            // Store database in Tauri state
            app.manage(database);
            app.manage(AiRuntime::default());
            app.manage(context::ContextRuntime::default());
            let native_status = native::setup(app)?;
            app.manage(native_status);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::native_get_status,
            commands::native_test_notification,
            commands::export_workspace_backup,
            commands::get_pulse,
            commands::create_source,
            commands::universal_search,
            commands::list_sources,
            commands::update_source_space,
            commands::revoke_source,
            commands::list_indexed_files,
            commands::scan_source,
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
            commands::create_task,
            commands::create_tasks_batch,
            commands::get_task,
            commands::list_tasks,
            commands::update_task,
            commands::list_task_attention,
            commands::archive_task,
            commands::restore_task,
            commands::delete_task,
            commands::create_memory,
            commands::get_memory,
            commands::list_memory,
            commands::update_memory,
            commands::delete_memory,
            commands::import_vault_item,
            commands::get_vault_item,
            commands::list_vault_items,
            commands::update_vault_item,
            commands::remove_vault_item,
            commands::open_vault_item,
            commands::reveal_vault_item,
            commands::list_activity,
            commands::get_space_continuity,
            // AI
            commands::ai_get_key_status,
            commands::ai_set_api_key,
            commands::ai_remove_api_key,
            commands::ai_test_connection,
            commands::ai_list_models,
            commands::ai_create_conversation,
            commands::ai_get_conversation,
            commands::ai_list_conversations,
            commands::ai_update_conversation,
            commands::ai_delete_conversation,
            commands::ai_list_messages,
            commands::ai_stream_message,
            commands::ai_cancel_request,
            commands::ai_add_context,
            commands::ai_list_context,
            commands::ai_remove_context,
            commands::ai_clear_context,
            commands::ai_resolve_context,
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
