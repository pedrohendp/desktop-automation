pub mod control_types;
pub mod element_ref;
pub mod error;

#[allow(unused_imports)]
pub use element_ref::ElementRef;
pub use element_ref::parse_element_ref;
pub use error::McpToolError;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Information about a top-level window.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WindowInfo {
    pub handle: i64,
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub process_name: String,
    pub is_visible: bool,
    pub bounding_rect: Option<BoundingRect>,
}

/// Bounding rectangle of a UI element.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundingRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub width: i32,
    pub height: i32,
}

/// Properties of a UI element.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElementInfo {
    pub name: String,
    pub automation_id: String,
    pub control_type: String,
    pub class_name: String,
    pub is_enabled: bool,
    pub is_offscreen: bool,
    pub bounding_rect: Option<BoundingRect>,
    pub runtime_id: Vec<i32>,
    pub supported_patterns: Vec<String>,
    pub tree_path: Vec<usize>,
}

/// A node in the UI element tree.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreeNode {
    pub name: String,
    pub automation_id: String,
    pub control_type: String,
    pub children: Vec<TreeNode>,
    pub tree_path: Vec<usize>,
}

/// Data read from a grid/table control.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GridData {
    pub headers: Vec<String>,
    pub rows: Vec<GridRow>,
    pub total_rows: i32,
    pub start_row: i32,
    pub end_row: i32,
}

/// A single row of grid data.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GridRow {
    pub index: i32,
    pub cells: Vec<String>,
}
