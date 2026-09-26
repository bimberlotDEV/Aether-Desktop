mod errors;
mod execution;
mod registry;
mod scope;

pub use errors::{ToolError, ToolErrorCode};
pub use execution::{
    execute_native_ai_tool, execute_native_ai_tool_named, NativeToolOutput, NativeToolResult,
    ToolExecutionContext,
};
pub use registry::{
    descriptor, native_tool_registry, NativeToolDescriptor, NativeToolId, ToolExecutionType,
    ToolResultLimits, ToolScopeRequirement,
};
pub use scope::ToolScope;

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate, Utc};
    use rusqlite::{params, Connection};
    use serde_json::{json, Value};

    use crate::{ai::privacy::DataClass, db::migrations};

    use super::*;

    fn instant(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    fn context() -> ToolExecutionContext {
        ToolExecutionContext {
            now: instant("2026-09-23T08:30:00Z"),
            local_date: date("2026-09-23"),
        }
    }

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrations::run(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO spaces(id,name,template_type) VALUES
                ('school-a','School A','school'),
                ('school-b','School B','school'),
                ('work','Work','work');
             INSERT INTO integrations(id,provider_id,auth_type,enabled,connection_status,sync_status) VALUES
                ('mtt-a','my_timetable','ics_feed',1,'connected','succeeded'),
                ('mtt-b','my_timetable','ics_feed',1,'connected','succeeded');
             INSERT INTO subscribed_calendars(id,connection_id,display_name) VALUES
                ('calendar-a','mtt-a','Timetable A'),
                ('calendar-b','mtt-b','Timetable B');
             INSERT INTO school_space_sources(school_space_id,connection_id) VALUES
                ('school-a','mtt-a'),('school-b','mtt-b');
             INSERT INTO school_space_source_groups(school_space_id,connection_id,group_reference) VALUES
                ('school-a','mtt-a','SAME'),('school-b','mtt-b','SAME');",
        )
        .unwrap();
        conn
    }

    struct EventSeed<'a> {
        id: &'a str,
        connection: &'a str,
        title: &'a str,
        time_kind: &'a str,
        start: &'a str,
        end: &'a str,
        status: &'a str,
        location: Option<&'a str>,
    }

    fn event(conn: &Connection, seed: EventSeed<'_>) {
        let (start_at, end_at, start_date, end_date) = if seed.time_kind == "timed" {
            (Some(seed.start), Some(seed.end), None, None)
        } else {
            (None, None, Some(seed.start), Some(seed.end))
        };
        conn.execute(
            "INSERT INTO external_events(
                id,connection_id,external_id,title,time_kind,start_at_utc,end_at_utc,
                start_date,end_date,timezone,location,group_references_json,event_kind,status,
                ingestion_provenance,source_version,content_hash,first_seen_at,last_seen_at,synchronized_at
             ) VALUES (?1,?2,?1,?3,?4,?5,?6,?7,?8,'Europe/Berlin',?9,'[\"SAME\"]','lesson',?10,'ics','1','hash','2026-09-23T00:00:00Z','2026-09-23T00:00:00Z','2026-09-23T00:00:00Z')",
            params![
                seed.id,
                seed.connection,
                seed.title,
                seed.time_kind,
                start_at,
                end_at,
                start_date,
                end_date,
                seed.location,
                seed.status
            ],
        )
        .unwrap();
    }

    fn task(
        conn: &Connection,
        id: &str,
        title: &str,
        status: &str,
        priority: &str,
        due_date: Option<&str>,
        description: &str,
    ) {
        let completed_at = (status == "done").then_some("2026-09-20T00:00:00Z");
        conn.execute(
            "INSERT INTO tasks(id,space_id,title,description,status,priority,due_date,completed_at)
             VALUES (?1,'work',?2,?3,?4,?5,?6,?7)",
            params![
                id,
                title,
                description,
                status,
                priority,
                due_date,
                completed_at
            ],
        )
        .unwrap();
    }

    fn calendar_scope(space: &str) -> ToolScope {
        ToolScope::calendar(
            vec![
                NativeToolId::CalendarGetEvents,
                NativeToolId::CalendarGetNextEvent,
            ],
            space,
            instant("2026-09-22T00:00:00Z"),
            instant("2026-10-20T00:00:00Z"),
            date("2026-09-22"),
            date("2026-10-20"),
            50,
        )
        .unwrap()
    }

    fn tasks_scope(allow_overdue: bool, allow_open: bool) -> ToolScope {
        ToolScope::tasks(
            vec![NativeToolId::TasksGetDue, NativeToolId::TasksGetOpen],
            Some((date("2026-09-20"), date("2026-10-20"))),
            allow_overdue,
            allow_open,
            50,
        )
        .unwrap()
    }

    fn execute(
        conn: &Connection,
        id: NativeToolId,
        arguments: Value,
        scope: &ToolScope,
    ) -> Result<NativeToolResult, ToolError> {
        execute_native_ai_tool(conn, id, arguments, scope, context())
    }

    #[test]
    fn registry_is_closed_stable_read_only_and_sensitive() {
        let registry = native_tool_registry();
        assert_eq!(registry.len(), NativeToolId::ALL.len());
        assert_eq!(
            registry
                .iter()
                .map(|item| item.public_name)
                .collect::<Vec<_>>(),
            vec![
                "calendar.get_events",
                "calendar.get_next_event",
                "tasks.get_due",
                "tasks.get_open"
            ]
        );
        for item in registry {
            assert_eq!(item.public_name, item.id.public_name());
            assert_eq!(item.privacy_class, DataClass::Sensitive);
            assert_eq!(item.execution_type, ToolExecutionType::ReadOnlyLocal);
            assert_eq!(item.output_schema_version, 1);
            assert_eq!(item.result_limits.max_serialized_bytes, 64 * 1024);
            assert_eq!(item.input_schema["additionalProperties"], false);
            assert_eq!(item.output_schema["additionalProperties"], false);
            let schema = item.input_schema.to_string();
            for prohibited in [
                "provider",
                "connection",
                "group",
                "sql",
                "table",
                "filesystem",
                "path",
                "url",
            ] {
                assert!(!schema.to_lowercase().contains(prohibited), "{prohibited}");
            }
            assert!(!item.public_name.contains("create"));
            assert!(!item.public_name.contains("update"));
            assert!(!item.public_name.contains("delete"));
        }
        assert_eq!(
            NativeToolId::from_public_name("tasks.delete")
                .unwrap_err()
                .code,
            ToolErrorCode::UnknownTool
        );
        assert!(serde_json::from_str::<NativeToolId>("\"run.shell\"").is_err());
    }

    #[test]
    fn schemas_are_stable_and_versioned() {
        let descriptors = native_tool_registry();
        assert_eq!(
            descriptors[0].input_schema,
            json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["startAt", "endAt", "startDate", "endDate"],
                "properties": {
                    "startAt": { "type": "string", "format": "date-time" },
                    "endAt": { "type": "string", "format": "date-time" },
                    "startDate": { "type": "string", "format": "date" },
                    "endDate": { "type": "string", "format": "date" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50 },
                    "includeAllDay": { "type": "boolean" }
                }
            })
        );
        assert_eq!(
            descriptors[1].input_schema,
            json!({
                "type": "object", "additionalProperties": false
            })
        );
        assert_eq!(descriptors[2].result_limits.max_window_days, Some(31));
        assert_eq!(descriptors[3].result_limits.default_items, 20);
        assert_eq!(descriptors[1].result_limits.max_items, 1);
    }

    #[test]
    fn strict_arguments_reject_unknown_types_limits_dates_and_windows() {
        let conn = db();
        let calendar = calendar_scope("school-a");
        let invalid_calendar = [
            json!({"startAt":"2026-09-23T00:00:00Z","endAt":"2026-09-24T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24","sql":"SELECT *"}),
            json!({"startAt":"bad","endAt":"2026-09-24T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24"}),
            json!({"startAt":"2026-09-24T00:00:00Z","endAt":"2026-09-23T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24"}),
            json!({"startAt":"2026-09-23T00:00:00Z","endAt":"2026-09-24T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24","limit":0}),
            json!({"startAt":"2026-09-23T00:00:00Z","endAt":"2026-09-24T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24","limit":51}),
            json!({"startAt":"2026-09-23T00:00:00Z","endAt":"2026-09-24T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24","includeAllDay":"yes"}),
        ];
        for arguments in invalid_calendar {
            assert!(execute(&conn, NativeToolId::CalendarGetEvents, arguments, &calendar).is_err());
        }
        let oversized = execute(
            &conn,
            NativeToolId::CalendarGetEvents,
            json!({"startAt":"2026-09-01T00:00:00Z","endAt":"2026-10-03T00:00:00Z","startDate":"2026-09-01","endDate":"2026-10-03"}),
            &calendar,
        )
        .unwrap_err();
        assert_eq!(oversized.code, ToolErrorCode::WindowTooLarge);

        let tasks = tasks_scope(true, true);
        for arguments in [
            json!({"startDate":"2026-99-01"}),
            json!({"startDate":"2026-09-23"}),
            json!({"startDate":"2026-09-25","endDate":"2026-09-24"}),
            json!({"limit":-1}),
            json!({"limit":51}),
            json!({"ranking":"ai"}),
        ] {
            assert!(execute(&conn, NativeToolId::TasksGetDue, arguments, &tasks).is_err());
        }
        assert!(execute(&conn, NativeToolId::TasksGetOpen, json!({}), &tasks).is_ok());
        assert!(execute(
            &conn,
            NativeToolId::CalendarGetNextEvent,
            json!({}),
            &calendar
        )
        .is_ok());
    }

    #[test]
    fn native_scope_rejects_ungranted_tools_ranges_and_capability_escalation() {
        let conn = db();
        let calendar = calendar_scope("school-a");
        assert_eq!(
            execute(&conn, NativeToolId::TasksGetOpen, json!({}), &calendar)
                .unwrap_err()
                .code,
            ToolErrorCode::UnauthorizedScope
        );
        assert_eq!(
            execute(
                &conn,
                NativeToolId::CalendarGetEvents,
                json!({"startAt":"2026-09-21T00:00:00Z","endAt":"2026-09-23T00:00:00Z","startDate":"2026-09-22","endDate":"2026-09-23"}),
                &calendar,
            )
            .unwrap_err()
            .code,
            ToolErrorCode::UnauthorizedScope
        );
        assert!(ToolScope::calendar(
            vec![NativeToolId::TasksGetOpen],
            "school-a",
            instant("2026-09-22T00:00:00Z"),
            instant("2026-09-23T00:00:00Z"),
            date("2026-09-22"),
            date("2026-09-23"),
            20,
        )
        .is_err());
        assert!(
            ToolScope::tasks(vec![NativeToolId::TasksGetOpen], None, false, false, 20,).is_err()
        );
        assert_eq!(ToolScope::default(), ToolScope::none());
    }

    #[test]
    fn calendar_projection_is_ordered_minimized_and_space_isolated() {
        let conn = db();
        event(
            &conn,
            EventSeed {
                id: "later",
                connection: "mtt-a",
                title: "Later",
                time_kind: "timed",
                start: "2026-09-23T10:00:00Z",
                end: "2026-09-23T11:00:00Z",
                status: "active",
                location: Some("Room 2"),
            },
        );
        event(
            &conn,
            EventSeed {
                id: "earlier",
                connection: "mtt-a",
                title: "Earlier",
                time_kind: "timed",
                start: "2026-09-23T09:00:00Z",
                end: "2026-09-23T10:00:00Z",
                status: "active",
                location: None,
            },
        );
        event(
            &conn,
            EventSeed {
                id: "all-day",
                connection: "mtt-a",
                title: "All day",
                time_kind: "all_day",
                start: "2026-09-24",
                end: "2026-09-25",
                status: "active",
                location: None,
            },
        );
        event(
            &conn,
            EventSeed {
                id: "cancelled",
                connection: "mtt-a",
                title: "Cancelled",
                time_kind: "timed",
                start: "2026-09-23T08:45:00Z",
                end: "2026-09-23T09:00:00Z",
                status: "cancelled",
                location: None,
            },
        );
        event(
            &conn,
            EventSeed {
                id: "other-space",
                connection: "mtt-b",
                title: "Private B",
                time_kind: "timed",
                start: "2026-09-23T09:30:00Z",
                end: "2026-09-23T10:30:00Z",
                status: "active",
                location: None,
            },
        );
        let result = execute(
            &conn,
            NativeToolId::CalendarGetEvents,
            json!({
                "startAt":"2026-09-23T00:00:00Z",
                "endAt":"2026-09-25T00:00:00Z",
                "startDate":"2026-09-23",
                "endDate":"2026-09-25",
                "includeAllDay":true,
                "limit":50
            }),
            &calendar_scope("school-a"),
        )
        .unwrap();
        let NativeToolOutput::CalendarEvents(output) = result.output else {
            panic!("calendar output expected")
        };
        assert_eq!(
            output
                .events
                .iter()
                .map(|event| event.id.as_str())
                .collect::<Vec<_>>(),
            vec!["earlier", "later", "all-day"]
        );
        assert!(output.events[2].all_day);
        assert!(output.events.iter().all(|event| !event.cancelled));
        let serialized = serde_json::to_value(&output).unwrap();
        let text = serialized.to_string();
        for prohibited in [
            "description",
            "sourceUrl",
            "connectionId",
            "providerId",
            "group",
            "credential",
            "ingestionProvenance",
            "contentHash",
        ] {
            assert!(!text.contains(prohibited), "{prohibited}");
        }

        let without_all_day = execute(
            &conn,
            NativeToolId::CalendarGetEvents,
            json!({
                "startAt":"2026-09-23T00:00:00Z",
                "endAt":"2026-09-25T00:00:00Z",
                "startDate":"2026-09-23",
                "endDate":"2026-09-25",
                "includeAllDay":false
            }),
            &calendar_scope("school-a"),
        )
        .unwrap();
        let NativeToolOutput::CalendarEvents(output) = without_all_day.output else {
            panic!("calendar output expected")
        };
        assert_eq!(output.events.len(), 2);
    }

    #[test]
    fn next_event_matches_pulse_timed_semantics_and_has_clear_no_result() {
        let conn = db();
        event(
            &conn,
            EventSeed {
                id: "all-day",
                connection: "mtt-a",
                title: "All day",
                time_kind: "all_day",
                start: "2026-09-23",
                end: "2026-09-24",
                status: "active",
                location: None,
            },
        );
        event(
            &conn,
            EventSeed {
                id: "past",
                connection: "mtt-a",
                title: "Past",
                time_kind: "timed",
                start: "2026-09-23T07:00:00Z",
                end: "2026-09-23T08:00:00Z",
                status: "active",
                location: None,
            },
        );
        event(
            &conn,
            EventSeed {
                id: "ongoing",
                connection: "mtt-a",
                title: "Ongoing",
                time_kind: "timed",
                start: "2026-09-23T08:00:00Z",
                end: "2026-09-23T08:45:00Z",
                status: "active",
                location: None,
            },
        );
        event(
            &conn,
            EventSeed {
                id: "next",
                connection: "mtt-a",
                title: "Next",
                time_kind: "timed",
                start: "2026-09-23T09:00:00Z",
                end: "2026-09-23T10:00:00Z",
                status: "active",
                location: None,
            },
        );
        let result = execute(
            &conn,
            NativeToolId::CalendarGetNextEvent,
            json!({}),
            &calendar_scope("school-a"),
        )
        .unwrap();
        let NativeToolOutput::CalendarNextEvent(output) = result.output else {
            panic!("next-event output expected")
        };
        assert_eq!(output.event.unwrap().id, "next");

        conn.execute("DELETE FROM external_events WHERE id='next'", [])
            .unwrap();
        let result = execute(
            &conn,
            NativeToolId::CalendarGetNextEvent,
            json!({}),
            &calendar_scope("school-a"),
        )
        .unwrap();
        let NativeToolOutput::CalendarNextEvent(output) = result.output else {
            panic!("next-event output expected")
        };
        assert!(output.event.is_none());
    }

    #[test]
    fn task_tools_cover_due_overdue_open_order_caps_and_minimization() {
        let conn = db();
        task(
            &conn,
            "overdue",
            "Overdue",
            "inbox",
            "low",
            Some("2026-09-22"),
            "private body",
        );
        task(
            &conn,
            "today-low",
            "Today low",
            "planned",
            "low",
            Some("2026-09-23"),
            "private body",
        );
        task(
            &conn,
            "today-high",
            "Today high",
            "in_progress",
            "high",
            Some("2026-09-23"),
            "private body",
        );
        task(
            &conn,
            "future",
            "Future",
            "inbox",
            "none",
            Some("2026-09-27"),
            "private body",
        );
        task(
            &conn,
            "no-due",
            "No due",
            "inbox",
            "medium",
            None,
            "private body",
        );
        task(
            &conn,
            "done",
            "Done",
            "done",
            "high",
            Some("2026-09-23"),
            "private body",
        );

        let scope = tasks_scope(true, true);
        let due = execute(
            &conn,
            NativeToolId::TasksGetDue,
            json!({"startDate":"2026-09-23","endDate":"2026-09-28","includeOverdue":true}),
            &scope,
        )
        .unwrap();
        let NativeToolOutput::Tasks(output) = due.output else {
            panic!("tasks output expected")
        };
        assert_eq!(
            output
                .tasks
                .iter()
                .map(|task| task.id.as_str())
                .collect::<Vec<_>>(),
            vec!["overdue", "today-high", "today-low", "future"]
        );
        assert!(output.tasks.iter().all(|task| !task.completed));
        let serialized = serde_json::to_value(&output).unwrap();
        assert!(!serialized.to_string().contains("private body"));
        assert!(serialized["tasks"][0].get("description").is_none());

        let due_without_overdue = execute(
            &conn,
            NativeToolId::TasksGetDue,
            json!({"startDate":"2026-09-23","endDate":"2026-09-28"}),
            &scope,
        )
        .unwrap();
        let NativeToolOutput::Tasks(output) = due_without_overdue.output else {
            panic!("tasks output expected")
        };
        assert_eq!(output.tasks.len(), 3);

        let open = execute(
            &conn,
            NativeToolId::TasksGetOpen,
            json!({"limit":2}),
            &scope,
        )
        .unwrap();
        let NativeToolOutput::Tasks(output) = open.output else {
            panic!("tasks output expected")
        };
        assert_eq!(output.tasks.len(), 2);
        assert_eq!(output.tasks[0].id, "overdue");
        assert_eq!(output.tasks[1].id, "today-high");
    }

    #[test]
    fn privacy_is_native_owned_and_cannot_be_lowered() {
        let conn = db();
        let scope = tasks_scope(false, true);
        let result = execute(&conn, NativeToolId::TasksGetOpen, json!({}), &scope).unwrap();
        assert_eq!(result.privacy_class, DataClass::Sensitive);
        let lowered = execute(
            &conn,
            NativeToolId::TasksGetOpen,
            json!({"privacyClass":"general"}),
            &scope,
        )
        .unwrap_err();
        assert_eq!(lowered.code, ToolErrorCode::InvalidArguments);
        let serialized = serde_json::to_value(result).unwrap();
        assert_eq!(serialized["privacyClass"], "sensitive");
        assert_eq!(serialized["outputSchemaVersion"], 1);
    }

    #[test]
    fn result_size_cap_fails_without_partial_truncation() {
        let conn = db();
        let huge_location = "x".repeat(66 * 1024);
        event(
            &conn,
            EventSeed {
                id: "huge",
                connection: "mtt-a",
                title: "Huge",
                time_kind: "timed",
                start: "2026-09-23T09:00:00Z",
                end: "2026-09-23T10:00:00Z",
                status: "active",
                location: Some(&huge_location),
            },
        );
        let error = execute(
            &conn,
            NativeToolId::CalendarGetEvents,
            json!({"startAt":"2026-09-23T00:00:00Z","endAt":"2026-09-24T00:00:00Z","startDate":"2026-09-23","endDate":"2026-09-24"}),
            &calendar_scope("school-a"),
        )
        .unwrap_err();
        assert_eq!(error.code, ToolErrorCode::ResultTooLarge);
        assert!(!error.message.contains("SQLite"));
        assert!(!error.message.contains("external_events"));
    }

    #[test]
    fn named_execution_rejects_unknown_ids_before_reading() {
        let conn = db();
        let error = execute_native_ai_tool_named(
            &conn,
            "filesystem.read",
            json!({"path":"C:/secret"}),
            &ToolScope::none(),
            context(),
        )
        .unwrap_err();
        assert_eq!(error.code, ToolErrorCode::UnknownTool);

        conn.execute("DROP TABLE tasks", []).unwrap();
        let error = execute_native_ai_tool(
            &conn,
            NativeToolId::TasksGetOpen,
            json!({}),
            &tasks_scope(false, true),
            context(),
        )
        .unwrap_err();
        assert_eq!(error.code, ToolErrorCode::InternalReadFailed);
        assert_eq!(error.message, "The local data could not be read.");
        assert!(!error.message.contains("SQL"));
        assert!(!error.message.contains("tasks"));
    }
}
