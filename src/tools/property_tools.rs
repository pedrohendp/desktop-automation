use rmcp::model::{CallToolResult, Content};

use crate::automation;
use crate::com_thread::ComThreadHandle;
use crate::types::{McpToolError, parse_element_ref};

/// Get all properties of a UI element.
pub async fn get_element_properties_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    let info = com
        .run(move |auto| {
            let element = automation::resolve_element(auto, &element_ref)?;
            Ok(automation::element_to_info(&element, vec![]))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}
