//! Shared lifecycle for the small closed set of renewable-ICS providers.
//!
//! Transport, parsing, reconciliation, scheduling, and persistence remain owned by
//! CAL-ICS, Calendar Core, Integration Sync, and their repositories respectively.
use crate::{
    ai::credentials,
    calendar_ics::{self, FetchResult, IcsValidation},
    db::{
        repositories::{integrations, subscribed_calendars},
        Database,
    },
    integration_sync::{IntegrationSyncRuntime, SyncTrigger},
};
use std::future::Future;

pub const CAPABILITIES: &[&str] = &["calendar_read", "manual_refresh"];
const SYNC_MODES: &[&str] = &["manual", "periodic", "app_start", "app_resume"];

#[derive(Debug, Clone, Copy)]
pub struct ProviderConfig {
    pub id: &'static str,
    pub name: &'static str,
}

pub async fn validate(config: ProviderConfig, url: &str) -> Result<IcsValidation, String> {
    let host = reqwest::Url::parse(url)
        .ok()
        .and_then(|value| value.host_str().map(str::to_string));
    match calendar_ics::fetch(url, None).await? {
        FetchResult::Complete { bytes, .. } => Ok(calendar_ics::validate_with_host(&bytes, host)),
        FetchResult::NotModified => {
            Err("Calendar feed validation needs a complete response".into())
        }
        FetchResult::RateLimited { .. } => Err(format!(
            "{} temporarily limited calendar validation",
            config.name
        )),
    }
}

pub fn candidate(url: &str) -> Result<(), String> {
    subscribed_calendars::validate_feed_url(url)
        .map_err(|_| "Enter a valid HTTPS calendar subscription link.".to_string())
}

pub async fn connect(
    config: ProviderConfig,
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    url: String,
) -> Result<integrations::Integration, String> {
    connect_with_validation(config, db, runtime, url, |url| async move {
        validate(config, &url).await
    })
    .await
}

pub(crate) async fn connect_with_validation<F, Fut>(
    config: ProviderConfig,
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    url: String,
    validate_feed: F,
) -> Result<integrations::Integration, String>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<IcsValidation, String>>,
{
    candidate(&url)?;
    let inspected = validate_feed(url.clone()).await?;
    if !inspected.usable {
        return Err("The subscription did not contain usable calendar data.".into());
    }
    let integration = {
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local connection state is unavailable".to_string())?;
        integrations::create(
            &conn,
            &integrations::IntegrationCreateInput {
                provider_id: config.id.into(),
                enabled: true,
                advertised_capabilities: CAPABILITIES.iter().map(|value| (*value).into()).collect(),
                auth_type: "ics_feed".into(),
                sync_modes: SYNC_MODES.iter().map(|value| (*value).into()).collect(),
                sync_config: serde_json::json!({}),
            },
        )?
    };
    let key = {
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local connection state is unavailable".to_string())?;
        if let Err(error) = subscribed_calendars::create(
            &conn,
            &subscribed_calendars::SubscribedCalendarInput {
                connection_id: integration.id.clone(),
                feed_url: url.clone(),
                display_name: inspected.display_name,
            },
        ) {
            let _ = conn.execute("DELETE FROM integrations WHERE id=?1", [&integration.id]);
            return Err(error);
        }
        subscribed_calendars::credential_key(&conn, &integration.id)?
    };
    if credentials::store(db, &key, &url).is_err() {
        if let Ok(conn) = db.conn.lock() {
            let _ = conn.execute("DELETE FROM integrations WHERE id=?1", [&integration.id]);
        }
        return Err("Calendar subscription secret could not be stored".into());
    }
    let integration = mark_configured(config, db, &integration.id)?;
    let _ = runtime.request(integration.id.clone(), SyncTrigger::Manual);
    Ok(integration)
}

pub async fn replace(
    config: ProviderConfig,
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
    url: String,
) -> Result<(), String> {
    replace_with_validation(config, db, runtime, connection_id, url, |url| async move {
        validate(config, &url).await
    })
    .await
}

pub(crate) async fn replace_with_validation<F, Fut>(
    config: ProviderConfig,
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
    url: String,
    validate_feed: F,
) -> Result<(), String>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<IcsValidation, String>>,
{
    candidate(&url)?;
    let inspected = validate_feed(url.clone()).await?;
    if !inspected.usable {
        return Err("The subscription did not contain usable calendar data.".into());
    }
    replace_validated_secret(config, db, connection_id, &url)?;
    mark_configured(config, db, connection_id)?;
    let _ = runtime.request(connection_id.to_string(), SyncTrigger::Manual);
    Ok(())
}

fn mark_configured(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
) -> Result<integrations::Integration, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|_| "Local connection state is unavailable".to_string())?;
    integrations::mark_configured(&conn, connection_id)?
        .ok_or_else(|| format!("{} connection was not found", config.name))
}

pub(crate) fn replace_validated_secret(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
    url: &str,
) -> Result<(), String> {
    let key = credential_key_for_provider(config, db, connection_id)?
        .ok_or_else(|| format!("{} connection was not found", config.name))?;
    credentials::store(db, &key, url)
        .map_err(|_| "Calendar subscription secret could not be stored".to_string())
}

fn credential_key_for_provider(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
) -> Result<Option<String>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|_| "Local connection state is unavailable".to_string())?;
    let Some(provider) = integrations::get_by_id(&conn, connection_id)?.filter(|integration| {
        integration.provider_id == config.id && integration.auth_type == "ics_feed"
    }) else {
        return Ok(None);
    };
    subscribed_calendars::credential_key(&conn, &provider.id).map(Some)
}

pub fn disconnect(
    config: ProviderConfig,
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
) -> Result<bool, String> {
    runtime.cancel_connection(connection_id);
    disconnect_persisted_connection(config, db, connection_id)
}

pub(crate) fn disconnect_persisted_connection(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
) -> Result<bool, String> {
    let Some(key) = credential_key_for_provider(config, db, connection_id)? else {
        return Ok(false);
    };
    credentials::remove(db, &key)
        .map_err(|_| "Calendar subscription secret could not be removed".to_string())?;
    let conn = db
        .conn
        .lock()
        .map_err(|_| "Local connection state is unavailable".to_string())?;
    conn.execute("DELETE FROM integrations WHERE id=?1", [connection_id])
        .map_err(|_| format!("{} connection could not be removed", config.name))
        .map(|count| count == 1)
}
