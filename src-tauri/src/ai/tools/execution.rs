use chrono::{DateTime, Days, NaiveDate, Utc};
use rusqlite::Connection;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ai::privacy::DataClass,
    db::repositories::{school_schedule, tasks},
};

use super::{
    errors::ToolError,
    registry::{
        descriptor, NativeToolId, DEFAULT_RESULT_LIMIT, MAX_RESULT_LIMIT,
        MAX_SERIALIZED_RESULT_BYTES, MAX_WINDOW_DAYS, OUTPUT_SCHEMA_VERSION,
    },
    scope::ToolScope,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolExecutionContext {
    pub now: DateTime<Utc>,
    pub local_date: NaiveDate,
}

impl ToolExecutionContext {
    #[allow(dead_code)] // Used by the future router coordinator; tests inject a deterministic clock.
    pub fn capture() -> Self {
        let local_now = chrono::Local::now();
        Self {
            now: local_now.with_timezone(&Utc),
            local_date: local_now.date_naive(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarToolEvent {
    pub id: String,
    pub title: String,
    pub start: String,
    pub end: String,
    pub all_day: bool,
    pub location: Option<String>,
    pub source_label: String,
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskToolItem {
    pub id: String,
    pub title: String,
    pub due_date: Option<String>,
    pub status: String,
    pub priority: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventsOutput {
    pub schema_version: u16,
    pub events: Vec<CalendarToolEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarNextEventOutput {
    pub schema_version: u16,
    pub event: Option<CalendarToolEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksOutput {
    pub schema_version: u16,
    pub tasks: Vec<TaskToolItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum NativeToolOutput {
    CalendarEvents(CalendarEventsOutput),
    CalendarNextEvent(CalendarNextEventOutput),
    Tasks(TasksOutput),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeToolResult {
    pub tool_id: NativeToolId,
    pub output_schema_version: u16,
    pub privacy_class: DataClass,
    pub output: NativeToolOutput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CalendarGetEventsArguments {
    start_at: String,
    end_at: String,
    start_date: String,
    end_date: String,
    limit: Option<u32>,
    #[serde(default = "default_true")]
    include_all_day: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyArguments {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TasksGetDueArguments {
    start_date: Option<String>,
    end_date: Option<String>,
    #[serde(default)]
    include_overdue: bool,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TasksGetOpenArguments {
    limit: Option<u32>,
}

fn default_true() -> bool {
    true
}

pub fn execute_native_ai_tool_named(
    conn: &Connection,
    public_name: &str,
    arguments: Value,
    scope: &ToolScope,
    context: ToolExecutionContext,
) -> Result<NativeToolResult, ToolError> {
    let tool_id = NativeToolId::from_public_name(public_name)?;
    execute_native_ai_tool(conn, tool_id, arguments, scope, context)
}

pub fn execute_native_ai_tool(
    conn: &Connection,
    tool_id: NativeToolId,
    arguments: Value,
    scope: &ToolScope,
    context: ToolExecutionContext,
) -> Result<NativeToolResult, ToolError> {
    if !scope.permits(tool_id) {
        return Err(ToolError::unauthorized_scope());
    }
    let output = match tool_id {
        NativeToolId::CalendarGetEvents => calendar_get_events(conn, arguments, scope)?,
        NativeToolId::CalendarGetNextEvent => {
            calendar_get_next_event(conn, arguments, scope, context)?
        }
        NativeToolId::TasksGetDue => tasks_get_due(conn, arguments, scope, context)?,
        NativeToolId::TasksGetOpen => tasks_get_open(conn, arguments, scope)?,
    };
    let descriptor = descriptor(tool_id);
    let result = NativeToolResult {
        tool_id,
        output_schema_version: descriptor.output_schema_version,
        privacy_class: descriptor.privacy_class,
        output,
    };
    let serialized = serde_json::to_vec(&result).map_err(|_| ToolError::internal_read_failed())?;
    if serialized.len() > MAX_SERIALIZED_RESULT_BYTES {
        return Err(ToolError::result_too_large());
    }
    Ok(result)
}

fn calendar_get_events(
    conn: &Connection,
    arguments: Value,
    scope: &ToolScope,
) -> Result<NativeToolOutput, ToolError> {
    let arguments: CalendarGetEventsArguments = parse(arguments)?;
    let start_at = instant(&arguments.start_at)?;
    let end_at = instant(&arguments.end_at)?;
    validate_instant_window(start_at, end_at)?;
    let start_date = date(&arguments.start_date)?;
    let end_date = date(&arguments.end_date)?;
    validate_date_window(start_date, end_date)?;
    let grant = scope.calendar_grant()?;
    if start_at < grant.start_at()
        || end_at > grant.end_at()
        || start_date < grant.start_date()
        || end_date > grant.end_date()
    {
        return Err(ToolError::unauthorized_scope());
    }
    let limit = limit(arguments.limit, grant.max_results())?;
    let start_utc = format_instant(start_at);
    let end_utc = format_instant(end_at);
    let start_date = format_date(start_date);
    let end_date = format_date(end_date);
    let rows = school_schedule::list_ai_calendar_events(
        conn,
        &school_schedule::AiCalendarProjectionRequest {
            school_space_id: grant.school_space_id(),
            start_utc: &start_utc,
            end_utc: &end_utc,
            start_date: &start_date,
            end_date: &end_date,
            include_all_day: arguments.include_all_day,
            timed_first: false,
            starts_at_or_after: false,
            limit,
        },
    )
    .map_err(|_| ToolError::internal_read_failed())?;
    let events = rows
        .into_iter()
        .map(calendar_event)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NativeToolOutput::CalendarEvents(CalendarEventsOutput {
        schema_version: OUTPUT_SCHEMA_VERSION,
        events,
    }))
}

fn calendar_get_next_event(
    conn: &Connection,
    arguments: Value,
    scope: &ToolScope,
    context: ToolExecutionContext,
) -> Result<NativeToolOutput, ToolError> {
    let _: EmptyArguments = parse(arguments)?;
    let grant = scope.calendar_grant()?;
    let start_at = context.now.max(grant.start_at());
    let event = if start_at >= grant.end_at() {
        None
    } else {
        let start_utc = format_instant(start_at);
        let end_utc = format_instant(grant.end_at());
        let start_date = format_date(grant.start_date());
        let end_date = format_date(grant.end_date());
        school_schedule::list_ai_calendar_events(
            conn,
            &school_schedule::AiCalendarProjectionRequest {
                school_space_id: grant.school_space_id(),
                start_utc: &start_utc,
                end_utc: &end_utc,
                start_date: &start_date,
                end_date: &end_date,
                include_all_day: false,
                timed_first: true,
                starts_at_or_after: true,
                limit: 1,
            },
        )
        .map_err(|_| ToolError::internal_read_failed())?
        .into_iter()
        .next()
        .map(calendar_event)
        .transpose()?
    };
    Ok(NativeToolOutput::CalendarNextEvent(
        CalendarNextEventOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            event,
        },
    ))
}

fn tasks_get_due(
    conn: &Connection,
    arguments: Value,
    scope: &ToolScope,
    context: ToolExecutionContext,
) -> Result<NativeToolOutput, ToolError> {
    let arguments: TasksGetDueArguments = parse(arguments)?;
    let (start, end) = match (
        arguments.start_date.as_deref(),
        arguments.end_date.as_deref(),
    ) {
        (Some(start), Some(end)) => (date(start)?, date(end)?),
        (None, None) => (
            context.local_date,
            context
                .local_date
                .checked_add_days(Days::new(7))
                .ok_or_else(ToolError::invalid_arguments)?,
        ),
        _ => return Err(ToolError::invalid_arguments()),
    };
    validate_date_window(start, end)?;
    let grant = scope.tasks_grant()?;
    let (scope_start, scope_end) = grant
        .due_window()
        .ok_or_else(ToolError::unauthorized_scope)?;
    if start < scope_start
        || end > scope_end
        || (arguments.include_overdue && !grant.allow_overdue())
    {
        return Err(ToolError::unauthorized_scope());
    }
    let limit = limit(arguments.limit, grant.max_results())?;
    let query_start = if arguments.include_overdue {
        scope_start
    } else {
        start
    };
    let rows = tasks::list_ai_due(conn, &format_date(query_start), &format_date(end), limit)
        .map_err(|_| ToolError::internal_read_failed())?;
    Ok(NativeToolOutput::Tasks(TasksOutput {
        schema_version: OUTPUT_SCHEMA_VERSION,
        tasks: rows.into_iter().map(task_item).collect(),
    }))
}

fn tasks_get_open(
    conn: &Connection,
    arguments: Value,
    scope: &ToolScope,
) -> Result<NativeToolOutput, ToolError> {
    let arguments: TasksGetOpenArguments = parse(arguments)?;
    let grant = scope.tasks_grant()?;
    if !grant.allow_open() {
        return Err(ToolError::unauthorized_scope());
    }
    let limit = limit(arguments.limit, grant.max_results())?;
    let rows = tasks::list_ai_open(conn, limit).map_err(|_| ToolError::internal_read_failed())?;
    Ok(NativeToolOutput::Tasks(TasksOutput {
        schema_version: OUTPUT_SCHEMA_VERSION,
        tasks: rows.into_iter().map(task_item).collect(),
    }))
}

fn parse<T: DeserializeOwned>(value: Value) -> Result<T, ToolError> {
    serde_json::from_value(value).map_err(|_| ToolError::invalid_arguments())
}

fn limit(requested: Option<u32>, scope_max: u32) -> Result<u32, ToolError> {
    let requested = requested.unwrap_or(DEFAULT_RESULT_LIMIT.min(scope_max));
    if requested == 0 || requested > MAX_RESULT_LIMIT {
        return Err(ToolError::invalid_arguments());
    }
    if requested > scope_max {
        return Err(ToolError::unauthorized_scope());
    }
    Ok(requested)
}

fn instant(value: &str) -> Result<DateTime<Utc>, ToolError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| ToolError::invalid_arguments())
}

fn date(value: &str) -> Result<NaiveDate, ToolError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| ToolError::invalid_arguments())
}

fn validate_instant_window(
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
) -> Result<(), ToolError> {
    if start_at >= end_at {
        return Err(ToolError::invalid_arguments());
    }
    if end_at.signed_duration_since(start_at).num_seconds() > MAX_WINDOW_DAYS * 86_400 {
        return Err(ToolError::window_too_large());
    }
    Ok(())
}

fn validate_date_window(start: NaiveDate, end: NaiveDate) -> Result<(), ToolError> {
    if start >= end {
        return Err(ToolError::invalid_arguments());
    }
    if end.signed_duration_since(start).num_days() > MAX_WINDOW_DAYS {
        return Err(ToolError::window_too_large());
    }
    Ok(())
}

fn format_instant(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn format_date(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn calendar_event(
    row: school_schedule::AiCalendarEventProjection,
) -> Result<CalendarToolEvent, ToolError> {
    let all_day = row.time_kind == "all_day";
    let (start, end) = if all_day {
        (
            row.start_date.ok_or_else(ToolError::internal_read_failed)?,
            row.end_date.ok_or_else(ToolError::internal_read_failed)?,
        )
    } else if row.time_kind == "timed" {
        (
            row.start_at.ok_or_else(ToolError::internal_read_failed)?,
            row.end_at.ok_or_else(ToolError::internal_read_failed)?,
        )
    } else {
        return Err(ToolError::internal_read_failed());
    };
    Ok(CalendarToolEvent {
        id: row.id,
        title: row.title,
        start,
        end,
        all_day,
        location: row.location,
        source_label: row.source_label,
        cancelled: false,
    })
}

fn task_item(row: tasks::AiTaskProjection) -> TaskToolItem {
    TaskToolItem {
        id: row.id,
        title: row.title,
        due_date: row.due_date,
        status: row.status,
        priority: row.priority,
        completed: false,
    }
}
