use rusqlite::{params, Connection};
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
const CONNECTION_STATUSES: &[&str] = &[
    "connected",
    "syncing",
    "degraded",
    "reauthentication_required",
    "permission_denied",
    "rate_limited",
    "institution_configuration_required",
    "unsupported",
    "disconnected",
];
const SYNC_STATUSES: &[&str] = &["idle", "pending", "syncing", "succeeded", "failed"];
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationUpdateInput {
    pub enabled: bool,
    pub advertised_capabilities: Vec<String>,
    pub effective_capabilities: Vec<String>,
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

fn validate_timestamp(value: &Option<String>, field: &str) -> Result<Option<String>, String> {
    match value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) if value.len() <= 64 => Ok(Some(value.to_string())),
        Some(_) => Err(format!("Integration {field} must not exceed 64 characters")),
        None => Ok(None),
    }
}

fn bounded(value: &Option<String>, limit: usize, field: &str) -> Result<Option<String>, String> {
    match value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) if value.chars().count() <= limit => Ok(Some(value.to_string())),
        Some(_) => Err(format!(
            "Integration {field} must not exceed {limit} characters"
        )),
        None => Ok(None),
    }
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

pub fn update(
    conn: &Connection,
    id: &str,
    input: &IntegrationUpdateInput,
) -> Result<Option<Integration>, String> {
    if get_by_id(conn, id)?.is_none() {
        return Ok(None);
    }
    let advertised_capabilities = validate_tokens(
        &input.advertised_capabilities,
        None,
        "advertised capabilities",
    )?;
    let effective_capabilities = validate_tokens(
        &input.effective_capabilities,
        None,
        "effective capabilities",
    )?;
    let sync_modes = validate_tokens(&input.sync_modes, Some(SYNC_MODES), "sync modes")?;
    let sync_config = validate_config(&input.sync_config)?;
    if !CONNECTION_STATUSES.contains(&input.connection_status.as_str())
        || !SYNC_STATUSES.contains(&input.sync_status.as_str())
    {
        return Err("Invalid Integration status".to_string());
    }
    let error_code = bounded(&input.last_sync_error_code, 100, "last sync error code")?;
    let error_message = bounded(
        &input.last_sync_error_message,
        500,
        "last sync error message",
    )?;
    let etag = bounded(&input.last_sync_etag, 512, "ETag")?;
    let last_modified = bounded(&input.last_sync_last_modified, 128, "Last-Modified")?;
    let cursor = bounded(&input.sync_cursor, 2048, "sync cursor")?;
    if input
        .disconnect_reason
        .as_deref()
        .is_some_and(|reason| !["local", "remote_revoke"].contains(&reason))
    {
        return Err("Invalid Integration disconnect reason".to_string());
    }
    conn.execute(
        "UPDATE integrations SET enabled=?1, advertised_capabilities_json=?2, effective_capabilities_json=?3, sync_modes_json=?4, sync_config_json=?5, connection_status=?6, sync_status=?7, disconnect_reason=?8, last_attempted_at=?9, last_successful_sync_at=?10, next_allowed_sync_at=?11, last_sync_error_code=?12, last_sync_error_message=?13, last_sync_etag=?14, last_sync_last_modified=?15, sync_cursor=?16, rate_limit_remaining=?17, retry_after_at=?18, credential_expires_at=?19, credential_rotated_at=?20, updated_at=datetime('now') WHERE id=?21",
        params![input.enabled, serde_json::to_string(&advertised_capabilities).unwrap(), serde_json::to_string(&effective_capabilities).unwrap(), serde_json::to_string(&sync_modes).unwrap(), sync_config, input.connection_status, input.sync_status, input.disconnect_reason, validate_timestamp(&input.last_attempted_at, "last attempted timestamp")?, validate_timestamp(&input.last_successful_sync_at, "last successful sync timestamp")?, validate_timestamp(&input.next_allowed_sync_at, "next allowed sync timestamp")?, error_code, error_message, etag, last_modified, cursor, input.rate_limit_remaining, validate_timestamp(&input.retry_after_at, "retry-after timestamp")?, validate_timestamp(&input.credential_expires_at, "credential expiry timestamp")?, validate_timestamp(&input.credential_rotated_at, "credential rotation timestamp")?, id],
    ).map_err(|error| format!("Integration update error: {error}"))?;
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
    fn updates_sync_lifecycle_and_rejects_unbounded_errors() {
        let conn = setup();
        let created = create(&conn, &input()).unwrap();
        let updated = update(
            &conn,
            &created.id,
            &IntegrationUpdateInput {
                enabled: false,
                advertised_capabilities: vec!["events_read".into()],
                effective_capabilities: vec!["events_read".into()],
                sync_modes: vec!["manual".into()],
                sync_config: serde_json::json!({}),
                connection_status: "connected".into(),
                sync_status: "succeeded".into(),
                disconnect_reason: None,
                last_attempted_at: Some("2026-09-21T12:00:00Z".into()),
                last_successful_sync_at: Some("2026-09-21T12:00:00Z".into()),
                next_allowed_sync_at: Some("2026-09-21T13:00:00Z".into()),
                last_sync_error_code: None,
                last_sync_error_message: None,
                last_sync_etag: Some("abc".into()),
                last_sync_last_modified: None,
                sync_cursor: Some("cursor".into()),
                rate_limit_remaining: Some(20),
                retry_after_at: None,
                credential_expires_at: Some("2026-10-01T00:00:00Z".into()),
                credential_rotated_at: None,
            },
        )
        .unwrap()
        .unwrap();
        assert!(!updated.enabled);
        assert_eq!(updated.sync_status, "succeeded");
        let mut invalid = input();
        invalid.sync_modes = vec!["unknown".into()];
        assert!(create(&conn, &invalid).is_err());
        invalid.sync_modes = vec!["manual".into()];
        invalid.sync_config = serde_json::json!({"feedUrl": "https://secret.example/feed"});
        assert!(create(&conn, &invalid).is_err());
    }
}
