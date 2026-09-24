use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const AUTH_TYPES: &[&str] = &[
    "none",
    "api_key",
    "api_token",
    "oauth",
    "ics_feed",
    "feed_url",
    "oauth_authorization_code",
];
const SYNC_MODES: &[&str] = &["manual", "periodic", "app_start", "app_resume", "webhook"];
const INTEGRATION_COLS: &str = "id, provider_id, enabled, advertised_capabilities_json, effective_capabilities_json, auth_type, sync_modes_json, sync_config_json, connection_status, sync_status, disconnect_reason, last_attempted_at, last_successful_sync_at, next_allowed_sync_at, last_sync_error_code, last_sync_error_message, last_sync_etag, last_sync_last_modified, sync_cursor, rate_limit_remaining, retry_after_at, credential_expires_at, credential_rotated_at, sync_execution_scope, created_at, updated_at";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Integration {
    pub id: String,
    pub provider_id: String,
    pub enabled: bool,
    pub advertised_capabilities: Vec<String>,
    pub effective_capabilities: Vec<String>,
    pub auth_type: String,
    pub sync_modes: Vec<String>,
    pub sync_config: Value,
    pub connection_status: String,
    pub sync_status: String,
    pub disconnect_reason: Option<String>,
    pub last_attempted_at: Option<String>,
    pub last_successful_sync_at: Option<String>,
    pub next_allowed_sync_at: Option<String>,
    pub last_sync_error_code: Option<String>,
    pub last_sync_error_message: Option<String>,
    pub last_sync_etag: Option<String>,
    pub last_sync_last_modified: Option<String>,
    pub sync_cursor: Option<String>,
    pub rate_limit_remaining: Option<u32>,
    pub retry_after_at: Option<String>,
    pub credential_expires_at: Option<String>,
    pub credential_rotated_at: Option<String>,
    pub sync_execution_scope: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationCreateInput {
    pub provider_id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub advertised_capabilities: Vec<String>,
    pub auth_type: String,
    #[serde(default)]
    pub sync_modes: Vec<String>,
    #[serde(default = "default_sync_config")]
    pub sync_config: Value,
}

fn default_enabled() -> bool {
    true
}
fn default_sync_config() -> Value {
    Value::Object(Default::default())
}

fn parse_strings(raw: String, field: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(&raw).map_err(|_| format!("Integration {field} is invalid"))
}

fn row_to_integration(row: &rusqlite::Row) -> rusqlite::Result<Integration> {
    let advertised_capabilities_json: String = row.get(3)?;
    let effective_capabilities_json: String = row.get(4)?;
    let sync_modes_json: String = row.get(6)?;
    let sync_config_json: String = row.get(7)?;
    let _credential_key: Option<String> = row.get(26)?;
    Ok(Integration {
        id: row.get(0)?,
        provider_id: row.get(1)?,
        enabled: row.get(2)?,
        advertised_capabilities: parse_strings(
            advertised_capabilities_json,
            "advertised capabilities",
        )
        .map_err(|_| rusqlite::Error::InvalidQuery)?,
        effective_capabilities: parse_strings(
            effective_capabilities_json,
            "effective capabilities",
        )
        .map_err(|_| rusqlite::Error::InvalidQuery)?,
        auth_type: row.get(5)?,
        sync_modes: parse_strings(sync_modes_json, "sync modes")
            .map_err(|_| rusqlite::Error::InvalidQuery)?,
        sync_config: serde_json::from_str(&sync_config_json)
            .map_err(|_| rusqlite::Error::InvalidQuery)?,
        connection_status: row.get(8)?,
        sync_status: row.get(9)?,
        disconnect_reason: row.get(10)?,
        last_attempted_at: row.get(11)?,
        last_successful_sync_at: row.get(12)?,
        next_allowed_sync_at: row.get(13)?,
        last_sync_error_code: row.get(14)?,
        last_sync_error_message: row.get(15)?,
        last_sync_etag: row.get(16)?,
        last_sync_last_modified: row.get(17)?,
        sync_cursor: row.get(18)?,
        rate_limit_remaining: row.get(19)?,
        retry_after_at: row.get(20)?,
        credential_expires_at: row.get(21)?,
        credential_rotated_at: row.get(22)?,
        sync_execution_scope: row.get(23)?,
        created_at: row.get(24)?,
        updated_at: row.get(25)?,
    })
}

fn validate_tokens(
    tokens: &[String],
    allowed: Option<&[&str]>,
    field: &str,
) -> Result<Vec<String>, String> {
    if tokens.len() > 32
        || tokens
            .iter()
            .any(|token| token.trim().is_empty() || token.chars().count() > 100)
    {
        return Err(format!("Integration {field} must contain at most 32 non-empty values of 100 characters or fewer"));
    }
    if let Some(allowed) = allowed {
        if tokens
            .iter()
            .any(|token| !allowed.contains(&token.as_str()))
        {
            return Err(format!("Invalid Integration {field}"));
        }
    }
    let mut normalized = tokens
        .iter()
        .map(|token| token.trim().to_string())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn validate_config(config: &Value) -> Result<String, String> {
    if !config.is_object() {
        return Err("Integration sync configuration must be an object".to_string());
    }
    fn contains_secret_key(value: &Value) -> bool {
        match value {
            Value::Object(object) => object.iter().any(|(key, value)| {
                let normalized = key.to_ascii_lowercase().replace(['_', '-'], "");
                matches!(
                    normalized.as_str(),
                    "feedurl"
                        | "token"
                        | "accesstoken"
                        | "refreshtoken"
                        | "clientsecret"
                        | "apikey"
                        | "password"
                        | "credential"
                ) || contains_secret_key(value)
            }),
            Value::Array(values) => values.iter().any(contains_secret_key),
            _ => false,
        }
    }
    if contains_secret_key(config) {
        return Err(
            "Integration configuration must not contain credentials or feed URLs".to_string(),
        );
    }
    let encoded = serde_json::to_string(config)
        .map_err(|error| format!("Integration sync configuration error: {error}"))?;
    if encoded.len() > 10_000 {
        return Err("Integration sync configuration must not exceed 10000 bytes".to_string());
    }
    Ok(encoded)
}

fn secret_key(id: &str, auth_type: &str) -> Option<String> {
    (auth_type != "none").then(|| format!("integration:{id}:credential"))
}

pub fn create(conn: &Connection, input: &IntegrationCreateInput) -> Result<Integration, String> {
    let provider_id = input.provider_id.trim();
    if provider_id.is_empty() || provider_id.chars().count() > 100 {
        return Err("Integration provider ID must contain 1 to 100 characters".to_string());
    }
    if !AUTH_TYPES.contains(&input.auth_type.as_str()) {
        return Err("Invalid Integration authentication type".to_string());
    }
    let advertised_capabilities = validate_tokens(
        &input.advertised_capabilities,
        None,
        "advertised capabilities",
    )?;
    let sync_modes = validate_tokens(&input.sync_modes, Some(SYNC_MODES), "sync modes")?;
    let sync_config = validate_config(&input.sync_config)?;
    let id = Uuid::now_v7().to_string();
    let credential_key = secret_key(&id, &input.auth_type);
    conn.execute(
        "INSERT INTO integrations (id, provider_id, enabled, advertised_capabilities_json, effective_capabilities_json, auth_type, credential_key, sync_modes_json, sync_config_json) VALUES (?1, ?2, ?3, ?4, '[]', ?5, ?6, ?7, ?8)",
        params![id, provider_id, input.enabled, serde_json::to_string(&advertised_capabilities).unwrap(), input.auth_type, credential_key, serde_json::to_string(&sync_modes).unwrap(), sync_config],
    ).map_err(|error| format!("Integration create error: {error}"))?;
    get_by_id(conn, &id)?.ok_or_else(|| "Integration not found after create".to_string())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Integration>, String> {
    let sql = format!("SELECT {INTEGRATION_COLS}, credential_key FROM integrations WHERE id = ?1");
    match conn.query_row(&sql, params![id], row_to_integration) {
        Ok(integration) => Ok(Some(integration)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(format!("Integration get error: {error}")),
    }
}

pub fn list(conn: &Connection) -> Result<Vec<Integration>, String> {
    let sql = format!("SELECT {INTEGRATION_COLS}, credential_key FROM integrations ORDER BY provider_id, created_at");
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| format!("Integration list error: {error}"))?;
    let rows = statement
        .query_map([], row_to_integration)
        .map_err(|error| format!("Integration list error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Integration row error: {error}"))
}

#[derive(Debug, Clone)]
pub struct SyncRuntimeRecord {
    pub integration: Integration,
    pub credential_key: Option<String>,
    pub validator_origin: Option<String>,
    pub consecutive_failures: u32,
}

pub fn sync_runtime_record(
    conn: &Connection,
    id: &str,
) -> Result<Option<SyncRuntimeRecord>, String> {
    let sql = format!("SELECT {INTEGRATION_COLS}, credential_key, last_sync_validator_origin, consecutive_sync_failures FROM integrations WHERE id=?1");
    conn.query_row(&sql, [id], |row| {
        Ok(SyncRuntimeRecord {
            integration: row_to_integration(row)?,
            credential_key: row.get(26)?,
            validator_origin: row.get(27)?,
            consecutive_failures: row.get::<_, i64>(28)?.max(0) as u32,
        })
    })
    .optional()
    .map_err(|error| format!("Integration runtime lookup error: {error}"))
}

pub fn list_sync_candidates(conn: &Connection) -> Result<Vec<Integration>, String> {
    list(conn)
}

/// Marks work left pending by a terminated desktop process as safely retryable.
pub fn recover_interrupted_runs(conn: &Connection, next_allowed: &str) -> Result<usize, String> {
    conn.execute(
        "UPDATE integrations SET sync_status='failed', connection_status=CASE WHEN connection_status='syncing' THEN 'degraded' ELSE connection_status END, last_sync_error_code='interrupted', last_sync_error_message='Sync was interrupted when Aether closed', last_sync_finished_at=datetime('now'), next_allowed_sync_at=?1, consecutive_sync_failures=consecutive_sync_failures+1, updated_at=datetime('now') WHERE sync_status IN ('pending','syncing')",
        params![next_allowed],
    ).map_err(|error| format!("Integration interrupted recovery error: {error}"))
}

pub fn runtime_mark_running(
    conn: &Connection,
    id: &str,
    trigger: &str,
    now: &str,
) -> Result<bool, String> {
    let changed = conn.execute(
        "UPDATE integrations SET sync_status='syncing', connection_status='syncing', last_attempted_at=?1, last_sync_trigger=?2, updated_at=datetime('now') WHERE id=?3 AND enabled=1 AND connection_status NOT IN ('disconnected','unsupported','reauthentication_required','permission_denied','institution_configuration_required')",
        params![now, trigger, id],
    ).map_err(|error| format!("Integration runtime start error: {error}"))?;
    Ok(changed == 1)
}

pub struct SyncValidators<'a> {
    pub origin: &'a str,
    pub etag: Option<&'a str>,
    pub last_modified: Option<&'a str>,
}

pub fn runtime_finish_success(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
    now: &str,
    validators: Option<SyncValidators<'_>>,
    next_allowed: &str,
) -> Result<(), String> {
    let replace_validators = validators.is_some();
    let origin = validators.as_ref().and_then(|validators| {
        (validators.etag.is_some() || validators.last_modified.is_some())
            .then_some(validators.origin)
    });
    let etag = validators.as_ref().and_then(|validators| validators.etag);
    let last_modified = validators
        .as_ref()
        .and_then(|validators| validators.last_modified);
    let changed = tx.execute(
        "UPDATE integrations SET sync_status='succeeded', connection_status='connected', last_successful_sync_at=?1, last_sync_finished_at=?1, next_allowed_sync_at=?2, last_sync_error_code=NULL, last_sync_error_message=NULL, last_sync_etag=CASE WHEN ?3 THEN ?4 ELSE last_sync_etag END, last_sync_last_modified=CASE WHEN ?3 THEN ?5 ELSE last_sync_last_modified END, last_sync_validator_origin=CASE WHEN ?3 THEN ?6 ELSE last_sync_validator_origin END, rate_limit_remaining=NULL, retry_after_at=NULL, consecutive_sync_failures=0, updated_at=datetime('now') WHERE id=?7 AND enabled=1",
        params![now, next_allowed, replace_validators, etag, last_modified, origin, id],
    ).map_err(|error| format!("Integration runtime success error: {error}"))?;
    if changed != 1 {
        return Err("Integration was disabled before sync completion".to_string());
    }
    Ok(())
}

pub fn runtime_finish_failure(
    conn: &Connection,
    id: &str,
    code: &str,
    message: &str,
    next_allowed: &str,
    connection_status: &str,
    retry_after: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE integrations SET sync_status='failed', connection_status=?1, last_sync_finished_at=?2, next_allowed_sync_at=?3, retry_after_at=?4, last_sync_error_code=?5, last_sync_error_message=?6, consecutive_sync_failures=consecutive_sync_failures+1, updated_at=datetime('now') WHERE id=?7",
        params![connection_status, Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true), next_allowed, retry_after, code, message, id],
    ).map_err(|error| format!("Integration runtime failure error: {error}"))?;
    Ok(())
}

pub fn set_enabled(
    conn: &Connection,
    id: &str,
    enabled: bool,
) -> Result<Option<Integration>, String> {
    let changed = conn.execute(
        "UPDATE integrations SET enabled=?1, sync_status=CASE WHEN ?1=0 THEN 'idle' ELSE sync_status END, updated_at=datetime('now') WHERE id=?2",
        params![enabled, id],
    ).map_err(|error| format!("Integration enabled update error: {error}"))?;
    if changed == 0 {
        return Ok(None);
    }
    get_by_id(conn, id)
}

/// Records that a connection has completed its native configuration flow.
/// Configuration is distinct from a successful synchronization, so this does
/// not change sync status or successful-sync metadata.
pub fn mark_configured(conn: &Connection, id: &str) -> Result<Option<Integration>, String> {
    let changed = conn.execute(
        "UPDATE integrations SET connection_status='connected', effective_capabilities_json=advertised_capabilities_json, disconnect_reason=NULL, updated_at=datetime('now') WHERE id=?1",
        [id],
    )
    .map_err(|error| format!("Integration configuration update error: {error}"))?;
    if changed == 0 {
        return Ok(None);
    }
    get_by_id(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        conn
    }
    fn input() -> IntegrationCreateInput {
        IntegrationCreateInput {
            provider_id: "calendar".into(),
            enabled: true,
            advertised_capabilities: vec!["events_read".into()],
            auth_type: "oauth_authorization_code".into(),
            sync_modes: vec!["manual".into(), "periodic".into()],
            sync_config: serde_json::json!({"institutionBaseUrl": "https://school.example"}),
        }
    }

    #[test]
    fn persists_generic_metadata_without_exposing_credential_key() {
        let conn = setup();
        let created = create(&conn, &input()).unwrap();
        assert_eq!(created.provider_id, "calendar");
        let key: String = conn
            .query_row(
                "SELECT credential_key FROM integrations WHERE id=?1",
                [&created.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(key, format!("integration:{}:credential", created.id));
        assert!(!serde_json::to_string(&created)
            .unwrap()
            .contains("credential_key"));
    }

    #[test]
    fn only_exposes_narrow_enabled_mutation_and_rejects_secret_config() {
        let conn = setup();
        let created = create(&conn, &input()).unwrap();
        let updated = set_enabled(&conn, &created.id, false).unwrap().unwrap();
        assert!(!updated.enabled);
        assert_eq!(updated.sync_status, "idle");
        let mut invalid = input();
        invalid.sync_modes = vec!["unknown".into()];
        assert!(create(&conn, &invalid).is_err());
        invalid.sync_modes = vec!["manual".into()];
        invalid.sync_config = serde_json::json!({"feedUrl": "https://secret.example/feed"});
        assert!(create(&conn, &invalid).is_err());
    }

    #[test]
    fn runtime_replaces_and_origin_binds_validators_without_public_origin_exposure() {
        let conn = setup();
        let created = create(&conn, &input()).unwrap();
        let tx = conn.unchecked_transaction().unwrap();
        runtime_finish_success(
            &tx,
            &created.id,
            "2026-09-24T10:00:00Z",
            Some(SyncValidators {
                origin: "https://calendar.example",
                etag: Some("\"new\""),
                last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT"),
            }),
            "2026-09-24T10:30:00Z",
        )
        .unwrap();
        tx.commit().unwrap();

        let record = sync_runtime_record(&conn, &created.id).unwrap().unwrap();
        assert_eq!(
            record.validator_origin.as_deref(),
            Some("https://calendar.example")
        );
        assert_eq!(
            record.integration.last_sync_etag.as_deref(),
            Some("\"new\"")
        );
        assert!(!serde_json::to_string(&record.integration)
            .unwrap()
            .contains("validator_origin"));

        let tx = conn.unchecked_transaction().unwrap();
        runtime_finish_success(
            &tx,
            &created.id,
            "2026-09-24T10:30:00Z",
            None,
            "2026-09-24T11:00:00Z",
        )
        .unwrap();
        tx.commit().unwrap();
        assert_eq!(
            sync_runtime_record(&conn, &created.id)
                .unwrap()
                .unwrap()
                .validator_origin
                .as_deref(),
            Some("https://calendar.example")
        );

        let tx = conn.unchecked_transaction().unwrap();
        runtime_finish_success(
            &tx,
            &created.id,
            "2026-09-24T11:00:00Z",
            Some(SyncValidators {
                origin: "https://other.example",
                etag: None,
                last_modified: None,
            }),
            "2026-09-24T11:30:00Z",
        )
        .unwrap();
        tx.commit().unwrap();
        let record = sync_runtime_record(&conn, &created.id).unwrap().unwrap();
        assert!(record.validator_origin.is_none());
        assert!(record.integration.last_sync_etag.is_none());
        assert!(record.integration.last_sync_last_modified.is_none());
    }

    #[test]
    fn marks_persisted_configuration_connected_without_claiming_sync_success() {
        let conn = setup();
        let created = create(&conn, &input()).unwrap();
        let configured = mark_configured(&conn, &created.id).unwrap().unwrap();
        assert_eq!(configured.connection_status, "connected");
        assert_eq!(
            configured.effective_capabilities,
            configured.advertised_capabilities
        );
        assert_eq!(configured.sync_status, "idle");
        assert!(configured.last_successful_sync_at.is_none());
    }
    #[test]
    fn recovers_interrupted_pending_and_syncing_without_touching_data() {
        let conn = setup();
        let first = create(&conn, &input()).unwrap();
        let second = create(&conn, &input()).unwrap();
        conn.execute(
            "UPDATE integrations SET sync_status='pending' WHERE id=?1",
            [&first.id],
        )
        .unwrap();
        conn.execute("UPDATE integrations SET sync_status='syncing', connection_status='syncing' WHERE id=?1", [&second.id]).unwrap();
        assert_eq!(
            recover_interrupted_runs(&conn, "2026-01-01T00:01:00Z").unwrap(),
            2
        );
        for id in [&first.id, &second.id] {
            let item = get_by_id(&conn, id).unwrap().unwrap();
            assert_eq!(item.sync_status, "failed");
            assert_eq!(
                item.next_allowed_sync_at.as_deref(),
                Some("2026-01-01T00:01:00Z")
            );
        }
    }
}
