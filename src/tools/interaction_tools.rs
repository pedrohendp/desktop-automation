use rmcp::model::{CallToolResult, Content};
use uiautomation::patterns::{UIInvokePattern, UIValuePattern};

use crate::automation;
use crate::com_thread::ComThreadHandle;
use crate::types::{McpToolError, parse_element_ref};

/// Click or invoke a UI element.
/// Strategy: InvokePattern first, fallback to element.click().
pub async fn click_element_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    let msg = com
        .run(move |auto| {
            let element = automation::resolve_element(auto, &element_ref)?;

            // Try InvokePattern first
            if let Ok(invoke) = element.get_pattern::<UIInvokePattern>() {
                invoke
                    .invoke()
                    .map_err(|e| McpToolError::UiAutomation(format!("Invoke failed: {}", e)))?;
                return Ok("Invoked element via InvokePattern".to_string());
            }

            // Fallback: mouse click on element center
            element
                .click()
                .map_err(|e| McpToolError::UiAutomation(format!("Click failed: {}", e)))?;
            Ok("Clicked element via mouse click".to_string())
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Set the value/text of a UI element.
/// Strategy: ValuePattern.set_value() first, fallback to set_focus + send_keys.
pub async fn set_value_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
    value: String,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    let msg = com
        .run(move |auto| {
            let element = automation::resolve_element(auto, &element_ref)?;

            // Try ValuePattern first
            if let Ok(vp) = element.get_pattern::<UIValuePattern>() {
                vp.set_value(&value)
                    .map_err(|e| McpToolError::UiAutomation(format!("SetValue failed: {}", e)))?;
                return Ok(format!("Set value via ValuePattern: '{}'", value));
            }

            // Fallback: focus + select all + type
            element
                .set_focus()
                .map_err(|e| McpToolError::UiAutomation(format!("SetFocus failed: {}", e)))?;

            // Select all existing text with Ctrl+A, then type new value
            element
                .send_keys("{Ctrl}a", 10)
                .map_err(|e| McpToolError::UiAutomation(format!("send_keys Ctrl+A failed: {}", e)))?;

            element
                .send_keys(&value, 10)
                .map_err(|e| McpToolError::UiAutomation(format!("send_keys value failed: {}", e)))?;

            Ok(format!("Set value via focus+keys: '{}'", value))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Read the current value/text of a UI element.
/// Strategy: ValuePattern.get_value() first, fallback to get_name().
pub async fn get_value_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    let result = com
        .run(move |auto| {
            let element = automation::resolve_element(auto, &element_ref)?;

            // Try ValuePattern first
            if let Ok(vp) = element.get_pattern::<UIValuePattern>() {
                if let Ok(val) = vp.get_value() {
                    return Ok(serde_json::json!({
                        "value": val,
                        "source": "ValuePattern"
                    }));
                }
            }

            // Fallback: get_name()
            let name = element.get_name().unwrap_or_default();
            Ok(serde_json::json!({
                "value": name,
                "source": "Name property"
            }))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    let json = serde_json::to_string_pretty(&result)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Send keystrokes to an element or the currently focused element.
pub async fn send_keys_impl(
    com: &ComThreadHandle,
    element_ref_json: Option<String>,
    keys: String,
    interval_ms: Option<u64>,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = match element_ref_json {
        Some(ref json) => Some(parse_element_ref(json)?),
        None => None,
    };

    let interval = interval_ms.unwrap_or(10);

    let msg = com
        .run(move |auto| {
            let element = match element_ref {
                Some(ref eref) => automation::resolve_element(auto, eref)?,
                None => auto
                    .get_focused_element()
                    .map_err(|e| McpToolError::UiAutomation(format!("get_focused_element failed: {}", e)))?,
            };

            // Focus the element first
            let _ = element.set_focus();

            element
                .send_keys(&keys, interval)
                .map_err(|e| McpToolError::UiAutomation(format!("send_keys failed: {}", e)))?;

            Ok(format!("Sent keys: '{}'", keys))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}
