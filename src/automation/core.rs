use uiautomation::UIAutomation;
use uiautomation::types::ControlType;

use crate::types::{BoundingRect, TreeNode, WindowInfo};
use crate::types::control_types::control_type_to_string;

/// Get the process name for a given process ID using Win32 API.
fn get_process_name(pid: u32) -> String {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
    use windows::Win32::System::ProcessStatus::K32GetModuleBaseNameW;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
        match handle {
            Ok(h) => {
                let mut buf = [0u16; 260];
                let len = K32GetModuleBaseNameW(h, None, &mut buf);
                let _ = windows::Win32::Foundation::CloseHandle(h);
                if len > 0 {
                    String::from_utf16_lossy(&buf[..len as usize])
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        }
    }
}

/// List all visible top-level windows.
pub fn list_top_level_windows(automation: &UIAutomation) -> Vec<WindowInfo> {
    let mut windows = Vec::new();

    let root = match automation.get_root_element() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to get root element: {}", e);
            return windows;
        }
    };

    let walker = match automation.get_control_view_walker() {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("Failed to create tree walker: {}", e);
            return windows;
        }
    };

    // Iterate top-level children of the desktop
    let mut child = match walker.get_first_child(&root) {
        Ok(c) => c,
        Err(_) => return windows,
    };

    loop {
        let ct = child.get_control_type().unwrap_or(ControlType::Custom);
        // Accept Window and Pane type top-level elements
        if ct == ControlType::Window || ct == ControlType::Pane {
            let name = child.get_name().unwrap_or_default();
            let class_name = child.get_classname().unwrap_or_default();
            let pid = child.get_process_id().unwrap_or(0);
            let is_offscreen = child.is_offscreen().unwrap_or(true);

            // Only include visible windows with a name
            if !is_offscreen || !name.is_empty() {
                let handle = child.get_native_window_handle()
                    .map(|h| {
                        let val: isize = h.into();
                        val as i64
                    })
                    .unwrap_or(0);

                let bounding_rect = child.get_bounding_rectangle().ok().map(|r| BoundingRect {
                    left: r.get_left() as i32,
                    top: r.get_top() as i32,
                    right: r.get_right() as i32,
                    bottom: r.get_bottom() as i32,
                    width: r.get_width() as i32,
                    height: r.get_height() as i32,
                });

                let process_name = get_process_name(pid);

                windows.push(WindowInfo {
                    handle,
                    title: name,
                    class_name,
                    process_id: pid,
                    process_name,
                    is_visible: !is_offscreen,
                    bounding_rect,
                });
            }
        }

        match walker.get_next_sibling(&child) {
            Ok(next) => child = next,
            Err(_) => break,
        }
    }

    windows
}

/// Recursively walk the control tree starting from a root element.
pub fn walk_control_tree(
    automation: &UIAutomation,
    root: &uiautomation::UIElement,
    max_depth: u32,
    max_nodes: u32,
) -> TreeNode {
    let walker = match automation.get_control_view_walker() {
        Ok(w) => w,
        Err(_) => {
            return make_tree_node(root, vec![]);
        }
    };

    let mut node_count: u32 = 0;
    walk_recursive(&walker, root, 0, max_depth, max_nodes, &mut node_count, vec![])
}

fn walk_recursive(
    walker: &uiautomation::UITreeWalker,
    element: &uiautomation::UIElement,
    current_depth: u32,
    max_depth: u32,
    max_nodes: u32,
    node_count: &mut u32,
    path: Vec<usize>,
) -> TreeNode {
    *node_count += 1;

    let mut node = make_tree_node(element, path.clone());

    if current_depth >= max_depth || *node_count >= max_nodes {
        return node;
    }

    // Walk children
    let first_child = walker.get_first_child(element);
    if let Ok(child) = first_child {
        let mut child_index: usize = 0;
        let mut current = child;

        loop {
            if *node_count >= max_nodes {
                break;
            }

            let mut child_path = path.clone();
            child_path.push(child_index);

            let child_node = walk_recursive(
                walker,
                &current,
                current_depth + 1,
                max_depth,
                max_nodes,
                node_count,
                child_path,
            );
            node.children.push(child_node);

            child_index += 1;

            match walker.get_next_sibling(&current) {
                Ok(next) => current = next,
                Err(_) => break,
            }
        }
    }

    node
}

fn make_tree_node(element: &uiautomation::UIElement, tree_path: Vec<usize>) -> TreeNode {
    let name = element.get_name().unwrap_or_default();
    let automation_id = element.get_automation_id().unwrap_or_default();
    let control_type = element
        .get_control_type()
        .map(control_type_to_string)
        .unwrap_or("Unknown")
        .to_string();

    TreeNode {
        name,
        automation_id,
        control_type,
        children: Vec::new(),
        tree_path,
    }
}
