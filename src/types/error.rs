use rmcp::model::{ErrorCode, ErrorData};

#[derive(Debug, thiserror::Error)]
pub enum McpToolError {
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Window not found: handle {0}")]
    WindowNotFound(i64),

    #[error("Pattern not supported: {0}")]
    PatternNotSupported(String),

    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    #[error("UI Automation error: {0}")]
    UiAutomation(String),

    #[error("COM thread is dead")]
    ComThreadDead,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),
}

impl From<McpToolError> for ErrorData {
    fn from(err: McpToolError) -> Self {
        let code = match &err {
            McpToolError::ElementNotFound(_) => ErrorCode(-32001),
            McpToolError::WindowNotFound(_) => ErrorCode(-32002),
            McpToolError::PatternNotSupported(_) => ErrorCode(-32003),
            McpToolError::Timeout(_) => ErrorCode(-32004),
            McpToolError::UiAutomation(_) => ErrorCode(-32005),
            McpToolError::ComThreadDead => ErrorCode(-32006),
            McpToolError::InvalidParameter(_) => ErrorCode(-32602),
            McpToolError::ScreenshotFailed(_) => ErrorCode(-32007),
        };
        ErrorData::new(code, err.to_string(), None::<serde_json::Value>)
    }
}
