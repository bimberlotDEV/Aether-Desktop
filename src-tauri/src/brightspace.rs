//! Thin, calendar-only Brightspace provider profile.
//!
//! Brightspace V1 deliberately exposes no LMS API, OAuth, course, assignment,
//! deadline, content, or material semantics. The renewable ICS URL remains a
//! native bearer credential and all ingestion uses the shared calendar pipeline.
use crate::{
    calendar_ics::IcsValidation,
    db::{repositories::integrations, Database},
    integration_sync::IntegrationSyncRuntime,
    subscribed_calendar_provider::{self, ProviderConfig},
};
use serde::Serialize;
use tauri::State;

pub const PROVIDER_ID: &str = "brightspace";
const CONFIG: ProviderConfig = ProviderConfig {
    id: PROVIDER_ID,
    name: "Brightspace",
};

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
        name: CONFIG.name,
        capabilities: subscribed_calendar_provider::CAPABILITIES.to_vec(),
        auth_type: "ics_feed",
    }
}

pub async fn validate(url: &str) -> Result<IcsValidation, String> {
    subscribed_calendar_provider::candidate(url)?;
    subscribed_calendar_provider::validate(CONFIG, url).await
}

pub async fn connect(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    url: String,
) -> Result<integrations::Integration, String> {
    subscribed_calendar_provider::connect(CONFIG, db, runtime, url).await
}

pub async fn replace(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
    url: String,
) -> Result<(), String> {
    subscribed_calendar_provider::replace(CONFIG, db, runtime, connection_id, url).await
}

pub fn disconnect(
    db: &Database,
    runtime: &IntegrationSyncRuntime,
    connection_id: &str,
) -> Result<bool, String> {
    subscribed_calendar_provider::disconnect(CONFIG, db, runtime, connection_id)
}

#[tauri::command]
pub fn brightspace_profile() -> ProviderProfile {
    profile()
}

#[tauri::command]
pub async fn brightspace_validate(url: String) -> Result<IcsValidation, String> {
    validate(&url).await
}

#[tauri::command]
pub async fn brightspace_connect(
    db: State<'_, Database>,
    runtime: State<'_, IntegrationSyncRuntime>,
    url: String,
) -> Result<integrations::Integration, String> {
    connect(&db, &runtime, url).await
}

#[tauri::command]
pub async fn brightspace_replace(
    db: State<'_, Database>,
    runtime: State<'_, IntegrationSyncRuntime>,
    connection_id: String,
    url: String,
) -> Result<(), String> {
    replace(&db, &runtime, &connection_id, url).await
}

#[tauri::command]
pub fn brightspace_disconnect(
    db: State<Database>,
    runtime: State<IntegrationSyncRuntime>,
    connection_id: String,
) -> Result<bool, String> {
    disconnect(&db, &runtime, &connection_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ai::credentials::{self, SecretCrypto},
        integration_sync::{
            PreparedSync, RuntimeHost, SyncFailure, SyncPolicy, SyncStateEvent, SyncTrigger,
        },
    };
    use async_trait::async_trait;
    use futures_util::future::BoxFuture;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use tokio_util::sync::CancellationToken;

    struct TestCrypto(Arc<AtomicBool>);

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

    struct ObservingHost {
        db: Arc<Database>,
        credentials: Mutex<Vec<Option<String>>>,
    }

    #[async_trait]
    impl RuntimeHost for ObservingHost {
        fn record(&self, id: &str) -> Result<Option<integrations::SyncRuntimeRecord>, String> {
            let record = integrations::sync_runtime_record(&self.db.conn.lock().unwrap(), id)?;
            let credential = record
                .as_ref()
                .and_then(|value| value.credential_key.as_deref())
                .map(|key| credentials::get(&self.db, key))
                .transpose()?
                .flatten();
            self.credentials.lock().unwrap().push(credential);
            Ok(record)
        }

        fn candidates(&self) -> Result<Vec<integrations::Integration>, String> {
            Ok(Vec::new())
        }

        fn recover_interrupted(&self, _next_allowed: &str) -> Result<usize, String> {
            Ok(0)
        }

        fn mark_running(
            &self,
            _id: &str,
            _trigger: SyncTrigger,
            _now: &str,
        ) -> Result<bool, String> {
            Ok(false)
        }

        async fn prepare(
            &self,
            _record: &integrations::SyncRuntimeRecord,
            _cancel: CancellationToken,
        ) -> Result<(PreparedSync, SyncPolicy), SyncFailure> {
            unreachable!()
        }

        fn commit_success(
            &self,
            _id: &str,
            _prepared: PreparedSync,
            _now: &str,
            _next_allowed: &str,
        ) -> Result<(), String> {
            unreachable!()
        }

        fn finish_failure(
            &self,
            _id: &str,
            _failure: &SyncFailure,
            _next_allowed: &str,
            _retry_after: Option<&str>,
        ) -> Result<(), String> {
            unreachable!()
        }

        fn emit(&self, _event: SyncStateEvent) {}

        fn spawn(&self, _task: BoxFuture<'static, ()>) {}
    }

    fn test_db() -> (Arc<Database>, Arc<AtomicBool>) {
        let fail_writes = Arc::new(AtomicBool::new(false));
        let db = Database::open(
            tempfile::tempdir().unwrap().path().join("aether.db"),
            Box::new(TestCrypto(fail_writes.clone())),
        )
        .unwrap();
        (Arc::new(db), fail_writes)
    }

    fn usable_validation() -> IcsValidation {
        IcsValidation {
            usable: true,
            event_count: 1,
            error_code: None,
            display_name: Some("Brightspace calendar".into()),
            covered_start: None,
            covered_end: None,
            public_host: Some("learn.example.edu".into()),
            warnings: vec![],
        }
    }

    #[test]
    fn profile_is_truthfully_calendar_only() {
        let value = profile();
        assert_eq!(value.id, "brightspace");
        assert_eq!(value.auth_type, "ics_feed");
        assert_eq!(value.capabilities, vec!["calendar_read", "manual_refresh"]);
        for unsupported in [
            "course_read",
            "assignment_read",
            "deadline_read",
            "content_metadata_read",
            "material_file_read",
        ] {
            assert!(!value.capabilities.contains(&unsupported));
        }
    }

    #[test]
    fn arbitrary_lms_like_text_remains_a_general_external_event() {
        let feed = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:ordinary\r\nDTSTART:20260923T090000Z\r\nDTEND:20260923T100000Z\r\nSUMMARY:Assignment 1 deadline\r\nDESCRIPTION:Course ABC material\r\nURL:https://learn.example.edu/course/123\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        let events =
            crate::calendar_ics::normalize(feed, "brightspace-id", chrono::Utc::now()).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_kind, "general");
        assert!(events[0].group_references.is_empty());
    }

    #[tokio::test]
    async fn connect_replace_failure_and_disconnect_preserve_the_native_secret_boundary() {
        let (db, fail_writes) = test_db();
        let host = Arc::new(ObservingHost {
            db: db.clone(),
            credentials: Mutex::new(Vec::new()),
        });
        let runtime = IntegrationSyncRuntime::new(host.clone());

        let old_url = "https://learn.example.edu/d2l/le/calendar/feed.ics?token=old";
        let integration = subscribed_calendar_provider::connect_with_validation(
            CONFIG,
            &db,
            &runtime,
            old_url.into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .unwrap();
        assert_eq!(integration.provider_id, PROVIDER_ID);
        assert_eq!(integration.connection_status, "connected");
        assert_eq!(
            integration.effective_capabilities,
            integration.advertised_capabilities
        );
        assert_eq!(
            host.credentials.lock().unwrap()[0].as_deref(),
            Some(old_url)
        );
        let cached = crate::calendar_ics::normalize(
            b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:cached\r\nDTSTART:20260923T090000Z\r\nSUMMARY:Cached\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            &integration.id,
            chrono::Utc::now(),
        )
        .unwrap();
        crate::db::repositories::external_events::reconcile(
            &mut db.conn.lock().unwrap(),
            &integration.id,
            &cached,
            crate::db::repositories::external_events::ReconciliationMode::ObservedOnly,
            None,
            "2026-09-23T00:00:00Z",
        )
        .unwrap();

        fail_writes.store(true, Ordering::SeqCst);
        assert!(subscribed_calendar_provider::replace_with_validation(
            CONFIG,
            &db,
            &runtime,
            &integration.id,
            "https://learn.example.edu/new.ics?token=new".into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .is_err());
        let key = integrations::sync_runtime_record(&db.conn.lock().unwrap(), &integration.id)
            .unwrap()
            .unwrap()
            .credential_key
            .unwrap();
        assert_eq!(
            credentials::get(&db, &key).unwrap().as_deref(),
            Some(old_url)
        );
        let cached_count: i64 = db
            .conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1",
                [&integration.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cached_count, 1);

        fail_writes.store(false, Ordering::SeqCst);
        let replacement_url = "https://learn.example.edu/new.ics?token=replacement";
        subscribed_calendar_provider::replace_with_validation(
            CONFIG,
            &db,
            &runtime,
            &integration.id,
            replacement_url.into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .unwrap();
        assert_eq!(
            host.credentials.lock().unwrap().last().unwrap().as_deref(),
            Some(replacement_url)
        );
        assert!(disconnect(&db, &runtime, &integration.id).unwrap());
        assert_eq!(credentials::get(&db, &key).unwrap(), None);
        assert!(
            integrations::get_by_id(&db.conn.lock().unwrap(), &integration.id)
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn failed_validation_or_credential_write_creates_no_connection() {
        let (db, fail_writes) = test_db();
        let host = Arc::new(ObservingHost {
            db: db.clone(),
            credentials: Mutex::new(Vec::new()),
        });
        let runtime = IntegrationSyncRuntime::new(host);

        assert!(subscribed_calendar_provider::connect_with_validation(
            CONFIG,
            &db,
            &runtime,
            "https://learn.example.edu/invalid.ics".into(),
            |_| async { Err("invalid feed".into()) },
        )
        .await
        .is_err());
        fail_writes.store(true, Ordering::SeqCst);
        assert!(subscribed_calendar_provider::connect_with_validation(
            CONFIG,
            &db,
            &runtime,
            "https://learn.example.edu/valid.ics".into(),
            |_| async { Ok(usable_validation()) },
        )
        .await
        .is_err());
        assert!(integrations::list(&db.conn.lock().unwrap())
            .unwrap()
            .is_empty());
    }
}
