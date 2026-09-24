//! Thin MyTimetable provider profile. Transport, parsing, reconciliation, and scheduling
//! remain owned by the shared subscribed-calendar and integration-sync modules.
use crate::{
    calendar_ics::{self, IcsValidation},
    db::{repositories::integrations, Database},
    integration_sync::IntegrationSyncRuntime,
    subscribed_calendar_provider::{self, ProviderConfig},
};
use serde::Serialize;
#[cfg(test)]
use std::future::Future;
use tauri::State;

pub const PROVIDER_ID: &str = "my_timetable";
const CONFIG: ProviderConfig = ProviderConfig {
    id: PROVIDER_ID,
    name: "MyTimetable",
};
const CAPABILITIES: &[&str] = subscribed_calendar_provider::CAPABILITIES;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub capabilities: Vec<&'static str>,
    pub auth_type: &'static str,
}

pub fn profile() -> ProviderProfile {
    ProviderProfile {
        id: PROVIDER_ID,
        name: "MyTimetable",
        capabilities: CAPABILITIES.to_vec(),
        auth_type: "ics_feed",
    }
}

pub fn classify(categories: &[String]) -> String {
    let normalized = categories
        .iter()
        .map(|value| value.trim().to_ascii_uppercase().replace(['-', ' '], "_"))
        .collect::<Vec<_>>();
    if normalized
        .iter()
        .any(|value| matches!(value.as_str(), "EXAM" | "EXAMINATION"))
    {
        "exam".into()
    } else if normalized
        .iter()
        .any(|value| matches!(value.as_str(), "LESSON" | "LECTURE" | "TEACHING_ACTIVITY"))
    {
        "lesson".into()
    } else {
        "general".into()
    }
}

pub fn metadata(categories: &[String], description: Option<&str>) -> calendar_ics::EventMetadata {
    let mut group_references = description
        .into_iter()
        .flat_map(str::lines)
        .filter_map(|line| line.trim().strip_prefix("Groep(en):"))
        .flat_map(|value| value.split([',', ';']))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    group_references.sort_unstable();
    group_references.dedup();
    calendar_ics::EventMetadata {
        event_kind: classify(categories),
        group_references,
    }
}

pub async fn validate(url: &str) -> Result<IcsValidation, String> {
    subscribed_calendar_provider::validate(CONFIG, url).await
}

fn candidate(url: &str) -> Result<(), String> {
    subscribed_calendar_provider::candidate(url)
}

pub async fn connect(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    url: String,
) -> Result<integrations::Integration, String> {
    subscribed_calendar_provider::connect(CONFIG, db, runtime, url).await
}

#[cfg(test)]
async fn connect_with_validation<F, Fut>(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    url: String,
    validate_feed: F,
) -> Result<integrations::Integration, String>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Result<IcsValidation, String>>,
{
    subscribed_calendar_provider::connect_with_validation(CONFIG, db, runtime, url, validate_feed)
        .await
}

pub async fn replace(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
    url: String,
) -> Result<(), String> {
    subscribed_calendar_provider::replace(CONFIG, db, runtime, connection_id, url).await
}

/// Internal composition seam: public production calls retain the shared CAL-ICS
/// validator, while provider-path tests can supply a deterministic result.
#[cfg(test)]
async fn replace_with_validation<F, Fut>(
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
    subscribed_calendar_provider::replace_with_validation(
        CONFIG,
        db,
        runtime,
        connection_id,
        url,
        validate_feed,
    )
    .await
}

#[cfg(test)]
fn replace_validated_secret(db: &Database, connection_id: &str, url: &str) -> Result<(), String> {
    subscribed_calendar_provider::replace_validated_secret(CONFIG, db, connection_id, url)
}

pub fn disconnect(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
) -> Result<bool, String> {
    subscribed_calendar_provider::disconnect(CONFIG, db, runtime, connection_id)
}

#[cfg(test)]
fn disconnect_persisted_connection(db: &Database, connection_id: &str) -> Result<bool, String> {
    subscribed_calendar_provider::disconnect_persisted_connection(CONFIG, db, connection_id)
}

#[tauri::command]
pub fn my_timetable_profile() -> ProviderProfile {
    profile()
}
#[tauri::command]
pub async fn my_timetable_validate(url: String) -> Result<IcsValidation, String> {
    candidate(&url)?;
    validate(&url).await
}
#[tauri::command]
pub async fn my_timetable_connect(
    db: State<'_, Database>,
    runtime: State<'_, IntegrationSyncRuntime>,
    url: String,
) -> Result<integrations::Integration, String> {
    connect(&db, &runtime, url).await
}
#[tauri::command]
pub async fn my_timetable_replace(
    db: State<'_, Database>,
    runtime: State<'_, IntegrationSyncRuntime>,
    connection_id: String,
    url: String,
) -> Result<(), String> {
    replace(&db, &runtime, &connection_id, url).await
}
#[tauri::command]
pub fn my_timetable_disconnect(
    db: State<Database>,
    runtime: State<IntegrationSyncRuntime>,
    connection_id: String,
) -> Result<bool, String> {
    disconnect(&db, &runtime, &connection_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration_sync::{
        PreparedSync, RuntimeHost, SyncFailure, SyncPolicy, SyncStateEvent, SyncTrigger,
    };
    use crate::{
        ai::credentials::{self, SecretCrypto},
        db::repositories::subscribed_calendars,
    };
    use async_trait::async_trait;
    use futures_util::future::BoxFuture;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use tokio_util::sync::CancellationToken;

    struct TestCrypto(Arc<AtomicBool>);

    #[derive(Clone)]
    struct RuntimeObservation {
        connection_id: String,
        record: integrations::SyncRuntimeRecord,
        credential: Option<String>,
    }

    /// SQLite-backed implementation of the production runtime contract used only by
    /// provider-path tests. It observes the persisted state at `request` lookup time.
    struct SqliteRuntimeHost {
        db: Arc<Database>,
        observed: Mutex<Vec<RuntimeObservation>>,
        fail_prepare: AtomicBool,
        block_prepare: AtomicBool,
        prepare_count: std::sync::atomic::AtomicUsize,
        entered_prepare: tokio::sync::Notify,
        entered_terminal: tokio::sync::Notify,
    }

    impl SqliteRuntimeHost {
        fn new(db: Arc<Database>) -> Arc<Self> {
            Arc::new(Self {
                db,
                observed: Mutex::new(Vec::new()),
                fail_prepare: AtomicBool::new(false),
                block_prepare: AtomicBool::new(false),
                prepare_count: std::sync::atomic::AtomicUsize::new(0),
                entered_prepare: tokio::sync::Notify::new(),
                entered_terminal: tokio::sync::Notify::new(),
            })
        }
    }

    #[async_trait]
    impl RuntimeHost for SqliteRuntimeHost {
        fn record(&self, id: &str) -> Result<Option<integrations::SyncRuntimeRecord>, String> {
            let record = integrations::sync_runtime_record(&self.db.conn.lock().unwrap(), id)?;
            if let Some(record) = &record {
                let credential = record
                    .credential_key
                    .as_deref()
                    .map(|key| credentials::get(&self.db, key))
                    .transpose()?
                    .flatten();
                self.observed.lock().unwrap().push(RuntimeObservation {
                    connection_id: id.into(),
                    record: record.clone(),
                    credential,
                });
            }
            Ok(record)
        }
        fn candidates(&self) -> Result<Vec<integrations::Integration>, String> {
            integrations::list_sync_candidates(&self.db.conn.lock().unwrap())
        }
        fn recover_interrupted(&self, next: &str) -> Result<usize, String> {
            integrations::recover_interrupted_runs(&self.db.conn.lock().unwrap(), next)
        }
        fn mark_running(
            &self,
            id: &str,
            configuration_generation: i64,
            trigger: SyncTrigger,
            now: &str,
        ) -> Result<bool, String> {
            let trigger = match trigger {
                SyncTrigger::Manual => "manual",
                SyncTrigger::Periodic => "periodic",
                SyncTrigger::Startup => "app_start",
                SyncTrigger::Resume => "app_resume",
            };
            integrations::runtime_mark_running(
                &self.db.conn.lock().unwrap(),
                id,
                configuration_generation,
                trigger,
                now,
            )
        }
        async fn prepare(
            &self,
            record: &integrations::SyncRuntimeRecord,
            cancel: CancellationToken,
        ) -> Result<(PreparedSync, SyncPolicy), SyncFailure> {
            self.prepare_count.fetch_add(1, Ordering::SeqCst);
            self.entered_prepare.notify_one();
            if self.block_prepare.load(Ordering::SeqCst) {
                tokio::time::sleep(std::time::Duration::from_millis(80)).await;
            }
            if cancel.is_cancelled() || self.fail_prepare.load(Ordering::SeqCst) {
                return Err(SyncFailure {
                    code: "transient",
                    message: "fixture failure",
                    connection_status: "degraded",
                });
            }
            Ok((
                PreparedSync {
                    connection_id: record.integration.id.clone(),
                    configuration_generation: record.configuration_generation,
                    outcome: crate::integration_sync::PreparedSyncOutcome::NotModified,
                },
                SyncPolicy {
                    success_interval_minutes: 30,
                },
            ))
        }
        fn commit_success(
            &self,
            prepared: PreparedSync,
            now: &str,
            next: &str,
        ) -> Result<integrations::SyncCompletion, String> {
            let conn = self.db.conn.lock().unwrap();
            let transaction = conn
                .unchecked_transaction()
                .map_err(|error| error.to_string())?;
            let completion = integrations::runtime_finish_success(
                &transaction,
                &prepared.connection_id,
                prepared.configuration_generation,
                now,
                None,
                next,
            )?;
            if completion == integrations::SyncCompletion::Applied {
                transaction.commit().map_err(|error| error.to_string())?;
            }
            Ok(completion)
        }
        fn finish_failure(
            &self,
            id: &str,
            configuration_generation: i64,
            failure: &SyncFailure,
            next: &str,
            retry: Option<&str>,
        ) -> Result<integrations::SyncCompletion, String> {
            integrations::runtime_finish_failure(
                &self.db.conn.lock().unwrap(),
                id,
                configuration_generation,
                integrations::SyncFailureUpdate {
                    code: failure.code,
                    message: failure.message,
                    next_allowed: next,
                    connection_status: failure.connection_status,
                    retry_after: retry,
                },
            )
        }
        fn emit(&self, event: SyncStateEvent) {
            if event.state == "terminal" {
                self.entered_terminal.notify_one();
            }
        }
        fn spawn(&self, task: BoxFuture<'static, ()>) {
            tokio::spawn(task);
        }
    }

    impl SecretCrypto for TestCrypto {
        fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            if self.0.load(Ordering::SeqCst) {
                Err("test_secret_write_failed".into())
            } else {
                Ok(data.to_vec())
            }
        }

        fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
            Ok(data.to_vec())
        }
    }

    fn test_db() -> (Database, Arc<AtomicBool>) {
        let fail_writes = Arc::new(AtomicBool::new(false));
        let db = Database::open(
            tempfile::tempdir().unwrap().path().join("aether.db"),
            Box::new(TestCrypto(fail_writes.clone())),
        )
        .unwrap();
        (db, fail_writes)
    }

    fn connected_calendar(db: &Database) -> (String, String) {
        let integration = {
            let conn = db.conn.lock().unwrap();
            integrations::create(
                &conn,
                &integrations::IntegrationCreateInput {
                    provider_id: PROVIDER_ID.into(),
                    enabled: true,
                    advertised_capabilities: CAPABILITIES
                        .iter()
                        .map(|value| (*value).into())
                        .collect(),
                    auth_type: "ics_feed".into(),
                    sync_modes: vec!["manual".into()],
                    sync_config: serde_json::json!({}),
                },
            )
            .unwrap()
        };
        db.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET connection_status='connected' WHERE id=?1",
                [&integration.id],
            )
            .unwrap();
        let key = {
            let conn = db.conn.lock().unwrap();
            subscribed_calendars::create(
                &conn,
                &subscribed_calendars::SubscribedCalendarInput {
                    connection_id: integration.id.clone(),
                    feed_url: "https://calendar.example/feed.ics".into(),
                    display_name: None,
                },
            )
            .unwrap();
            subscribed_calendars::credential_key(&conn, &integration.id).unwrap()
        };
        credentials::store(db, &key, "https://calendar.example/old?token=old-secret").unwrap();
        (integration.id, key)
    }

    fn cache_event(db: &Database, connection_id: &str) {
        let feed = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:cached\r\nDTSTART:20260101T090000Z\r\nDTEND:20260101T100000Z\r\nSUMMARY:Cached\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let snapshot = calendar_ics::normalize(feed, connection_id, chrono::Utc::now()).unwrap();
        let mut conn = db.conn.lock().unwrap();
        crate::db::repositories::external_events::reconcile(
            &mut conn,
            connection_id,
            &snapshot,
            crate::db::repositories::external_events::ReconciliationMode::ObservedOnly,
            None,
            "2026-01-01T00:00:00Z",
        )
        .unwrap();
    }

    fn cached_event_count(db: &Database, connection_id: &str) -> i64 {
        db.conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1",
                [connection_id],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn usable_validation() -> IcsValidation {
        IcsValidation {
            usable: true,
            event_count: 1,
            error_code: None,
            display_name: None,
            covered_start: None,
            covered_end: None,
            public_host: None,
            warnings: vec![],
        }
    }

    #[test]
    fn profile_is_read_only_and_manual_only() {
        let value = profile();
        assert_eq!(value.id, PROVIDER_ID);
        assert_eq!(value.auth_type, "ics_feed");
        assert_eq!(value.capabilities, vec!["calendar_read", "manual_refresh"]);
        assert!(!value.capabilities.iter().any(|v| matches!(
            *v,
            "oauth" | "account_profile" | "external_write" | "webhook" | "api"
        )));
    }

    #[test]
    fn sqlite_runtime_host_observes_sqlite_backed_request_state() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let host = SqliteRuntimeHost::new(db.clone());
        let (connection_id, _) = connected_calendar(&db);
        assert!(host.record(&connection_id).unwrap().is_some());
        let observed = host.observed.lock().unwrap();
        assert_eq!(observed[0].connection_id, connection_id);
        assert_eq!(observed[0].record.integration.provider_id, PROVIDER_ID);
        assert!(observed[0].credential.is_some());
    }

    #[tokio::test]
    async fn successful_connect_marks_configuration_connected_before_runtime_starts() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let host = SqliteRuntimeHost::new(db.clone());
        let runtime = IntegrationSyncRuntime::new(host.clone());

        let integration = connect_with_validation(
            &db,
            &runtime,
            "https://calendar.example/feed?token=new-secret".into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .unwrap();

        assert_eq!(integration.connection_status, "connected");
        assert_eq!(integration.sync_status, "idle");
        {
            let observed = host.observed.lock().unwrap();
            assert_eq!(
                observed[0].record.integration.connection_status,
                "connected"
            );
        }
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            host.entered_prepare.notified(),
        )
        .await
        .expect("runtime did not proceed beyond mark_running");
        assert_eq!(host.prepare_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_connect_validation_never_creates_a_connected_integration() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let host = SqliteRuntimeHost::new(db.clone());
        let runtime = IntegrationSyncRuntime::new(host);

        assert!(connect_with_validation(
            &db,
            &runtime,
            "https://calendar.example/feed?token=invalid".into(),
            |_| async { Err("validation failed".into()) },
        )
        .await
        .is_err());
        assert!(integrations::list(&db.conn.lock().unwrap())
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn failed_replacement_validation_preserves_generation_validators_retry_and_cache() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, key) = connected_calendar(&db);
        cache_event(&db, &connection_id);
        db.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET last_sync_etag='old-etag', last_sync_last_modified='old-modified', last_sync_validator_origin='https://calendar.example', next_allowed_sync_at='2099-01-01T00:00:00Z', retry_after_at='2099-01-01T00:00:00Z' WHERE id=?1",
                [&connection_id],
            )
            .unwrap();
        let host = SqliteRuntimeHost::new(db.clone());
        let runtime = IntegrationSyncRuntime::new(host);

        assert!(replace_with_validation(
            &db,
            &runtime,
            &connection_id,
            "https://calendar.example/rejected?token=new".into(),
            |_| async { Err("candidate rejected".into()) },
        )
        .await
        .is_err());

        assert_eq!(
            credentials::get(&db, &key).unwrap().as_deref(),
            Some("https://calendar.example/old?token=old-secret")
        );
        let record = integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
            .unwrap()
            .unwrap();
        assert_eq!(record.configuration_generation, 1);
        assert_eq!(
            record.integration.last_sync_etag.as_deref(),
            Some("old-etag")
        );
        assert_eq!(
            record.validator_origin.as_deref(),
            Some("https://calendar.example")
        );
        assert_eq!(
            record.integration.next_allowed_sync_at.as_deref(),
            Some("2099-01-01T00:00:00Z")
        );
        assert_eq!(cached_event_count(&db, &connection_id), 1);
    }

    #[tokio::test]
    async fn slower_concurrent_replacement_cannot_overwrite_a_newer_generation() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, key) = connected_calendar(&db);
        let host = SqliteRuntimeHost::new(db.clone());
        host.fail_prepare.store(true, Ordering::SeqCst);
        let runtime = Arc::new(IntegrationSyncRuntime::new(host));
        let validation_started = Arc::new(tokio::sync::Notify::new());
        let release_validation = Arc::new(tokio::sync::Notify::new());
        let stale_task = {
            let db = db.clone();
            let runtime = runtime.clone();
            let connection_id = connection_id.clone();
            let validation_started = validation_started.clone();
            let release_validation = release_validation.clone();
            tokio::spawn(async move {
                replace_with_validation(
                    &db,
                    &runtime,
                    &connection_id,
                    "https://calendar.example/b?token=b".into(),
                    |_| async move {
                        validation_started.notify_one();
                        release_validation.notified().await;
                        Ok(usable_validation())
                    },
                )
                .await
            })
        };
        validation_started.notified().await;

        replace_with_validation(
            &db,
            &runtime,
            &connection_id,
            "https://calendar.example/c?token=c".into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .unwrap();
        release_validation.notify_one();
        assert!(stale_task.await.unwrap().is_err());

        assert_eq!(
            credentials::get(&db, &key).unwrap().as_deref(),
            Some("https://calendar.example/c?token=c")
        );
        assert_eq!(
            integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap()
                .configuration_generation,
            2
        );
    }

    #[tokio::test]
    async fn validating_the_same_feed_does_not_advance_generation_or_clear_validators() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, _) = connected_calendar(&db);
        db.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET last_sync_etag='same-etag', last_sync_validator_origin='https://calendar.example' WHERE id=?1",
                [&connection_id],
            )
            .unwrap();
        let host = SqliteRuntimeHost::new(db.clone());
        let runtime = IntegrationSyncRuntime::new(host);

        replace_with_validation(
            &db,
            &runtime,
            &connection_id,
            "https://calendar.example/old?token=old-secret".into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .unwrap();

        let record = integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
            .unwrap()
            .unwrap();
        assert_eq!(record.configuration_generation, 1);
        assert_eq!(
            record.integration.last_sync_etag.as_deref(),
            Some("same-etag")
        );
        assert_eq!(
            record.validator_origin.as_deref(),
            Some("https://calendar.example")
        );
    }

    #[tokio::test]
    async fn replacement_persists_new_secret_before_runtime_observes_a_failed_refresh() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, key) = connected_calendar(&db);
        db.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET connection_status='disconnected', last_successful_sync_at='2026-01-01T00:00:00Z', last_sync_etag='old-etag', last_sync_last_modified='old-modified', last_sync_validator_origin='https://calendar.example', next_allowed_sync_at='2099-01-01T00:00:00Z', retry_after_at='2099-01-01T00:00:00Z', consecutive_sync_failures=4 WHERE id=?1",
                [&connection_id],
            )
            .unwrap();
        cache_event(&db, &connection_id);
        let host = SqliteRuntimeHost::new(db.clone());
        host.fail_prepare.store(true, Ordering::SeqCst);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let new = "https://calendar.example/new?token=new-secret";
        replace_with_validation(&db, &runtime, &connection_id, new.into(), |_| async {
            Ok(usable_validation())
        })
        .await
        .unwrap();
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            host.entered_prepare.notified(),
        )
        .await
        .expect("runtime did not proceed beyond mark_running after replacement");
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            host.entered_terminal.notified(),
        )
        .await
        .expect("replacement failure did not reach a terminal state");
        assert_eq!(credentials::get(&db, &key).unwrap().as_deref(), Some(new));
        assert_eq!(cached_event_count(&db, &connection_id), 1);
        let observed = host.observed.lock().unwrap();
        assert!(observed
            .iter()
            .all(|observation| observation.credential.as_deref() == Some(new)));
        assert!(observed
            .iter()
            .all(|observation| observation.connection_id == connection_id));
        assert_eq!(
            observed[0].record.integration.connection_status,
            "connected"
        );
        assert_eq!(observed[0].record.configuration_generation, 2);
        assert!(observed[0].record.integration.last_sync_etag.is_none());
        assert!(observed[0]
            .record
            .integration
            .last_sync_last_modified
            .is_none());
        assert!(observed[0].record.validator_origin.is_none());
        drop(observed);
        let current = integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
            .unwrap()
            .unwrap();
        assert_eq!(current.configuration_generation, 2);
        assert_eq!(current.integration.connection_status, "degraded");
        assert!(current.integration.last_successful_sync_at.is_none());
        assert!(current.integration.last_sync_etag.is_none());
        assert!(current.integration.last_sync_last_modified.is_none());
        assert!(current.validator_origin.is_none());
        assert_eq!(current.consecutive_failures, 1);
    }

    #[test]
    fn disconnect_uses_generic_cascade_and_makes_future_manual_sync_ineligible() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, key) = connected_calendar(&db);
        cache_event(&db, &connection_id);
        let host = SqliteRuntimeHost::new(db.clone());
        let runtime = IntegrationSyncRuntime::new(host);

        assert!(disconnect(&db, &runtime, &connection_id).unwrap());
        assert_eq!(credentials::get(&db, &key).unwrap(), None);
        assert!(
            integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .is_none()
        );
        let external_count: i64 = db
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1",
                [&connection_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            external_count, 0,
            "generic foreign-key cascade owns cleanup"
        );
        assert!(matches!(
            runtime.request(connection_id, crate::integration_sync::SyncTrigger::Manual),
            crate::integration_sync::SyncRequestResult::Rejected { .. }
        ));
    }

    #[test]
    fn provider_normalized_lifecycle_uses_shared_identity_and_reconciliation_contract() {
        let (db, _) = test_db();
        let (connection_id, _) = connected_calendar(&db);
        let feed = |start: &str, location: &str, status: &str| {
            format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:provider-event\r\nDTSTART:{start}\r\nLOCATION:{location}\r\nSTATUS:{status}\r\nSUMMARY:Lesson\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        )
        };
        let reconcile = |bytes: Vec<u8>, mode| {
            let snapshot = calendar_ics::normalize_with_classifier(
                &bytes,
                &connection_id,
                chrono::Utc::now(),
                classify,
            )
            .unwrap();
            let mut conn = db.conn.lock().unwrap();
            crate::db::repositories::external_events::reconcile(
                &mut conn,
                &connection_id,
                &snapshot,
                mode,
                Some(("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z")),
                "2026-01-01T00:00:00Z",
            )
            .unwrap();
        };
        reconcile(
            feed("20260101T090000Z", "Old room", "CONFIRMED").into_bytes(),
            crate::db::repositories::external_events::ReconciliationMode::Authoritative,
        );
        reconcile(
            feed("20260101T110000Z", "New room", "CONFIRMED").into_bytes(),
            crate::db::repositories::external_events::ReconciliationMode::Authoritative,
        );
        let conn = db.conn.lock().unwrap();
        let (count, start, location): (i64, String, String) = conn
            .query_row(
                "SELECT count(*), min(start_at_utc), min(location) FROM external_events",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(start, "2026-01-01T11:00:00Z");
        assert_eq!(location, "New room");
    }

    #[test]
    fn provider_cancellation_removal_reappearance_and_partial_snapshots_follow_shared_contract() {
        let (db, _) = test_db();
        let (connection_id, _) = connected_calendar(&db);
        let normalize = |status: &str| {
            calendar_ics::normalize_with_classifier(
            format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:lifecycle\r\nDTSTART:20260101T090000Z\r\nSUMMARY:Ordinary title\r\nDESCRIPTION:No heuristic\r\nSTATUS:{status}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n").as_bytes(),
            &connection_id, chrono::Utc::now(), classify).unwrap()
        };
        let mut conn = db.conn.lock().unwrap();
        use crate::db::repositories::external_events::{reconcile, ReconciliationMode};
        reconcile(
            &mut conn,
            &connection_id,
            &normalize("CONFIRMED"),
            ReconciliationMode::Authoritative,
            Some(("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z")),
            "2026-01-01T00:00:00Z",
        )
        .unwrap();
        reconcile(
            &mut conn,
            &connection_id,
            &normalize("CANCELLED"),
            ReconciliationMode::Authoritative,
            Some(("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z")),
            "2026-01-02T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT status FROM external_events", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "cancelled"
        );
        reconcile(
            &mut conn,
            &connection_id,
            &[],
            ReconciliationMode::Authoritative,
            Some(("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z")),
            "2026-01-03T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT status FROM external_events", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "removed"
        );
        reconcile(
            &mut conn,
            &connection_id,
            &normalize("CONFIRMED"),
            ReconciliationMode::Authoritative,
            Some(("2026-01-01T00:00:00Z", "2026-01-02T00:00:00Z")),
            "2026-01-04T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT count(*), min(status) FROM external_events",
                [],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            )
            .unwrap(),
            (1, "active".into())
        );
        reconcile(
            &mut conn,
            &connection_id,
            &[],
            ReconciliationMode::ObservedOnly,
            None,
            "2026-01-05T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM external_events", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn provider_runtime_not_modified_preserves_cached_identity_without_reconciliation() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, _) = connected_calendar(&db);
        cache_event(&db, &connection_id);
        let host = SqliteRuntimeHost::new(db.clone());
        let runtime = IntegrationSyncRuntime::new(host);
        assert!(matches!(
            runtime.request(
                connection_id.clone(),
                crate::integration_sync::SyncTrigger::Manual
            ),
            crate::integration_sync::SyncRequestResult::Accepted
        ));
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert_eq!(cached_event_count(&db, &connection_id), 1);
        let state = integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
            .unwrap()
            .unwrap();
        assert_eq!(state.sync_status, "succeeded");
        assert_eq!(state.last_sync_error_code, None);
    }

    #[tokio::test]
    async fn duplicate_manual_provider_requests_use_shared_runtime_single_flight() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, _) = connected_calendar(&db);
        let host = SqliteRuntimeHost::new(db);
        host.block_prepare.store(true, Ordering::SeqCst);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        assert!(matches!(
            runtime.request(
                connection_id.clone(),
                crate::integration_sync::SyncTrigger::Manual
            ),
            crate::integration_sync::SyncRequestResult::Accepted
        ));
        assert!(matches!(
            runtime.request(connection_id, crate::integration_sync::SyncTrigger::Manual),
            crate::integration_sync::SyncRequestResult::Coalesced
        ));
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            host.entered_prepare.notified(),
        )
        .await
        .expect("runtime did not enter provider preparation");
        assert_eq!(host.prepare_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn provider_runtime_failure_preserves_cached_events_and_credential() {
        let (db, _) = test_db();
        let db = Arc::new(db);
        let (connection_id, key) = connected_calendar(&db);
        cache_event(&db, &connection_id);
        let original = credentials::get(&db, &key).unwrap();
        let host = SqliteRuntimeHost::new(db.clone());
        host.fail_prepare.store(true, Ordering::SeqCst);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        assert!(matches!(
            runtime.request(
                connection_id.clone(),
                crate::integration_sync::SyncTrigger::Manual
            ),
            crate::integration_sync::SyncRequestResult::Accepted
        ));
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            host.entered_prepare.notified(),
        )
        .await
        .expect("runtime did not enter failing provider preparation");
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            host.entered_terminal.notified(),
        )
        .await
        .expect("runtime did not finish failing provider preparation");
        assert_eq!(cached_event_count(&db, &connection_id), 1);
        assert_eq!(credentials::get(&db, &key).unwrap(), original);
        let state = integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
            .unwrap()
            .unwrap();
        assert_eq!(state.sync_status, "failed");
        assert_eq!(state.last_sync_error_code.as_deref(), Some("transient"));
    }

    #[test]
    fn adversarial_parser_budget_failure_cannot_erase_cached_events() {
        let (db, _) = test_db();
        let (connection_id, _) = connected_calendar(&db);
        cache_event(&db, &connection_id);
        let feed = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:attack\r\nDTSTART:20260102T000000Z\r\nRRULE:FREQ=HOURLY;COUNT={}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            calendar_ics::MAX_FEED_OCCURRENCES + 1
        );

        assert_eq!(
            calendar_ics::normalize(
                feed.as_bytes(),
                &connection_id,
                chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            )
            .unwrap_err(),
            "occurrence_limit"
        );
        assert_eq!(cached_event_count(&db, &connection_id), 1);
        let cached_status: String = db
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT status FROM external_events WHERE connection_id=?1 AND external_id='cached'",
                [&connection_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cached_status, "active");
    }

    #[test]
    fn classification_uses_only_exact_structured_categories() {
        assert_eq!(classify(&["Exam".into()]), "exam");
        assert_eq!(classify(&["Teaching activity".into()]), "lesson");
        assert_eq!(classify(&["Institutional test".into()]), "general");
        assert_eq!(classify(&[]), "general");
    }

    #[test]
    fn metadata_extracts_only_explicit_group_lines_and_supports_multiple_groups() {
        let value = metadata(
            &["Exam".into()],
            Some(
                "Vak: Machine Learning\nGroep(en): ADSAI-ZM-1.a, ADSAI-DH-1.a; ADSAI-ZM-1.a\nLocatie: ADSAI-ZM-9.a",
            ),
        );
        assert_eq!(value.event_kind, "exam");
        assert_eq!(value.group_references, vec!["ADSAI-DH-1.a", "ADSAI-ZM-1.a"]);
        assert!(metadata(&[], Some("ADSAI-ZM-1.a in free text"))
            .group_references
            .is_empty());
    }

    #[test]
    fn provider_normalization_persists_structured_groups_without_title_or_location_inference() {
        let feed = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:grouped\r\nDTSTART:20260923T090000Z\r\nDTEND:20260923T100000Z\r\nSUMMARY:ADSAI-ZM-9.a title\r\nLOCATION:ADSAI-ZM-8.a\r\nDESCRIPTION:Groep(en): ADSAI-ZM-1.a\\, ADSAI-ZM-2.a\r\nCATEGORIES:EXAM\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events =
            calendar_ics::normalize_with_metadata(feed, "connection", chrono::Utc::now(), metadata)
                .unwrap();
        assert_eq!(events[0].event_kind, "exam");
        assert_eq!(
            events[0].group_references,
            vec!["ADSAI-ZM-1.a", "ADSAI-ZM-2.a"]
        );
    }

    #[test]
    fn title_text_is_not_classification_evidence() {
        let feed = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:title-only\r\nDTSTART:20260101T090000Z\r\nDTEND:20260101T100000Z\r\nSUMMARY:Exam preparation\r\nDESCRIPTION:Exam at 9\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events = crate::calendar_ics::normalize_with_classifier(
            feed,
            "connection",
            chrono::Utc::now(),
            classify,
        )
        .unwrap();
        assert_eq!(events[0].event_kind, "general");
    }

    #[test]
    fn replacement_persistence_keeps_old_secret_on_candidate_or_write_failure_and_exposes_no_secret(
    ) {
        let (db, fail_writes) = test_db();
        let (connection_id, key) = connected_calendar(&db);
        db.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET connection_status='disconnected' WHERE id=?1",
                [&connection_id],
            )
            .unwrap();
        cache_event(&db, &connection_id);
        let old = "https://calendar.example/old?token=old-secret";
        let new = "https://calendar.example/new?token=new-secret";

        assert!(candidate("http://calendar.example/new?token=new-secret").is_err());
        assert_eq!(credentials::get(&db, &key).unwrap().as_deref(), Some(old));
        assert_eq!(cached_event_count(&db, &connection_id), 1);
        assert_eq!(
            integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap()
                .connection_status,
            "disconnected"
        );

        fail_writes.store(true, Ordering::SeqCst);
        assert!(replace_validated_secret(&db, &connection_id, new).is_err());
        assert_eq!(credentials::get(&db, &key).unwrap().as_deref(), Some(old));
        assert_eq!(
            integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap()
                .configuration_generation,
            1
        );
        assert_eq!(cached_event_count(&db, &connection_id), 1);
        assert_eq!(
            integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap()
                .connection_status,
            "disconnected"
        );

        fail_writes.store(false, Ordering::SeqCst);
        replace_validated_secret(&db, &connection_id, new).unwrap();
        assert_eq!(credentials::get(&db, &key).unwrap().as_deref(), Some(new));
        assert_eq!(
            integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap()
                .configuration_generation,
            2
        );
        let count: i64 = db
            .conn
            .lock()
            .unwrap()
            .query_row("SELECT count(*) FROM secrets", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
        let integration = integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
            .unwrap()
            .unwrap();
        let public = serde_json::to_string(&integration).unwrap();
        assert!(!public.contains("old-secret"));
        assert!(!public.contains("new-secret"));
        assert!(!public.contains("calendar.example"));
    }

    #[test]
    fn rapid_replacements_increment_once_each_and_current_generation_survives_restart() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("aether.db");
        let fail_writes = Arc::new(AtomicBool::new(false));
        let db = Database::open(path.clone(), Box::new(TestCrypto(fail_writes.clone()))).unwrap();
        let (connection_id, key) = connected_calendar(&db);

        replace_validated_secret(&db, &connection_id, "https://calendar.example/b?token=b")
            .unwrap();
        replace_validated_secret(&db, &connection_id, "https://calendar.example/c?token=c")
            .unwrap();
        assert_eq!(
            integrations::sync_runtime_record(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap()
                .configuration_generation,
            3
        );
        drop(db);

        let reopened = Database::open(path, Box::new(TestCrypto(fail_writes))).unwrap();
        let record =
            integrations::sync_runtime_record(&reopened.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .unwrap();
        assert_eq!(record.configuration_generation, 3);
        assert_eq!(
            credentials::get(&reopened, &key).unwrap().as_deref(),
            Some("https://calendar.example/c?token=c")
        );
        assert!(record.integration.last_sync_etag.is_none());
        assert!(record.validator_origin.is_none());
    }

    #[test]
    fn disconnect_removes_secret_and_generic_cascade_removes_connection_data() {
        let (db, _) = test_db();
        let (connection_id, key) = connected_calendar(&db);
        assert!(disconnect_persisted_connection(&db, &connection_id).unwrap());
        assert_eq!(credentials::get(&db, &key).unwrap(), None);
        assert!(
            integrations::get_by_id(&db.conn.lock().unwrap(), &connection_id)
                .unwrap()
                .is_none()
        );
        let calendar_count: i64 = db
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM subscribed_calendars WHERE connection_id=?1",
                [&connection_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(calendar_count, 0);
    }
}
