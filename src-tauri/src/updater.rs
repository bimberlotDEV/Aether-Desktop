use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{ipc::Channel, AppHandle, Runtime};
use tauri_plugin_updater::{Update, UpdaterExt};
use uuid::Uuid;

const APPROVAL_TTL: Duration = Duration::from_secs(10 * 60);
const UPDATE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const MAX_RELEASE_NOTES_CHARS: usize = 12_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdatePhase {
    Idle,
    Checking,
    Ready,
    Installing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub configured: bool,
    pub channel: &'static str,
    pub current_version: String,
    pub phase: UpdatePhase,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePreview {
    pub token: String,
    pub current_version: String,
    pub version: String,
    pub notes: Option<String>,
    pub published_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum UpdateProgressEvent {
    Started { content_length: Option<u64> },
    Progress { downloaded: u64, chunk_length: u64 },
    Downloaded,
    Verified,
    Installing,
}

struct Pending<T> {
    token: String,
    expires_at: Instant,
    value: T,
}

struct ApprovalGate<T> {
    phase: UpdatePhase,
    pending: Option<Pending<T>>,
}

impl<T> Default for ApprovalGate<T> {
    fn default() -> Self {
        Self {
            phase: UpdatePhase::Idle,
            pending: None,
        }
    }
}

impl<T> ApprovalGate<T> {
    fn clear_expired(&mut self, now: Instant) {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.expires_at <= now)
        {
            self.pending = None;
            if self.phase == UpdatePhase::Ready {
                self.phase = UpdatePhase::Idle;
            }
        }
    }

    fn begin_check(&mut self, now: Instant) -> Result<(), String> {
        self.clear_expired(now);
        if matches!(self.phase, UpdatePhase::Checking | UpdatePhase::Installing) {
            return Err("Another update operation is already running.".to_string());
        }
        self.pending = None;
        self.phase = UpdatePhase::Checking;
        Ok(())
    }

    fn finish_without_update(&mut self) {
        self.pending = None;
        self.phase = UpdatePhase::Idle;
    }

    fn finish_with_update(&mut self, token: String, expires_at: Instant, value: T) {
        self.pending = Some(Pending {
            token,
            expires_at,
            value,
        });
        self.phase = UpdatePhase::Ready;
    }

    fn cancel(&mut self, token: &str, now: Instant) -> bool {
        self.clear_expired(now);
        if self.phase != UpdatePhase::Ready
            || !self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.token == token)
        {
            return false;
        }
        self.pending = None;
        self.phase = UpdatePhase::Idle;
        true
    }

    fn take_for_install(&mut self, token: &str, now: Instant) -> Result<T, String> {
        self.clear_expired(now);
        if self.phase != UpdatePhase::Ready {
            return Err("There is no approved update ready to install.".to_string());
        }
        let pending = self
            .pending
            .take()
            .ok_or_else(|| "There is no approved update ready to install.".to_string())?;
        if pending.token != token {
            self.pending = Some(pending);
            return Err("The update approval is invalid or no longer current.".to_string());
        }
        self.phase = UpdatePhase::Installing;
        Ok(pending.value)
    }

    fn finish_install(&mut self) {
        self.phase = UpdatePhase::Idle;
        self.pending = None;
    }
}

pub struct UpdateRuntime {
    configured: bool,
    gate: Mutex<ApprovalGate<Update>>,
}

impl UpdateRuntime {
    pub fn new(configured: bool) -> Self {
        Self {
            configured,
            gate: Mutex::new(ApprovalGate::default()),
        }
    }

    pub fn configured(&self) -> bool {
        self.configured
    }

    fn phase(&self) -> UpdatePhase {
        self.gate
            .lock()
            .map(|gate| gate.phase)
            .unwrap_or(UpdatePhase::Idle)
    }
}

pub fn status<R: Runtime>(app: &AppHandle<R>, runtime: &UpdateRuntime) -> UpdateStatus {
    UpdateStatus {
        configured: runtime.configured(),
        channel: "Stable",
        current_version: app.package_info().version.to_string(),
        phase: runtime.phase(),
    }
}

pub async fn check<R: Runtime>(
    app: &AppHandle<R>,
    runtime: &UpdateRuntime,
) -> Result<Option<UpdatePreview>, String> {
    if !runtime.configured() {
        return Err("Signed updates are not configured in this Aether build.".to_string());
    }
    {
        let mut gate = runtime
            .gate
            .lock()
            .map_err(|_| "Update state is temporarily unavailable.".to_string())?;
        gate.begin_check(Instant::now())?;
    }

    let updater = match app.updater_builder().timeout(UPDATE_TIMEOUT).build() {
        Ok(updater) => updater,
        Err(error) => {
            reset_after_error(runtime);
            return Err(map_updater_error(&error));
        }
    };
    let update = match updater.check().await {
        Ok(update) => update,
        Err(error) => {
            reset_after_error(runtime);
            return Err(map_updater_error(&error));
        }
    };

    let Some(update) = update else {
        reset_after_error(runtime);
        return Ok(None);
    };

    let token = Uuid::now_v7().to_string();
    let expires_at = Instant::now() + APPROVAL_TTL;
    let expires_at_utc: DateTime<Utc> = Utc::now()
        + ChronoDuration::from_std(APPROVAL_TTL)
            .map_err(|_| "Could not create the update approval window.".to_string())?;
    let preview = UpdatePreview {
        token: token.clone(),
        current_version: update.current_version.clone(),
        version: update.version.clone(),
        notes: update.body.as_deref().map(sanitize_release_notes),
        published_at: update.date.map(|date| date.to_string()),
        expires_at: expires_at_utc.to_rfc3339(),
    };

    runtime
        .gate
        .lock()
        .map_err(|_| "Update state is temporarily unavailable.".to_string())?
        .finish_with_update(token, expires_at, update);
    Ok(Some(preview))
}

pub fn cancel(runtime: &UpdateRuntime, token: &str) -> Result<bool, String> {
    runtime
        .gate
        .lock()
        .map_err(|_| "Update state is temporarily unavailable.".to_string())
        .map(|mut gate| gate.cancel(token, Instant::now()))
}

pub async fn install(
    runtime: &UpdateRuntime,
    token: &str,
    on_event: Channel<UpdateProgressEvent>,
) -> Result<(), String> {
    let update = runtime
        .gate
        .lock()
        .map_err(|_| "Update state is temporarily unavailable.".to_string())?
        .take_for_install(token, Instant::now())?;

    let result = download_and_install(update, &on_event).await;
    runtime
        .gate
        .lock()
        .map_err(|_| "Update state is temporarily unavailable.".to_string())?
        .finish_install();
    result
}

async fn download_and_install(
    update: Update,
    on_event: &Channel<UpdateProgressEvent>,
) -> Result<(), String> {
    let mut downloaded = 0_u64;
    let mut started = false;
    let bytes = update
        .download(
            |chunk_length, content_length| {
                if !started {
                    started = true;
                    let _ = on_event.send(UpdateProgressEvent::Started { content_length });
                }
                let chunk_length = chunk_length as u64;
                downloaded = downloaded.saturating_add(chunk_length);
                let _ = on_event.send(UpdateProgressEvent::Progress {
                    downloaded,
                    chunk_length,
                });
            },
            || {
                let _ = on_event.send(UpdateProgressEvent::Downloaded);
            },
        )
        .await
        .map_err(|error| map_updater_error(&error))?;

    let _ = on_event.send(UpdateProgressEvent::Verified);
    let _ = on_event.send(UpdateProgressEvent::Installing);
    update
        .install(bytes)
        .map_err(|error| map_updater_error(&error))
}

fn reset_after_error(runtime: &UpdateRuntime) {
    if let Ok(mut gate) = runtime.gate.lock() {
        gate.finish_without_update();
    }
}

fn sanitize_release_notes(notes: &str) -> String {
    notes
        .chars()
        .filter(|character| *character == '\n' || *character == '\t' || !character.is_control())
        .take(MAX_RELEASE_NOTES_CHARS)
        .collect::<String>()
        .trim()
        .to_string()
}

fn map_updater_error(error: &tauri_plugin_updater::Error) -> String {
    use tauri_plugin_updater::Error;
    match error {
        Error::EmptyEndpoints => {
            "Signed updates are not configured in this Aether build.".to_string()
        }
        Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_) => {
            "The downloaded update could not be verified and was not installed.".to_string()
        }
        Error::Reqwest(_) | Error::Network(_) | Error::ReleaseNotFound => {
            "Aether could not reach the Stable update service. Check your connection and try again."
                .to_string()
        }
        Error::TargetNotFound(_) | Error::TargetsNotFound(_) => {
            "The available release does not contain a compatible Windows installer.".to_string()
        }
        Error::InsecureTransportProtocol => {
            "The update service configuration is insecure and has been refused.".to_string()
        }
        Error::InvalidUpdaterFormat | Error::BinaryNotFoundInArchive | Error::Extract(_) => {
            "The verified update package is not a valid Aether Windows installer.".to_string()
        }
        Error::AuthenticationFailed => {
            "Windows cancelled or refused the update installation.".to_string()
        }
        _ => "Aether could not prepare or install this update safely.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_is_one_time_and_bound_to_the_exact_token() {
        let now = Instant::now();
        let mut gate = ApprovalGate::default();
        gate.begin_check(now).unwrap();
        gate.finish_with_update("right".to_string(), now + APPROVAL_TTL, "update");

        assert!(gate.take_for_install("wrong", now).is_err());
        assert_eq!(gate.phase, UpdatePhase::Ready);
        assert_eq!(gate.take_for_install("right", now).unwrap(), "update");
        assert_eq!(gate.phase, UpdatePhase::Installing);
        assert!(gate.take_for_install("right", now).is_err());
    }

    #[test]
    fn expired_and_cancelled_approvals_never_install() {
        let now = Instant::now();
        let mut expired = ApprovalGate::default();
        expired.begin_check(now).unwrap();
        expired.finish_with_update("expired".to_string(), now, "update");
        assert!(expired.take_for_install("expired", now).is_err());
        assert_eq!(expired.phase, UpdatePhase::Idle);

        let mut cancelled = ApprovalGate::default();
        cancelled.begin_check(now).unwrap();
        cancelled.finish_with_update("cancel".to_string(), now + APPROVAL_TTL, "update");
        assert!(cancelled.cancel("cancel", now));
        assert!(!cancelled.cancel("cancel", now));
        assert!(cancelled.take_for_install("cancel", now).is_err());
    }

    #[test]
    fn concurrent_operations_are_rejected_and_errors_reset_state() {
        let now = Instant::now();
        let mut gate: ApprovalGate<()> = ApprovalGate::default();
        gate.begin_check(now).unwrap();
        assert!(gate.begin_check(now).is_err());
        gate.finish_without_update();
        assert_eq!(gate.phase, UpdatePhase::Idle);
        gate.begin_check(now).unwrap();
    }

    #[test]
    fn release_notes_are_bounded_and_strip_control_characters() {
        let notes = format!(
            "safe\u{0000}\n{}",
            "x".repeat(MAX_RELEASE_NOTES_CHARS + 100)
        );
        let sanitized = sanitize_release_notes(&notes);
        assert!(!sanitized.contains('\u{0000}'));
        assert!(sanitized.chars().count() <= MAX_RELEASE_NOTES_CHARS);
    }
}
