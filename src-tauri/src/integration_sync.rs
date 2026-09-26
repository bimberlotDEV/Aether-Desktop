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
    collections::{HashMap, HashSet, VecDeque},
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
        PROVIDER_ICS => Some(IcsNormalization::Default),
        PROVIDER_MY_TIMETABLE => Some(IcsNormalization::MyTimetable),
        _ => None,
    }
}

pub(crate) fn supports(provider: &str, auth_type: &str) -> bool {
    ics_handler(provider, auth_type).is_some()
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
    Queued,
    Coalesced,
    Deferred { eligible_at: String },
    Rejected { reason: String },
}
#[cfg(test)]
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
pub struct PreparedSync {
    pub connection_id: String,
    pub configuration_generation: i64,
    pub outcome: PreparedSyncOutcome,
}

#[derive(Debug)]
pub enum PreparedSyncOutcome {
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
    fn mark_running(
        &self,
        id: &str,
        configuration_generation: i64,
        trigger: SyncTrigger,
        now: &str,
    ) -> Result<bool, String>;
    async fn prepare(
        &self,
        record: &integrations::SyncRuntimeRecord,
        cancel: CancellationToken,
    ) -> Result<(PreparedSync, SyncPolicy), SyncFailure>;
    fn commit_success(
        &self,
        prepared: PreparedSync,
        now: &str,
        next_allowed: &str,
    ) -> Result<integrations::SyncCompletion, String>;
    fn finish_failure(
        &self,
        id: &str,
        configuration_generation: i64,
        failure: &SyncFailure,
        next_allowed: &str,
        retry_after: Option<&str>,
    ) -> Result<integrations::SyncCompletion, String>;
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
    fn mark_running(
        &self,
        id: &str,
        configuration_generation: i64,
        trigger: SyncTrigger,
        now: &str,
    ) -> Result<bool, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::runtime_mark_running(
            &conn,
            id,
            configuration_generation,
            trigger.as_str(),
            now,
        )
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
        let url = {
            let conn = db.conn.lock().map_err(|_| SyncFailure {
                code: "internal",
                message: "Local sync state is unavailable",
                connection_status: "degraded",
            })?;
            if !integrations::generation_is_current(
                &conn,
                &record.integration.id,
                record.configuration_generation,
            )
            .map_err(|_| SyncFailure {
                code: "internal",
                message: "Local sync state is unavailable",
                connection_status: "degraded",
            })? {
                return Err(SyncFailure {
                    code: "stale_generation",
                    message: "Sync configuration changed",
                    connection_status: "connected",
                });
            }
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
            credentials::get_from_connection(&conn, db.crypto.as_ref(), key)
                .map_err(|_| SyncFailure {
                    code: "configuration",
                    message: "Calendar subscription is not configured",
                    connection_status: "institution_configuration_required",
                })?
                .ok_or(SyncFailure {
                    code: "configuration",
                    message: "Calendar subscription is not configured",
                    connection_status: "institution_configuration_required",
                })?
        };
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
        let outcome = match fetched {
            calendar_ics::FetchResult::NotModified => PreparedSyncOutcome::NotModified,
            calendar_ics::FetchResult::RateLimited {
                retry_after_at,
                rate_limit_reset_at,
            } => PreparedSyncOutcome::RateLimited {
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
                PreparedSyncOutcome::Ics {
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
            PreparedSync {
                connection_id: record.integration.id.clone(),
                configuration_generation: record.configuration_generation,
                outcome,
            },
            SyncPolicy {
                success_interval_minutes: SUCCESS_INTERVAL_MINUTES,
            },
        ))
    }
    fn commit_success(
        &self,
        prepared: PreparedSync,
        now: &str,
        next: &str,
    ) -> Result<integrations::SyncCompletion, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        commit_prepared(&conn, prepared, now, next)
    }
    fn finish_failure(
        &self,
        id: &str,
        configuration_generation: i64,
        failure: &SyncFailure,
        next: &str,
        retry: Option<&str>,
    ) -> Result<integrations::SyncCompletion, String> {
        let db = self.db();
        let conn = db
            .conn
            .lock()
            .map_err(|_| "Local sync state is unavailable".to_string())?;
        integrations::runtime_finish_failure(
            &conn,
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
        let _ = self.app.emit("integration-sync-state-changed", event);
    }
    fn spawn(&self, task: BoxFuture<'static, ()>) {
        tauri::async_runtime::spawn(task);
    }
}

#[derive(Default)]
struct State {
    active: HashSet<String>,
    queued: HashSet<String>,
    provider_active: HashSet<String>,
    provider_queues: HashMap<String, VecDeque<QueuedRequest>>,
    tokens: HashMap<String, CancellationToken>,
    replacement_followups: HashSet<String>,
    shutting_down: bool,
}

#[derive(Clone)]
struct QueuedRequest {
    id: String,
    provider: String,
    configuration_generation: i64,
    trigger: SyncTrigger,
}

fn commit_prepared(
    conn: &rusqlite::Connection,
    prepared: PreparedSync,
    now: &str,
    next: &str,
) -> Result<integrations::SyncCompletion, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    if !integrations::generation_is_current(
        &tx,
        &prepared.connection_id,
        prepared.configuration_generation,
    )? {
        return Ok(integrations::SyncCompletion::StaleGeneration);
    }
    let completion = match prepared.outcome {
        PreparedSyncOutcome::Ics {
            events,
            etag,
            last_modified,
            validator_origin,
            window_start,
            window_end,
        } => calendar::reconcile_in_transaction(
            &tx,
            &prepared.connection_id,
            &events,
            calendar::ReconciliationMode::Authoritative,
            Some((&window_start, &window_end)),
            now,
        )
        .and_then(|_| {
            integrations::runtime_finish_success(
                &tx,
                &prepared.connection_id,
                prepared.configuration_generation,
                now,
                Some(integrations::SyncValidators {
                    origin: &validator_origin,
                    etag: etag.as_deref(),
                    last_modified: last_modified.as_deref(),
                }),
                next,
            )
        })?,
        PreparedSyncOutcome::NotModified => integrations::runtime_finish_success(
            &tx,
            &prepared.connection_id,
            prepared.configuration_generation,
            now,
            None,
            next,
        )?,
        PreparedSyncOutcome::RateLimited { .. } => {
            return Err("Rate-limited result cannot commit success".to_string())
        }
    };
    if completion == integrations::SyncCompletion::Applied {
        tx.commit().map_err(|error| error.to_string())?;
    }
    Ok(completion)
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
    #[cfg(test)]
    pub fn status(&self) -> RuntimeStatus {
        let state = self.inner.state.lock().expect("sync state poisoned");
        RuntimeStatus {
            running_count: state.active.len(),
            queued_count: state.queued.len(),
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
        let mut state = self.inner.state.lock().expect("sync state poisoned");
        if let Some(token) = state.tokens.get(id) {
            token.cancel();
        }
        remove_queued(&mut state, id);
        state.replacement_followups.remove(id);
    }
    pub fn request_replacement_sync(&self, id: String) {
        let request_now = {
            let mut state = self.inner.state.lock().expect("sync state poisoned");
            if let Some(token) = state.tokens.get(&id) {
                token.cancel();
            }
            if state.active.contains(&id) {
                state.replacement_followups.insert(id.clone());
                false
            } else {
                remove_queued(&mut state, &id);
                true
            }
        };
        if request_now {
            let _ = self.request(id, SyncTrigger::Manual);
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
    {
        let state = inner.state.lock().expect("sync state poisoned");
        if state.shutting_down {
            return SyncRequestResult::Rejected {
                reason: "Sync runtime is shutting down".into(),
            };
        }
        if state.active.contains(&id) || state.queued.contains(&id) {
            return SyncRequestResult::Coalesced;
        }
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
    if let Err(result) = validate_request(&record, None, trigger) {
        return result;
    }
    let queued = QueuedRequest {
        id: id.clone(),
        provider: record.integration.provider_id.clone(),
        configuration_generation: record.configuration_generation,
        trigger,
    };
    let token = inner.root.child_token();
    {
        let mut state = inner.state.lock().expect("sync state poisoned");
        if state.shutting_down {
            return SyncRequestResult::Rejected {
                reason: "Sync runtime is shutting down".into(),
            };
        }
        if state.active.contains(&id) || state.queued.contains(&id) {
            return SyncRequestResult::Coalesced;
        }
        if state.provider_active.contains(&queued.provider) {
            state.queued.insert(id);
            state
                .provider_queues
                .entry(queued.provider.clone())
                .or_default()
                .push_back(queued);
            return SyncRequestResult::Queued;
        }
        state.active.insert(id.clone());
        state.provider_active.insert(queued.provider.clone());
        state.tokens.insert(id.clone(), token.clone());
    }
    spawn_reserved(inner, host, queued, record, token);
    SyncRequestResult::Accepted
}

fn validate_request(
    record: &integrations::SyncRuntimeRecord,
    expected_generation: Option<i64>,
    trigger: SyncTrigger,
) -> Result<(), SyncRequestResult> {
    if expected_generation.is_some_and(|expected| expected != record.configuration_generation) {
        return Err(SyncRequestResult::Rejected {
            reason: "Sync configuration changed".into(),
        });
    }
    if !record.integration.enabled {
        return Err(SyncRequestResult::Rejected {
            reason: "Connection is disabled".into(),
        });
    }
    if !record
        .integration
        .sync_modes
        .iter()
        .any(|mode| mode == trigger.as_str())
    {
        return Err(SyncRequestResult::Rejected {
            reason: "Sync trigger is not enabled for this connection".into(),
        });
    }
    if matches!(
        record.integration.connection_status.as_str(),
        "disconnected"
            | "unsupported"
            | "reauthentication_required"
            | "permission_denied"
            | "institution_configuration_required"
    ) {
        return Err(SyncRequestResult::Rejected {
            reason: "Connection is not eligible for synchronization".into(),
        });
    }
    if ics_handler(
        &record.integration.provider_id,
        &record.integration.auth_type,
    )
    .is_none()
    {
        return Err(SyncRequestResult::Rejected {
            reason: "Provider is unsupported".into(),
        });
    }
    if let Some(eligible) = record
        .integration
        .next_allowed_sync_at
        .as_deref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .filter(|value| *value > Utc::now())
    {
        return Err(SyncRequestResult::Deferred {
            eligible_at: eligible.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        });
    }
    Ok(())
}

fn spawn_reserved(
    inner: Arc<Inner>,
    host: Arc<dyn RuntimeHost>,
    queued_request: QueuedRequest,
    record: integrations::SyncRuntimeRecord,
    token: CancellationToken,
) {
    let semaphore = inner.global.clone();
    let task_host = host.clone();
    host.spawn(Box::pin(async move {
        tokio::select! {
            _ = token.cancelled() => {}
            permit = semaphore.acquire_owned() => {
                if let Ok(_permit) = permit {
                    run_one(
                        inner.clone(),
                        task_host.clone(),
                        queued_request.id.clone(),
                        record,
                        queued_request.trigger,
                        token,
                    )
                    .await;
                }
            }
        }
        let followup = finish_active(&inner, &queued_request.id);
        if followup {
            let _ = request(
                inner.clone(),
                task_host.clone(),
                queued_request.id.clone(),
                SyncTrigger::Manual,
            );
        }
        dispatch_next(inner, task_host, queued_request.provider);
    }));
}
fn schedule(inner: Arc<Inner>, host: Arc<dyn RuntimeHost>, trigger: SyncTrigger) {
    if let Ok(items) = host.candidates() {
        for item in items.into_iter().filter(|item| {
            item.enabled && item.sync_modes.iter().any(|mode| mode == trigger.as_str())
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
fn finish_active(inner: &Inner, id: &str) -> bool {
    let mut state = inner.state.lock().expect("sync state poisoned");
    state.active.remove(id);
    state.tokens.remove(id);
    state.replacement_followups.remove(id)
}

fn dispatch_next(inner: Arc<Inner>, host: Arc<dyn RuntimeHost>, provider: String) {
    loop {
        let next = {
            let mut state = inner.state.lock().expect("sync state poisoned");
            if state.shutting_down {
                state.provider_active.remove(&provider);
                state.provider_queues.remove(&provider);
                return;
            }
            let next = state
                .provider_queues
                .get_mut(&provider)
                .and_then(VecDeque::pop_front);
            if let Some(next) = &next {
                state.queued.remove(&next.id);
            } else {
                state.provider_queues.remove(&provider);
                state.provider_active.remove(&provider);
            }
            next
        };
        let Some(next) = next else {
            return;
        };
        let record = match host.record(&next.id) {
            Ok(Some(record)) => record,
            _ => continue,
        };
        if record.integration.provider_id != provider
            || validate_request(&record, Some(next.configuration_generation), next.trigger).is_err()
        {
            continue;
        }
        let token = inner.root.child_token();
        {
            let mut state = inner.state.lock().expect("sync state poisoned");
            if state.shutting_down {
                state.provider_active.remove(&provider);
                return;
            }
            state.active.insert(next.id.clone());
            state.tokens.insert(next.id.clone(), token.clone());
        }
        spawn_reserved(inner, host, next, record, token);
        return;
    }
}

fn remove_queued(state: &mut State, id: &str) {
    if !state.queued.remove(id) {
        return;
    }
    state.provider_queues.retain(|_, queue| {
        queue.retain(|request| request.id != id);
        !queue.is_empty()
    });
}

async fn shutdown(inner: Arc<Inner>) {
    {
        let mut state = inner.state.lock().expect("sync state poisoned");
        state.shutting_down = true;
        state.queued.clear();
        state.provider_queues.clear();
        state.replacement_followups.clear();
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
    record: integrations::SyncRuntimeRecord,
    trigger: SyncTrigger,
    cancel: CancellationToken,
) {
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    if host
        .mark_running(&id, record.configuration_generation, trigger, &now)
        .ok()
        != Some(true)
    {
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
        Ok((
            PreparedSync {
                outcome: PreparedSyncOutcome::RateLimited { eligible_at },
                ..
            },
            _,
        )) => {
            let fallback = backoff(record.consecutive_failures);
            let next = eligible_at.as_deref().unwrap_or(&fallback);
            let failure = SyncFailure {
                code: "rate_limited",
                message: "Calendar provider temporarily limited sync requests",
                connection_status: "rate_limited",
            };
            let _completion = host.finish_failure(
                &id,
                record.configuration_generation,
                &failure,
                next,
                eligible_at.as_deref(),
            );
        }
        Ok((prepared, policy)) => {
            if cancel.is_cancelled() || shutting_down(&inner) {
                let failure = SyncFailure {
                    code: if shutting_down(&inner) {
                        "interrupted"
                    } else {
                        "cancelled"
                    },
                    message: if shutting_down(&inner) {
                        "Sync was interrupted when Aether closed"
                    } else {
                        "Sync was cancelled"
                    },
                    connection_status: "connected",
                };
                let next = backoff(record.consecutive_failures);
                let _completion = host.finish_failure(
                    &id,
                    record.configuration_generation,
                    &failure,
                    &next,
                    None,
                );
            } else {
                let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let next = (Utc::now() + Duration::minutes(policy.success_interval_minutes))
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                if host.commit_success(prepared, &now, &next).is_err() {
                    let failure = SyncFailure {
                        code: "local_commit",
                        message: "Local synchronized data could not be committed",
                        connection_status: "degraded",
                    };
                    let retry = backoff(record.consecutive_failures);
                    let _completion = host.finish_failure(
                        &id,
                        record.configuration_generation,
                        &failure,
                        &retry,
                        None,
                    );
                }
            }
        }
        Err(failure) => {
            let next = backoff(record.consecutive_failures);
            let _completion =
                host.finish_failure(&id, record.configuration_generation, &failure, &next, None);
        }
    }
    host.emit(SyncStateEvent {
        connection_id: id,
        state: "terminal",
    });
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
            ics_handler(PROVIDER_ICS, "ics_feed"),
            Some(IcsNormalization::Default)
        );
        assert_eq!(
            ics_handler(PROVIDER_MY_TIMETABLE, "ics_feed"),
            Some(IcsNormalization::MyTimetable)
        );
        assert_eq!(ics_handler(PROVIDER_ICS, "oauth"), None);
        assert_eq!(ics_handler("brightspace", "ics_feed"), None);
        assert_eq!(ics_handler("unregistered", "ics_feed"), None);
    }

    #[derive(Clone, Copy)]
    enum HandlerMode {
        Success,
        Blocking,
        Controlled,
        ControlledFailure(&'static str),
        RateLimited,
    }

    struct FakeRuntimeHost {
        conn: Mutex<Connection>,
        mode: Mutex<HandlerMode>,
        connection_modes: Mutex<HashMap<String, HandlerMode>>,
        started: Arc<Notify>,
        started_order: Mutex<Vec<String>>,
        releases: Mutex<HashMap<String, Arc<Notify>>>,
        events: Mutex<Vec<SyncStateEvent>>,
        domain_commits: Mutex<usize>,
        success_commits: Mutex<usize>,
        commit_failures: Mutex<HashSet<String>>,
        cancelled: AtomicBool,
    }

    impl FakeRuntimeHost {
        fn new(mode: HandlerMode) -> Arc<Self> {
            let conn = Connection::open_in_memory().unwrap();
            migrations::run(&conn).unwrap();
            Arc::new(Self {
                conn: Mutex::new(conn),
                mode: Mutex::new(mode),
                connection_modes: Mutex::new(HashMap::new()),
                started: Arc::new(Notify::new()),
                started_order: Mutex::new(Vec::new()),
                releases: Mutex::new(HashMap::new()),
                events: Mutex::new(Vec::new()),
                domain_commits: Mutex::new(0),
                success_commits: Mutex::new(0),
                commit_failures: Mutex::new(HashSet::new()),
                cancelled: AtomicBool::new(false),
            })
        }

        fn integration(&self, next_allowed: Option<&str>) -> String {
            self.integration_for_provider(PROVIDER_ICS, next_allowed)
        }

        fn integration_for_provider(&self, provider: &str, next_allowed: Option<&str>) -> String {
            let conn = self.conn.lock().unwrap();
            let id = integrations::create(
                &conn,
                &integrations::IntegrationCreateInput {
                    provider_id: provider.into(),
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

        fn legacy_integration(&self, provider: &str) -> String {
            let id = uuid::Uuid::now_v7().to_string();
            self.conn
                .lock()
                .unwrap()
                .execute(
                    "INSERT INTO integrations(id,provider_id,enabled,auth_type,sync_modes_json,connection_status) VALUES (?1,?2,1,'ics_feed','[\"manual\",\"app_start\"]','connected')",
                    rusqlite::params![id, provider],
                )
                .unwrap();
            id
        }

        fn set_mode(&self, id: &str, mode: HandlerMode) {
            self.connection_modes
                .lock()
                .unwrap()
                .insert(id.to_string(), mode);
        }

        fn release(&self, id: &str) {
            self.releases
                .lock()
                .unwrap()
                .entry(id.to_string())
                .or_insert_with(|| Arc::new(Notify::new()))
                .notify_one();
        }

        fn fail_commit(&self, id: &str) {
            self.commit_failures.lock().unwrap().insert(id.to_string());
        }

        async fn wait_for_starts(&self, count: usize) {
            for _ in 0..100 {
                if self.started_order.lock().unwrap().len() >= count {
                    return;
                }
                sleep(TokioDuration::from_millis(10)).await;
            }
            panic!("expected {count} started syncs");
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
        fn mark_running(
            &self,
            id: &str,
            configuration_generation: i64,
            trigger: SyncTrigger,
            now: &str,
        ) -> Result<bool, String> {
            integrations::runtime_mark_running(
                &self.conn.lock().unwrap(),
                id,
                configuration_generation,
                trigger.as_str(),
                now,
            )
        }
        async fn prepare(
            &self,
            record: &integrations::SyncRuntimeRecord,
            cancel: CancellationToken,
        ) -> Result<(PreparedSync, SyncPolicy), SyncFailure> {
            self.started_order
                .lock()
                .unwrap()
                .push(record.integration.id.clone());
            self.started.notify_waiters();
            let mode = self
                .connection_modes
                .lock()
                .unwrap()
                .get(&record.integration.id)
                .copied()
                .unwrap_or_else(|| *self.mode.lock().unwrap());
            match mode {
                HandlerMode::Success => Ok((
                    PreparedSync {
                        connection_id: record.integration.id.clone(),
                        configuration_generation: record.configuration_generation,
                        outcome: PreparedSyncOutcome::NotModified,
                    },
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
                HandlerMode::Controlled => {
                    let release = self
                        .releases
                        .lock()
                        .unwrap()
                        .entry(record.integration.id.clone())
                        .or_insert_with(|| Arc::new(Notify::new()))
                        .clone();
                    tokio::select! {
                        _ = release.notified() => Ok((
                            PreparedSync {
                                connection_id: record.integration.id.clone(),
                                configuration_generation: record.configuration_generation,
                                outcome: PreparedSyncOutcome::NotModified,
                            },
                            SyncPolicy { success_interval_minutes: 30 },
                        )),
                        _ = cancel.cancelled() => {
                            self.cancelled.store(true, Ordering::SeqCst);
                            Err(SyncFailure {
                                code: "cancelled",
                                message: "cancelled",
                                connection_status: "connected",
                            })
                        }
                    }
                }
                HandlerMode::ControlledFailure(code) => {
                    let release = self
                        .releases
                        .lock()
                        .unwrap()
                        .entry(record.integration.id.clone())
                        .or_insert_with(|| Arc::new(Notify::new()))
                        .clone();
                    tokio::select! {
                        _ = release.notified() => Err(SyncFailure {
                            code,
                            message: "prepared work failed",
                            connection_status: "degraded",
                        }),
                        _ = cancel.cancelled() => Err(SyncFailure {
                            code: "cancelled",
                            message: "cancelled",
                            connection_status: "connected",
                        }),
                    }
                }
                HandlerMode::RateLimited => Ok((
                    PreparedSync {
                        connection_id: record.integration.id.clone(),
                        configuration_generation: record.configuration_generation,
                        outcome: PreparedSyncOutcome::RateLimited {
                            eligible_at: latest_allowed(
                                Some("2999-01-01T00:00:30Z".into()),
                                Some("2999-01-01T00:01:00Z".into()),
                            ),
                        },
                    },
                    SyncPolicy {
                        success_interval_minutes: 30,
                    },
                )),
            }
        }
        fn commit_success(
            &self,
            prepared: PreparedSync,
            now: &str,
            next_allowed: &str,
        ) -> Result<integrations::SyncCompletion, String> {
            if self
                .commit_failures
                .lock()
                .unwrap()
                .remove(&prepared.connection_id)
            {
                return Err("injected commit failure".into());
            }
            let conn = self.conn.lock().unwrap();
            let tx = conn
                .unchecked_transaction()
                .map_err(|error| error.to_string())?;
            if !integrations::generation_is_current(
                &tx,
                &prepared.connection_id,
                prepared.configuration_generation,
            )? {
                return Ok(integrations::SyncCompletion::StaleGeneration);
            }
            *self.domain_commits.lock().unwrap() += 1;
            let completion = match prepared.outcome {
                PreparedSyncOutcome::NotModified => integrations::runtime_finish_success(
                    &tx,
                    &prepared.connection_id,
                    prepared.configuration_generation,
                    now,
                    None,
                    next_allowed,
                )?,
                _ => return Err("Fake handler only commits no-change results".into()),
            };
            if completion == integrations::SyncCompletion::Applied {
                tx.commit().map_err(|error| error.to_string())?;
                *self.success_commits.lock().unwrap() += 1;
            }
            Ok(completion)
        }
        fn finish_failure(
            &self,
            id: &str,
            configuration_generation: i64,
            failure: &SyncFailure,
            next_allowed: &str,
            retry_after: Option<&str>,
        ) -> Result<integrations::SyncCompletion, String> {
            integrations::runtime_finish_failure(
                &self.conn.lock().unwrap(),
                id,
                configuration_generation,
                integrations::SyncFailureUpdate {
                    code: failure.code,
                    message: failure.message,
                    next_allowed,
                    connection_status: failure.connection_status,
                    retry_after,
                },
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
            runtime.request(id.clone(), SyncTrigger::Manual),
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
    async fn same_provider_connections_run_fifo_without_duplicate_starvation() {
        let host = FakeRuntimeHost::new(HandlerMode::Controlled);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let a = host.integration(None);
        let b = host.integration(None);
        let c = host.integration(None);

        assert!(matches!(
            runtime.request(a.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        host.wait_for_starts(1).await;
        assert!(matches!(
            runtime.request(b.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        assert!(matches!(
            runtime.request(c.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        assert!(matches!(
            runtime.request(a.clone(), SyncTrigger::Manual),
            SyncRequestResult::Coalesced
        ));
        assert_eq!(runtime.status().queued_count, 2);

        host.release(&a);
        host.wait_for_starts(2).await;
        assert_eq!(
            &*host.started_order.lock().unwrap(),
            &[a.clone(), b.clone()]
        );
        host.conn
            .lock()
            .unwrap()
            .execute(
                "UPDATE integrations SET next_allowed_sync_at=NULL WHERE id=?1",
                [&a],
            )
            .unwrap();
        assert!(matches!(
            runtime.request(a.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));

        host.release(&b);
        host.wait_for_starts(3).await;
        assert_eq!(
            &*host.started_order.lock().unwrap(),
            &[a.clone(), b.clone(), c.clone()]
        );
        host.release(&c);
        host.wait_for_starts(4).await;
        assert_eq!(
            &*host.started_order.lock().unwrap(),
            &[a.clone(), b, c, a.clone()]
        );
        host.release(&a);
        for _ in 0..100 {
            if runtime.status().running_count == 0 {
                break;
            }
            sleep(TokioDuration::from_millis(10)).await;
        }
        assert_eq!(runtime.status().queued_count, 0);
    }

    #[tokio::test]
    async fn different_providers_can_hold_global_slots_together() {
        let host = FakeRuntimeHost::new(HandlerMode::Controlled);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let timetable = host.integration_for_provider(PROVIDER_MY_TIMETABLE, None);
        let generic = host.integration_for_provider(PROVIDER_ICS, None);

        assert!(matches!(
            runtime.request(timetable.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        assert!(matches!(
            runtime.request(generic.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        host.wait_for_starts(2).await;
        assert_eq!(runtime.status().running_count, 2);
        host.release(&timetable);
        host.release(&generic);
    }

    #[tokio::test]
    async fn supported_calendar_providers_each_queue_distinct_connections() {
        for provider in [PROVIDER_MY_TIMETABLE, PROVIDER_ICS] {
            let host = FakeRuntimeHost::new(HandlerMode::Controlled);
            let runtime = IntegrationSyncRuntime::new(host.clone());
            let first = host.integration_for_provider(provider, None);
            let second = host.integration_for_provider(provider, None);

            assert!(matches!(
                runtime.request(first.clone(), SyncTrigger::Manual),
                SyncRequestResult::Accepted
            ));
            host.wait_for_starts(1).await;
            assert!(matches!(
                runtime.request(second.clone(), SyncTrigger::Manual),
                SyncRequestResult::Queued
            ));
            host.release(&first);
            host.wait_for_starts(2).await;
            assert_eq!(
                &*host.started_order.lock().unwrap(),
                &[first, second.clone()]
            );
            host.release(&second);
        }
    }

    #[tokio::test]
    async fn legacy_brightspace_is_rejected_for_manual_and_startup_sync() {
        let host = FakeRuntimeHost::new(HandlerMode::Success);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let legacy = host.legacy_integration("brightspace");

        assert!(matches!(
            runtime.request(legacy, SyncTrigger::Manual),
            SyncRequestResult::Rejected { reason } if reason == "Provider is unsupported"
        ));
        runtime.start();
        sleep(TokioDuration::from_millis(25)).await;
        assert!(host.started_order.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn lifecycle_scheduling_uses_only_the_matching_declared_trigger() {
        let host = FakeRuntimeHost::new(HandlerMode::Success);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let startup = host.integration(None);
        let resume = host.integration(None);
        let periodic = host.integration(None);
        {
            let conn = host.conn.lock().unwrap();
            conn.execute(
                "UPDATE integrations SET sync_modes_json='[\"app_start\"]' WHERE id=?1",
                [&startup],
            )
            .unwrap();
            conn.execute(
                "UPDATE integrations SET sync_modes_json='[\"app_resume\"]' WHERE id=?1",
                [&resume],
            )
            .unwrap();
            conn.execute(
                "UPDATE integrations SET sync_modes_json='[\"periodic\"]' WHERE id=?1",
                [&periodic],
            )
            .unwrap();
        }

        schedule(runtime.inner.clone(), host.clone(), SyncTrigger::Startup);
        host.wait_for_starts(1).await;
        schedule(runtime.inner.clone(), host.clone(), SyncTrigger::Resume);
        host.wait_for_starts(2).await;
        schedule(runtime.inner.clone(), host.clone(), SyncTrigger::Periodic);
        host.wait_for_starts(3).await;
        assert_eq!(
            &*host.started_order.lock().unwrap(),
            &[startup, resume, periodic]
        );
    }

    #[tokio::test]
    async fn queued_disconnect_and_disable_are_removed_before_dispatch() {
        let host = FakeRuntimeHost::new(HandlerMode::Controlled);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let active = host.integration(None);
        let disconnected = host.integration(None);
        let disabled = host.integration(None);

        assert!(matches!(
            runtime.request(active.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        host.wait_for_starts(1).await;
        assert!(matches!(
            runtime.request(disconnected.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        assert!(matches!(
            runtime.request(disabled.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        runtime.cancel_connection(&disconnected);
        host.conn
            .lock()
            .unwrap()
            .execute("DELETE FROM integrations WHERE id=?1", [&disconnected])
            .unwrap();
        integrations::set_enabled(&host.conn.lock().unwrap(), &disabled, false).unwrap();
        runtime.cancel_connection(&disabled);
        assert_eq!(runtime.status().queued_count, 0);

        host.release(&active);
        sleep(TokioDuration::from_millis(50)).await;
        assert_eq!(&*host.started_order.lock().unwrap(), &[active]);
    }

    #[tokio::test]
    async fn queued_replacement_runs_only_the_current_generation() {
        let host = FakeRuntimeHost::new(HandlerMode::Controlled);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let active = host.integration(None);
        let replacement = host.integration(None);

        assert!(matches!(
            runtime.request(active.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        host.wait_for_starts(1).await;
        assert!(matches!(
            runtime.request(replacement.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        {
            let conn = host.conn.lock().unwrap();
            let tx = conn.unchecked_transaction().unwrap();
            assert_eq!(
                integrations::replace_ics_configuration(&tx, &replacement, 1).unwrap(),
                Some(2)
            );
            tx.commit().unwrap();
        }
        runtime.request_replacement_sync(replacement.clone());
        assert_eq!(runtime.status().queued_count, 1);

        host.release(&active);
        host.wait_for_starts(2).await;
        host.release(&replacement);
        for _ in 0..100 {
            if runtime.status().running_count == 0 {
                break;
            }
            sleep(TokioDuration::from_millis(10)).await;
        }
        let record = integrations::sync_runtime_record(&host.conn.lock().unwrap(), &replacement)
            .unwrap()
            .unwrap();
        assert_eq!(record.configuration_generation, 2);
        assert_eq!(record.integration.sync_status, "succeeded");
    }

    #[tokio::test]
    async fn prepare_failures_and_cancellation_advance_the_provider_queue() {
        for failure_code in ["transient", "provider_data"] {
            let host = FakeRuntimeHost::new(HandlerMode::Success);
            let runtime = IntegrationSyncRuntime::new(host.clone());
            let first = host.integration(None);
            let second = host.integration(None);
            host.set_mode(&first, HandlerMode::ControlledFailure(failure_code));

            assert!(matches!(
                runtime.request(first.clone(), SyncTrigger::Manual),
                SyncRequestResult::Accepted
            ));
            host.wait_for_starts(1).await;
            assert!(matches!(
                runtime.request(second.clone(), SyncTrigger::Manual),
                SyncRequestResult::Queued
            ));
            host.release(&first);
            host.wait_for_starts(2).await;
            assert_eq!(
                host.record_state(&first).last_sync_error_code.as_deref(),
                Some(failure_code)
            );
            assert_eq!(host.record_state(&second).sync_status, "succeeded");
        }

        let host = FakeRuntimeHost::new(HandlerMode::Success);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let first = host.integration(None);
        let second = host.integration(None);
        host.set_mode(&first, HandlerMode::Blocking);
        assert!(matches!(
            runtime.request(first.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        host.wait_for_starts(1).await;
        assert!(matches!(
            runtime.request(second.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        runtime.cancel_connection(&first);
        host.wait_for_starts(2).await;
        assert_eq!(
            host.record_state(&first).last_sync_error_code.as_deref(),
            Some("cancelled")
        );
        assert_eq!(host.record_state(&second).sync_status, "succeeded");
    }

    #[tokio::test]
    async fn commit_failure_terminalizes_and_advances_the_provider_queue() {
        let host = FakeRuntimeHost::new(HandlerMode::Controlled);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let first = host.integration(None);
        let second = host.integration(None);
        host.fail_commit(&first);

        assert!(matches!(
            runtime.request(first.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        host.wait_for_starts(1).await;
        assert!(matches!(
            runtime.request(second.clone(), SyncTrigger::Manual),
            SyncRequestResult::Queued
        ));
        host.release(&first);
        host.wait_for_starts(2).await;
        host.release(&second);
        for _ in 0..100 {
            if runtime.status().running_count == 0 {
                break;
            }
            sleep(TokioDuration::from_millis(10)).await;
        }

        let failed = host.record_state(&first);
        assert_eq!(failed.sync_status, "failed");
        assert_eq!(failed.last_sync_error_code.as_deref(), Some("local_commit"));
        assert!(failed.last_successful_sync_at.is_none());
        assert_eq!(host.record_state(&second).sync_status, "succeeded");
        let events = host.events.lock().unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|event| event.connection_id == first && event.state == "terminal")
                .count(),
            1
        );
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
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Rejected { .. }
        ));
        assert_eq!(*host.domain_commits.lock().unwrap(), 0);
        assert_eq!(*host.success_commits.lock().unwrap(), 0);
        let persisted = host.record_state(&id);
        assert_eq!(persisted.sync_status, "failed");
        assert_eq!(persisted.last_sync_error_code.as_deref(), Some("cancelled"));
        assert_eq!(
            host.events
                .lock()
                .unwrap()
                .iter()
                .filter(|event| event.connection_id == id && event.state == "terminal")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn replacement_cancels_old_work_and_runs_one_current_generation_followup() {
        let host = FakeRuntimeHost::new(HandlerMode::Blocking);
        let runtime = IntegrationSyncRuntime::new(host.clone());
        let id = host.integration(None);
        let started = host.started.notified();
        assert!(matches!(
            runtime.request(id.clone(), SyncTrigger::Manual),
            SyncRequestResult::Accepted
        ));
        tokio::time::timeout(TokioDuration::from_secs(1), started)
            .await
            .expect("old generation did not start");
        {
            let conn = host.conn.lock().unwrap();
            let tx = conn.unchecked_transaction().unwrap();
            assert_eq!(
                integrations::replace_ics_configuration(&tx, &id, 1).unwrap(),
                Some(2)
            );
            tx.commit().unwrap();
        }
        *host.mode.lock().unwrap() = HandlerMode::Success;
        runtime.request_replacement_sync(id.clone());

        for _ in 0..100 {
            sleep(TokioDuration::from_millis(10)).await;
            if runtime.status().running_count == 0 && *host.success_commits.lock().unwrap() == 1 {
                break;
            }
        }
        assert!(host.cancelled.load(Ordering::SeqCst));
        assert_eq!(*host.domain_commits.lock().unwrap(), 1);
        assert_eq!(*host.success_commits.lock().unwrap(), 1);
        let record = integrations::sync_runtime_record(&host.conn.lock().unwrap(), &id)
            .unwrap()
            .unwrap();
        assert_eq!(record.configuration_generation, 2);
        assert_eq!(record.integration.sync_status, "succeeded");
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

    #[test]
    fn reconciliation_commit_failure_rolls_back_snapshot_and_records_bounded_failure() {
        let host = FakeRuntimeHost::new(HandlerMode::Success);
        let id = host.integration(None);
        let old_events = calendar_ics::normalize(
            b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:old\r\nDTSTART:20260924T090000Z\r\nDTEND:20260924T100000Z\r\nSUMMARY:Old\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            &id,
            Utc::now(),
        )
        .unwrap();
        crate::db::repositories::external_events::reconcile(
            &mut host.conn.lock().unwrap(),
            &id,
            &old_events,
            crate::db::repositories::external_events::ReconciliationMode::ObservedOnly,
            None,
            "2026-09-24T08:00:00Z",
        )
        .unwrap();
        let new_events = calendar_ics::normalize(
            b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:new\r\nDTSTART:20260924T110000Z\r\nDTEND:20260924T120000Z\r\nSUMMARY:New\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            &id,
            Utc::now(),
        )
        .unwrap();
        let conn = host.conn.lock().unwrap();
        integrations::runtime_mark_running(&conn, &id, 1, "manual", "2026-09-24T10:00:00Z")
            .unwrap();
        conn.execute_batch(
            "CREATE TRIGGER fail_integration_success BEFORE UPDATE ON integrations
             WHEN NEW.sync_status='succeeded'
             BEGIN SELECT RAISE(ABORT, 'injected commit failure'); END;",
        )
        .unwrap();
        let result = commit_prepared(
            &conn,
            PreparedSync {
                connection_id: id.clone(),
                configuration_generation: 1,
                outcome: PreparedSyncOutcome::Ics {
                    events: new_events,
                    etag: Some("new-etag".into()),
                    last_modified: None,
                    validator_origin: "https://calendar.example".into(),
                    window_start: "2026-09-24T00:00:00Z".into(),
                    window_end: "2026-09-25T00:00:00Z".into(),
                },
            },
            "2026-09-24T10:01:00Z",
            "2026-09-24T10:31:00Z",
        );
        assert!(result.is_err());
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1 AND external_id='old' AND status='active'",
                [&id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1 AND external_id='new'",
                [&id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        let after_rollback = integrations::get_by_id(&conn, &id).unwrap().unwrap();
        assert_eq!(after_rollback.sync_status, "syncing");
        assert!(after_rollback.last_successful_sync_at.is_none());
        assert!(after_rollback.last_sync_etag.is_none());
        conn.execute_batch("DROP TRIGGER fail_integration_success;")
            .unwrap();
        let failure = SyncFailure {
            code: "local_commit",
            message: "Local synchronized data could not be committed",
            connection_status: "degraded",
        };
        integrations::runtime_finish_failure(
            &conn,
            &id,
            1,
            integrations::SyncFailureUpdate {
                code: failure.code,
                message: failure.message,
                next_allowed: "2026-09-24T10:02:00Z",
                connection_status: failure.connection_status,
                retry_after: None,
            },
        )
        .unwrap();
        let terminal = integrations::get_by_id(&conn, &id).unwrap().unwrap();
        assert_eq!(terminal.sync_status, "failed");
        assert_eq!(
            terminal.last_sync_error_code.as_deref(),
            Some("local_commit")
        );
        assert!(terminal.last_successful_sync_at.is_none());
        assert!(terminal.last_sync_etag.is_none());
    }

    #[test]
    fn stale_authoritative_work_cannot_reconcile_or_overwrite_current_generation() {
        let host = FakeRuntimeHost::new(HandlerMode::Success);
        let id = host.integration(None);
        let old_events = calendar_ics::normalize(
            b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:old\r\nDTSTART:20260924T090000Z\r\nDTEND:20260924T100000Z\r\nSUMMARY:Old\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            &id,
            Utc::now(),
        )
        .unwrap();
        crate::db::repositories::external_events::reconcile(
            &mut host.conn.lock().unwrap(),
            &id,
            &old_events,
            crate::db::repositories::external_events::ReconciliationMode::ObservedOnly,
            None,
            "2026-09-24T08:00:00Z",
        )
        .unwrap();
        let new_events = calendar_ics::normalize(
            b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:new\r\nDTSTART:20260924T110000Z\r\nDTEND:20260924T120000Z\r\nSUMMARY:New\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
            &id,
            Utc::now(),
        )
        .unwrap();

        {
            let conn = host.conn.lock().unwrap();
            let tx = conn.unchecked_transaction().unwrap();
            assert_eq!(
                integrations::replace_ics_configuration(&tx, &id, 1).unwrap(),
                Some(2)
            );
            tx.commit().unwrap();
        }
        assert_eq!(
            commit_prepared(
                &host.conn.lock().unwrap(),
                PreparedSync {
                    connection_id: id.clone(),
                    configuration_generation: 1,
                    outcome: PreparedSyncOutcome::Ics {
                        events: new_events.clone(),
                        etag: Some("stale-etag".into()),
                        last_modified: None,
                        validator_origin: "https://stale.example".into(),
                        window_start: "2026-09-24T00:00:00Z".into(),
                        window_end: "2026-09-25T00:00:00Z".into(),
                    },
                },
                "2026-09-24T10:00:00Z",
                "2026-09-24T10:30:00Z",
            )
            .unwrap(),
            integrations::SyncCompletion::StaleGeneration
        );
        let conn = host.conn.lock().unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1 AND external_id='old' AND status='active'",
                [&id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1 AND external_id='new'",
                [&id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            0
        );
        drop(conn);

        assert_eq!(
            commit_prepared(
                &host.conn.lock().unwrap(),
                PreparedSync {
                    connection_id: id.clone(),
                    configuration_generation: 2,
                    outcome: PreparedSyncOutcome::Ics {
                        events: new_events,
                        etag: Some("current-etag".into()),
                        last_modified: None,
                        validator_origin: "https://current.example".into(),
                        window_start: "2026-09-24T00:00:00Z".into(),
                        window_end: "2026-09-25T00:00:00Z".into(),
                    },
                },
                "2026-09-24T10:01:00Z",
                "2026-09-24T10:31:00Z",
            )
            .unwrap(),
            integrations::SyncCompletion::Applied
        );
        let conn = host.conn.lock().unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1 AND external_id='old' AND status='removed'",
                [&id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT count(*) FROM external_events WHERE connection_id=?1 AND external_id='new' AND status='active'",
                [&id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap(),
            1
        );
        let record = integrations::sync_runtime_record(&conn, &id)
            .unwrap()
            .unwrap();
        assert_eq!(record.configuration_generation, 2);
        assert_eq!(
            record.integration.last_sync_etag.as_deref(),
            Some("current-etag")
        );
        assert_eq!(
            record.validator_origin.as_deref(),
            Some("https://current.example")
        );
        drop(conn);

        host.conn
            .lock()
            .unwrap()
            .execute("DELETE FROM integrations WHERE id=?1", [&id])
            .unwrap();
        assert_eq!(
            commit_prepared(
                &host.conn.lock().unwrap(),
                PreparedSync {
                    connection_id: id,
                    configuration_generation: 2,
                    outcome: PreparedSyncOutcome::NotModified,
                },
                "2026-09-24T10:02:00Z",
                "2026-09-24T10:32:00Z",
            )
            .unwrap(),
            integrations::SyncCompletion::StaleGeneration
        );
    }
}
