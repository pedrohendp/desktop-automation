use rmcp::model::{CallToolResult, Content};
use uiautomation::patterns::UIWindowPattern;
use uiautomation::types::WindowVisualState;

use crate::automation;
use crate::com_thread::ComThreadHandle;
use crate::types::McpToolError;

/// List all visible top-level windows.
pub async fn list_windows_impl(com: &ComThreadHandle) -> Result<CallToolResult, McpToolError> {
    let windows = com
        .run(|auto| automation::list_top_level_windows(auto))
        .await?;

    let json = serde_json::to_string_pretty(&windows)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Get the control tree of a specific window.
pub async fn get_window_tree_impl(
    com: &ComThreadHandle,
    window_handle: i64,
    max_depth: Option<u32>,
) -> Result<CallToolResult, McpToolError> {
    let depth = max_depth.unwrap_or(5);
    let max_nodes: u32 = 2000;

    let tree = com
        .run(move |auto| {
            let handle = uiautomation::types::Handle::from(window_handle as isize);
            let root = auto
                .element_from_handle(handle)
                .map_err(|_| McpToolError::WindowNotFound(window_handle))?;

            Ok(automation::walk_control_tree(auto, &root, depth, max_nodes))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    let json = serde_json::to_string_pretty(&tree)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Change the window state: minimize, maximize, restore, or bring to foreground.
pub async fn set_window_state_impl(
    com: &ComThreadHandle,
    window_handle: i64,
    state: &str,
) -> Result<CallToolResult, McpToolError> {
    let state_str = state.to_lowercase();

    match state_str.as_str() {
        "minimize" | "maximize" | "restore" | "foreground" => {}
        _ => {
            return Err(McpToolError::InvalidParameter(
                "state must be 'minimize', 'maximize', 'restore', or 'foreground'".to_string(),
            ));
        }
    }

    let msg = com
        .run(move |auto| {
            let handle = uiautomation::types::Handle::from(window_handle as isize);
            let element = auto
                .element_from_handle(handle)
                .map_err(|_| McpToolError::WindowNotFound(window_handle))?;

            if state_str == "foreground" {
                // Use Win32 SetForegroundWindow
                use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;
                let hwnd = windows::Win32::Foundation::HWND(window_handle as *mut _);
                unsafe {
                    let _ = SetForegroundWindow(hwnd);
                }
                return Ok("Brought window to foreground".to_string());
            }

            // Use WindowPattern for minimize/maximize/restore
            let wp = element
                .get_pattern::<UIWindowPattern>()
                .map_err(|_| McpToolError::PatternNotSupported("WindowPattern".to_string()))?;

            let visual_state = match state_str.as_str() {
                "minimize" => WindowVisualState::Minimized,
                "maximize" => WindowVisualState::Maximized,
                "restore" => WindowVisualState::Normal,
                _ => unreachable!(),
            };

            wp.set_window_visual_state(visual_state)
                .map_err(|e| McpToolError::UiAutomation(format!("set_window_visual_state failed: {}", e)))?;

            Ok(format!("Window state set to '{}'", state_str))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}
