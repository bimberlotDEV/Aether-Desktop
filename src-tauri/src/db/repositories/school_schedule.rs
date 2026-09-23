use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::external_events::ExternalEvent;

const SCHOOL_PROVIDER_ID: &str = "my_timetable";
const EVENT_COLUMNS: &str = "e.id, e.connection_id, e.external_id, e.occurrence_id, e.title, e.description, e.time_kind, e.start_at_utc, e.end_at_utc, e.start_date, e.end_date, e.timezone, e.location, e.course_reference, e.group_references_json, e.event_kind, e.status, e.source_url, e.ingestion_provenance, e.source_version, e.content_hash, e.first_seen_at, e.last_seen_at, e.synchronized_at, e.created_at, e.updated_at";
const SELECTED_GROUP_KEY: &str = "schoolGroup";

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
    pub group_options: Vec<String>,
    pub selected_group: Option<String>,
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

fn settings(value: Option<String>) -> Result<Map<String, Value>, String> {
    match value {
        None => Ok(Map::new()),
        Some(value) => serde_json::from_str::<Value>(&value)
            .map_err(|_| "School Space settings are not valid JSON".to_string())?
            .as_object()
            .cloned()
            .ok_or_else(|| "School Space settings must be a JSON object".to_string()),
    }
}

fn school_settings(conn: &Connection, space_id: &str) -> Result<Map<String, Value>, String> {
    let value = conn
        .query_row(
            "SELECT settings_json FROM spaces WHERE id=?1 AND template_type='school' AND parent_space_id IS NULL AND archived_at IS NULL",
            [space_id.trim()],
            |row| row.get::<_, Option<String>>(0),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                "School schedule is available only for an active parent School Space".into()
            }
            _ => format!("School Space lookup error: {error}"),
        })?;
    settings(value)
}

fn group_options(conn: &Connection) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare(
            "SELECT DISTINCT CAST(groups.value AS TEXT)
             FROM external_events e
             JOIN integrations i ON i.id=e.connection_id
             JOIN subscribed_calendars sc ON sc.connection_id=i.id
             JOIN json_each(e.group_references_json) groups
             WHERE i.provider_id=?1 AND e.status!='removed'
               AND json_type(e.group_references_json)='array'
               AND length(trim(CAST(groups.value AS TEXT))) BETWEEN 1 AND 200
             ORDER BY CAST(groups.value AS TEXT) COLLATE NOCASE
             LIMIT 500",
        )
        .map_err(|error| format!("School group query error: {error}"))?;
    let options = statement
        .query_map([SCHOOL_PROVIDER_ID], |row| row.get(0))
        .map_err(|error| format!("School group query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School group row error: {error}"))?;
    Ok(options)
}

/// Returns a bounded, local-only School schedule. Provider transport remains outside this read
/// path: the fixed provider association is used solely to prevent unrelated calendars leaking in.
pub fn get(conn: &Connection, request: &SchoolScheduleRequest) -> Result<SchoolSchedule, String> {
    let settings = school_settings(conn, &request.space_id)?;
    let selected_group = settings
        .get(SELECTED_GROUP_KEY)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

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

    let group_options = group_options(conn)?;

    let Some(selected_group_value) = selected_group.as_deref() else {
        return Ok(SchoolSchedule {
            events: Vec::new(),
            sources,
            group_options,
            selected_group,
        });
    };

    let mut event_statement = conn
        .prepare(&format!(
            "SELECT {EVENT_COLUMNS}
             FROM external_events e
             JOIN integrations i ON i.id=e.connection_id
             JOIN subscribed_calendars sc ON sc.connection_id=i.id
             WHERE i.provider_id=?1 AND e.status!='removed'
               AND EXISTS (SELECT 1 FROM json_each(e.group_references_json) groups WHERE groups.value=?6)
               AND ((e.time_kind='timed' AND e.start_at_utc<?3 AND e.end_at_utc>?2)
                 OR (e.time_kind='all_day' AND e.start_date<?5 AND e.end_date>?4))
             ORDER BY CASE WHEN e.time_kind='all_day' THEN e.start_date || 'T00:00:00Z' ELSE e.start_at_utc END, e.id
             LIMIT ?7"
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
                selected_group_value,
                limit
            ],
            event_row,
        )
        .map_err(|error| format!("School event query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School event row error: {error}"))?;

    Ok(SchoolSchedule {
        events,
        sources,
        group_options,
        selected_group,
    })
}

pub fn set_selected_group(
    conn: &Connection,
    space_id: &str,
    selected_group: Option<&str>,
) -> Result<(), String> {
    let mut settings = school_settings(conn, space_id)?;
    match selected_group
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(group) => {
            if group.chars().count() > 200 || !group_options(conn)?.iter().any(|v| v == group) {
                return Err("Select a group present in the local School timetable".into());
            }
            settings.insert(SELECTED_GROUP_KEY.into(), Value::String(group.into()));
        }
        None => {
            settings.remove(SELECTED_GROUP_KEY);
        }
    }
    let encoded = serde_json::to_string(&settings)
        .map_err(|error| format!("School Space settings serialization error: {error}"))?;
    conn.execute(
        "UPDATE spaces SET settings_json=?2,updated_at=datetime('now') WHERE id=?1",
        params![space_id.trim(), encoded],
    )
    .map_err(|error| format!("School group update error: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::run(&conn).unwrap();
        conn.execute(
            "INSERT INTO spaces(id,name,template_type,settings_json) VALUES ('school','School','school','{\"schoolGroup\":\"ADSAI-ZM-1.a\"}'),('regular','Work','work',NULL)",
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
            "INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_at_utc,end_at_utc,timezone,location,group_references_json,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at)
             VALUES (?1,?2,?1,?3,'timed',?4,?5,'Europe/Berlin',?6,'[\"ADSAI-ZM-1.a\"]','lesson',?7,'ics','1','hash','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z')",
            params![id, source, title, start, end, location, status],
        )
        .unwrap();
    }

    fn all_day(conn: &Connection, id: &str, start: &str, end: &str) {
        conn.execute(
            "INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_date,end_date,timezone,group_references_json,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at)
             VALUES (?1,'school-source',?1,?1,'all_day',?2,?3,'Europe/Berlin','[\"ADSAI-ZM-1.a\"]','general','active','ics','1','hash','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z')",
            params![id, start, end],
        )
        .unwrap();
    }

    fn groups(conn: &Connection, id: &str, values: &[&str]) {
        conn.execute(
            "UPDATE external_events SET group_references_json=?2 WHERE id=?1",
            params![id, serde_json::to_string(values).unwrap()],
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
        let mut ids = result
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, vec!["inside-local-day", "local-day"]);
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

    #[test]
    fn selected_group_includes_multi_group_and_exam_but_excludes_unrelated_groups() {
        let conn = db();
        for (id, title) in [
            ("selected", "Selected lesson"),
            ("shared", "Shared lesson"),
            ("other", "Other lesson"),
            ("exam", "Selected exam"),
        ] {
            timed(
                &conn,
                id,
                "school-source",
                title,
                "2026-09-23T09:00:00Z",
                "2026-09-23T10:00:00Z",
                "active",
                None,
            );
        }
        groups(&conn, "shared", &["ADSAI-DH-1.a", "ADSAI-ZM-1.a"]);
        groups(&conn, "other", &["ADSAI-ZM-2.a"]);
        conn.execute(
            "UPDATE external_events SET event_kind='exam' WHERE id='exam'",
            [],
        )
        .unwrap();

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
        let ids = result
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["exam", "selected", "shared"]);
        assert_eq!(result.selected_group.as_deref(), Some("ADSAI-ZM-1.a"));
        assert_eq!(
            result.group_options,
            vec!["ADSAI-DH-1.a", "ADSAI-ZM-1.a", "ADSAI-ZM-2.a"]
        );
    }

    #[test]
    fn no_selection_returns_setup_data_without_dumping_all_events() {
        let conn = db();
        timed(
            &conn,
            "lesson",
            "school-source",
            "Lesson",
            "2026-09-23T09:00:00Z",
            "2026-09-23T10:00:00Z",
            "active",
            None,
        );
        conn.execute("UPDATE spaces SET settings_json=NULL WHERE id='school'", [])
            .unwrap();
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
        assert!(result.events.is_empty());
        assert_eq!(result.selected_group, None);
        assert_eq!(result.group_options, vec!["ADSAI-ZM-1.a"]);
    }

    #[test]
    fn selected_group_persists_in_space_settings_and_cached_failed_data_respects_it() {
        let conn = db();
        timed(
            &conn,
            "cached-selected",
            "school-source",
            "Cached selected",
            "2026-09-23T09:00:00Z",
            "2026-09-23T10:00:00Z",
            "active",
            None,
        );
        timed(
            &conn,
            "cached-other",
            "school-source",
            "Cached other",
            "2026-09-23T11:00:00Z",
            "2026-09-23T12:00:00Z",
            "active",
            None,
        );
        groups(&conn, "cached-other", &["ADSAI-ZM-2.a"]);
        conn.execute("UPDATE integrations SET enabled=0,connection_status='degraded',sync_status='failed' WHERE id='school-source'", []).unwrap();
        set_selected_group(&conn, "school", Some("ADSAI-ZM-1.a")).unwrap();

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
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].id, "cached-selected");
        assert_eq!(result.selected_group.as_deref(), Some("ADSAI-ZM-1.a"));
        let raw: String = conn
            .query_row(
                "SELECT settings_json FROM spaces WHERE id='school'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&raw).unwrap()[SELECTED_GROUP_KEY],
            "ADSAI-ZM-1.a"
        );
    }

    #[test]
    fn today_week_and_upcoming_ranges_all_apply_the_selected_group() {
        let conn = db();
        for (id, start, end) in [
            (
                "today-selected",
                "2026-09-23T09:00:00Z",
                "2026-09-23T10:00:00Z",
            ),
            (
                "today-other",
                "2026-09-23T11:00:00Z",
                "2026-09-23T12:00:00Z",
            ),
            (
                "week-selected",
                "2026-09-25T09:00:00Z",
                "2026-09-25T10:00:00Z",
            ),
            ("week-other", "2026-09-25T11:00:00Z", "2026-09-25T12:00:00Z"),
            (
                "upcoming-selected",
                "2026-10-15T09:00:00Z",
                "2026-10-15T10:00:00Z",
            ),
            (
                "upcoming-other",
                "2026-10-15T11:00:00Z",
                "2026-10-15T12:00:00Z",
            ),
        ] {
            timed(&conn, id, "school-source", id, start, end, "active", None);
            if id.ends_with("other") {
                groups(&conn, id, &["ADSAI-ZM-2.a"]);
            }
        }

        let ranges = [
            request(
                "2026-09-22T22:00:00Z",
                "2026-09-23T22:00:00Z",
                "2026-09-23",
                "2026-09-24",
            ),
            request(
                "2026-09-20T22:00:00Z",
                "2026-09-27T22:00:00Z",
                "2026-09-21",
                "2026-09-28",
            ),
            request(
                "2026-09-23T08:00:00Z",
                "2026-12-22T23:00:00Z",
                "2026-09-23",
                "2026-12-23",
            ),
        ];
        for value in ranges {
            let result = get(&conn, &value).unwrap();
            assert!(!result.events.is_empty());
            assert!(result
                .events
                .iter()
                .all(|event| event.id.ends_with("selected")));
            assert!(result
                .events
                .iter()
                .all(|event| event.group_references.contains(&"ADSAI-ZM-1.a".into())));
        }
    }
}
