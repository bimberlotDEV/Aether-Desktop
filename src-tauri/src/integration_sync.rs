//! Native-only integration synchronization. Tauri is isolated in TauriRuntimeHost.
use crate::{
    ai::credentials,
    calendar, calendar_ics,
    db::{
        repositories::{integrations, subscribed_calendars},
        Database,
    },
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use futures_util::future::BoxFuture;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    sync::Semaphore,
    time::{sleep, Duration as TokioDuration},
};
use tokio_util::sync::CancellationToken;

const PROVIDER_ICS: &str = "calendar_ics";
const PROVIDER_MY_TIMETABLE: &str = "my_timetable";
const PROVIDER_BRIGHTSPACE: &str = "brightspace";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IcsNormalization {
    Default,
    MyTimetable,
}

fn ics_handler(provider: &str, auth_type: &str) -> Option<IcsNormalization> {
    if auth_type != "ics_feed" {
        return None;
    }
    match provider {
        PROVIDER_ICS | PROVIDER_BRIGHTSPACE => Some(IcsNormalization::Default),
        PROVIDER_MY_TIMETABLE => Some(IcsNormalization::MyTimetable),
        _ => None,
    }
}
const PERIODIC_SECONDS: u64 = 15 * 60;
const TIMEOUT_SECONDS: u64 = 30;
const SUCCESS_INTERVAL_MINUTES: i64 = 30;
const SHUTDOWN_DRAIN_STEPS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyncTrigger {
    Periodic,
    Startup,
    Resume,
    Manual,
}
impl SyncTrigger {
    fn as_str(self) -> &'static str {
        match self {
            Self::Periodic => "periodic",
            Self::Startup => "app_start",
            Self::Resume => "app_resume",
            Self::Manual => "manual",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SyncRequestResult {
    Accepted,
    Coalesced,
    Deferred { eligible_at: String },
    Rejected { reason: String },
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub running_count: usize,
    pub queued_count: usize,
    pub shutting_down: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStateEvent {
    pub connection_id: String,
    pub state: &'static str,
}
#[derive(Debug, Clone)]
pub struct SyncPolicy {
    pub success_interval_minutes: i64,
}
#[derive(Debug)]
pub enum PreparedSync {
    Ics {
        events: Vec<calendar::ExternalEventInput>,
        etag: Option<String>,
        last_modified: Option<String>,
        validator_origin: String,
        window_start: String,
        window_end: String,
    },
    NotModified,
    RateLimited {
        eligible_at: Option<String>,
    },
}
#[derive(Debug)]
pub struct SyncFailure {
    pub code: &'static str,
    pub message: &'static str,
    pub connection_status: &'static str,
}

/// The complete runtime-facing platform boundary. It carries repository work,
/// child spawning and only sanitized state events; it does not expose windows.
#[async_trait]
pub trait RuntimeHost: Send + Sync {
    fn record(&self, id: &str) -> Result<Option<integrations::SyncRuntimeRecord>, String>;
    fn candidates(&self) -> Result<Vec<integrations::Integration>, String>;
    fn recover_interrupted(&self, next_allowed: &str) -> Result<usize, String>;
    fn mark_running(&self, id: &str, trigger: SyncTrigger, now: &str) -> Result<bool, String>;
    async fn prepare(
        &self,
        record: &integrations::SyncRuntimeRecord,
        cancel: CancellationToken,
    ) -> Result<(PreparedSync, SyncPolicy), SyncFailure>;
    fn commit_success(
        &self,
        id: &str,
        prepared: PreparedSync,
        now: &str,
        next_allowed: &str,
    ) -> Result<(), String>;
    fn finish_failure(
        &self,
        id: &str,
        failure: &SyncFailure,
        next_allowed: &str,
        retry_after: Option<&str>,
    ) -> Result<(), String>;
    fn emit(&self, event: SyncStateEvent);
    fn spawn(&self, task: BoxFuture<'static, ()>);
}

/// Production outer adapter. This is the only implementation coupled to Tauri.
pub struct TauriRuntimeHost {
    app: AppHandle,
}
impl TauriRuntimeHost {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
    fn db(&self) -> tauri::State<'_, Database> {
        self.app.state()
    }
}
#[async_trait]
impl RuntimeHost for TauriRuntimeHost {
    fn record(&self, id: &str) -> Result<Option<integrations::SyncRuntimeRecord>, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::sync_runtime_record(&conn, id)
    }
    fn candidates(&self) -> Result<Vec<integrations::Integration>, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::list_sync_candidates(&conn)
    }
    fn recover_interrupted(&self, next: &str) -> Result<usize, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::recover_interrupted_runs(&conn, next)
    }
    fn mark_running(&self, id: &str, trigger: SyncTrigger, now: &str) -> Result<bool, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::runtime_mark_running(&conn, id, trigger.as_str(), now)
    }
    async fn prepare(
        &self,
        record: &integrations::SyncRuntimeRecord,
        cancel: CancellationToken,
    ) -> Result<(PreparedSync, SyncPolicy), SyncFailure> {
        let normalization = ics_handler(
            &record.integration.provider_id,
            &record.integration.auth_type,
        )
        .ok_or(SyncFailure {
            code: "unsupported",
            message: "Provider is unsupported",
            connection_status: "unsupported",
        })?;
        let key = record.credential_key.as_deref().ok_or(SyncFailure {
            code: "configuration",
            message: "Calendar subscription is not configured",
            connection_status: "institution_configuration_required",
        })?;
        let db = self.db();
        {
            let conn = db.conn.lock().map_err(|_| SyncFailure {
                code: "internal",
                message: "Local sync state is unavailable",
                connection_status: "degraded",
            })?;
            subscribed_calendars::get_by_connection(&conn, &record.integration.id)
                .map_err(|_| SyncFailure {
                    code: "configuration",
                    message: "Calendar subscription is not configured",
                    connection_status: "institution_configuration_required",
                })?
                .ok_or(SyncFailure {
                    code: "configuration",
                    message: "Calendar subscription is not configured",
                    connection_status: "institution_configuration_required",
                })?;
        }
        let url = credentials::get(&db, key)
            .map_err(|_| SyncFailure {
                code: "configuration",
                message: "Calendar subscription is not configured",
                connection_status: "institution_configuration_required",
            })?
            .ok_or(SyncFailure {
                code: "configuration",
                message: "Calendar subscription is not configured",
                connection_status: "institution_configuration_required",
            })?;
        let validators =
            record
                .validator_origin
                .as_deref()
                .map(|origin| calendar_ics::ConditionalValidators {
                    origin,
                    etag: record.integration.last_sync_etag.as_deref(),
                    last_modified: record.integration.last_sync_last_modified.as_deref(),
                });
        let fetched = tokio::select! {
            _ = cancel.cancelled() => return Err(SyncFailure { code: "cancelled", message: "Sync was cancelled", connection_status: "connected" }),
            fetched = calendar_ics::fetch(&url, validators) => fetched,
        }.map_err(|_| SyncFailure { code: "transient", message: "Calendar feed could not be reached", connection_status: "degraded" })?;
        if cancel.is_cancelled() {
            return Err(SyncFailure {
                code: "cancelled",
                message: "Sync was cancelled",
                connection_status: "connected",
            });
        }
        let prepared = match fetched {
            calendar_ics::FetchResult::NotModified => PreparedSync::NotModified,
            calendar_ics::FetchResult::RateLimited {
                retry_after_at,
                rate_limit_reset_at,
            } => PreparedSync::RateLimited {
                eligible_at: latest_allowed(retry_after_at, rate_limit_reset_at),
            },
            calendar_ics::FetchResult::Complete {
                bytes,
                etag,
                last_modified,
                validator_origin,
                retry_after_at,
                rate_limit_reset_at,
            } => {
                // Successful payloads do not impose a failure gate, but consume only
                // the safe timing metadata here so the fetch contract remains closed.
                let _safe_provider_gate = latest_allowed(retry_after_at, rate_limit_reset_at);
                let now = Utc::now();
                let events = match normalization {
                    IcsNormalization::MyTimetable => calendar_ics::normalize_with_metadata(
                        &bytes,
                        &record.integration.id,
                        now,
                        crate::my_timetable::metadata,
                    ),
                    IcsNormalization::Default => {
                        calendar_ics::normalize(&bytes, &record.integration.id, now)
                    }
                }
                .map_err(|_| SyncFailure {
                    code: "provider_data",
                    message: "Calendar feed data could not be safely processed",
                    connection_status: "degraded",
                })?;
                PreparedSync::Ics {
                    events,
                    etag,
                    last_modified,
                    validator_origin,
                    window_start: (now - Duration::days(366))
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    window_end: (now + Duration::days(366))
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                }
            }
        };
        Ok((
            prepared,
            SyncPolicy {
                success_interval_minutes: SUCCESS_INTERVAL_MINUTES,
            },
        ))
    }
    fn commit_success(
        &self,
        id: &str,
        prepared: PreparedSync,
        now: &str,
        next: &str,
    ) -> Result<(), String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        match prepared {
            PreparedSync::Ics {
                events,
                etag,
                last_modified,
                validator_origin,
                window_start,
                window_end,
            } => calendar::reconcile_in_transaction(
                &tx,
                id,
                &events,
                calendar::ReconciliationMode::Authoritative,
                Some((&window_start, &window_end)),
                now,
            )
            .and_then(|_| {
                integrations::runtime_finish_success(
                    &tx,
                    id,
                    now,
                    Some(integrations::SyncValidators {
                        origin: &validator_origin,
                        etag: etag.as_deref(),
                        last_modified: last_modified.as_deref(),
                    }),
                    next,
                )
            })?,
            PreparedSync::NotModified => {
                integrations::runtime_finish_success(&tx, id, now, None, next)?
            }
            PreparedSync::RateLimited { .. } => {
                return Err("Rate-limited result cannot commit success".to_string())
            }
        }
        tx.commit().map_err(|error| error.to_string())
    }
    fn finish_failure(
        &self,
        id: &str,
        failure: &SyncFailure,
        next: &str,
        retry: Option<&str>,
    ) -> Result<(), String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::runtime_finish_failure(
            &conn,
            id,
            failure.code,
            failure.message,
            next,
            failure.connection_status,
            retry,
        )
    }
    fn emit(&self, event: SyncStateEvent) {
        let _ = self.app.emit("integration-sync-state-changed", event);
    }
    fn spawn(&self, task: BoxFuture<'static, ()>) {
        tauri::async_runtime::spawn(task);
    }
}

#[derive(Default)]
struct State {
    active: HashSet<String>,
    provider_active: HashMap<String, usize>,
    tokens: HashMap<String, CancellationToken>,
    shutting_down: bool,
}
struct Inner {
    state: Mutex<State>,
    global: Arc<Semaphore>,
    root: CancellationToken,
}
pub struct IntegrationSyncRuntime {
    inner: Arc<Inner>,
    host: Arc<dyn RuntimeHost>,
}
impl IntegrationSyncRuntime {
    pub fn new(host: Arc<dyn RuntimeHost>) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State::default()),
                global: Arc::new(Semaphore::new(3)),
                root: CancellationToken::new(),
            }),
            host,
        }
    }
    pub fn status(&self) -> RuntimeStatus {
        let state = self.inner.state.lock().expect("sync state poisoned");
        RuntimeStatus {
            running_count: state.active.len(),
            queued_count: 0,
            shutting_down: state.shutting_down,
        }
    }
    pub fn request(&self, id: String, trigger: SyncTrigger) -> SyncRequestResult {
        request(self.inner.clone(), self.host.clone(), id, trigger)
    }
    pub fn start(&self) {
        let recovery =
            (Utc::now() + Duration::seconds(30)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let _ = self.host.recover_interrupted(&recovery);
        schedule(self.inner.clone(), self.host.clone(), SyncTrigger::Startup);
        let inner = self.inner.clone();
        let host = self.host.clone();
        self.host.spawn(Box::pin(async move {
            loop {
                sleep(TokioDuration::from_secs(PERIODIC_SECONDS)).await;
                if shutting_down(&inner) {
                    break;
                }
                schedule(inner.clone(), host.clone(), SyncTrigger::Periodic);
            }
        }));
    }
    pub fn on_resume(&self) {
        schedule(self.inner.clone(), self.host.clone(), SyncTrigger::Resume);
    }
    pub fn cancel_connection(&self, id: &str) {
        if let Some(token) = self
            .inner
            .state
            .lock()
            .expect("sync state poisoned")
            .tokens
            .get(id)
        {
            token.cancel();
        }
    }
    pub async fn shutdown(&self) {
        shutdown(self.inner.clone()).await;
    }
}

pub struct RuntimeLifecycle<'a> {
    runtime: &'a IntegrationSyncRuntime,
}
impl<'a> RuntimeLifecycle<'a> {
    pub fn new(runtime: &'a IntegrationSyncRuntime) -> Self {
        Self { runtime }
    }
    pub async fn on_exit_requested(&self) {
        self.runtime.shutdown().await;
    }
    pub fn on_close_requested(&self) {}
}

fn request(
    inner: Arc<Inner>,
    host: Arc<dyn RuntimeHost>,
    id: String,
    trigger: SyncTrigger,
) -> SyncRequestResult {
    if shutting_down(&inner) {
        return SyncRequestResult::Rejected {
            reason: "Sync runtime is shutting down".into(),
        };
    }
    let record = match host.record(&id) {
        Ok(Some(record)) => record,
        Ok(None) => {
            return SyncRequestResult::Rejected {
                reason: "Connection was not found".into(),
            }
        }
        Err(_) => {
            return SyncRequestResult::Rejected {
                reason: "Local sync state is unavailable".into(),
            }
        }
    };
    if !record.integration.enabled {
        return SyncRequestResult::Rejected {
            reason: "Connection is disabled".into(),
        };
    }
    if ics_handler(
        &record.integration.provider_id,
        &record.integration.auth_type,
    )
    .is_none()
    {
        return SyncRequestResult::Rejected {
            reason: "Provider is unsupported".into(),
        };
    }
    if let Some(eligible) = record
        .integration
        .next_allowed_sync_at
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .filter(|value| *value > Utc::now())
    {
        return SyncRequestResult::Deferred {
            eligible_at: eligible.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };
    }
    {
        let mut state = inner.state.lock().expect("sync state poisoned");
        if state.shutting_down {
            return SyncRequestResult::Rejected {
                reason: "Sync runtime is shutting down".into(),
            };
        }
        if !state.active.insert(id.clone()) {
            return SyncRequestResult::Coalesced;
        }
        if state
            .provider_active
            .get(&record.integration.provider_id)
            .copied()
            .unwrap_or(0)
            >= 1
        {
            state.active.remove(&id);
            return SyncRequestResult::Coalesced;
        }
        *state
            .provider_active
            .entry(record.integration.provider_id.clone())
            .or_default() += 1;
    }
    let token = inner.root.child_token();
    inner
        .state
        .lock()
        .expect("sync state poisoned")
        .tokens
        .insert(id.clone(), token.clone());
    let provider = record.integration.provider_id.clone();
    let semaphore = inner.global.clone();
    let task_host = host.clone();
    host.spawn(Box::pin(async move {
        if let Ok(_permit) = semaphore.acquire_owned().await {
            run_one(inner.clone(), task_host, id.clone(), trigger, token).await;
        }
        cleanup(&inner, &id, &provider);
    }));
    SyncRequestResult::Accepted
}
fn schedule(inner: Arc<Inner>, host: Arc<dyn RuntimeHost>, trigger: SyncTrigger) {
    if let Ok(items) = host.candidates() {
        for item in items.into_iter().filter(|item| {
            item.enabled
                && item
                    .sync_modes
                    .iter()
                    .any(|mode| matches!(mode.as_str(), "app_start" | "app_resume" | "periodic"))
        }) {
            let _ = request(inner.clone(), host.clone(), item.id, trigger);
        }
    }
}
fn shutting_down(inner: &Inner) -> bool {
    inner
        .state
        .lock()
        .expect("sync state poisoned")
        .shutting_down
}
fn cleanup(inner: &Inner, id: &str, provider: &str) {
    let mut state = inner.state.lock().expect("sync state poisoned");
    state.active.remove(id);
    state.tokens.remove(id);
    if let Some(count) = state.provider_active.get_mut(provider) {
        *count = count.saturating_sub(1);
    }
}
async fn shutdown(inner: Arc<Inner>) {
    {
        let mut state = inner.state.lock().expect("sync state poisoned");
        state.shutting_down = true;
        for token in state.tokens.values() {
            token.cancel();
        }
    }
    inner.root.cancel();
    for _ in 0..SHUTDOWN_DRAIN_STEPS {
        if inner
            .state
            .lock()
            .expect("sync state poisoned")
            .active
            .is_empty()
        {
            break;
        }
        sleep(TokioDuration::from_millis(100)).await;
    }
}
async fn run_one(
    inner: Arc<Inner>,
    host: Arc<dyn RuntimeHost>,
    id: String,
    trigger: SyncTrigger,
    cancel: CancellationToken,
) {
    let record = match host.record(&id) {
        Ok(Some(record)) => record,
        _ => return,
    };
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    if host.mark_running(&id, trigger, &now).ok() != Some(true) {
        return;
    }
    host.emit(SyncStateEvent {
        connection_id: id.clone(),
        state: "active",
    });
    let prepared = tokio::select! {
        _ = sleep(TokioDuration::from_secs(TIMEOUT_SECONDS)) => Err(SyncFailure {
            code: "transient",
            message: "Sync timed out",
            connection_status: "degraded",
        }),
        prepared = host.prepare(&record, cancel.clone()) => prepared,
    };
    match prepared {
        Ok((PreparedSync::RateLimited { eligible_at }, _)) => {
            let fallback = backoff(record.consecutive_failures);
            let next = eligible_at.as_deref().unwrap_or(&fallback);
            let failure = SyncFailure {
                code: "rate_limited",
                message: "Calendar provider temporarily limited sync requests",
                connection_status: "rate_limited",
            };
            let _ = host.finish_failure(&id, &failure, next, eligible_at.as_deref());
            host.emit(SyncStateEvent {
                connection_id: id,
                state: "terminal",
            });
        }
        Ok((prepared, policy)) => {
            if cancel.is_cancelled() || shutting_down(&inner) {
                return;
            }
            let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let next = (Utc::now() + Duration::minutes(policy.success_interval_minutes))
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            if !cancel.is_cancelled()
                && !shutting_down(&inner)
                && host.commit_success(&id, prepared, &now, &next).is_ok()
            {
                host.emit(SyncStateEvent {
                    connection_id: id,
                    state: "terminal",
                });
            }
        }
        Err(failure) => {
            if !shutting_down(&inner) {
                let next = backoff(record.consecutive_failures);
                let _ = host.finish_failure(&id, &failure, &next, None);
                host.emit(SyncStateEvent {
                    connection_id: id,
                    state: "terminal",
                });
            }
        }
    }
}
fn backoff(prior: u32) -> String {
    (Utc::now() + Duration::seconds((30_i64.saturating_mul(2_i64.pow(prior.min(8)))).min(3600)))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
fn latest_allowed(first: Option<String>, second: Option<String>) -> Option<String> {
    match (first, second) {
        (Some(first), Some(second)) => Some(first.max(second)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use rusqlite::Connection;
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        time::Instant,
    };
    use tokio::sync::Notify;

    #[test]
    fn closed_ics_registry_requires_an_approved_provider_and_auth_pair() {
        assert_eq!(
            ics_handler(PROVIDER_BRIGHTSPACE, "ics_feed"),
            Some(IcsNormalization::Default)
        );
        assert_eq!(
            ics_handler(PROVIDER_MY_TIMETABLE, "ics_feed"),
            Some(IcsNormalization::MyTimetable)
        );
        assert_eq!(ics_handler(PROVIDER_BRIGHTSPACE, "oauth"), None);
        assert_eq!(ics_handler("unregistered", "ics_feed"), None);
    }

    #[derive(Clone, Copy)]
    enum HandlerMode {
        Success,
        Blocking,
        RateLimited,
    }

    struct FakeRuntimeHost {
        conn: Mutex<Connection>,
        mode: Mutex<HandlerMode>,
        started: Arc<Notify>,
        events: Mutex<Vec<SyncStateEvent>>,
        domain_commits: Mutex<usize>,
        success_commits: Mutex<usize>,
        cancelled: AtomicBool,
    }

    impl FakeRuntimeHost {
        fn new(mode: HandlerMode) -> Arc<Self> {
            let conn = Connection::open_in_memory().unwrap();
            migrations::run(&conn).unwrap();
            Arc::new(Self {
                conn: Mutex::new(conn),
                mode: Mutex::new(mode),
                started: Arc::new(Notify::new()),
                events: Mutex::new(Vec::new()),
                domain_commits: Mutex::new(0),
                success_commits: Mutex::new(0),
                cancelled: AtomicBool::new(false),
            })
        }

        fn integration(&self, next_allowed: Option<&str>) -> String {
            let conn = self.conn.lock().unwrap();
            let id = integrations::create(
                &conn,
                &integrations::IntegrationCreateInput {
                    provider_id: PROVIDER_ICS.into(),
                    enabled: true,
                    advertised_capabilities: Vec::new(),
                    auth_type: "ics_feed".into(),
                    sync_modes: vec!["manual".into()],
                    sync_config: serde_json::json!({}),
                },
            )
            .unwrap()
            .id;
            conn.execute(
                "UPDATE integrations SET connection_status='connected' WHERE id=?1",
                [&id],
            )
            .unwrap();
            if let Some(next_allowed) = next_allowed {
                conn.execute(
                    "UPDATE integrations SET next_allowed_sync_at=?1 WHERE id=?2",
                    [next_allowed, &id],
                )
                .unwrap();
            }
            id
        }

        fn record_state(&self, id: &str) -> integrations::Integration {
            integrations::get_by_id(&self.conn.lock().unwrap(), id)
                .unwrap()
                .unwrap()
        }
    }

    #[async_trait]
    impl RuntimeHost for FakeRuntimeHost {
        fn record(&self, id: &str) -> Result<Option<integrations::SyncRuntimeRecord>, String> {
            integrations::sync_runtime_record(&self.conn.lock().unwrap(), id)
        }
        fn candidates(&self) -> Result<Vec<integrations::Integration>, String> {
            integrations::list_sync_candidates(&self.conn.lock().unwrap())
        }
        fn recover_interrupted(&self, next_allowed: &str) -> Result<usize, String> {
            integrations::recover_interrupted_runs(&self.conn.lock().unwrap(), next_allowed)
        }
        fn mark_running(&self, id: &str, trigger: SyncTrigger, now: &str) -> Result<bool, String> {
            integrations::runtime_mark_running(
                &self.conn.lock().unwrap(),
                id,
                trigger.as_str(),
                now,
            )
        }
        async fn prepare(
            &self,
            _record: &integrations::SyncRuntimeRecord,
            cancel: CancellationToken,
        ) -> Result<(PreparedSync, SyncPolicy), SyncFailure> {
            self.started.notify_waiters();
            let mode = *self.mode.lock().unwrap();
            match mode {
                HandlerMode::Success => Ok((
                    PreparedSync::NotModified,
                    SyncPolicy {
                        success_interval_minutes: 30,
                    },
                )),
                HandlerMode::Blocking => {
                    cancel.cancelled().await;
                    self.cancelled.store(true, Ordering::SeqCst);
                    Err(SyncFailure {
                        code: "cancelled",
                        message: "cancelled",
                        connection_status: "connected",
                    })
                }
                HandlerMode::RateLimited => Ok((
                    PreparedSync::RateLimited {
                        eligible_at: latest_allowed(
                            Some("2999-01-01T00:00:30Z".into()),
                            Some("2999-01-01T00:01:00Z".into()),
                        ),
                    },
                    SyncPolicy {
                        success_interval_minutes: 30,
                    },
                )),
            }
        }
        fn commit_success(
            &self,
            id: &str,
            prepared: PreparedSync,
            now: &str,
            next_allowed: &str,
        ) -> Result<(), String> {
            *self.domain_commits.lock().unwrap() += 1;
            let conn = self.conn.lock().unwrap();
            let tx = conn
                .unchecked_transaction()
                .map_err(|error| error.to_string())?;
            match prepared {
                PreparedSync::NotModified => {
                    integrations::runtime_finish_success(&tx, id, now, None, next_allowed)?
                }
                _ => return Err("Fake handler only commits no-change results".into()),
            }
            tx.commit().map_err(|error| error.to_string())?;
            *self.success_commits.lock().unwrap() += 1;
            Ok(())
        }
        fn finish_failure(
            &self,
            id: &str,
            failure: &SyncFailure,
            next_allowed: &str,
            retry_after: Option<&str>,
        ) -> Result<(), String> {
            integrations::runtime_finish_failure(
                &self.conn.lock().unwrap(),
                id,
                failure.code,
                failure.message,
                next_allowed,
                failure.connection_status,
                retry_after,
            )
        }
        fn emit(&self, event: SyncStateEvent) {
            self.events.lock().unwrap().push(event);
        }
        fn spawn(&self, task: BoxFuture<'static, ()>) {
            tokio::spawn(task);
        }
    }

    #[test]
    fn maximum_rate_limit_gate_wins() {
        assert_eq!(
            latest_allowed(
                Some("2026-01-01T00:00:30Z".into()),
                Some("2026-01-01T00:01:00Z".into())
            )
            .as_deref(),
            Some("2026-01-01T00:01:00Z")
        );
    }

    #[tokio::test]
    async fn manual_requests_accept_coalesce_defer_and_reject() {
        let host = FakeRuntimeHost::new(HandlerMode::Blocking);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let id = host.integration(None);
        assert!(matches!(
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        assert!(matches!(
            runtime.request(id, SyncTrigger::Manual),
            SyncRequestResult::Coalesced
        ));
        assert!(matches!(
            runtime.request(
                host.integration(Some("2999-01-01T00:00:00Z")),
                SyncTrigger::Manual
            ),
            SyncRequestResult::Deferred { .. }
        ));
        assert!(matches!(
            runtime.request("does-not-exist".into(), SyncTrigger::Manual),
            SyncRequestResult::Rejected { .. }
        ));
        runtime.shutdown().await;
    }

    #[tokio::test]
    async fn shutdown_cancels_drains_rejects_and_prevents_late_commits() {
        let host = FakeRuntimeHost::new(HandlerMode::Blocking);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let id = host.integration(None);
        assert!(matches!(
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        sleep(TokioDuration::from_millis(25)).await;
        RuntimeLifecycle::new(&runtime).on_close_requested();
        assert!(!runtime.status().shutting_down);
        let started_at = Instant::now();
        RuntimeLifecycle::new(&runtime).on_exit_requested().await;
        assert!(started_at.elapsed() < std::time::Duration::from_secs(3));
        assert!(runtime.status().shutting_down);
        assert_eq!(runtime.status().running_count, 0);
        assert!(host.cancelled.load(Ordering::SeqCst));
        assert!(matches!(
            runtime.request(id, SyncTrigger::Manual),
            SyncRequestResult::Rejected { .. }
        ));
        assert_eq!(*host.domain_commits.lock().unwrap(), 0);
        assert_eq!(*host.success_commits.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn rate_limited_result_persists_the_maximum_gate_and_defers_manual_work() {
        let host = FakeRuntimeHost::new(HandlerMode::RateLimited);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let id = host.integration(None);
        assert!(matches!(
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        for _ in 0..50 {
            sleep(TokioDuration::from_millis(10)).await;
            if runtime.status().running_count == 0 {
                break;
            }
        }
        let state = host.record_state(&id);
        assert_eq!(
            state.next_allowed_sync_at.as_deref(),
            Some("2999-01-01T00:01:00Z")
        );
        assert_eq!(
            state.retry_after_at.as_deref(),
            Some("2999-01-01T00:01:00Z")
        );
        assert!(matches!(
            runtime.request(id, SyncTrigger::Manual),
            SyncRequestResult::Deferred { .. }
        ));
    }

    #[tokio::test]
    async fn success_clears_stale_retry_state_and_events_are_sanitized() {
        let host = FakeRuntimeHost::new(HandlerMode::Success);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let id = host.integration(None);
        host.conn.lock().unwrap().execute(
            "UPDATE integrations SET retry_after_at='2999-01-01T00:00:00Z', next_allowed_sync_at='2999-01-01T00:00:00Z', rate_limit_remaining=0 WHERE id=?1",
            [&id],
        ).unwrap();
        assert!(matches!(
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Deferred { .. }
        ));
        host.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET next_allowed_sync_at=NULL WHERE id=?1",
                [&id],
            )
            .unwrap();
        assert!(matches!(
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        for _ in 0..50 {
            sleep(TokioDuration::from_millis(10)).await;
            if runtime.status().running_count == 0 {
                break;
            }
        }
        let state = host.record_state(&id);
        assert!(state.retry_after_at.is_none());
        assert!(state.rate_limit_remaining.is_none());
        let serialized = serde_json::to_string(&*host.events.lock().unwrap()).unwrap();
        assert!(!serialized.contains("https://"));
        assert!(!serialized.contains("credential"));
        assert!(!serialized.contains("payload"));
    }
}
