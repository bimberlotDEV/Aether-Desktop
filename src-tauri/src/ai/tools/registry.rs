use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::privacy::DataClass;

use super::errors::ToolError;

pub const DEFAULT_RESULT_LIMIT: u32 = 20;
pub const MAX_RESULT_LIMIT: u32 = 50;
pub const MAX_WINDOW_DAYS: i64 = 31;
pub const MAX_SERIALIZED_RESULT_BYTES: usize = 64 * 1024;
pub const OUTPUT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeToolId {
    CalendarGetEvents,
    CalendarGetNextEvent,
    TasksGetDue,
    TasksGetOpen,
}

impl NativeToolId {
    pub const ALL: [Self; 4] = [
        Self::CalendarGetEvents,
        Self::CalendarGetNextEvent,
        Self::TasksGetDue,
        Self::TasksGetOpen,
    ];

    pub const fn public_name(self) -> &'static str {
        match self {
            Self::CalendarGetEvents => "calendar.get_events",
            Self::CalendarGetNextEvent => "calendar.get_next_event",
            Self::TasksGetDue => "tasks.get_due",
            Self::TasksGetOpen => "tasks.get_open",
        }
    }

    pub fn from_public_name(value: &str) -> Result<Self, ToolError> {
        Self::ALL
            .into_iter()
            .find(|tool_id| tool_id.public_name() == value)
            .ok_or_else(ToolError::unknown_tool)
    }
}

impl Serialize for NativeToolId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.public_name())
    }
}

impl<'de> Deserialize<'de> for NativeToolId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_public_name(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionType {
    ReadOnlyLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolScopeRequirement {
    Calendar,
    TasksDue,
    TasksOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultLimits {
    pub default_items: u32,
    pub max_items: u32,
    pub max_window_days: Option<u32>,
    pub max_serialized_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeToolDescriptor {
    pub id: NativeToolId,
    pub public_name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub output_schema: Value,
    pub output_schema_version: u16,
    pub privacy_class: DataClass,
    pub execution_type: ToolExecutionType,
    pub required_scope: ToolScopeRequirement,
    pub result_limits: ToolResultLimits,
}

fn limits(windowed: bool) -> ToolResultLimits {
    ToolResultLimits {
        default_items: DEFAULT_RESULT_LIMIT,
        max_items: MAX_RESULT_LIMIT,
        max_window_days: windowed.then_some(MAX_WINDOW_DAYS as u32),
        max_serialized_bytes: MAX_SERIALIZED_RESULT_BYTES,
    }
}

fn event_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["id", "title", "start", "end", "allDay", "location", "sourceLabel", "cancelled"],
        "properties": {
            "id": { "type": "string" },
            "title": { "type": "string" },
            "start": { "type": "string" },
            "end": { "type": "string" },
            "allDay": { "type": "boolean" },
            "location": { "type": ["string", "null"] },
            "sourceLabel": { "type": "string" },
            "cancelled": { "const": false }
        }
    })
}

fn task_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["id", "title", "dueDate", "status", "priority", "completed"],
        "properties": {
            "id": { "type": "string" },
            "title": { "type": "string" },
            "dueDate": { "type": ["string", "null"] },
            "status": { "enum": ["inbox", "planned", "in_progress"] },
            "priority": { "enum": ["none", "low", "medium", "high"] },
            "completed": { "const": false }
        }
    })
}

fn calendar_events_input_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["startAt", "endAt", "startDate", "endDate"],
        "properties": {
            "startAt": { "type": "string", "format": "date-time" },
            "endAt": { "type": "string", "format": "date-time" },
            "startDate": { "type": "string", "format": "date" },
            "endDate": { "type": "string", "format": "date" },
            "limit": { "type": "integer", "minimum": 1, "maximum": MAX_RESULT_LIMIT },
            "includeAllDay": { "type": "boolean" }
        }
    })
}

fn due_input_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "dependentRequired": {
            "startDate": ["endDate"],
            "endDate": ["startDate"]
        },
        "properties": {
            "startDate": { "type": "string", "format": "date" },
            "endDate": { "type": "string", "format": "date" },
            "includeOverdue": { "type": "boolean" },
            "limit": { "type": "integer", "minimum": 1, "maximum": MAX_RESULT_LIMIT }
        }
    })
}

fn output_list_schema(key: &str, item: Value) -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["schemaVersion", key],
        "properties": {
            "schemaVersion": { "const": OUTPUT_SCHEMA_VERSION },
            key: { "type": "array", "maxItems": MAX_RESULT_LIMIT, "items": item }
        }
    })
}

pub fn native_tool_registry() -> Vec<NativeToolDescriptor> {
    vec![
        NativeToolDescriptor {
            id: NativeToolId::CalendarGetEvents,
            public_name: NativeToolId::CalendarGetEvents.public_name(),
            description: "Read a bounded authorized School Calendar event window.",
            input_schema: calendar_events_input_schema(),
            output_schema: output_list_schema("events", event_schema()),
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            privacy_class: DataClass::Sensitive,
            execution_type: ToolExecutionType::ReadOnlyLocal,
            required_scope: ToolScopeRequirement::Calendar,
            result_limits: limits(true),
        },
        NativeToolDescriptor {
            id: NativeToolId::CalendarGetNextEvent,
            public_name: NativeToolId::CalendarGetNextEvent.public_name(),
            description: "Read at most one next timed authorized School Calendar event.",
            input_schema: json!({ "type": "object", "additionalProperties": false }),
            output_schema: json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["schemaVersion", "event"],
                "properties": {
                    "schemaVersion": { "const": OUTPUT_SCHEMA_VERSION },
                    "event": { "anyOf": [event_schema(), { "type": "null" }] }
                }
            }),
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            privacy_class: DataClass::Sensitive,
            execution_type: ToolExecutionType::ReadOnlyLocal,
            required_scope: ToolScopeRequirement::Calendar,
            result_limits: ToolResultLimits {
                default_items: 1,
                max_items: 1,
                max_window_days: Some(MAX_WINDOW_DAYS as u32),
                max_serialized_bytes: MAX_SERIALIZED_RESULT_BYTES,
            },
        },
        NativeToolDescriptor {
            id: NativeToolId::TasksGetDue,
            public_name: NativeToolId::TasksGetDue.public_name(),
            description: "Read local incomplete Tasks in a bounded due-date window.",
            input_schema: due_input_schema(),
            output_schema: output_list_schema("tasks", task_schema()),
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            privacy_class: DataClass::Sensitive,
            execution_type: ToolExecutionType::ReadOnlyLocal,
            required_scope: ToolScopeRequirement::TasksDue,
            result_limits: limits(true),
        },
        NativeToolDescriptor {
            id: NativeToolId::TasksGetOpen,
            public_name: NativeToolId::TasksGetOpen.public_name(),
            description: "Read a bounded deterministic list of local incomplete Tasks.",
            input_schema: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "limit": { "type": "integer", "minimum": 1, "maximum": MAX_RESULT_LIMIT }
                }
            }),
            output_schema: output_list_schema("tasks", task_schema()),
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            privacy_class: DataClass::Sensitive,
            execution_type: ToolExecutionType::ReadOnlyLocal,
            required_scope: ToolScopeRequirement::TasksOpen,
            result_limits: limits(false),
        },
    ]
}

pub fn descriptor(tool_id: NativeToolId) -> NativeToolDescriptor {
    native_tool_registry()
        .into_iter()
        .find(|descriptor| descriptor.id == tool_id)
        .expect("every closed native tool ID must have a descriptor")
}
