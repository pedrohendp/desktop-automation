use uiautomation::{UIAutomation, UIElement};
use uiautomation::types::Handle;

use crate::types::ElementRef;
use crate::types::McpToolError;
use crate::types::control_types::string_to_control_type;

/// Resolve an ElementRef to a live UIElement by re-querying the UI tree.
pub fn resolve_element(
    automation: &UIAutomation,
    element_ref: &ElementRef,
) -> Result<UIElement, McpToolError> {
    match element_ref {
        ElementRef::ByAutomationId {
            window_handle,
            automation_id,
        } => resolve_by_automation_id(automation, *window_handle, automation_id),

        ElementRef::ByTreePath {
            window_handle,
            path,
        } => resolve_by_tree_path(automation, *window_handle, path),

        ElementRef::ByNameAndType {
            window_handle,
            name,
            control_type,
        } => resolve_by_name_and_type(automation, *window_handle, name, control_type.as_deref()),

        ElementRef::ByRuntimeId {
            window_handle,
            runtime_id,
        } => resolve_by_runtime_id(automation, *window_handle, runtime_id),
    }
}

fn get_window_element(automation: &UIAutomation, window_handle: i64) -> Result<UIElement, McpToolError> {
    let handle = Handle::from(window_handle as isize);
    automation
        .element_from_handle(handle)
        .map_err(|_| McpToolError::WindowNotFound(window_handle))
}

fn resolve_by_automation_id(
    automation: &UIAutomation,
    window_handle: i64,
    automation_id: &str,
) -> Result<UIElement, McpToolError> {
    let window = get_window_element(automation, window_handle)?;

    let matcher = automation
        .create_matcher()
        .from(window)
        .depth(20)
        .timeout(3000)
        .filter_fn(Box::new({
            let aid = automation_id.to_string();
            move |element: &UIElement| -> Result<bool, uiautomation::Error> {
                let elem_aid = element.get_automation_id().unwrap_or_default();
                Ok(elem_aid == aid)
            }
        }));

    matcher
        .find_first()
        .map_err(|_| McpToolError::ElementNotFound(format!("AutomationId '{}'", automation_id)))
}

fn resolve_by_tree_path(
    automation: &UIAutomation,
    window_handle: i64,
    path: &[usize],
) -> Result<UIElement, McpToolError> {
    let window = get_window_element(automation, window_handle)?;

    let walker = automation
        .get_control_view_walker()
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    let mut current = window;

    for &index in path {
        let first_child = walker
            .get_first_child(&current)
            .map_err(|_| McpToolError::ElementNotFound(format!("TreePath {:?} - no children", path)))?;

        let mut child = first_child;
        for _ in 0..index {
            child = walker
                .get_next_sibling(&child)
                .map_err(|_| {
                    McpToolError::ElementNotFound(format!("TreePath {:?} - index {} out of bounds", path, index))
                })?;
        }

        current = child;
    }

    Ok(current)
}

fn resolve_by_name_and_type(
    automation: &UIAutomation,
    window_handle: i64,
    name: &str,
    control_type: Option<&str>,
) -> Result<UIElement, McpToolError> {
    let window = get_window_element(automation, window_handle)?;

    let mut matcher = automation
        .create_matcher()
        .from(window)
        .depth(20)
        .timeout(3000)
        .name(name);

    if let Some(ct_str) = control_type {
        if let Some(ct) = string_to_control_type(ct_str) {
            matcher = matcher.control_type(ct);
        }
    }

    matcher
        .find_first()
        .map_err(|_| {
            let desc = match control_type {
                Some(ct) => format!("Name '{}' with type '{}'", name, ct),
                None => format!("Name '{}'", name),
            };
            McpToolError::ElementNotFound(desc)
        })
}

fn resolve_by_runtime_id(
    automation: &UIAutomation,
    window_handle: i64,
    runtime_id: &[i32],
) -> Result<UIElement, McpToolError> {
    let window = get_window_element(automation, window_handle)?;
    let target_id = runtime_id.to_vec();

    let matcher = automation
        .create_matcher()
        .from(window)
        .depth(20)
        .timeout(3000)
        .filter_fn(Box::new({
            let target = target_id.clone();
            move |element: &UIElement| -> Result<bool, uiautomation::Error> {
                let elem_id = element.get_runtime_id().unwrap_or_default();
                Ok(elem_id == target)
            }
        }));

    matcher
        .find_first()
        .map_err(|_| McpToolError::ElementNotFound(format!("RuntimeId {:?}", runtime_id)))
}
