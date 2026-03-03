use rmcp::model::{CallToolResult, Content};

use crate::automation;
use crate::com_thread::ComThreadHandle;
use crate::types::McpToolError;
use crate::types::control_types::string_to_control_type;

/// Find an element by name, automation_id, and/or control_type within a window.
pub async fn find_element_impl(
    com: &ComThreadHandle,
    window_handle: i64,
    name: Option<String>,
    automation_id: Option<String>,
    control_type: Option<String>,
    max_depth: Option<u32>,
) -> Result<CallToolResult, McpToolError> {
    if name.is_none() && automation_id.is_none() && control_type.is_none() {
        return Err(McpToolError::InvalidParameter(
            "At least one of name, automation_id, or control_type must be provided".to_string(),
        ));
    }

    let depth = max_depth.unwrap_or(10);

    let results = com
        .run(move |auto| {
            let handle = uiautomation::types::Handle::from(window_handle as isize);
            let window = auto
                .element_from_handle(handle)
                .map_err(|_| McpToolError::WindowNotFound(window_handle))?;

            let mut matcher = auto.create_matcher().from(window).depth(depth).timeout(3000);

            if let Some(ref n) = name {
                matcher = matcher.contains_name(n);
            }

            if let Some(ref ct_str) = control_type {
                if let Some(ct) = string_to_control_type(ct_str) {
                    matcher = matcher.control_type(ct);
                }
            }

            // For automation_id, use filter_fn since UIMatcher has no built-in automation_id method
            if let Some(ref aid) = automation_id {
                let aid_clone = aid.clone();
                matcher = matcher.filter_fn(Box::new(move |element: &uiautomation::UIElement| -> Result<bool, uiautomation::Error> {
                    let elem_aid = element.get_automation_id().unwrap_or_default();
                    Ok(elem_aid == aid_clone)
                }));
            }

            let elements = matcher.find_all().map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

            let infos: Vec<_> = elements
                .iter()
                .map(|el| automation::element_to_info(el, vec![]))
                .collect();

            Ok(infos)
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Get the currently focused element.
pub async fn get_focused_element_impl(
    com: &ComThreadHandle,
) -> Result<CallToolResult, McpToolError> {
    let info = com
        .run(|auto| {
            let element = auto
                .get_focused_element()
                .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

            Ok(automation::element_to_info(&element, vec![]))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wait for an element to appear, polling with timeout.
pub async fn wait_for_element_impl(
    com: &ComThreadHandle,
    window_handle: i64,
    name: Option<String>,
    automation_id: Option<String>,
    control_type: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<CallToolResult, McpToolError> {
    if name.is_none() && automation_id.is_none() && control_type.is_none() {
        return Err(McpToolError::InvalidParameter(
            "At least one of name, automation_id, or control_type must be provided".to_string(),
        ));
    }

    let timeout = timeout_ms.unwrap_or(10000);
    let poll_interval = std::time::Duration::from_millis(500);
    let start = std::time::Instant::now();

    loop {
        let name_c = name.clone();
        let aid_c = automation_id.clone();
        let ct_c = control_type.clone();

        let result = com
            .run(move |auto| {
                let handle = uiautomation::types::Handle::from(window_handle as isize);
                let window = auto
                    .element_from_handle(handle)
                    .map_err(|_| McpToolError::WindowNotFound(window_handle))?;

                let mut matcher = auto.create_matcher().from(window).depth(10).timeout(1000);

                if let Some(ref n) = name_c {
                    matcher = matcher.contains_name(n);
                }

                if let Some(ref ct_str) = ct_c {
                    if let Some(ct) = string_to_control_type(ct_str) {
                        matcher = matcher.control_type(ct);
                    }
                }

                if let Some(ref aid) = aid_c {
                    let aid_clone = aid.clone();
                    matcher = matcher.filter_fn(Box::new(move |element: &uiautomation::UIElement| -> Result<bool, uiautomation::Error> {
                        let elem_aid = element.get_automation_id().unwrap_or_default();
                        Ok(elem_aid == aid_clone)
                    }));
                }

                match matcher.find_first() {
                    Ok(el) => Ok(Some(automation::element_to_info(&el, vec![]))),
                    Err(_) => Ok(None),
                }
            })
            .await?
            .map_err(|e: McpToolError| e)?;

        if let Some(info) = result {
            let json = serde_json::to_string_pretty(&info)
                .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;
            return Ok(CallToolResult::success(vec![Content::text(json)]));
        }

        if start.elapsed().as_millis() as u64 >= timeout {
            return Err(McpToolError::Timeout(timeout));
        }

        tokio::time::sleep(poll_interval).await;
    }
}
