use crate::native::NativeStatus;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaDiagnosticReport {
    pub app_version: String,
    pub database_schema: String,
    pub database_integrity: String,
    pub platform: String,
    pub updater_configured: bool,
    pub tray_available: bool,
    pub shortcut_registered: bool,
    pub notifications_available: bool,
}

pub fn generate(conn: &Connection, native: &NativeStatus) -> Result<BetaDiagnosticReport, String> {
    let database_schema = conn
        .query_row("SELECT MAX(name) FROM _migrations", [], |row| {
            row.get::<_, Option<String>>(0)
        })
        .optional()
        .map_err(|error| format!("Diagnostic schema check failed: {error}"))?
        .flatten()
        .unwrap_or_else(|| "uninitialized".to_string());

    let integrity: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|error| format!("Diagnostic integrity check failed: {error}"))?;

    Ok(BetaDiagnosticReport {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        database_schema,
        database_integrity: if integrity == "ok" {
            "ok".to_string()
        } else {
            "failed".to_string()
        },
        platform: format!("Windows {}", std::env::consts::ARCH),
        updater_configured: native.updater_configured,
        tray_available: native.tray_available,
        shortcut_registered: native.shortcut_registered,
        notifications_available: native.notifications_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use serde_json::Value;

    fn native_status() -> NativeStatus {
        NativeStatus {
            tray_available: true,
            shortcut: "must-not-leak".to_string(),
            shortcut_registered: false,
            notifications_available: true,
            updater_configured: false,
        }
    }

    #[test]
    fn report_has_a_closed_content_free_schema() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();

        let value = serde_json::to_value(generate(&conn, &native_status()).unwrap()).unwrap();
        let object = value.as_object().unwrap();
        let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "appVersion",
                "databaseIntegrity",
                "databaseSchema",
                "notificationsAvailable",
                "platform",
                "shortcutRegistered",
                "trayAvailable",
                "updaterConfigured",
            ]
        );
        assert_eq!(value["databaseSchema"], "010_ai_route_provenance");
        assert_eq!(value["databaseIntegrity"], "ok");
        assert_eq!(
            value["platform"],
            format!("Windows {}", std::env::consts::ARCH)
        );
        assert!(!value.to_string().contains("must-not-leak"));
    }

    #[test]
    fn report_uses_only_bounded_schema_and_integrity_labels() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE _migrations (name TEXT PRIMARY KEY NOT NULL);
             INSERT INTO _migrations (name) VALUES ('test_schema');",
        )
        .unwrap();

        let report = generate(&conn, &native_status()).unwrap();
        assert_eq!(report.database_schema, "test_schema");
        assert_eq!(report.database_integrity, "ok");
    }

    #[test]
    fn report_serialization_contains_no_null_or_nested_values() {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        let value = serde_json::to_value(generate(&conn, &native_status()).unwrap()).unwrap();
        assert!(value
            .as_object()
            .unwrap()
            .values()
            .all(|field| matches!(field, Value::String(_) | Value::Bool(_))));
    }
}
