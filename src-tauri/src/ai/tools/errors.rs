use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorCode {
    UnknownTool,
    InvalidArguments,
    UnauthorizedScope,
    UnsupportedScope,
    ResultTooLarge,
    WindowTooLarge,
    InternalReadFailed,
    TemporarilyUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolError {
    pub code: ToolErrorCode,
    pub message: String,
}

impl ToolError {
    pub(super) fn new(code: ToolErrorCode, message: &'static str) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(super) fn unknown_tool() -> Self {
        Self::new(
            ToolErrorCode::UnknownTool,
            "The requested native tool is not registered.",
        )
    }

    pub(super) fn invalid_arguments() -> Self {
        Self::new(
            ToolErrorCode::InvalidArguments,
            "The native tool arguments are invalid.",
        )
    }

    pub(super) fn unauthorized_scope() -> Self {
        Self::new(
            ToolErrorCode::UnauthorizedScope,
            "The native tool is not authorized for this request.",
        )
    }

    pub(super) fn unsupported_scope() -> Self {
        Self::new(
            ToolErrorCode::UnsupportedScope,
            "The requested native tool scope is unsupported.",
        )
    }

    pub(super) fn window_too_large() -> Self {
        Self::new(
            ToolErrorCode::WindowTooLarge,
            "The requested native tool date window is too large.",
        )
    }

    pub(super) fn internal_read_failed() -> Self {
        Self::new(
            ToolErrorCode::InternalReadFailed,
            "The local data could not be read.",
        )
    }

    pub(super) fn result_too_large() -> Self {
        Self::new(
            ToolErrorCode::ResultTooLarge,
            "The native tool result exceeds its safe size limit.",
        )
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ToolError {}
