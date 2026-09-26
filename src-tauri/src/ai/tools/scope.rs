use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use super::{
    errors::ToolError,
    registry::{NativeToolId, MAX_RESULT_LIMIT, MAX_WINDOW_DAYS},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarReadGrant {
    school_space_id: String,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    max_results: u32,
}

impl CalendarReadGrant {
    pub(super) fn school_space_id(&self) -> &str {
        &self.school_space_id
    }

    pub(super) fn start_at(&self) -> DateTime<Utc> {
        self.start_at
    }

    pub(super) fn end_at(&self) -> DateTime<Utc> {
        self.end_at
    }

    pub(super) fn max_results(&self) -> u32 {
        self.max_results
    }

    pub(super) fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    pub(super) fn end_date(&self) -> NaiveDate {
        self.end_date
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksReadGrant {
    due_start: Option<NaiveDate>,
    due_end: Option<NaiveDate>,
    allow_overdue: bool,
    allow_open: bool,
    max_results: u32,
}

impl TasksReadGrant {
    pub(super) fn due_window(&self) -> Option<(NaiveDate, NaiveDate)> {
        self.due_start.zip(self.due_end)
    }

    pub(super) fn allow_overdue(&self) -> bool {
        self.allow_overdue
    }

    pub(super) fn allow_open(&self) -> bool {
        self.allow_open
    }

    pub(super) fn max_results(&self) -> u32 {
        self.max_results
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolScope {
    allowed_tool_ids: Vec<NativeToolId>,
    calendar_read: Option<CalendarReadGrant>,
    tasks_read: Option<TasksReadGrant>,
}

impl ToolScope {
    pub fn none() -> Self {
        Self::default()
    }

    pub(crate) fn calendar(
        allowed_tool_ids: Vec<NativeToolId>,
        school_space_id: impl Into<String>,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        max_results: u32,
    ) -> Result<Self, ToolError> {
        let school_space_id = school_space_id.into();
        if school_space_id.trim().is_empty()
            || !allowed_tool_ids.iter().all(|tool_id| {
                matches!(
                    tool_id,
                    NativeToolId::CalendarGetEvents | NativeToolId::CalendarGetNextEvent
                )
            })
        {
            return Err(ToolError::unsupported_scope());
        }
        validate_result_limit(max_results)?;
        validate_instant_window(start_at, end_at)?;
        validate_date_window(start_date, end_date)?;
        Ok(Self {
            allowed_tool_ids,
            calendar_read: Some(CalendarReadGrant {
                school_space_id,
                start_at,
                end_at,
                start_date,
                end_date,
                max_results,
            }),
            tasks_read: None,
        })
    }

    pub(crate) fn tasks(
        allowed_tool_ids: Vec<NativeToolId>,
        due_window: Option<(NaiveDate, NaiveDate)>,
        allow_overdue: bool,
        allow_open: bool,
        max_results: u32,
    ) -> Result<Self, ToolError> {
        if !allowed_tool_ids.iter().all(|tool_id| {
            matches!(
                tool_id,
                NativeToolId::TasksGetDue | NativeToolId::TasksGetOpen
            )
        }) {
            return Err(ToolError::unsupported_scope());
        }
        validate_result_limit(max_results)?;
        if let Some((start, end)) = due_window {
            validate_date_window(start, end)?;
        }
        if allowed_tool_ids.contains(&NativeToolId::TasksGetDue) && due_window.is_none() {
            return Err(ToolError::unsupported_scope());
        }
        if allowed_tool_ids.contains(&NativeToolId::TasksGetOpen) && !allow_open {
            return Err(ToolError::unsupported_scope());
        }
        Ok(Self {
            allowed_tool_ids,
            calendar_read: None,
            tasks_read: Some(TasksReadGrant {
                due_start: due_window.map(|window| window.0),
                due_end: due_window.map(|window| window.1),
                allow_overdue,
                allow_open,
                max_results,
            }),
        })
    }

    pub(super) fn permits(&self, tool_id: NativeToolId) -> bool {
        self.allowed_tool_ids.contains(&tool_id)
    }

    pub(super) fn calendar_grant(&self) -> Result<&CalendarReadGrant, ToolError> {
        self.calendar_read
            .as_ref()
            .ok_or_else(ToolError::unauthorized_scope)
    }

    pub(super) fn tasks_grant(&self) -> Result<&TasksReadGrant, ToolError> {
        self.tasks_read
            .as_ref()
            .ok_or_else(ToolError::unauthorized_scope)
    }
}

fn validate_result_limit(max_results: u32) -> Result<(), ToolError> {
    if (1..=MAX_RESULT_LIMIT).contains(&max_results) {
        Ok(())
    } else {
        Err(ToolError::unsupported_scope())
    }
}

fn validate_instant_window(
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
) -> Result<(), ToolError> {
    if start_at >= end_at {
        return Err(ToolError::unsupported_scope());
    }
    if end_at.signed_duration_since(start_at).num_seconds() > MAX_WINDOW_DAYS * 86_400 {
        return Err(ToolError::window_too_large());
    }
    Ok(())
}

fn validate_date_window(start: NaiveDate, end: NaiveDate) -> Result<(), ToolError> {
    if start >= end {
        return Err(ToolError::unsupported_scope());
    }
    if end.signed_duration_since(start).num_days() > MAX_WINDOW_DAYS {
        return Err(ToolError::window_too_large());
    }
    Ok(())
}
