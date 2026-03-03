use rmcp::model::{CallToolResult, Content};
use uiautomation::patterns::{
    UIExpandCollapsePattern, UIGridPattern, UISelectionItemPattern, UITablePattern,
};

use crate::automation;
use crate::com_thread::ComThreadHandle;
use crate::types::{GridData, GridRow, McpToolError, parse_element_ref};

/// Expand or collapse a UI element using ExpandCollapsePattern.
pub async fn expand_collapse_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
    action: &str,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    let action_str = action.to_lowercase();
    if action_str != "expand" && action_str != "collapse" {
        return Err(McpToolError::InvalidParameter(
            "action must be 'expand' or 'collapse'".to_string(),
        ));
    }

    let msg = com
        .run(move |auto| {
            let element = automation::resolve_element(auto, &element_ref)?;

            let pattern = element
                .get_pattern::<UIExpandCollapsePattern>()
                .map_err(|_| {
                    McpToolError::PatternNotSupported("ExpandCollapsePattern".to_string())
                })?;

            if action_str == "expand" {
                pattern
                    .expand()
                    .map_err(|e| McpToolError::UiAutomation(format!("Expand failed: {}", e)))?;
            } else {
                pattern
                    .collapse()
                    .map_err(|e| McpToolError::UiAutomation(format!("Collapse failed: {}", e)))?;
            }

            // Read back the new state
            let new_state = pattern
                .get_state()
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|_| "Unknown".to_string());

            Ok(format!(
                "{}d element. New state: {}",
                if action_str == "expand" {
                    "Expande"
                } else {
                    "Collapse"
                },
                new_state
            ))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Select an item using SelectionItemPattern.
pub async fn select_item_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
) -> Result<CallToolResult, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    let msg = com
        .run(move |auto| {
            let element = automation::resolve_element(auto, &element_ref)?;

            let pattern = element
                .get_pattern::<UISelectionItemPattern>()
                .map_err(|_| {
                    McpToolError::PatternNotSupported("SelectionItemPattern".to_string())
                })?;

            pattern
                .select()
                .map_err(|e| McpToolError::UiAutomation(format!("Select failed: {}", e)))?;

            let name = element.get_name().unwrap_or_default();
            Ok(format!("Selected item: '{}'", name))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Read data from a DataGridView or Table control.
pub async fn read_grid_impl(
    com: &ComThreadHandle,
    element_ref_json: &str,
    start_row: Option<i32>,
    end_row: Option<i32>,
) -> Result<CallToolResult, McpToolError> {
    let elem_ref = parse_element_ref(element_ref_json)?;

    let grid_data = com
        .run(move |automation| -> Result<GridData, McpToolError> {
            let element = automation::resolve_element(automation, &elem_ref)?;

            // Try to get GridPattern
            let grid: UIGridPattern = element.get_pattern().map_err(|_| {
                McpToolError::PatternNotSupported(
                    "Element does not support GridPattern. Make sure the element is a DataGridView or Table control.".to_string(),
                )
            })?;

            let total_rows = grid
                .get_row_count()
                .map_err(|e| McpToolError::UiAutomation(format!("get_row_count failed: {}", e)))?;
            let col_count = grid
                .get_column_count()
                .map_err(|e| {
                    McpToolError::UiAutomation(format!("get_column_count failed: {}", e))
                })?;

            // Determine row range (cap at 100 rows per call)
            let start = start_row.unwrap_or(0).max(0);
            let max_end = total_rows.min(start + 100);
            let end = end_row.unwrap_or(max_end).min(max_end).min(total_rows);

            if start >= total_rows {
                return Ok(GridData {
                    headers: Vec::new(),
                    rows: Vec::new(),
                    total_rows,
                    start_row: start,
                    end_row: start,
                });
            }

            // Try to get column headers via TablePattern or child traversal
            let headers = get_column_headers(automation, &element, col_count);

            // Read cells row by row
            let mut rows = Vec::new();
            for row_idx in start..end {
                let mut cells = Vec::new();
                for col_idx in 0..col_count {
                    let cell_value = match grid.get_item(row_idx, col_idx) {
                        Ok(cell_element) => {
                            // Try ValuePattern first, then fall back to Name
                            match cell_element
                                .get_pattern::<uiautomation::patterns::UIValuePattern>()
                            {
                                Ok(vp) => vp
                                    .get_value()
                                    .unwrap_or_else(|_| {
                                        cell_element.get_name().unwrap_or_default()
                                    }),
                                Err(_) => cell_element.get_name().unwrap_or_default(),
                            }
                        }
                        Err(_) => String::new(),
                    };
                    cells.push(cell_value);
                }
                rows.push(GridRow {
                    index: row_idx,
                    cells,
                });
            }

            Ok(GridData {
                headers,
                rows,
                total_rows,
                start_row: start,
                end_row: end,
            })
        })
        .await??;

    let json = serde_json::to_string_pretty(&grid_data)
        .map_err(|e| McpToolError::UiAutomation(format!("JSON serialization failed: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Try to extract column headers from the element using TablePattern or child traversal.
fn get_column_headers(
    automation: &uiautomation::UIAutomation,
    element: &uiautomation::UIElement,
    col_count: i32,
) -> Vec<String> {
    use uiautomation::types::ControlType;

    // First try TablePattern which provides column headers directly
    if let Ok(table) = element.get_pattern::<UITablePattern>() {
        if let Ok(header_elements) = table.get_column_headers() {
            let headers: Vec<String> = header_elements
                .iter()
                .map(|h: &uiautomation::UIElement| h.get_name().unwrap_or_default())
                .collect();
            if !headers.is_empty() {
                return headers;
            }
        }
    }

    // Fallback: walk children to find Header element, then its HeaderItem children
    if let Ok(walker) = automation.get_control_view_walker() {
        if let Ok(first_child) = walker.get_first_child(element) {
            let mut current = first_child;
            loop {
                let ct = current.get_control_type().unwrap_or(ControlType::Custom);
                if ct == ControlType::Header {
                    let mut headers = Vec::new();
                    if let Ok(header_child) = walker.get_first_child(&current) {
                        let mut hc = header_child;
                        loop {
                            headers.push(hc.get_name().unwrap_or_default());
                            match walker.get_next_sibling(&hc) {
                                Ok(next) => hc = next,
                                Err(_) => break,
                            }
                        }
                    }
                    if !headers.is_empty() {
                        return headers;
                    }
                }
                match walker.get_next_sibling(&current) {
                    Ok(next) => current = next,
                    Err(_) => break,
                }
            }
        }
    }

    // If we can't find headers, return generic column labels
    (0..col_count).map(|i| format!("Column{}", i)).collect()
}
