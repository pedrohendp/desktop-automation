use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::McpToolError;

/// Identifies a UI element across tool calls. Always re-resolved at each invocation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ElementRef {
    /// Locate by UI Automation AutomationId property (most reliable for WinForms).
    ByAutomationId {
        window_handle: i64,
        automation_id: String,
    },
    /// Locate by walking child indices from the window root.
    ByTreePath {
        window_handle: i64,
        path: Vec<usize>,
    },
    /// Locate by Name (and optionally ControlType).
    ByNameAndType {
        window_handle: i64,
        name: String,
        control_type: Option<String>,
    },
    /// Locate by runtime id array.
    ByRuntimeId {
        window_handle: i64,
        runtime_id: Vec<i32>,
    },
}

/// Parse an element_ref JSON string, auto-detecting the variant from fields if `type` is missing.
pub fn parse_element_ref(json: &str) -> Result<ElementRef, McpToolError> {
    // Try direct deserialization first (works when "type" field is correct)
    if let Ok(element_ref) = serde_json::from_str::<ElementRef>(json) {
        return Ok(element_ref);
    }

    // Fallback: parse as generic JSON and infer variant from fields
    let obj: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        McpToolError::InvalidParameter(format!(
            "Invalid element_ref JSON: {}. Expected format: {{\"window_handle\":123,\"automation_id\":\"btnOK\"}}",
            e
        ))
    })?;

    let map = obj.as_object().ok_or_else(|| {
        McpToolError::InvalidParameter(
            "element_ref must be a JSON object".to_string(),
        )
    })?;

    let window_handle = map
        .get("window_handle")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            McpToolError::InvalidParameter(
                "element_ref missing required field 'window_handle' (integer). Examples:\n  \
                 {\"window_handle\":123,\"automation_id\":\"btnOK\"}\n  \
                 {\"window_handle\":123,\"name\":\"OK\",\"control_type\":\"Button\"}\n  \
                 {\"window_handle\":123,\"path\":[0,2,1]}\n  \
                 {\"window_handle\":123,\"runtime_id\":[42,567]}"
                    .to_string(),
            )
        })?;

    if let Some(aid) = map.get("automation_id").and_then(|v| v.as_str()) {
        return Ok(ElementRef::ByAutomationId {
            window_handle,
            automation_id: aid.to_string(),
        });
    }

    if let Some(path_val) = map.get("path").and_then(|v| v.as_array()) {
        let path: Vec<usize> = path_val
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as usize))
            .collect();
        return Ok(ElementRef::ByTreePath {
            window_handle,
            path,
        });
    }

    if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
        let control_type = map.get("control_type").and_then(|v| v.as_str()).map(String::from);
        return Ok(ElementRef::ByNameAndType {
            window_handle,
            name: name.to_string(),
            control_type,
        });
    }

    if let Some(rid_val) = map.get("runtime_id").and_then(|v| v.as_array()) {
        let runtime_id: Vec<i32> = rid_val
            .iter()
            .filter_map(|v| v.as_i64().map(|n| n as i32))
            .collect();
        return Ok(ElementRef::ByRuntimeId {
            window_handle,
            runtime_id,
        });
    }

    Err(McpToolError::InvalidParameter(
        "Could not determine element_ref type. Provide one of: automation_id, name, path, or runtime_id. Examples:\n  \
         {\"window_handle\":123,\"automation_id\":\"btnOK\"}\n  \
         {\"window_handle\":123,\"name\":\"OK\",\"control_type\":\"Button\"}\n  \
         {\"window_handle\":123,\"path\":[0,2,1]}\n  \
         {\"window_handle\":123,\"runtime_id\":[42,567]}"
            .to_string(),
    ))
}
