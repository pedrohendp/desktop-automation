use uiautomation::UIElement;
use uiautomation::patterns::{UIInvokePattern, UIValuePattern, UIExpandCollapsePattern, UIGridPattern, UISelectionItemPattern, UITogglePattern, UIScrollPattern, UITablePattern};

use crate::types::{BoundingRect, ElementInfo};
use crate::types::control_types::control_type_to_string;

/// Check which UI Automation patterns an element supports.
pub fn get_supported_patterns(element: &UIElement) -> Vec<String> {
    let mut patterns = Vec::new();

    if element.get_pattern::<UIInvokePattern>().is_ok() {
        patterns.push("Invoke".to_string());
    }
    if element.get_pattern::<UIValuePattern>().is_ok() {
        patterns.push("Value".to_string());
    }
    if element.get_pattern::<UIExpandCollapsePattern>().is_ok() {
        patterns.push("ExpandCollapse".to_string());
    }
    if element.get_pattern::<UIGridPattern>().is_ok() {
        patterns.push("Grid".to_string());
    }
    if element.get_pattern::<UISelectionItemPattern>().is_ok() {
        patterns.push("SelectionItem".to_string());
    }
    if element.get_pattern::<UITogglePattern>().is_ok() {
        patterns.push("Toggle".to_string());
    }
    if element.get_pattern::<UIScrollPattern>().is_ok() {
        patterns.push("Scroll".to_string());
    }
    if element.get_pattern::<UITablePattern>().is_ok() {
        patterns.push("Table".to_string());
    }

    patterns
}

/// Convert a UIElement into our ElementInfo struct.
pub fn element_to_info(element: &UIElement, tree_path: Vec<usize>) -> ElementInfo {
    let name = element.get_name().unwrap_or_default();
    let automation_id = element.get_automation_id().unwrap_or_default();
    let control_type = element
        .get_control_type()
        .map(control_type_to_string)
        .unwrap_or("Unknown")
        .to_string();
    let class_name = element.get_classname().unwrap_or_default();
    let is_enabled = element.is_enabled().unwrap_or(false);
    let is_offscreen = element.is_offscreen().unwrap_or(true);
    let bounding_rect = element.get_bounding_rectangle().ok().map(|r| BoundingRect {
        left: r.get_left() as i32,
        top: r.get_top() as i32,
        right: r.get_right() as i32,
        bottom: r.get_bottom() as i32,
        width: r.get_width() as i32,
        height: r.get_height() as i32,
    });
    let runtime_id = element.get_runtime_id().unwrap_or_default();
    let supported_patterns = get_supported_patterns(element);

    ElementInfo {
        name,
        automation_id,
        control_type,
        class_name,
        is_enabled,
        is_offscreen,
        bounding_rect,
        runtime_id,
        supported_patterns,
        tree_path,
    }
}
