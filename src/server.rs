use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    schemars, tool, tool_handler, tool_router,
};

use crate::com_thread::ComThreadHandle;
use crate::tools;

// ── Parameter structs ──

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetWindowTreeParams {
    /// The window handle (HWND) as returned by list_windows
    pub window_handle: i64,
    /// Maximum tree depth to traverse (default: 5)
    pub max_depth: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindElementParams {
    /// The window handle (HWND) to search within
    pub window_handle: i64,
    /// Element name to search for
    pub name: Option<String>,
    /// Automation ID to search for (most reliable for WinForms)
    pub automation_id: Option<String>,
    /// Control type filter (e.g. Button, Edit, ComboBox)
    pub control_type: Option<String>,
    /// Maximum search depth (default: 10)
    pub max_depth: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ElementRefParams {
    /// JSON-encoded ElementRef identifying the element
    pub element_ref: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetValueParams {
    /// JSON-encoded ElementRef identifying the target element
    pub element_ref: String,
    /// The value to set
    pub value: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SendKeysParams {
    /// JSON-encoded ElementRef to focus before typing (optional, uses current focus if omitted)
    pub element_ref: Option<String>,
    /// Keys to send. Use {Enter}, {Tab}, {Escape}, {Ctrl}, {Alt}, {Shift} for special keys
    pub keys: String,
    /// Delay between keystrokes in milliseconds (default: 10)
    pub interval_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WindowHandleParams {
    /// The window handle (HWND)
    pub window_handle: i64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExpandCollapseParams {
    /// JSON-encoded ElementRef identifying the element
    pub element_ref: String,
    /// Action to perform: 'expand' or 'collapse'
    pub action: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadGridParams {
    /// JSON-encoded ElementRef identifying the grid/table element
    pub element_ref: String,
    /// Starting row index (0-based, default: 0)
    pub start_row: Option<i32>,
    /// Ending row index (exclusive, default: min(start+100, total_rows))
    pub end_row: Option<i32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WaitForElementParams {
    /// The window handle (HWND) to search within
    pub window_handle: i64,
    /// Element name to wait for
    pub name: Option<String>,
    /// Automation ID to wait for
    pub automation_id: Option<String>,
    /// Control type filter
    pub control_type: Option<String>,
    /// Timeout in milliseconds (default: 10000)
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetWindowStateParams {
    /// The window handle (HWND) to change
    pub window_handle: i64,
    /// Target state: 'minimize', 'maximize', 'restore', or 'foreground'
    pub state: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunWorkflowParams {
    /// Array of steps to execute sequentially. Each step has an "action" field and action-specific parameters.
    /// Steps that find elements (find_element, wait_for_element) automatically set the element context
    /// for subsequent action steps (click, set_value, etc.), so you don't need to pass element_ref explicitly.
    pub steps: Vec<crate::tools::workflow_tools::WorkflowStep>,
}

// ── Server ──

#[derive(Clone)]
pub struct DesktopAutomationServer {
    pub com_thread: ComThreadHandle,
    tool_router: ToolRouter<DesktopAutomationServer>,
}

#[tool_router]
impl DesktopAutomationServer {
    pub fn new(com_thread: ComThreadHandle) -> Self {
        Self {
            com_thread,
            tool_router: Self::tool_router(),
        }
    }

    /// List all visible top-level windows with their handles, titles, class names, and process info
    #[tool(description = "List all visible top-level windows with their handles, titles, class names, and process info")]
    async fn list_windows(&self) -> Result<CallToolResult, McpError> {
        tools::list_windows_impl(&self.com_thread)
            .await
            .map_err(McpError::from)
    }

    /// Get the full UI element tree of a window. Returns a hierarchical view of all controls.
    #[tool(description = "Get the full UI element tree of a window. Returns a hierarchical view of all controls.")]
    async fn get_window_tree(
        &self,
        Parameters(params): Parameters<GetWindowTreeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::get_window_tree_impl(&self.com_thread, params.window_handle, params.max_depth)
            .await
            .map_err(McpError::from)
    }

    /// Search for a UI element by name, automation ID, and/or control type within a window
    #[tool(description = "Search for a UI element by name, automation ID, and/or control type within a window")]
    async fn find_element(
        &self,
        Parameters(params): Parameters<FindElementParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::find_element_impl(
            &self.com_thread,
            params.window_handle,
            params.name,
            params.automation_id,
            params.control_type,
            params.max_depth,
        )
        .await
        .map_err(McpError::from)
    }

    /// Click or invoke a UI element. Uses InvokePattern if available, falls back to mouse click.
    #[tool(description = "Click or invoke a UI element. Uses InvokePattern if available, falls back to mouse click.")]
    async fn click_element(
        &self,
        Parameters(params): Parameters<ElementRefParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::click_element_impl(&self.com_thread, &params.element_ref)
            .await
            .map_err(McpError::from)
    }

    /// Set the value/text of a UI element. Uses ValuePattern if available, falls back to select-all + type.
    #[tool(description = "Set the value/text of a UI element. Uses ValuePattern if available, falls back to select-all + type.")]
    async fn set_value(
        &self,
        Parameters(params): Parameters<SetValueParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::set_value_impl(&self.com_thread, &params.element_ref, params.value)
            .await
            .map_err(McpError::from)
    }

    /// Read the current value/text of a UI element. Uses ValuePattern if available, falls back to Name property.
    #[tool(description = "Read the current value/text of a UI element. Uses ValuePattern if available, falls back to Name property.")]
    async fn get_value(
        &self,
        Parameters(params): Parameters<ElementRefParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::get_value_impl(&self.com_thread, &params.element_ref)
            .await
            .map_err(McpError::from)
    }

    /// Get all properties of a UI element including name, type, automation ID, bounding rect, patterns, and state
    #[tool(description = "Get all properties of a UI element including name, type, automation ID, bounding rect, patterns, and state")]
    async fn get_element_properties(
        &self,
        Parameters(params): Parameters<ElementRefParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::get_element_properties_impl(&self.com_thread, &params.element_ref)
            .await
            .map_err(McpError::from)
    }

    /// Send keystrokes to an element or the focused window. Supports special keys like {Enter}, {Tab}, {Ctrl}c
    #[tool(description = "Send keystrokes to an element or the focused window. Supports special keys like {Enter}, {Tab}, {Ctrl}c")]
    async fn send_keys(
        &self,
        Parameters(params): Parameters<SendKeysParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::send_keys_impl(
            &self.com_thread,
            params.element_ref,
            params.keys,
            params.interval_ms,
        )
        .await
        .map_err(McpError::from)
    }

    /// Capture a screenshot of a window and return it as a base64-encoded PNG image
    #[tool(description = "Capture a screenshot of a window and return it as a base64-encoded PNG image")]
    async fn screenshot_window(
        &self,
        Parameters(params): Parameters<WindowHandleParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::screenshot_window_impl(&self.com_thread, params.window_handle)
            .await
            .map_err(McpError::from)
    }

    /// Expand or collapse a UI element (tree nodes, combo boxes, expanders)
    #[tool(description = "Expand or collapse a UI element (tree nodes, combo boxes, expanders)")]
    async fn expand_collapse(
        &self,
        Parameters(params): Parameters<ExpandCollapseParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::expand_collapse_impl(&self.com_thread, &params.element_ref, &params.action)
            .await
            .map_err(McpError::from)
    }

    /// Select an item in a list, combo box, tab control, or tree view
    #[tool(description = "Select an item in a list, combo box, tab control, or tree view")]
    async fn select_item(
        &self,
        Parameters(params): Parameters<ElementRefParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::select_item_impl(&self.com_thread, &params.element_ref)
            .await
            .map_err(McpError::from)
    }

    /// Read data from a DataGridView or Table control, including headers and cell values
    #[tool(description = "Read data from a DataGridView or Table control, including headers and cell values")]
    async fn read_grid(
        &self,
        Parameters(params): Parameters<ReadGridParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::read_grid_impl(
            &self.com_thread,
            &params.element_ref,
            params.start_row,
            params.end_row,
        )
        .await
        .map_err(McpError::from)
    }

    /// Wait for a UI element to appear in a window, polling until found or timeout
    #[tool(description = "Wait for a UI element to appear in a window, polling until found or timeout")]
    async fn wait_for_element(
        &self,
        Parameters(params): Parameters<WaitForElementParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wait_for_element_impl(
            &self.com_thread,
            params.window_handle,
            params.name,
            params.automation_id,
            params.control_type,
            params.timeout_ms,
        )
        .await
        .map_err(McpError::from)
    }

    /// Get the currently focused UI element across all windows
    #[tool(description = "Get the currently focused UI element across all windows")]
    async fn get_focused_element(&self) -> Result<CallToolResult, McpError> {
        tools::get_focused_element_impl(&self.com_thread)
            .await
            .map_err(McpError::from)
    }

    /// Change window state: minimize, maximize, restore, or bring to foreground
    #[tool(description = "Change window state: minimize, maximize, restore, or bring to foreground")]
    async fn set_window_state(
        &self,
        Parameters(params): Parameters<SetWindowStateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::set_window_state_impl(&self.com_thread, params.window_handle, &params.state)
            .await
            .map_err(McpError::from)
    }

    /// Execute a multi-step workflow in a single call. Each step runs sequentially; find_element and
    /// wait_for_element steps automatically pass their result as context to subsequent action steps
    /// (click, set_value, get_value, send_keys, etc.), so you don't need to specify element_ref
    /// for every step. Stops on first error and returns partial results.
    #[tool(description = "Execute a multi-step workflow in a single call. Steps run sequentially with automatic element context passing between find/wait steps and action steps. Supports: find_element, click, set_value, get_value, send_keys, wait_for_element, screenshot, expand_collapse, select_item, wait. Stops on first error and returns partial results.")]
    async fn run_workflow(
        &self,
        Parameters(params): Parameters<RunWorkflowParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::run_workflow_impl(&self.com_thread, params.steps)
            .await
            .map_err(McpError::from)
    }
}

#[tool_handler]
impl ServerHandler for DesktopAutomationServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP server for controlling Windows desktop applications (WinForms, WPF, Win32) \
                 via UI Automation. Use list_windows to discover apps, get_window_tree to explore \
                 controls, and interaction tools to click, type, and read values.\n\n\
                 IMPORTANT: For multi-step operations (filling forms, navigating menus, automating \
                 sequences), prefer the run_workflow tool over calling individual tools one at a time. \
                 run_workflow accepts an array of steps and executes them all server-side with automatic \
                 element context passing — find_element/wait_for_element steps set the context, and \
                 subsequent action steps (click, set_value, send_keys, etc.) use it automatically. \
                 This is faster, more reliable, and prevents context loss on long processes."
                    .to_string(),
            ),
        }
    }
}
