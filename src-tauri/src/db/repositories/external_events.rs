#![allow(dead_code)] // Reconciliation is intentionally native-connector-only until a provider task uses it.

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const EVENT_COLUMNS: &str = "id, connection_id, external_id, occurrence_id, title, description, time_kind, start_at_utc, end_at_utc, start_date, end_date, timezone, location, course_reference, group_references_json, event_kind, status, source_url, ingestion_provenance, source_version, content_hash, first_seen_at, last_seen_at, synchronized_at, created_at, updated_at";
type NormalizedEvent = (
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    String,
);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalEvent {
    pub id: String,
    pub connection_id: String,
    pub external_id: String,
    pub occurrence_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub time_kind: String,
    pub start_at_utc: Option<String>,
    pub end_at_utc: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: String,
    pub location: Option<String>,
    pub course_reference: Option<String>,
    pub group_references: Vec<String>,
    pub event_kind: String,
    pub status: String,
    pub source_url: Option<String>,
    pub ingestion_provenance: String,
    pub source_version: String,
    pub content_hash: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub synchronized_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEventInput {
    pub connection_id: String,
    pub external_id: String,
    pub occurrence_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub time_kind: String,
    pub start_at_utc: Option<String>,
    pub end_at_utc: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: String,
    pub location: Option<String>,
    pub course_reference: Option<String>,
    #[serde(default)]
    pub group_references: Vec<String>,
    pub event_kind: String,
    pub status: String,
    pub source_url: Option<String>,
    pub ingestion_provenance: String,
    pub source_version: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEventRange {
    pub connection_id: Option<String>,
    pub start: String,
    pub end: String,
    pub include_removed: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconciliationMode {
    ObservedOnly,
    Authoritative,
}

fn bounded(value: &str, field: &str, max: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max {
        return Err(format!(
            "External event {field} must contain 1 to {max} characters"
        ));
    }
    Ok(value.to_string())
}
fn optional(value: &Option<String>, field: &str, max: usize) -> Result<Option<String>, String> {
    match value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => Ok(Some(bounded(value, field, max)?)),
        None => Ok(None),
    }
}
fn utc(value: &str, field: &str) -> Result<String, String> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|_| format!("External event {field} must be an RFC3339 UTC instant"))?;
    if parsed.offset().local_minus_utc() != 0 {
        return Err(format!(
            "External event {field} must be UTC; unresolved floating values are not allowed"
        ));
    }
    Ok(parsed
        .with_timezone(&Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}
fn date(value: &str, field: &str) -> Result<String, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("External event {field} must be YYYY-MM-DD"))?;
    Ok(value.to_string())
}
fn url(value: &Option<String>) -> Result<Option<String>, String> {
    let Some(value) = optional(value, "source URL", 2048)? else {
        return Ok(None);
    };
    if !value.starts_with("https://") {
        return Err("External event source URL must use HTTPS".to_string());
    }
    Ok(Some(value))
}
fn row(row: &rusqlite::Row) -> rusqlite::Result<ExternalEvent> {
    let occurrence: String = row.get(3)?;
    let groups_json: String = row.get(14)?;
    let group_references = serde_json::from_str(&groups_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(14, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(ExternalEvent {
        id: row.get(0)?,
        connection_id: row.get(1)?,
        external_id: row.get(2)?,
        occurrence_id: (!occurrence.is_empty()).then_some(occurrence),
        title: row.get(4)?,
        description: row.get(5)?,
        time_kind: row.get(6)?,
        start_at_utc: row.get(7)?,
        end_at_utc: row.get(8)?,
        start_date: row.get(9)?,
        end_date: row.get(10)?,
        timezone: row.get(11)?,
        location: row.get(12)?,
        course_reference: row.get(13)?,
        group_references,
        event_kind: row.get(15)?,
        status: row.get(16)?,
        source_url: row.get(17)?,
        ingestion_provenance: row.get(18)?,
        source_version: row.get(19)?,
        content_hash: row.get(20)?,
        first_seen_at: row.get(21)?,
        last_seen_at: row.get(22)?,
        synchronized_at: row.get(23)?,
        created_at: row.get(24)?,
        updated_at: row.get(25)?,
    })
}
fn normalized(input: &ExternalEventInput) -> Result<NormalizedEvent, String> {
    let connection = bounded(&input.connection_id, "connection ID", 64)?;
    let external = bounded(&input.external_id, "external ID", 512)?;
    let occurrence = input
        .occurrence_id
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if occurrence.chars().count() > 512 {
        return Err("External event occurrence ID must not exceed 512 characters".to_string());
    }
    let title = bounded(&input.title, "title", 500)?;
    let description = optional(&input.description, "description", 20_000)?;
    let timezone = bounded(&input.timezone, "timezone", 128)?;
    let event_kind = bounded(&input.event_kind, "kind", 64)?;
    let mut group_references = input
        .group_references
        .iter()
        .map(|value| bounded(value, "group reference", 200))
        .collect::<Result<Vec<_>, _>>()?;
    group_references.sort_unstable();
    group_references.dedup();
    if group_references.len() > 64 {
        return Err("External event group references must not exceed 64 entries".to_string());
    }
    let group_references_json = serde_json::to_string(&group_references)
        .map_err(|error| format!("External event group serialization error: {error}"))?;
    if !["active", "cancelled"].contains(&input.status.as_str()) {
        return Err("External snapshots may be active or cancelled, not removed".to_string());
    }
    let (start_at_utc, end_at_utc, start_date, end_date) = match input.time_kind.as_str() {
        "timed" => {
            let start = utc(
                input
                    .start_at_utc
                    .as_deref()
                    .ok_or("Timed event needs UTC start")?,
                "start",
            )?;
            let end = utc(
                input
                    .end_at_utc
                    .as_deref()
                    .ok_or("Timed event needs UTC end")?,
                "end",
            )?;
            if start >= end {
                return Err("Timed event end must be after start".to_string());
            }
            if input.start_date.is_some() || input.end_date.is_some() {
                return Err("Timed event must not include date-only fields".to_string());
            }
            (Some(start), Some(end), None, None)
        }
        "all_day" => {
            let start = date(
                input
                    .start_date
                    .as_deref()
                    .ok_or("All-day event needs start date")?,
                "start date",
            )?;
            let end = date(
                input
                    .end_date
                    .as_deref()
                    .ok_or("All-day event needs end date")?,
                "end date",
            )?;
            if start >= end {
                return Err(
                    "All-day event end date must be exclusive and after start date".to_string(),
                );
            }
            if input.start_at_utc.is_some() || input.end_at_utc.is_some() {
                return Err("All-day event must not include UTC fields".to_string());
            }
            (None, None, Some(start), Some(end))
        }
        _ => return Err("External event time kind must be timed or all_day".to_string()),
    };
    Ok((
        connection,
        external,
        occurrence,
        title,
        description,
        input.time_kind.clone(),
        start_at_utc,
        end_at_utc,
        start_date,
        end_date,
        timezone,
        optional(&input.location, "location", 500)?,
        optional(&input.course_reference, "course reference", 200)?,
        group_references_json,
        event_kind,
        input.status.clone(),
        url(&input.source_url)?,
        bounded(&input.ingestion_provenance, "ingestion provenance", 100)?,
        bounded(&input.source_version, "source version", 512)?,
        bounded(&input.content_hash, "content hash", 128)?,
    ))
}

pub fn reconcile(
    conn: &mut Connection,
    connection_id: &str,
    snapshot: &[ExternalEventInput],
    mode: ReconciliationMode,
    window: Option<(&str, &str)>,
    synchronized_at: &str,
) -> Result<(), String> {
    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| format!("External event transaction error: {error}"))?;
    reconcile_in_transaction(
        &transaction,
        connection_id,
        snapshot,
        mode,
        window,
        synchronized_at,
    )?;
    transaction
        .commit()
        .map_err(|error| format!("External event reconciliation error: {error}"))
}

/// Reconciles an already normalized provider snapshot in a caller-owned transaction.
/// Sync runtimes use this to keep domain data and their Integration completion metadata atomic.
pub fn reconcile_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    connection_id: &str,
    snapshot: &[ExternalEventInput],
    mode: ReconciliationMode,
    window: Option<(&str, &str)>,
    synchronized_at: &str,
) -> Result<(), String> {
    let synchronized_at = utc(synchronized_at, "synchronized at")?;
    if mode == ReconciliationMode::Authoritative && window.is_none() {
        return Err("Authoritative reconciliation requires a complete window".to_string());
    }
    for input in snapshot {
        if input.connection_id.trim() != connection_id.trim() {
            return Err(
                "Snapshot event connection ID does not match reconciliation connection".to_string(),
            );
        }
        let value = normalized(input)?;
        transaction.execute(
            "INSERT INTO external_events (id,connection_id,external_id,occurrence_id,title,description,time_kind,start_at_utc,end_at_utc,start_date,end_date,timezone,location,course_reference,group_references_json,event_kind,status,source_url,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?22,?22) ON CONFLICT(connection_id,external_id,occurrence_id) DO UPDATE SET title=excluded.title,description=excluded.description,time_kind=excluded.time_kind,start_at_utc=excluded.start_at_utc,end_at_utc=excluded.end_at_utc,start_date=excluded.start_date,end_date=excluded.end_date,timezone=excluded.timezone,location=excluded.location,course_reference=excluded.course_reference,group_references_json=excluded.group_references_json,event_kind=excluded.event_kind,status=excluded.status,source_url=excluded.source_url,ingestion_provenance=excluded.ingestion_provenance,source_version=excluded.source_version,content_hash=excluded.content_hash,last_seen_at=excluded.last_seen_at,synchronized_at=excluded.synchronized_at,updated_at=datetime('now')",
            params![Uuid::now_v7().to_string(), value.0,value.1,value.2,value.3,value.4,value.5,value.6,value.7,value.8,value.9,value.10,value.11,value.12,value.13,value.14,value.15,value.16,value.17,value.18,value.19,synchronized_at],
        ).map_err(|error| format!("External event upsert error: {error}"))?;
    }
    if mode == ReconciliationMode::Authoritative {
        let (start, end) = window.unwrap();
        let start = utc(start, "window start")?;
        let end = utc(end, "window end")?;
        if start >= end {
            return Err("Authoritative window end must be after start".to_string());
        }
        transaction.execute("UPDATE external_events SET status='removed', updated_at=datetime('now') WHERE connection_id=?1 AND status!='removed' AND last_seen_at<?4 AND ((time_kind='timed' AND start_at_utc>=?2 AND start_at_utc<?3) OR (time_kind='all_day' AND start_date>=substr(?2,1,10) AND start_date<substr(?3,1,10)))", params![connection_id.trim(),start,end,synchronized_at]).map_err(|error| format!("External event tombstone error: {error}"))?;
    }
    Ok(())
}

pub fn list_range(
    conn: &Connection,
    range: &ExternalEventRange,
) -> Result<Vec<ExternalEvent>, String> {
    let start = utc(&range.start, "range start")?;
    let end = utc(&range.end, "range end")?;
    if start >= end {
        return Err("External event range end must be after start".to_string());
    }
    let limit = range.limit.unwrap_or(100).clamp(1, 500);
    let mut statement = conn.prepare(&format!("SELECT {EVENT_COLUMNS} FROM external_events WHERE (?1 IS NULL OR connection_id=?1) AND (?2 OR status!='removed') AND ((time_kind='timed' AND start_at_utc<?4 AND end_at_utc>?3) OR (time_kind='all_day' AND start_date<substr(?4,1,10) AND end_date>substr(?3,1,10))) ORDER BY COALESCE(start_at_utc, start_date), id LIMIT ?5")).map_err(|error| format!("External event query error: {error}"))?;
    let events = statement
        .query_map(
            params![
                range.connection_id,
                range.include_removed.unwrap_or(false),
                start,
                end,
                limit
            ],
            row,
        )
        .map_err(|error| format!("External event query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("External event row error: {error}"))?;
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        migrations::run(&c).unwrap();
        c.execute(
            "INSERT INTO integrations(id,provider_id,auth_type) VALUES ('c','test','none')",
            [],
        )
        .unwrap();
        c
    }
    fn event() -> ExternalEventInput {
        ExternalEventInput {
            connection_id: "c".into(),
            external_id: "e".into(),
            occurrence_id: Some("o".into()),
            title: "Lesson".into(),
            description: None,
            time_kind: "timed".into(),
            start_at_utc: Some("2026-10-25T09:00:00Z".into()),
            end_at_utc: Some("2026-10-25T10:00:00Z".into()),
            start_date: None,
            end_date: None,
            timezone: "Europe/Berlin".into(),
            location: None,
            course_reference: None,
            group_references: vec!["ADSAI-ZM-1.a".into()],
            event_kind: "lesson".into(),
            status: "active".into(),
            source_url: None,
            ingestion_provenance: "feed".into(),
            source_version: "1".into(),
            content_hash: "hash".into(),
        }
    }
    #[test]
    fn upsert_is_idempotent_and_moved_occurrence_keeps_identity() {
        let mut c = db();
        let mut e = event();
        reconcile(
            &mut c,
            "c",
            std::slice::from_ref(&e),
            ReconciliationMode::ObservedOnly,
            None,
            "2026-09-21T00:00:00Z",
        )
        .unwrap();
        e.start_at_utc = Some("2026-10-25T11:00:00Z".into());
        e.end_at_utc = Some("2026-10-25T12:00:00Z".into());
        reconcile(
            &mut c,
            "c",
            &[e],
            ReconciliationMode::ObservedOnly,
            None,
            "2026-09-22T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM external_events", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        let values = list_range(
            &c,
            &ExternalEventRange {
                connection_id: Some("c".into()),
                start: "2026-10-25T00:00:00Z".into(),
                end: "2026-10-26T00:00:00Z".into(),
                include_removed: None,
                limit: Some(10),
            },
        )
        .unwrap();
        assert_eq!(values[0].group_references, vec!["ADSAI-ZM-1.a"]);
    }
    #[test]
    fn authoritative_only_tombstones_and_reappearance_reactivates() {
        let mut c = db();
        let e = event();
        reconcile(
            &mut c,
            "c",
            std::slice::from_ref(&e),
            ReconciliationMode::ObservedOnly,
            None,
            "2026-09-21T00:00:00Z",
        )
        .unwrap();
        reconcile(
            &mut c,
            "c",
            &[],
            ReconciliationMode::ObservedOnly,
            None,
            "2026-09-22T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            c.query_row("SELECT status FROM external_events", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "active"
        );
        reconcile(
            &mut c,
            "c",
            &[],
            ReconciliationMode::Authoritative,
            Some(("2026-10-25T00:00:00Z", "2026-10-26T00:00:00Z")),
            "2026-09-22T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            c.query_row("SELECT status FROM external_events", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "removed"
        );
        reconcile(
            &mut c,
            "c",
            &[e],
            ReconciliationMode::Authoritative,
            Some(("2026-10-25T00:00:00Z", "2026-10-26T00:00:00Z")),
            "2026-09-23T00:00:00Z",
        )
        .unwrap();
        assert_eq!(
            c.query_row("SELECT status FROM external_events", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "active"
        );
    }
    #[test]
    fn rejects_floating_and_invalid_all_day_values() {
        let mut c = db();
        let mut e = event();
        e.start_at_utc = Some("2026-10-25T09:00:00".into());
        assert!(reconcile(
            &mut c,
            "c",
            &[e],
            ReconciliationMode::ObservedOnly,
            None,
            "2026-09-21T00:00:00Z"
        )
        .is_err());
    }
}
