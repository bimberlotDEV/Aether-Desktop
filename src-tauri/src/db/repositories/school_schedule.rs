use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::external_events::ExternalEvent;

const MY_TIMETABLE_PROVIDER_ID: &str = "my_timetable";
const EVENT_COLUMNS: &str = "e.id, e.connection_id, e.external_id, e.occurrence_id, e.title, e.description, e.time_kind, e.start_at_utc, e.end_at_utc, e.start_date, e.end_date, e.timezone, e.location, e.course_reference, e.group_references_json, e.event_kind, e.status, e.source_url, e.ingestion_provenance, e.source_version, e.content_hash, e.first_seen_at, e.last_seen_at, e.synchronized_at, e.created_at, e.updated_at";

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
    pub provider_id: String,
    pub display_name: Option<String>,
    pub associated: bool,
    pub enabled: bool,
    pub connection_status: String,
    pub sync_status: String,
    pub last_successful_sync_at: Option<String>,
    pub last_sync_error_code: Option<String>,
    pub last_sync_error_message: Option<String>,
    pub selected_groups: Vec<String>,
    pub group_options: Vec<String>,
    pub group_selection_valid: bool,
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

fn ensure_parent_school(conn: &Connection, space_id: &str) -> Result<String, String> {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return Err("School Space identity is required".into());
    }
    conn.query_row(
        "SELECT id FROM spaces
         WHERE id=?1 AND template_type='school' AND parent_space_id IS NULL
           AND archived_at IS NULL",
        [space_id],
        |row| row.get(0),
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            "School schedule is available only for an active parent School Space".into()
        }
        _ => format!("School Space lookup error: {error}"),
    })
}

fn source_group_options(conn: &Connection, connection_id: &str) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare(
            "SELECT DISTINCT CAST(groups.value AS TEXT)
             FROM external_events e
             JOIN json_each(e.group_references_json) groups
             WHERE e.connection_id=?1 AND e.status!='removed'
               AND json_type(e.group_references_json)='array'
               AND length(trim(CAST(groups.value AS TEXT))) BETWEEN 1 AND 200
             ORDER BY CAST(groups.value AS TEXT) COLLATE NOCASE
             LIMIT 500",
        )
        .map_err(|error| format!("School group query error: {error}"))?;
    let values = statement
        .query_map([connection_id], |row| row.get(0))
        .map_err(|error| format!("School group query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School group row error: {error}"))?;
    Ok(values)
}

fn selected_groups(
    conn: &Connection,
    school_space_id: &str,
    connection_id: &str,
) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare(
            "SELECT group_reference FROM school_space_source_groups
             WHERE school_space_id=?1 AND connection_id=?2
             ORDER BY group_reference COLLATE NOCASE",
        )
        .map_err(|error| format!("School selected-group query error: {error}"))?;
    let values = statement
        .query_map(params![school_space_id, connection_id], |row| row.get(0))
        .map_err(|error| format!("School selected-group query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School selected-group row error: {error}"))?;
    Ok(values)
}

fn sources(conn: &Connection, school_space_id: &str) -> Result<Vec<SchoolCalendarSource>, String> {
    let mut statement = conn
        .prepare(
            "SELECT i.id, i.provider_id, sc.display_name,
                    CASE WHEN binding.connection_id IS NULL THEN 0 ELSE 1 END,
                    i.enabled, i.connection_status, i.sync_status,
                    i.last_successful_sync_at, i.last_sync_error_code,
                    i.last_sync_error_message
             FROM integrations i
             JOIN subscribed_calendars sc ON sc.connection_id=i.id
             LEFT JOIN school_space_sources binding
               ON binding.connection_id=i.id AND binding.school_space_id=?1
             WHERE i.provider_id=?2
             ORDER BY i.provider_id, i.created_at, i.id",
        )
        .map_err(|error| format!("School calendar source query error: {error}"))?;
    let rows = statement
        .query_map(params![school_space_id, MY_TIMETABLE_PROVIDER_ID], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, bool>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<String>>(9)?,
            ))
        })
        .map_err(|error| format!("School calendar source query error: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("School calendar source row error: {error}"))?;

    rows.into_iter()
        .map(
            |(
                connection_id,
                provider_id,
                display_name,
                associated,
                enabled,
                connection_status,
                sync_status,
                last_successful_sync_at,
                last_sync_error_code,
                last_sync_error_message,
            )| {
                let selected_groups = if associated && provider_id == MY_TIMETABLE_PROVIDER_ID {
                    selected_groups(conn, school_space_id, &connection_id)?
                } else {
                    Vec::new()
                };
                let group_options = if associated && provider_id == MY_TIMETABLE_PROVIDER_ID {
                    source_group_options(conn, &connection_id)?
                } else {
                    Vec::new()
                };
                let group_selection_valid = provider_id != MY_TIMETABLE_PROVIDER_ID
                    || (associated
                        && !selected_groups.is_empty()
                        && selected_groups
                            .iter()
                            .all(|selected| group_options.contains(selected)));
                Ok(SchoolCalendarSource {
                    connection_id,
                    provider_id,
                    display_name,
                    associated,
                    enabled,
                    connection_status,
                    sync_status,
                    last_successful_sync_at,
                    last_sync_error_code,
                    last_sync_error_message,
                    selected_groups,
                    group_options,
                    group_selection_valid,
                })
            },
        )
        .collect()
}

/// Returns a bounded local timetable whose authority is derived only from the requested parent
/// School Space. Provider and group labels are filters inside persisted connection bindings, never
/// ownership boundaries.
pub fn get(conn: &Connection, request: &SchoolScheduleRequest) -> Result<SchoolSchedule, String> {
    let school_space_id = ensure_parent_school(conn, &request.space_id)?;
    let start_utc = utc(&request.start_utc, "UTC start")?;
    let end_utc = utc(&request.end_utc, "UTC end")?;
    let start_date = date(&request.start_date, "local start date")?;
    let end_date = date(&request.end_date, "local end date")?;
    if start_utc >= end_utc || start_date >= end_date {
        return Err("School schedule range end must be after start".into());
    }
    let limit = request.limit.unwrap_or(250).clamp(1, 500);
    let sources = sources(conn, &school_space_id)?;

    let mut event_statement = conn
        .prepare(&format!(
            "SELECT DISTINCT {EVENT_COLUMNS}
             FROM school_space_sources binding
             JOIN integrations i ON i.id=binding.connection_id
             JOIN external_events e ON e.connection_id=binding.connection_id
             JOIN json_each(e.group_references_json) event_group
             JOIN school_space_source_groups selected_group
               ON selected_group.school_space_id=binding.school_space_id
              AND selected_group.connection_id=binding.connection_id
              AND selected_group.group_reference=CAST(event_group.value AS TEXT)
             WHERE binding.school_space_id=?1
               AND i.provider_id='my_timetable'
               AND i.connection_status!='disconnected'
               AND e.status!='removed'
               AND json_type(e.group_references_json)='array'
               AND ((e.time_kind='timed' AND e.start_at_utc<?3 AND e.end_at_utc>?2)
                 OR (e.time_kind='all_day' AND e.start_date<?5 AND e.end_date>?4))
             ORDER BY CASE WHEN e.time_kind='all_day' THEN e.start_date || 'T00:00:00Z' ELSE e.start_at_utc END, e.id
             LIMIT ?6"
        ))
        .map_err(|error| format!("School event query error: {error}"))?;
    let events = event_statement
        .query_map(
            params![
                school_space_id,
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

fn ensure_supported_source(conn: &Connection, connection_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT i.provider_id
         FROM integrations i
         JOIN subscribed_calendars sc ON sc.connection_id=i.id
         WHERE i.id=?1 AND i.provider_id=?2",
        params![connection_id.trim(), MY_TIMETABLE_PROVIDER_ID],
        |row| row.get(0),
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            "Select an available School calendar connection".into()
        }
        _ => format!("School calendar source lookup error: {error}"),
    })
}

pub fn set_source_association(
    conn: &Connection,
    space_id: &str,
    connection_id: &str,
    associated: bool,
) -> Result<(), String> {
    let school_space_id = ensure_parent_school(conn, space_id)?;
    let connection_id = connection_id.trim();
    ensure_supported_source(conn, connection_id)?;
    if associated {
        conn.execute(
            "INSERT INTO school_space_sources(school_space_id, connection_id)
             VALUES (?1, ?2)
             ON CONFLICT(school_space_id, connection_id)
             DO UPDATE SET updated_at=datetime('now')",
            params![school_space_id, connection_id],
        )
        .map_err(|error| format!("School source association error: {error}"))?;
    } else {
        conn.execute(
            "DELETE FROM school_space_sources WHERE school_space_id=?1 AND connection_id=?2",
            params![school_space_id, connection_id],
        )
        .map_err(|error| format!("School source removal error: {error}"))?;
    }
    Ok(())
}

pub fn set_selected_groups(
    conn: &Connection,
    space_id: &str,
    connection_id: &str,
    selected_groups: &[String],
) -> Result<(), String> {
    let school_space_id = ensure_parent_school(conn, space_id)?;
    let connection_id = connection_id.trim();
    let provider_id = ensure_supported_source(conn, connection_id)?;
    if provider_id != MY_TIMETABLE_PROVIDER_ID {
        return Err("Timetable groups are available only for MyTimetable sources".into());
    }
    let binding_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM school_space_sources
             WHERE school_space_id=?1 AND connection_id=?2",
            params![school_space_id, connection_id],
            |row| row.get(0),
        )
        .map_err(|error| format!("School source association lookup error: {error}"))?;
    if binding_count != 1 {
        return Err("Associate the MyTimetable source before selecting a group".into());
    }
    if selected_groups.len() > 32 {
        return Err("Select at most 32 School groups".into());
    }
    let mut normalized = selected_groups
        .iter()
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    if normalized
        .iter()
        .any(|value| value.is_empty() || value.chars().count() > 200)
    {
        return Err("School groups must contain 1 to 200 characters".into());
    }
    normalized.sort();
    normalized.dedup();
    let options = source_group_options(conn, connection_id)?;
    if normalized.iter().any(|group| !options.contains(group)) {
        return Err("Select a group present in this MyTimetable source".into());
    }

    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| format!("School group transaction error: {error}"))?;
    transaction
        .execute(
            "DELETE FROM school_space_source_groups
             WHERE school_space_id=?1 AND connection_id=?2",
            params![school_space_id, connection_id],
        )
        .map_err(|error| format!("School group update error: {error}"))?;
    for group in normalized {
        transaction
            .execute(
                "INSERT INTO school_space_source_groups
                    (school_space_id, connection_id, group_reference)
                 VALUES (?1, ?2, ?3)",
                params![school_space_id, connection_id, group],
            )
            .map_err(|error| format!("School group update error: {error}"))?;
    }
    transaction
        .commit()
        .map_err(|error| format!("School group commit error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn seed(conn: &Connection) {
        conn.execute_batch(
            "INSERT INTO spaces(id,name,template_type) VALUES
                ('school-a','School A','school'),
                ('school-b','School B','school'),
                ('regular','Work','work'),
                ('subject','Math','subject');
             UPDATE spaces SET parent_space_id='school-a' WHERE id='subject';
             INSERT INTO integrations(id,provider_id,auth_type,enabled,connection_status,sync_status,last_successful_sync_at) VALUES
                ('mtt-x','my_timetable','ics_feed',1,'connected','succeeded','2026-09-23T06:00:00Z'),
                ('mtt-y','my_timetable','ics_feed',1,'connected','succeeded','2026-09-23T06:00:00Z'),
                ('bsp','brightspace','ics_feed',1,'connected','succeeded','2026-09-23T06:00:00Z');
             INSERT INTO subscribed_calendars(id,connection_id,display_name) VALUES
                ('calendar-x','mtt-x','MyTimetable'),
                ('calendar-y','mtt-y','MyTimetable'),
                ('calendar-bsp','bsp','Brightspace');",
        )
        .unwrap();
    }

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        seed(&conn);
        conn
    }

    fn timed(conn: &Connection, id: &str, source: &str, group_values: &[&str]) {
        conn.execute(
            "INSERT INTO external_events(id,connection_id,external_id,title,time_kind,start_at_utc,end_at_utc,timezone,group_references_json,event_kind,status,ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at)
             VALUES (?1,?2,?1,?1,'timed','2026-09-23T09:00:00Z','2026-09-23T10:00:00Z','Europe/Berlin',?3,'lesson','active','ics','1','hash','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z','2026-09-23T06:00:00Z')",
            params![id, source, serde_json::to_string(group_values).unwrap()],
        )
        .unwrap();
    }

    fn request(space_id: &str) -> SchoolScheduleRequest {
        SchoolScheduleRequest {
            space_id: space_id.into(),
            start_utc: "2026-09-22T22:00:00Z".into(),
            end_utc: "2026-09-23T22:00:00Z".into(),
            start_date: "2026-09-23".into(),
            end_date: "2026-09-24".into(),
            limit: Some(250),
        }
    }

    fn associate(conn: &Connection, school: &str, source: &str, group: &str) {
        set_source_association(conn, school, source, true).unwrap();
        set_selected_groups(conn, school, source, &[group.into()]).unwrap();
    }

    #[test]
    fn no_association_returns_zero_events_and_no_group_options() {
        let conn = db();
        timed(&conn, "x", "mtt-x", &["A"]);
        let result = get(&conn, &request("school-a")).unwrap();
        assert!(result.events.is_empty());
        assert!(result
            .sources
            .iter()
            .all(|source| !source.associated && source.group_options.is_empty()));
    }

    #[test]
    fn same_group_name_cannot_cross_connection_or_space_boundaries() {
        let conn = db();
        timed(&conn, "x-only", "mtt-x", &["SAME"]);
        timed(&conn, "y-only", "mtt-y", &["SAME"]);
        associate(&conn, "school-a", "mtt-x", "SAME");
        associate(&conn, "school-b", "mtt-y", "SAME");

        assert_eq!(
            get(&conn, &request("school-a")).unwrap().events[0].id,
            "x-only"
        );
        assert_eq!(
            get(&conn, &request("school-b")).unwrap().events[0].id,
            "y-only"
        );

        set_selected_groups(&conn, "school-a", "mtt-x", &[]).unwrap();
        assert!(get(&conn, &request("school-a")).unwrap().events.is_empty());
        assert_eq!(
            get(&conn, &request("school-b")).unwrap().events[0].id,
            "y-only"
        );
    }

    #[test]
    fn shared_event_within_one_source_appears_once() {
        let conn = db();
        timed(&conn, "shared", "mtt-x", &["A", "B"]);
        set_source_association(&conn, "school-a", "mtt-x", true).unwrap();
        set_selected_groups(&conn, "school-a", "mtt-x", &["A".into(), "B".into()]).unwrap();
        assert_eq!(get(&conn, &request("school-a")).unwrap().events.len(), 1);
    }

    #[test]
    fn one_space_can_read_multiple_explicitly_bound_mytimetable_sources() {
        let conn = db();
        timed(&conn, "x", "mtt-x", &["A"]);
        timed(&conn, "y", "mtt-y", &["B"]);
        associate(&conn, "school-a", "mtt-x", "A");
        associate(&conn, "school-a", "mtt-y", "B");

        let result = get(&conn, &request("school-a")).unwrap();
        assert_eq!(
            result
                .events
                .iter()
                .map(|event| event.id.as_str())
                .collect::<Vec<_>>(),
            vec!["x", "y"]
        );
        let x = result
            .sources
            .iter()
            .find(|source| source.connection_id == "mtt-x")
            .unwrap();
        let y = result
            .sources
            .iter()
            .find(|source| source.connection_id == "mtt-y")
            .unwrap();
        assert_eq!(x.group_options, vec!["A"]);
        assert_eq!(y.group_options, vec!["B"]);
    }

    #[test]
    fn source_change_clears_only_that_spaces_group_selection() {
        let conn = db();
        timed(&conn, "x", "mtt-x", &["A"]);
        timed(&conn, "y", "mtt-y", &["B"]);
        associate(&conn, "school-a", "mtt-x", "A");
        associate(&conn, "school-b", "mtt-y", "B");
        set_source_association(&conn, "school-a", "mtt-x", false).unwrap();
        set_source_association(&conn, "school-a", "mtt-y", true).unwrap();

        let a = get(&conn, &request("school-a")).unwrap();
        assert!(a.events.is_empty());
        assert!(a
            .sources
            .iter()
            .find(|source| source.connection_id == "mtt-y")
            .unwrap()
            .selected_groups
            .is_empty());
        assert_eq!(get(&conn, &request("school-b")).unwrap().events[0].id, "y");
    }

    #[test]
    fn disabled_source_remains_bound_without_alternate_fallback() {
        let conn = db();
        timed(&conn, "cached-x", "mtt-x", &["A"]);
        timed(&conn, "unrelated-y", "mtt-y", &["A"]);
        associate(&conn, "school-a", "mtt-x", "A");
        conn.execute(
            "UPDATE integrations SET enabled=0,connection_status='degraded',sync_status='failed' WHERE id='mtt-x'",
            [],
        )
        .unwrap();
        let result = get(&conn, &request("school-a")).unwrap();
        assert_eq!(result.events[0].id, "cached-x");
        let source = result
            .sources
            .iter()
            .find(|source| source.associated)
            .unwrap();
        assert!(!source.enabled);
        assert_eq!(source.sync_status, "failed");
    }

    #[test]
    fn deleted_source_cascades_binding_and_never_falls_back() {
        let conn = db();
        timed(&conn, "x", "mtt-x", &["A"]);
        timed(&conn, "y", "mtt-y", &["A"]);
        associate(&conn, "school-a", "mtt-x", "A");
        conn.execute("DELETE FROM integrations WHERE id='mtt-x'", [])
            .unwrap();
        let result = get(&conn, &request("school-a")).unwrap();
        assert!(result.events.is_empty());
        assert!(result.sources.iter().all(|source| !source.associated));
    }

    #[test]
    fn disconnected_source_returns_no_cached_or_alternate_events() {
        let conn = db();
        timed(&conn, "cached-x", "mtt-x", &["A"]);
        timed(&conn, "unrelated-y", "mtt-y", &["A"]);
        associate(&conn, "school-a", "mtt-x", "A");
        conn.execute(
            "UPDATE integrations SET connection_status='disconnected',sync_status='idle' WHERE id='mtt-x'",
            [],
        )
        .unwrap();

        let result = get(&conn, &request("school-a")).unwrap();
        assert!(result.events.is_empty());
        let source = result
            .sources
            .iter()
            .find(|source| source.associated)
            .unwrap();
        assert_eq!(source.connection_status, "disconnected");
    }

    #[test]
    fn same_connection_feed_replacement_state_preserves_binding() {
        let conn = db();
        timed(&conn, "x", "mtt-x", &["A"]);
        associate(&conn, "school-a", "mtt-x", "A");
        conn.execute(
            "UPDATE integrations SET configuration_generation=configuration_generation+1 WHERE id='mtt-x'",
            [],
        )
        .unwrap();
        assert_eq!(get(&conn, &request("school-a")).unwrap().events[0].id, "x");
    }

    #[test]
    fn legacy_brightspace_binding_is_hidden_and_adds_no_timetable_events() {
        let conn = db();
        timed(&conn, "brightspace-general", "bsp", &[]);
        timed(&conn, "mtt-lesson", "mtt-x", &["A"]);
        conn.execute(
            "INSERT INTO school_space_sources(school_space_id,connection_id) VALUES ('school-a','bsp')",
            [],
        )
        .unwrap();
        associate(&conn, "school-a", "mtt-x", "A");
        let result = get(&conn, &request("school-a")).unwrap();
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].id, "mtt-lesson");
        assert!(result
            .sources
            .iter()
            .all(|source| source.connection_id != "bsp"));
        assert!(set_source_association(&conn, "school-a", "bsp", true).is_err());
        assert!(set_selected_groups(&conn, "school-a", "bsp", &["fake".into()]).is_err());
    }

    #[test]
    fn non_school_and_subject_child_cannot_read_or_own_sources() {
        let conn = db();
        for space in ["regular", "subject"] {
            assert!(get(&conn, &request(space)).is_err());
            assert!(set_source_association(&conn, space, "mtt-x", true).is_err());
        }
    }

    #[test]
    fn invalid_persisted_group_is_truthful_and_returns_no_events() {
        let conn = db();
        timed(&conn, "x", "mtt-x", &["A"]);
        set_source_association(&conn, "school-a", "mtt-x", true).unwrap();
        conn.execute(
            "INSERT INTO school_space_source_groups(school_space_id,connection_id,group_reference) VALUES ('school-a','mtt-x','MISSING')",
            [],
        )
        .unwrap();
        let result = get(&conn, &request("school-a")).unwrap();
        assert!(result.events.is_empty());
        let source = result
            .sources
            .iter()
            .find(|source| source.associated)
            .unwrap();
        assert!(!source.group_selection_valid);
        assert_eq!(source.selected_groups, vec!["MISSING"]);
    }

    #[test]
    fn source_association_and_group_selection_persist_after_restart() {
        let temporary = tempfile::NamedTempFile::new().unwrap();
        {
            let conn = Connection::open(temporary.path()).unwrap();
            conn.pragma_update(None, "foreign_keys", "ON").unwrap();
            migrations::run(&conn).unwrap();
            seed(&conn);
            timed(&conn, "persisted", "mtt-x", &["A"]);
            associate(&conn, "school-a", "mtt-x", "A");
        }

        let reopened = Connection::open(temporary.path()).unwrap();
        reopened.pragma_update(None, "foreign_keys", "ON").unwrap();
        let result = get(&reopened, &request("school-a")).unwrap();
        assert_eq!(result.events[0].id, "persisted");
        let source = result
            .sources
            .iter()
            .find(|source| source.connection_id == "mtt-x")
            .unwrap();
        assert!(source.associated);
        assert_eq!(source.selected_groups, vec!["A"]);
    }
}
