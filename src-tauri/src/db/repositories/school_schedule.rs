use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::external_events::ExternalEvent;

const SCHOOL_PROVIDER_ID: &str = "my_timetable";
const EVENT_COLUMNS: &str = "e.id, e.connection_id, e.external_id, e.occurrence_id, e.title, e.description, e.time_kind, e.start_at_utc, e.end_at_utc, e.start_date, e.end_date, e.timezone, e.location, e.course_reference, e.event_kind, e.status, e.source_url, e.ingestion_provenance, e.source_version, e.content_hash, e.first_seen_at, e.last_seen_at, e.synchronized_at, e.created_at, e.updated_at";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchoolScheduleRequest {
    pub space_id: String,
    pub start_utc: String,
    pub end_utc: String,
    pub start_date: String,
    pub end_date: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SchoolCalendarSource {
    pub connection_id: String,
    pub enabled: bool,
    pub connection_status: String,
    pub sync_status: String,
    pub last_successful_sync_at: Option<String>,
    pub last_sync_error_code: Option<String>,
    pub last_sync_error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SchoolSchedule {
    pub events: Vec<ExternalEvent>,
    pub sources: Vec<SchoolCalendarSource>,
}

fn utc(value: &str, field: &str) -> Result<String, String> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|_| format!("School schedule {field} must be an RFC3339 UTC instant"))?;
    if parsed.offset().local_minus_utc() != 0 {
        return Err(format!("School schedule {field} must be UTC"));
    }
    Ok(parsed
        .with_timezone(&Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

fn date(value: &str, field: &str) -> Result<String, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("School schedule {field} must be YYYY-MM-DD"))?;
    Ok(value.to_string())
}

fn event_row(row: &rusqlite::Row) -> rusqlite::Result<ExternalEvent> {
    let occurrence: String = row.get(3)?;
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
        event_kind: row.get(14)?,
        status: row.get(15)?,
        source_url: row.get(16)?,
        ingestion_provenance: row.get(17)?,
        source_version: row.get(18)?,
        content_hash: row.get(19)?,
        first_seen_at: row.get(20)?,
        last_seen_at: row.get(21)?,
        synchronized_at: row.get(22)?,
        created_at: row.get(23)?,
        updated_at: row.get(24)?,
    })
}

/// Returns a bounded, local-only School schedule. Provider transport remains outside this read
/// path: the fixed provider association is used solely to prevent unrelated calendars leaking in.
pub fn get(conn: &Connection, request: &SchoolScheduleRequest) -> Result<SchoolSchedule, String> {
    let is_school_parent: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM spaces WHERE id=?1 AND template_type='school' AND parent_space_id IS NULL AND archived_at IS NULL)",
            [request.space_id.trim()],
            |row| row.get(0),
        )
        .map_err(|error| format!("School Space lookup error: {error}"))?;
    if !is_school_parent {
        return Err("School schedule is available only for an active parent School Space".into());
    }

    let start_utc = utc(&request.start_utc, "UTC start")?;
    let end_utc = utc(&request.end_utc, "UTC end")?;
    let start_date = date(&request.start_date, "local start date")?;
    let end_date = date(&request.end_date, "local end date")?;
    if start_utc >= end_utc || start_date >= end_date {
        return Err("School schedule range end must be after start".into());
    }
    let limit = request.limit.unwrap_or(250).clamp(1, 500);

    let mut source_statement = conn
        .prepare(
            "SELECT i.id, i.enabled, i.connection_status, i.sync_status, i.last_successful_sync_at, i.last_sync_error_code, i.last_sync_error_message
             FROM integrations i
             JOIN subscribed_calendars sc ON sc.connection_id=i.id
             WHERE i.provider_id=?1 ORDER BY i.created_at, i.id",
        )
        .map_err(|error| format!("School calendar source query error: {error}"))?;
    let sources = source_statement
        .query_map([SCHOOL_PROVIDER_ID], |row| {
            Ok(SchoolCalendarSource {
                connection_id: row.get(0)?,
                enabled: row.get(1)?,
                connection_status: row.get(2)?,
                sync_status: row.get(3)?,
                last_successful_sync_at: row.get(4)?,
                last_sync_error_code: row.get(5)?,
                last_sync_error_message: row.get(6)?,
            })
        })
        .map_err(|error| format!("School calendar source query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School calendar source row error: {error}"))?;

    let mut event_statement = conn
        .prepare(&format!(
            "SELECT {EVENT_COLUMNS}
             FROM external_events e
             JOIN integrations i ON i.id=e.connection_id
             JOIN subscribed_calendars sc ON sc.connection_id=i.id
             WHERE i.provider_id=?1 AND e.status!='removed'
               AND ((e.time_kind='timed' AND e.start_at_utc<?3 AND e.end_at_utc>?2)
                 OR (e.time_kind='all_day' AND e.start_date<?5 AND e.end_date>?4))
             ORDER BY CASE WHEN e.time_kind='all_day' THEN 0 ELSE 1 END,
                      CASE WHEN e.time_kind='all_day' THEN e.start_date || 'T00:00:00' ELSE e.start_at_utc END,
                      e.id
             LIMIT ?6"
        ))
        .map_err(|error| format!("School event query error: {error}"))?;
    let events = event_statement
        .query_map(
            params![
                SCHOOL_PROVIDER_ID,
                start_utc,
                end_utc,
                start_date,
                end_date,
                limit
            ],
            event_row,
        )
        .map_err(|error| format!("School event query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School event row error: {error}"))?;

    Ok(SchoolSchedule { events, sources })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        conn.execute(
            "INSERT INTO spaces(id,name,template_type) VALUES ('school','School','school'),('regular','Work','work')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO integrations(id,provider_id,auth_type,enabled,connection_status,sync_status,last_successful_sync_at) VALUES
             ('school-source','my_timetable','ics_feed',1,'connected','succeeded','2026-09-23T06:00:00Z'),
             ('other-source','calendar','ics_feed',1,'connected','succeeded','2026-09-23T06:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO subscribed_calendars(id,connection_id,display_name) VALUES ('school-calendar','school-source','School')",
            [],
        )
        .unwrap();
        conn
    }

    #[allow(clippy::too_many_arguments)]
    fn timed(
        conn: &Connection,
        id: &str,
        source: &str,
        title: &str,
        start: &str,
        end: &str,
        status: &str,
        location: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_at_utc,end_at_utc,timezone,location,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at)
             VALUES (?1,?2,?1,?3,'timed',?4,?5,'Europe/Berlin',?6,'lesson',?7,'ics','1','hash','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z')",
            params![id, source, title, start, end, location, status],
        )
        .unwrap();
    }

    fn all_day(conn: &Connection, id: &str, start: &str, end: &str) {
        conn.execute(
            "INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_date,end_date,timezone,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at)
             VALUES (?1,'school-source',?1,?1,'all_day',?2,?3,'Europe/Berlin','general','active','ics','1','hash','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z')",
            params![id, start, end],
        )
        .unwrap();
    }

    fn request(
        start_utc: &str,
        end_utc: &str,
        start_date: &str,
        end_date: &str,
    ) -> SchoolScheduleRequest {
        SchoolScheduleRequest {
            space_id: "school".into(),
            start_utc: start_utc.into(),
            end_utc: end_utc.into(),
            start_date: start_date.into(),
            end_date: end_date.into(),
            limit: Some(250),
        }
    }

    #[test]
    fn today_is_chronological_excludes_removed_and_unrelated_calendars() {
        let conn = db();
        timed(
            &conn,
            "late",
            "school-source",
            "Late",
            "2026-09-23T12:00:00Z",
            "2026-09-23T13:00:00Z",
            "active",
            Some("B2"),
        );
        timed(
            &conn,
            "early",
            "school-source",
            "Early",
            "2026-09-23T07:00:00Z",
            "2026-09-23T08:00:00Z",
            "cancelled",
            None,
        );
        timed(
            &conn,
            "removed",
            "school-source",
            "Removed",
            "2026-09-23T09:00:00Z",
            "2026-09-23T10:00:00Z",
            "removed",
            None,
        );
        timed(
            &conn,
            "private",
            "other-source",
            "Private",
            "2026-09-23T08:00:00Z",
            "2026-09-23T09:00:00Z",
            "active",
            None,
        );

        let result = get(
            &conn,
            &request(
                "2026-09-22T22:00:00Z",
                "2026-09-23T22:00:00Z",
                "2026-09-23",
                "2026-09-24",
            ),
        )
        .unwrap();
        assert_eq!(
            result
                .events
                .iter()
                .map(|event| event.id.as_str())
                .collect::<Vec<_>>(),
            vec!["early", "late"]
        );
        assert_eq!(result.events[0].status, "cancelled");
        assert_eq!(result.sources.len(), 1);
        assert_eq!(result.sources[0].connection_id, "school-source");
    }

    #[test]
    fn current_week_returns_overlapping_occurrences_without_hiding_either() {
        let conn = db();
        timed(
            &conn,
            "first",
            "school-source",
            "First",
            "2026-09-23T08:00:00Z",
            "2026-09-23T10:00:00Z",
            "active",
            None,
        );
        timed(
            &conn,
            "overlap",
            "school-source",
            "Overlap",
            "2026-09-23T09:00:00Z",
            "2026-09-23T11:00:00Z",
            "active",
            None,
        );
        let result = get(
            &conn,
            &request(
                "2026-09-20T22:00:00Z",
                "2026-09-27T22:00:00Z",
                "2026-09-21",
                "2026-09-28",
            ),
        )
        .unwrap();
        assert_eq!(result.events.len(), 2);
    }

    #[test]
    fn upcoming_query_is_bounded() {
        let conn = db();
        for hour in 7..12 {
            timed(
                &conn,
                &format!("event-{hour}"),
                "school-source",
                "Lesson",
                &format!("2026-09-24T{hour:02}:00:00Z"),
                &format!("2026-09-24T{hour:02}:30:00Z"),
                "active",
                None,
            );
        }
        let mut value = request(
            "2026-09-23T10:00:00Z",
            "2026-12-22T23:00:00Z",
            "2026-09-23",
            "2026-12-23",
        );
        value.limit = Some(3);
        assert_eq!(get(&conn, &value).unwrap().events.len(), 3);
    }

    #[test]
    fn local_date_bounds_preserve_all_day_and_dst_boundary_semantics() {
        let conn = db();
        all_day(&conn, "local-day", "2026-10-25", "2026-10-26");
        timed(
            &conn,
            "before-local-day",
            "school-source",
            "Before",
            "2026-10-24T21:30:00Z",
            "2026-10-24T21:45:00Z",
            "active",
            None,
        );
        timed(
            &conn,
            "inside-local-day",
            "school-source",
            "Inside",
            "2026-10-24T22:30:00Z",
            "2026-10-24T23:00:00Z",
            "active",
            None,
        );
        let result = get(
            &conn,
            &request(
                "2026-10-24T22:00:00Z",
                "2026-10-25T23:00:00Z",
                "2026-10-25",
                "2026-10-26",
            ),
        )
        .unwrap();
        assert_eq!(
            result
                .events
                .iter()
                .map(|event| event.id.as_str())
                .collect::<Vec<_>>(),
            vec!["local-day", "inside-local-day"]
        );
    }

    #[test]
    fn rejects_non_school_spaces_and_returns_truthful_source_state() {
        let conn = db();
        conn.execute("UPDATE integrations SET enabled=0,connection_status='degraded',sync_status='failed',last_sync_error_code='network',last_sync_error_message='Offline' WHERE id='school-source'", []).unwrap();
        let result = get(
            &conn,
            &request(
                "2026-09-22T22:00:00Z",
                "2026-09-23T22:00:00Z",
                "2026-09-23",
                "2026-09-24",
            ),
        )
        .unwrap();
        assert!(!result.sources[0].enabled);
        assert_eq!(result.sources[0].sync_status, "failed");
        assert_eq!(
            result.sources[0].last_sync_error_code.as_deref(),
            Some("network")
        );
        let mut invalid = request(
            "2026-09-22T22:00:00Z",
            "2026-09-23T22:00:00Z",
            "2026-09-23",
            "2026-09-24",
        );
        invalid.space_id = "regular".into();
        assert!(get(&conn, &invalid).is_err());
    }
}
