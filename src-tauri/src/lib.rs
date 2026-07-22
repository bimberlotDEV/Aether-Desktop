mod commands;
mod db;

use db::Database;
use tauri::Manager;
use std::path::PathBuf;

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
            let database = Database::open(db_path)
                .map_err(|e| {
                    log::error!("Database initialization failed: {}", e);
                    e
                })?;

            log::info!(
                "Database initialized at: {}",
                app_data_dir.display()
            );

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
            commands::get_space,
            commands::list_spaces,
            commands::update_space,
            commands::archive_space,
            commands::restore_space,
            commands::delete_space,
            commands::favourite_space,
            commands::reorder_spaces,
            commands::record_activity,
            commands::list_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
