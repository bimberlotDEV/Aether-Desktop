pub mod settings;
pub mod profile;
pub mod spaces;
pub mod activity;

use rusqlite::Connection;
use std::sync::Mutex;

/// Helper to get a connection from the Mutex
pub(crate) fn with_conn<T>(
    db: &Mutex<Connection>,
    f: impl FnOnce(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    let conn = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    f(&conn)
}
