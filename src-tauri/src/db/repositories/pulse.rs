use std::collections::HashSet;

use chrono::{DateTime, Days, Local, LocalResult, NaiveDate, TimeZone, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

const TODAY_LIMIT: i64 = 30;
const UPCOMING_LIMIT: i64 = 30;
const TASK_LIMIT: i64 = 20;
const CONFLICT_LIMIT: usize = 10;
const CONFLICT_CANDIDATE_LIMIT: i64 = 128;
const CONTINUITY_LIMIT: i64 = 5;
const FRESHNESS_THRESHOLD_SECONDS: i64 = 2 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseEvent {
    pub id: String,
    pub source_id: String,
    pub source_label: String,
    pub source_type: String,
    pub title: String,
    pub time_kind: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub location: Option<String>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseTask {
    pub id: String,
    pub title: String,
    pub space_id: Option<String>,
    pub space_name: Option<String>,
    pub due_date: String,
    pub priority: String,
    pub category: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseContinuityItem {
    pub id: String,
    pub name: String,
    pub reason: String,
    pub last_worked_at: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseConflict {
    pub id: String,
    pub first: PulseEvent,
    pub second: PulseEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseTrustItem {
    pub source_id: String,
    pub source_label: String,
    pub provider_label: String,
    pub enabled: bool,
    pub state: String,
    pub freshness: String,
    pub last_successful_sync_at: Option<String>,
    pub last_attempted_at: Option<String>,
    pub error_category: Option<String>,
    pub showing_cached_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseSectionIssue {
    pub section: String,
    pub state: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PulseSnapshot {
    pub generated_at: String,
    pub local_date: String,
    pub now: Vec<PulseEvent>,
    pub next: Option<PulseEvent>,
    pub today: Vec<PulseEvent>,
    pub upcoming: Vec<PulseEvent>,
    pub tasks: Vec<PulseTask>,
    pub conflicts: Vec<PulseConflict>,
    pub continuity: Vec<PulseContinuityItem>,
    pub trust: Vec<PulseTrustItem>,
    pub academic_deadlines_available: bool,
    pub issues: Vec<PulseSectionIssue>,
}

#[derive(Debug, Clone)]
struct SnapshotClock {
    now: DateTime<Utc>,
    today: NaiveDate,
    today_start: DateTime<Utc>,
    tomorrow_start: DateTime<Utc>,
    horizon_end: DateTime<Utc>,
}

impl SnapshotClock {
    fn capture() -> Result<Self, String> {
        let local_now = Local::now();
        let today = local_now.date_naive();
        let tomorrow = today
            .checked_add_days(Days::new(1))
            .ok_or_else(|| "Pulse local date is unavailable".to_string())?;
        let horizon = today
            .checked_add_days(Days::new(8))
            .ok_or_else(|| "Pulse horizon is unavailable".to_string())?;
        Ok(Self {
            now: local_now.with_timezone(&Utc),
            today,
            today_start: local_midnight(today)?,
            tomorrow_start: local_midnight(tomorrow)?,
            horizon_end: local_midnight(horizon)?,
        })
    }

    #[cfg(test)]
    fn utc(now: &str, today: &str) -> Self {
        let now = DateTime::parse_from_rfc3339(now)
            .unwrap()
            .with_timezone(&Utc);
        let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap();
        let at = |date: NaiveDate| Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
        Self {
            now,
            today,
            today_start: at(today),
            tomorrow_start: at(today.checked_add_days(Days::new(1)).unwrap()),
            horizon_end: at(today.checked_add_days(Days::new(8)).unwrap()),
        }
    }
}

fn local_midnight(date: NaiveDate) -> Result<DateTime<Utc>, String> {
    let midnight = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "Pulse local midnight is unavailable".to_string())?;
    let local = match Local.from_local_datetime(&midnight) {
        LocalResult::Single(value) => value,
        LocalResult::Ambiguous(first, second) => first.min(second),
        LocalResult::None => {
            return Err("Pulse local day boundary is unavailable".to_string());
        }
    };
    Ok(local.with_timezone(&Utc))
}

pub fn get(conn: &Connection) -> Result<PulseSnapshot, String> {
    get_at(conn, SnapshotClock::capture()?)
}

fn get_at(conn: &Connection, clock: SnapshotClock) -> Result<PulseSnapshot, String> {
    let mut issues = Vec::new();
    let today = section(
        "schedule",
        &mut issues,
        list_events(
            conn,
            &clock.today_start,
            &clock.tomorrow_start,
            clock.today,
            clock.today.checked_add_days(Days::new(1)).unwrap(),
            TODAY_LIMIT,
        ),
    );
    let upcoming = section(
        "upcoming",
        &mut issues,
        list_events(
            conn,
            &clock.tomorrow_start,
            &clock.horizon_end,
            clock.today.checked_add_days(Days::new(1)).unwrap(),
            clock.today.checked_add_days(Days::new(8)).unwrap(),
            UPCOMING_LIMIT,
        ),
    );
    let tasks = section(
        "tasks",
        &mut issues,
        list_tasks(conn, clock.today, TASK_LIMIT),
    );
    let continuity = section(
        "continuity",
        &mut issues,
        list_continuity(conn, CONTINUITY_LIMIT),
    );
    let trust = section("trust", &mut issues, list_trust(conn, clock.now));

    let now = today
        .iter()
        .filter(|event| is_active(event, clock.now))
        .cloned()
        .collect::<Vec<_>>();
    let next = today
        .iter()
        .chain(upcoming.iter())
        .filter(|event| is_future_timed(event, clock.now))
        .min_by(|left, right| event_order(left, right))
        .cloned();

    let conflict_candidates = section(
        "conflicts",
        &mut issues,
        list_events(
            conn,
            &clock.today_start,
            &clock.horizon_end,
            clock.today,
            clock.today.checked_add_days(Days::new(8)).unwrap(),
            CONFLICT_CANDIDATE_LIMIT,
        ),
    );
    let conflicts = conflicts(&conflict_candidates);

    Ok(PulseSnapshot {
        generated_at: instant(clock.now),
        local_date: clock.today.format("%Y-%m-%d").to_string(),
        now,
        next,
        today,
        upcoming,
        tasks,
        conflicts,
        continuity,
        trust,
        academic_deadlines_available: false,
        issues,
    })
}

fn section<T: Default>(
    name: &str,
    issues: &mut Vec<PulseSectionIssue>,
    result: Result<T, String>,
) -> T {
    match result {
        Ok(value) => value,
        Err(_) => {
            issues.push(PulseSectionIssue {
                section: name.to_string(),
                state: "degraded".to_string(),
                message: format!("{name} is temporarily unavailable"),
            });
            T::default()
        }
    }
}

fn instant(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn date(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn list_events(
    conn: &Connection,
    start_utc: &DateTime<Utc>,
    end_utc: &DateTime<Utc>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    limit: i64,
) -> Result<Vec<PulseEvent>, String> {
    let mut statement = conn
        .prepare(
            "SELECT DISTINCT e.id, e.connection_id,
                    coalesce(nullif(trim(sc.display_name), ''), 'MyTimetable'),
                    'school_calendar',
                    e.title, e.time_kind, e.start_at_utc, e.end_at_utc,
                    e.start_date, e.end_date, e.location, e.status
             FROM external_events e
             JOIN integrations i ON i.id=e.connection_id
             LEFT JOIN subscribed_calendars sc ON sc.connection_id=i.id
             WHERE e.status!='removed'
               AND i.provider_id='my_timetable'
               AND EXISTS (
                   SELECT 1
                   FROM school_space_sources binding
                   JOIN spaces school ON school.id=binding.school_space_id
                   JOIN school_space_source_groups selected
                     ON selected.school_space_id=binding.school_space_id
                    AND selected.connection_id=binding.connection_id
                   JOIN json_each(e.group_references_json) event_group
                   WHERE binding.connection_id=e.connection_id
                     AND school.template_type='school'
                     AND school.parent_space_id IS NULL
                     AND school.archived_at IS NULL
                     AND selected.group_reference=CAST(event_group.value AS TEXT)
                 )
               AND ((e.time_kind='timed' AND e.start_at_utc<?2 AND e.end_at_utc>?1)
                 OR (e.time_kind='all_day' AND e.start_date<?4 AND e.end_date>?3))
             ORDER BY
               CASE WHEN e.time_kind='all_day' THEN e.start_date || 'T00:00:00Z' ELSE e.start_at_utc END,
               CASE WHEN e.time_kind='all_day' THEN e.end_date || 'T00:00:00Z' ELSE e.end_at_utc END,
               e.id
             LIMIT ?5",
        )
        .map_err(|error| format!("Pulse event projection error: {error}"))?;
    let rows = statement
        .query_map(
            params![
                instant(*start_utc),
                instant(*end_utc),
                date(start_date),
                date(end_date),
                limit
            ],
            |row| {
                Ok(PulseEvent {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    source_label: row.get(2)?,
                    source_type: row.get(3)?,
                    title: row.get(4)?,
                    time_kind: row.get(5)?,
                    start_at: row.get(6)?,
                    end_at: row.get(7)?,
                    start_date: row.get(8)?,
                    end_date: row.get(9)?,
                    location: row.get(10)?,
                    cancelled: row.get::<_, String>(11)? == "cancelled",
                })
            },
        )
        .map_err(|error| format!("Pulse event projection error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Pulse event projection row error: {error}"))
}

fn list_tasks(conn: &Connection, today: NaiveDate, limit: i64) -> Result<Vec<PulseTask>, String> {
    let horizon = today.checked_add_days(Days::new(7)).unwrap();
    let mut statement = conn
        .prepare(
            "SELECT t.id, t.title, t.space_id, s.name, t.due_date, t.priority,
                    CASE WHEN t.due_date<?1 THEN 'overdue'
                         WHEN t.due_date=?1 THEN 'today' ELSE 'soon' END
             FROM tasks t
             LEFT JOIN spaces s ON s.id=t.space_id
             WHERE t.archived_at IS NULL AND t.status!='done' AND t.due_date IS NOT NULL
               AND t.due_date<=?2
               AND (t.space_id IS NULL OR s.archived_at IS NULL)
             ORDER BY CASE WHEN t.due_date<?1 THEN 0 WHEN t.due_date=?1 THEN 1 ELSE 2 END,
               t.due_date,
               CASE t.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END,
               t.updated_at DESC, t.id
             LIMIT ?3",
        )
        .map_err(|error| format!("Pulse task projection error: {error}"))?;
    let rows = statement
        .query_map(params![date(today), date(horizon), limit], |row| {
            Ok(PulseTask {
                id: row.get(0)?,
                title: row.get(1)?,
                space_id: row.get(2)?,
                space_name: row.get(3)?,
                due_date: row.get(4)?,
                priority: row.get(5)?,
                category: row.get(6)?,
                destination: "/tasks".to_string(),
            })
        })
        .map_err(|error| format!("Pulse task projection error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Pulse task projection row error: {error}"))
}

fn list_continuity(conn: &Connection, limit: i64) -> Result<Vec<PulseContinuityItem>, String> {
    let mut statement = conn
        .prepare(
            "WITH work(space_id, worked_at, reason) AS (
                SELECT space_id, updated_at, 'Recent Note work' FROM notes WHERE archived_at IS NULL
                UNION ALL SELECT space_id, updated_at, 'Open Task activity' FROM tasks WHERE archived_at IS NULL AND status!='done' AND space_id IS NOT NULL
                UNION ALL SELECT space_id, updated_at, 'Recent AI conversation' FROM ai_conversations WHERE archived_at IS NULL AND space_id IS NOT NULL
                UNION ALL SELECT id, last_opened_at, 'Recently opened' FROM spaces WHERE last_opened_at IS NOT NULL
             ), ranked AS (
                SELECT space_id, worked_at, reason,
                  row_number() OVER (PARTITION BY space_id ORDER BY worked_at DESC, reason) position
                FROM work
             )
             SELECT s.id, s.name, ranked.reason, ranked.worked_at
             FROM ranked JOIN spaces s ON s.id=ranked.space_id
             WHERE ranked.position=1 AND s.archived_at IS NULL
             ORDER BY ranked.worked_at DESC, s.name, s.id LIMIT ?1",
        )
        .map_err(|error| format!("Pulse continuity projection error: {error}"))?;
    let rows = statement
        .query_map([limit], |row| {
            let id: String = row.get(0)?;
            Ok(PulseContinuityItem {
                destination: format!("/spaces/{id}"),
                id,
                name: row.get(1)?,
                reason: row.get(2)?,
                last_worked_at: row.get(3)?,
            })
        })
        .map_err(|error| format!("Pulse continuity projection error: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Pulse continuity projection row error: {error}"))
}

fn list_trust(conn: &Connection, now: DateTime<Utc>) -> Result<Vec<PulseTrustItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT DISTINCT i.id,
                    coalesce(nullif(trim(sc.display_name), ''), 'MyTimetable'),
                    'MyTimetable',
                    i.enabled, i.connection_status, i.sync_status,
                    i.last_successful_sync_at, i.last_attempted_at, i.last_sync_error_code,
                    EXISTS(SELECT 1 FROM external_events e WHERE e.connection_id=i.id AND e.status!='removed')
             FROM integrations i
             JOIN subscribed_calendars sc ON sc.connection_id=i.id
             WHERE i.provider_id='my_timetable'
               AND EXISTS (
                 SELECT 1 FROM school_space_sources binding
                 JOIN spaces school ON school.id=binding.school_space_id
                 WHERE binding.connection_id=i.id
                   AND school.template_type='school'
                   AND school.parent_space_id IS NULL
                   AND school.archived_at IS NULL
               )
             ORDER BY 2 COLLATE NOCASE, i.id",
        )
        .map_err(|error| format!("Pulse trust projection error: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, bool>(9)?,
            ))
        })
        .map_err(|error| format!("Pulse trust projection error: {error}"))?;
    rows.map(|row| {
        let (
            source_id,
            source_label,
            provider_label,
            enabled,
            connection_status,
            sync_status,
            last_successful_sync_at,
            last_attempted_at,
            error_code,
            has_cached_data,
        ) = row.map_err(|error| format!("Pulse trust projection row error: {error}"))?;
        let freshness = freshness(last_successful_sync_at.as_deref(), now);
        let state = trust_state(enabled, &connection_status, &sync_status, &freshness);
        Ok(PulseTrustItem {
            source_id,
            source_label,
            provider_label,
            enabled,
            showing_cached_data: has_cached_data
                && matches!(
                    state.as_str(),
                    "disabled" | "disconnected" | "degraded" | "stale"
                ),
            state,
            freshness,
            last_successful_sync_at,
            last_attempted_at,
            error_category: sanitized_error(error_code.as_deref()),
        })
    })
    .collect()
}

fn freshness(last_successful: Option<&str>, now: DateTime<Utc>) -> String {
    let Some(last_successful) = last_successful else {
        return "never_synced".to_string();
    };
    let parsed = DateTime::parse_from_rfc3339(last_successful)
        .map(|value| value.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(last_successful, "%Y-%m-%d %H:%M:%S")
                .map(|value| Utc.from_utc_datetime(&value))
        });
    match parsed {
        Ok(last)
            if now.signed_duration_since(last).num_seconds() <= FRESHNESS_THRESHOLD_SECONDS =>
        {
            "fresh".to_string()
        }
        Ok(_) => "stale".to_string(),
        Err(_) => "never_synced".to_string(),
    }
}

fn trust_state(
    enabled: bool,
    connection_status: &str,
    sync_status: &str,
    freshness: &str,
) -> String {
    if !enabled {
        "disabled"
    } else if connection_status == "disconnected" {
        "disconnected"
    } else if sync_status == "syncing" || sync_status == "pending" {
        "syncing"
    } else if connection_status == "degraded" || sync_status == "failed" {
        "degraded"
    } else if freshness == "never_synced" {
        "never_synced"
    } else if freshness == "stale" {
        "stale"
    } else {
        "fresh"
    }
    .to_string()
}

fn sanitized_error(code: Option<&str>) -> Option<String> {
    let code = code?.to_ascii_lowercase();
    Some(
        if code.contains("auth") || code.contains("credential") || code.contains("permission") {
            "authorization"
        } else if code.contains("rate") {
            "rate_limited"
        } else if code.contains("network") || code.contains("http") || code.contains("timeout") {
            "network"
        } else if code.contains("parse") || code.contains("invalid") {
            "invalid_data"
        } else {
            "sync_error"
        }
        .to_string(),
    )
}

fn is_active(event: &PulseEvent, now: DateTime<Utc>) -> bool {
    if event.cancelled || event.time_kind != "timed" {
        return false;
    }
    match (
        parse_instant(event.start_at.as_deref()),
        parse_instant(event.end_at.as_deref()),
    ) {
        (Some(start), Some(end)) => start <= now && now < end,
        _ => false,
    }
}

fn is_future_timed(event: &PulseEvent, now: DateTime<Utc>) -> bool {
    !event.cancelled
        && event.time_kind == "timed"
        && parse_instant(event.start_at.as_deref()).is_some_and(|start| start > now)
}

fn parse_instant(value: Option<&str>) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value?)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn event_order(left: &PulseEvent, right: &PulseEvent) -> std::cmp::Ordering {
    left.start_at
        .cmp(&right.start_at)
        .then_with(|| left.end_at.cmp(&right.end_at))
        .then_with(|| left.id.cmp(&right.id))
}

fn conflicts(events: &[PulseEvent]) -> Vec<PulseConflict> {
    let mut timed = events
        .iter()
        .filter(|event| !event.cancelled && event.time_kind == "timed")
        .cloned()
        .collect::<Vec<_>>();
    timed.sort_by(event_order);
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for (index, first) in timed.iter().enumerate() {
        let (Some(first_start), Some(first_end)) = (
            parse_instant(first.start_at.as_deref()),
            parse_instant(first.end_at.as_deref()),
        ) else {
            continue;
        };
        for second in timed.iter().skip(index + 1) {
            let (Some(second_start), Some(second_end)) = (
                parse_instant(second.start_at.as_deref()),
                parse_instant(second.end_at.as_deref()),
            ) else {
                continue;
            };
            if second_start >= first_end {
                break;
            }
            if first.id == second.id || !(first_start < second_end && second_start < first_end) {
                continue;
            }
            let id = format!("{}:{}", first.id, second.id);
            if seen.insert(id.clone()) {
                result.push(PulseConflict {
                    id,
                    first: first.clone(),
                    second: second.clone(),
                });
                if result.len() == CONFLICT_LIMIT {
                    return result;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn database() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        migrations::run(&connection).unwrap();
        connection
    }

    fn bind_school(connection: &Connection, school: &str, source: &str, group: &str) {
        connection
            .execute(
                "INSERT INTO spaces(id,name,template_type) VALUES (?1,?1,'school')",
                [school],
            )
            .unwrap();
        connection.execute("INSERT INTO integrations(id,provider_id,auth_type,connection_status,sync_status,last_successful_sync_at) VALUES (?1,'my_timetable','ics_feed','connected','idle','2026-09-26T09:30:00Z')", [source]).unwrap();
        connection.execute("INSERT INTO subscribed_calendars(id,connection_id,display_name) VALUES (?1||'-calendar',?1,?1||' timetable')", [source]).unwrap();
        connection
            .execute(
                "INSERT INTO school_space_sources(school_space_id,connection_id) VALUES (?1,?2)",
                params![school, source],
            )
            .unwrap();
        connection.execute("INSERT INTO school_space_source_groups(school_space_id,connection_id,group_reference) VALUES (?1,?2,?3)", params![school,source,group]).unwrap();
    }

    fn event(
        connection: &Connection,
        id: &str,
        source: &str,
        group: &str,
        start: &str,
        end: &str,
        status: &str,
    ) {
        connection.execute("INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_at_utc,end_at_utc,timezone,group_references_json,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at) VALUES (?1,?2,?1,?1,'timed',?3,?4,'Europe/Berlin',json_array(?5),'lesson',?6,'ics','1',?1,'2026-09-26T09:00:00Z','2026-09-26T09:00:00Z','2026-09-26T09:00:00Z')", params![id,source,start,end,group,status]).unwrap();
    }

    fn all_day(
        connection: &Connection,
        id: &str,
        source: &str,
        group: &str,
        start: &str,
        end: &str,
    ) {
        connection.execute("INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_date,end_date,timezone,group_references_json,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at) VALUES (?1,?2,?1,?1,'all_day',?3,?4,'Europe/Berlin',json_array(?5),'lesson','active','ics','1',?1,'2026-09-26T09:00:00Z','2026-09-26T09:00:00Z','2026-09-26T09:00:00Z')", params![id,source,start,end,group]).unwrap();
    }

    #[test]
    fn empty_snapshot_is_truthful_and_bounded() {
        let snapshot = get_at(
            &database(),
            SnapshotClock::utc("2026-09-26T10:00:00Z", "2026-09-26"),
        )
        .unwrap();
        assert!(snapshot.now.is_empty());
        assert!(snapshot.next.is_none());
        assert!(snapshot.today.is_empty());
        assert!(!snapshot.academic_deadlines_available);
        assert!(snapshot.issues.is_empty());
    }

    #[test]
    fn now_next_ordering_boundaries_cancellation_and_isolation_are_correct() {
        let connection = database();
        bind_school(&connection, "school-a", "source-a", "SAME");
        bind_school(&connection, "school-b", "source-b", "OTHER");
        event(
            &connection,
            "ended",
            "source-a",
            "SAME",
            "2026-09-26T09:00:00Z",
            "2026-09-26T10:00:00Z",
            "active",
        );
        event(
            &connection,
            "active-a",
            "source-a",
            "SAME",
            "2026-09-26T10:00:00Z",
            "2026-09-26T11:00:00Z",
            "active",
        );
        event(
            &connection,
            "active-b",
            "source-a",
            "SAME",
            "2026-09-26T09:30:00Z",
            "2026-09-26T10:30:00Z",
            "active",
        );
        event(
            &connection,
            "cancelled",
            "source-a",
            "SAME",
            "2026-09-26T10:00:00Z",
            "2026-09-26T12:00:00Z",
            "cancelled",
        );
        event(
            &connection,
            "next-b",
            "source-a",
            "SAME",
            "2026-09-26T12:00:00Z",
            "2026-09-26T13:30:00Z",
            "active",
        );
        event(
            &connection,
            "next-a",
            "source-a",
            "SAME",
            "2026-09-26T12:00:00Z",
            "2026-09-26T13:00:00Z",
            "active",
        );
        event(
            &connection,
            "leak",
            "source-b",
            "SAME",
            "2026-09-26T10:00:00Z",
            "2026-09-26T11:00:00Z",
            "active",
        );

        let snapshot = get_at(
            &connection,
            SnapshotClock::utc("2026-09-26T10:00:00Z", "2026-09-26"),
        )
        .unwrap();
        assert_eq!(
            snapshot
                .now
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["active-b", "active-a"]
        );
        assert_eq!(snapshot.next.as_ref().unwrap().id, "next-a");
        assert!(snapshot.today.iter().all(|item| item.id != "leak"));
        assert!(
            snapshot
                .today
                .iter()
                .find(|item| item.id == "cancelled")
                .unwrap()
                .cancelled
        );
    }

    #[test]
    fn unsupported_legacy_calendar_provider_never_enters_pulse() {
        let connection = database();
        connection
            .execute(
                "INSERT INTO integrations(id,provider_id,auth_type,connection_status,sync_status) VALUES ('legacy','retired_calendar','ics_feed','connected','succeeded')",
                [],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO subscribed_calendars(id,connection_id,display_name) VALUES ('legacy-calendar','legacy','Legacy calendar')",
                [],
            )
            .unwrap();
        event(
            &connection,
            "legacy-event",
            "legacy",
            "IGNORED",
            "2026-09-26T10:00:00Z",
            "2026-09-26T11:00:00Z",
            "active",
        );

        let snapshot = get_at(
            &connection,
            SnapshotClock::utc("2026-09-26T10:30:00Z", "2026-09-26"),
        )
        .unwrap();
        assert!(snapshot
            .today
            .iter()
            .all(|event| event.id != "legacy-event"));
        assert!(snapshot
            .trust
            .iter()
            .all(|source| source.source_id != "legacy"));
    }

    #[test]
    fn conflicts_exclude_touching_cancelled_all_day_and_duplicate_pairs() {
        let base = |id: &str, start: &str, end: &str| PulseEvent {
            id: id.into(),
            source_id: "s".into(),
            source_label: "School".into(),
            source_type: "school_calendar".into(),
            title: id.into(),
            time_kind: "timed".into(),
            start_at: Some(start.into()),
            end_at: Some(end.into()),
            start_date: None,
            end_date: None,
            location: None,
            cancelled: false,
        };
        let first = base("a", "2026-09-26T10:00:00Z", "2026-09-26T11:00:00Z");
        let overlap = base("b", "2026-09-26T10:30:00Z", "2026-09-26T11:30:00Z");
        let touching = base("c", "2026-09-26T11:30:00Z", "2026-09-26T12:00:00Z");
        let mut cancelled = base("d", "2026-09-26T10:15:00Z", "2026-09-26T10:45:00Z");
        cancelled.cancelled = true;
        let result = conflicts(&[first, overlap, touching, cancelled]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "a:b");
    }

    #[test]
    fn tasks_are_categorized_ordered_and_capped() {
        let connection = database();
        for index in 0..25 {
            connection.execute("INSERT INTO tasks(id,title,status,priority,due_date) VALUES (?1,?2,'planned',?3,?4)", params![format!("t-{index:02}"),format!("Task {index}"),if index==0 {"high"} else {"none"},if index<2 {"2026-09-25"} else if index<4 {"2026-09-26"} else {"2026-09-27"}]).unwrap();
        }
        connection.execute("INSERT INTO tasks(id,title,status,priority,due_date,completed_at) VALUES ('done','Done','done','high','2026-09-25','2026-09-25')", []).unwrap();
        let tasks = list_tasks(
            &connection,
            NaiveDate::from_ymd_opt(2026, 9, 26).unwrap(),
            TASK_LIMIT,
        )
        .unwrap();
        assert_eq!(tasks.len(), 20);
        assert_eq!(tasks[0].category, "overdue");
        assert_eq!(tasks[2].category, "today");
        assert!(tasks.iter().all(|item| item.id != "done"));
    }

    #[test]
    fn trust_states_cover_policy_and_hide_raw_error_details() {
        let now = DateTime::parse_from_rfc3339("2026-09-26T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(freshness(None, now), "never_synced");
        assert_eq!(freshness(Some("2026-09-26T11:00:00Z"), now), "fresh");
        assert_eq!(freshness(Some("2026-09-26T08:00:00Z"), now), "stale");
        assert_eq!(trust_state(false, "connected", "idle", "fresh"), "disabled");
        assert_eq!(
            trust_state(true, "disconnected", "idle", "fresh"),
            "disconnected"
        );
        assert_eq!(
            trust_state(true, "connected", "syncing", "stale"),
            "syncing"
        );
        assert_eq!(trust_state(true, "degraded", "failed", "fresh"), "degraded");
        assert_eq!(
            sanitized_error(Some("http_body_contained_private_detail")).as_deref(),
            Some("network")
        );
    }

    #[test]
    fn serialized_projection_contains_no_broad_persistence_fields() {
        let connection = database();
        bind_school(&connection, "school", "source", "G");
        event(
            &connection,
            "safe",
            "source",
            "G",
            "2026-09-26T10:00:00Z",
            "2026-09-26T11:00:00Z",
            "active",
        );
        let value = serde_json::to_value(
            get_at(
                &connection,
                SnapshotClock::utc("2026-09-26T10:00:00Z", "2026-09-26"),
            )
            .unwrap(),
        )
        .unwrap();
        let serialized = value.to_string();
        for forbidden in [
            "description",
            "sourceUrl",
            "credential",
            "syncConfig",
            "contentHash",
            "rawPayload",
            "lastSyncErrorMessage",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "projection leaked {forbidden}"
            );
        }
    }

    #[test]
    fn all_day_and_cross_midnight_events_preserve_semantics() {
        let connection = database();
        bind_school(&connection, "school", "source", "G");
        all_day(
            &connection,
            "all-day",
            "source",
            "G",
            "2026-09-26",
            "2026-09-27",
        );
        event(
            &connection,
            "overnight",
            "source",
            "G",
            "2026-09-25T23:30:00Z",
            "2026-09-26T00:30:00Z",
            "active",
        );
        let snapshot = get_at(
            &connection,
            SnapshotClock::utc("2026-09-26T00:00:00Z", "2026-09-26"),
        )
        .unwrap();
        assert!(snapshot
            .today
            .iter()
            .any(|item| item.id == "all-day" && item.start_date.as_deref() == Some("2026-09-26")));
        assert!(snapshot.now.iter().any(|item| item.id == "overnight"));
        assert!(snapshot.now.iter().all(|item| item.id != "all-day"));
    }

    #[test]
    fn event_and_continuity_results_respect_hard_caps() {
        let connection = database();
        bind_school(&connection, "school", "source", "G");
        for index in 0..40 {
            event(
                &connection,
                &format!("event-{index:02}"),
                "source",
                "G",
                "2026-09-27T10:00:00Z",
                "2026-09-27T11:00:00Z",
                "active",
            );
            connection
                .execute(
                    "INSERT INTO spaces(id,name,last_opened_at) VALUES (?1,?2,?3)",
                    params![
                        format!("space-{index:02}"),
                        format!("Space {index}"),
                        format!("2026-09-{:02} 10:00:00", (index % 25) + 1)
                    ],
                )
                .unwrap();
        }
        let snapshot = get_at(
            &connection,
            SnapshotClock::utc("2026-09-26T10:00:00Z", "2026-09-26"),
        )
        .unwrap();
        assert_eq!(snapshot.upcoming.len(), UPCOMING_LIMIT as usize);
        assert_eq!(snapshot.continuity.len(), CONTINUITY_LIMIT as usize);
        assert_eq!(snapshot.upcoming.first().unwrap().id, "event-00");
    }

    #[test]
    fn optional_section_failure_does_not_blank_independent_sections() {
        let connection = database();
        bind_school(&connection, "school", "source", "G");
        event(
            &connection,
            "class",
            "source",
            "G",
            "2026-09-26T10:00:00Z",
            "2026-09-26T11:00:00Z",
            "active",
        );
        connection.execute("DROP TABLE tasks", []).unwrap();
        let snapshot = get_at(
            &connection,
            SnapshotClock::utc("2026-09-26T10:00:00Z", "2026-09-26"),
        )
        .unwrap();
        assert_eq!(snapshot.now[0].id, "class");
        assert!(snapshot.tasks.is_empty());
        assert!(snapshot.issues.iter().any(|issue| issue.section == "tasks"));
        assert!(snapshot
            .issues
            .iter()
            .any(|issue| issue.section == "continuity"));
        assert!(snapshot
            .issues
            .iter()
            .all(|issue| !issue.message.contains("SQL")));
    }

    #[test]
    fn disabled_source_with_cached_data_is_labeled_truthfully() {
        let connection = database();
        bind_school(&connection, "school", "source", "G");
        event(
            &connection,
            "cached",
            "source",
            "G",
            "2026-09-26T10:00:00Z",
            "2026-09-26T11:00:00Z",
            "active",
        );
        connection
            .execute("UPDATE integrations SET enabled=0 WHERE id='source'", [])
            .unwrap();
        let now = DateTime::parse_from_rfc3339("2026-09-26T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let trust = list_trust(&connection, now).unwrap();
        assert_eq!(trust[0].state, "disabled");
        assert!(trust[0].showing_cached_data);
    }

    #[test]
    fn dst_local_day_boundaries_keep_calendar_dates_authoritative() {
        use chrono_tz::Europe::Berlin;

        let spring = Berlin
            .with_ymd_and_hms(2026, 3, 29, 0, 0, 0)
            .single()
            .unwrap();
        let spring_next = Berlin
            .with_ymd_and_hms(2026, 3, 30, 0, 0, 0)
            .single()
            .unwrap();
        let autumn = Berlin
            .with_ymd_and_hms(2026, 10, 25, 0, 0, 0)
            .single()
            .unwrap();
        let autumn_next = Berlin
            .with_ymd_and_hms(2026, 10, 26, 0, 0, 0)
            .single()
            .unwrap();

        assert_eq!((spring_next - spring).num_hours(), 23);
        assert_eq!((autumn_next - autumn).num_hours(), 25);
    }
}
