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
    let target = replacement_target(config, db, connection_id)?
        .ok_or_else(|| format!("{} connection was not found", config.name))?;
    let inspected = validate_feed(url.clone()).await?;
    if !inspected.usable {
        return Err("The subscription did not contain usable calendar data.".into());
    }
    if target.current_url == url {
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local connection state is unavailable".to_string())?;
        if !integrations::mark_configured_at_generation(
            &conn,
            connection_id,
            target.configuration_generation,
        )? {
            return Err(format!(
                "{} connection configuration changed during replacement",
                config.name
            ));
        }
        drop(conn);
        let _ = runtime.request(connection_id.to_string(), SyncTrigger::Manual);
        return Ok(());
    }
    replace_validated_secret_at_generation(
        config,
        db,
        connection_id,
        target.configuration_generation,
        &target.credential_key,
        &url,
    )?;
    runtime.request_replacement_sync(connection_id.to_string());
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

#[cfg(test)]
pub(crate) fn replace_validated_secret(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
    url: &str,
) -> Result<(), String> {
    let target = replacement_target(config, db, connection_id)?
        .ok_or_else(|| format!("{} connection was not found", config.name))?;
    replace_validated_secret_at_generation(
        config,
        db,
        connection_id,
        target.configuration_generation,
        &target.credential_key,
        url,
    )
}

struct ReplacementTarget {
    credential_key: String,
    current_url: String,
    configuration_generation: i64,
}

fn replacement_target(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
) -> Result<Option<ReplacementTarget>, String> {
    let conn = db
        .conn
        .lock()
        .map_err(|_| "Local connection state is unavailable".to_string())?;
    let Some(record) = integrations::sync_runtime_record(&conn, connection_id)?.filter(|record| {
        record.integration.provider_id == config.id && record.integration.auth_type == "ics_feed"
    }) else {
        return Ok(None);
    };
    if subscribed_calendars::get_by_connection(&conn, connection_id)?.is_none() {
        return Ok(None);
    }
    let credential_key = record
        .credential_key
        .ok_or_else(|| "Calendar subscription is not configured".to_string())?;
    let current_url = credentials::get_from_connection(&conn, db.crypto.as_ref(), &credential_key)?
        .ok_or_else(|| "Calendar subscription is not configured".to_string())?;
    Ok(Some(ReplacementTarget {
        credential_key,
        current_url,
        configuration_generation: record.configuration_generation,
    }))
}

fn replace_validated_secret_at_generation(
    config: ProviderConfig,
    db: &Database,
    connection_id: &str,
    expected_generation: i64,
    credential_key: &str,
    url: &str,
) -> Result<(), String> {
    let encoded = credentials::prepare_value(db.crypto.as_ref(), url)
        .map_err(|_| "Calendar subscription secret could not be stored".to_string())?;
    let conn = db
        .conn
        .lock()
        .map_err(|_| "Local connection state is unavailable".to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| "Calendar subscription could not be replaced".to_string())?;
    let Some(_) = integrations::replace_ics_configuration(&tx, connection_id, expected_generation)?
    else {
        return Err(format!(
            "{} connection configuration changed during replacement",
            config.name
        ));
    };
    credentials::store_prepared(&tx, credential_key, &encoded)
        .map_err(|_| "Calendar subscription secret could not be stored".to_string())?;
    tx.commit()
        .map_err(|_| "Calendar subscription could not be replaced".to_string())
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

pub fn remove_unsupported_connection(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
) -> Result<bool, String> {
    runtime.cancel_connection(connection_id);
    remove_unsupported_persisted_connection(db, connection_id)
}

fn remove_unsupported_persisted_connection(
    db: &Database,
    connection_id: &str,
) -> Result<bool, String> {
    let record = {
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local connection state is unavailable".to_string())?;
        integrations::sync_runtime_record(&conn, connection_id)?
    };
    let Some(record) = record else {
        return Ok(false);
    };
    if record.integration.auth_type != "ics_feed"
        || crate::integration_sync::supports(
            &record.integration.provider_id,
            &record.integration.auth_type,
        )
    {
        return Err("Only unsupported calendar connections can be removed here".into());
    }
    let key = record
        .credential_key
        .ok_or_else(|| "Unsupported calendar connection has no credential reference".to_string())?;
    credentials::remove(db, &key)
        .map_err(|_| "Calendar subscription secret could not be removed".to_string())?;
    let conn = db
        .conn
        .lock()
        .map_err(|_| "Local connection state is unavailable".to_string())?;
    conn.execute("DELETE FROM integrations WHERE id=?1", [connection_id])
        .map_err(|_| "Unsupported calendar connection could not be removed".to_string())
        .map(|count| count == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::credentials::{self, SecretCrypto};

    struct TestCrypto;

    impl SecretCrypto for TestCrypto {
        fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            Ok(data.to_vec())
        }

        fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            Ok(data.to_vec())
        }
    }

    fn db() -> Database {
        Database::open(
            tempfile::tempdir().unwrap().path().join("aether.db"),
            Box::new(TestCrypto),
        )
        .unwrap()
    }

    #[test]
    fn unsupported_legacy_calendar_cleanup_removes_secret_and_cascaded_data() {
        let db = db();
        let key = "integration:legacy:credential";
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO integrations(id,provider_id,enabled,auth_type,credential_key,connection_status) VALUES ('legacy','brightspace',1,'ics_feed',?1,'connected')",
                [key],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO subscribed_calendars(id,connection_id) VALUES ('legacy-calendar','legacy')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO spaces(id,name,template_type) VALUES ('school','School','school')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO school_space_sources(school_space_id,connection_id) VALUES ('school','legacy')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO external_events(id,connection_id,external_id,occurrence_id,title,time_kind,start_at_utc,end_at_utc,timezone,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at) VALUES ('legacy-event','legacy','event','','Legacy','timed','2026-09-25T08:00:00Z','2026-09-25T09:00:00Z','Europe/Berlin','general','active','ics','1','hash','2026-09-25T00:00:00Z','2026-09-25T00:00:00Z','2026-09-25T00:00:00Z')",
                [],
            )
            .unwrap();
        }
        credentials::store(&db, key, "https://example.edu/private.ics").unwrap();

        assert!(remove_unsupported_persisted_connection(&db, "legacy").unwrap());
        assert_eq!(credentials::get(&db, key).unwrap(), None);
        let conn = db.conn.lock().unwrap();
        assert!(integrations::get_by_id(&conn, "legacy").unwrap().is_none());
        let remaining: i64 = conn
            .query_row(
                "SELECT count(*) FROM external_events WHERE connection_id='legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);
        let bindings: i64 = conn
            .query_row(
                "SELECT count(*) FROM school_space_sources WHERE connection_id='legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(bindings, 0);
    }

    #[test]
    fn unsupported_cleanup_refuses_supported_my_timetable_connection() {
        let db = db();
        db.conn
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO integrations(id,provider_id,enabled,auth_type,credential_key,connection_status) VALUES ('mtt','my_timetable',1,'ics_feed','integration:mtt:credential','connected')",
                [],
            )
            .unwrap();
        assert_eq!(
            remove_unsupported_persisted_connection(&db, "mtt").unwrap_err(),
            "Only unsupported calendar connections can be removed here"
        );
        assert!(integrations::get_by_id(&db.conn.lock().unwrap(), "mtt")
            .unwrap()
            .is_some());
    }
}
