use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uiautomation::patterns::{
    UIExpandCollapsePattern, UIInvokePattern, UISelectionItemPattern, UIValuePattern,
};

use crate::automation;
use crate::com_thread::ComThreadHandle;
use crate::types::control_types::string_to_control_type;
use crate::types::{McpToolError, parse_element_ref};

// ── Workflow Step Definitions ──

/// A single step in a workflow sequence.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WorkflowStep {
    /// Find an element. Updates the element context for subsequent steps.
    FindElement {
        window_handle: i64,
        name: Option<String>,
        automation_id: Option<String>,
        control_type: Option<String>,
        max_depth: Option<u32>,
    },
    /// Click or invoke an element. Uses element context if element_ref is omitted.
    Click {
        element_ref: Option<String>,
    },
    /// Set the value/text of an element. Uses element context if element_ref is omitted.
    SetValue {
        element_ref: Option<String>,
        value: String,
    },
    /// Read the current value of an element. Uses element context if element_ref is omitted.
    GetValue {
        element_ref: Option<String>,
    },
    /// Send keystrokes to an element or the focused window.
    SendKeys {
        element_ref: Option<String>,
        keys: String,
        interval_ms: Option<u64>,
    },
    /// Wait for an element to appear. Updates the element context.
    WaitForElement {
        window_handle: i64,
        name: Option<String>,
        automation_id: Option<String>,
        control_type: Option<String>,
        timeout_ms: Option<u64>,
    },
    /// Take a screenshot of a window.
    Screenshot {
        window_handle: i64,
    },
    /// Expand or collapse a tree node, combo box, etc.
    ExpandCollapse {
        element_ref: Option<String>,
        /// 'expand' or 'collapse'
        mode: String,
    },
    /// Select an item in a list, combo box, tab, or tree view.
    SelectItem {
        element_ref: Option<String>,
    },
    /// Wait a fixed amount of milliseconds (no UI interaction).
    Wait {
        ms: u64,
    },
}

/// Result of a single workflow step.
#[derive(Debug, Clone, Serialize)]
struct StepResult {
    step: usize,
    action: String,
    status: String,
    message: String,
}

/// Overall workflow result.
#[derive(Debug, Clone, Serialize)]
struct WorkflowResult {
    completed: usize,
    total: usize,
    status: String,
    results: Vec<StepResult>,
}

// ── Main Entry Point ──

/// Execute a sequence of workflow steps, passing element context between them.
pub async fn run_workflow_impl(
    com: &ComThreadHandle,
    steps: Vec<WorkflowStep>,
) -> Result<CallToolResult, McpToolError> {
    if steps.is_empty() {
        return Err(McpToolError::InvalidParameter(
            "Workflow must contain at least one step".to_string(),
        ));
    }

    let total = steps.len();
    let mut results: Vec<StepResult> = Vec::with_capacity(total);
    let mut element_context: Option<String> = None;
    let mut extra_content: Vec<Content> = Vec::new();

    for (i, step) in steps.into_iter().enumerate() {
        let action_name = step_action_name(&step);

        match execute_step(com, step, &mut element_context, &mut extra_content).await {
            Ok(msg) => {
                results.push(StepResult {
                    step: i + 1,
                    action: action_name,
                    status: "ok".to_string(),
                    message: msg,
                });
            }
            Err(e) => {
                results.push(StepResult {
                    step: i + 1,
                    action: action_name,
                    status: "error".to_string(),
                    message: e.to_string(),
                });

                let workflow_result = WorkflowResult {
                    completed: i,
                    total,
                    status: "error".to_string(),
                    results,
                };

                let json = serde_json::to_string_pretty(&workflow_result)
                    .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

                let mut content = vec![Content::text(json)];
                content.append(&mut extra_content);
                return Ok(CallToolResult::success(content));
            }
        }
    }

    let workflow_result = WorkflowResult {
        completed: total,
        total,
        status: "ok".to_string(),
        results,
    };

    let json = serde_json::to_string_pretty(&workflow_result)
        .map_err(|e| McpToolError::UiAutomation(e.to_string()))?;

    let mut content = vec![Content::text(json)];
    content.append(&mut extra_content);
    Ok(CallToolResult::success(content))
}

// ── Step Dispatcher ──

async fn execute_step(
    com: &ComThreadHandle,
    step: WorkflowStep,
    context: &mut Option<String>,
    extra_content: &mut Vec<Content>,
) -> Result<String, McpToolError> {
    match step {
        WorkflowStep::FindElement {
            window_handle,
            name,
            automation_id,
            control_type,
            max_depth,
        } => {
            execute_find_element(com, window_handle, name, automation_id, control_type, max_depth, context)
                .await
        }
        WorkflowStep::Click { element_ref } => {
            let resolved = resolve_context(element_ref, context)?;
            execute_click(com, &resolved).await
        }
        WorkflowStep::SetValue { element_ref, value } => {
            let resolved = resolve_context(element_ref, context)?;
            execute_set_value(com, &resolved, value).await
        }
        WorkflowStep::GetValue { element_ref } => {
            let resolved = resolve_context(element_ref, context)?;
            execute_get_value(com, &resolved).await
        }
        WorkflowStep::SendKeys {
            element_ref,
            keys,
            interval_ms,
        } => execute_send_keys(com, element_ref.or_else(|| context.clone()), keys, interval_ms).await,
        WorkflowStep::WaitForElement {
            window_handle,
            name,
            automation_id,
            control_type,
            timeout_ms,
        } => {
            execute_wait_for_element(
                com,
                window_handle,
                name,
                automation_id,
                control_type,
                timeout_ms,
                context,
            )
            .await
        }
        WorkflowStep::Screenshot { window_handle } => {
            execute_screenshot(com, window_handle, extra_content).await
        }
        WorkflowStep::ExpandCollapse { element_ref, mode } => {
            let resolved = resolve_context(element_ref, context)?;
            execute_expand_collapse(com, &resolved, &mode).await
        }
        WorkflowStep::SelectItem { element_ref } => {
            let resolved = resolve_context(element_ref, context)?;
            execute_select_item(com, &resolved).await
        }
        WorkflowStep::Wait { ms } => {
            tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            Ok(format!("Waited {}ms", ms))
        }
    }
}

// ── Context Helpers ──

/// Resolve an explicit element_ref or fall back to the current context.
fn resolve_context(
    explicit: Option<String>,
    context: &Option<String>,
) -> Result<String, McpToolError> {
    explicit.or_else(|| context.clone()).ok_or_else(|| {
        McpToolError::InvalidParameter(
            "No element_ref provided and no element context from a previous find_element/wait_for_element step"
                .to_string(),
        )
    })
}

/// Build a JSON-encoded ElementRef from element info. Prefers automation_id > runtime_id > name.
fn build_element_ref_json(
    window_handle: i64,
    automation_id: &str,
    runtime_id: &[i32],
    name: &str,
    control_type: &str,
) -> String {
    if !automation_id.is_empty() {
        serde_json::json!({
            "window_handle": window_handle,
            "automation_id": automation_id,
        })
        .to_string()
    } else if !runtime_id.is_empty() {
        serde_json::json!({
            "window_handle": window_handle,
            "runtime_id": runtime_id,
        })
        .to_string()
    } else {
        let mut obj = serde_json::json!({
            "window_handle": window_handle,
            "name": name,
        });
        if !control_type.is_empty() {
            obj["control_type"] = serde_json::json!(control_type);
        }
        obj.to_string()
    }
}

fn step_action_name(step: &WorkflowStep) -> String {
    match step {
        WorkflowStep::FindElement { .. } => "find_element",
        WorkflowStep::Click { .. } => "click",
        WorkflowStep::SetValue { .. } => "set_value",
        WorkflowStep::GetValue { .. } => "get_value",
        WorkflowStep::SendKeys { .. } => "send_keys",
        WorkflowStep::WaitForElement { .. } => "wait_for_element",
        WorkflowStep::Screenshot { .. } => "screenshot",
        WorkflowStep::ExpandCollapse { .. } => "expand_collapse",
        WorkflowStep::SelectItem { .. } => "select_item",
        WorkflowStep::Wait { .. } => "wait",
    }
    .to_string()
}

// ── Per-Action Handlers ──

async fn execute_find_element(
    com: &ComThreadHandle,
    window_handle: i64,
    name: Option<String>,
    automation_id: Option<String>,
    control_type: Option<String>,
    max_depth: Option<u32>,
    context: &mut Option<String>,
) -> Result<String, McpToolError> {
    if name.is_none() && automation_id.is_none() && control_type.is_none() {
        return Err(McpToolError::InvalidParameter(
            "find_element: at least one of name, automation_id, or control_type required".to_string(),
        ));
    }

    let depth = max_depth.unwrap_or(10);

    let info = com
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

            if let Some(ref aid) = automation_id {
                let aid_clone = aid.clone();
                matcher = matcher.filter_fn(Box::new(
                    move |element: &uiautomation::UIElement| -> Result<bool, uiautomation::Error> {
                        let elem_aid = element.get_automation_id().unwrap_or_default();
                        Ok(elem_aid == aid_clone)
                    },
                ));
            }

            let element = matcher
                .find_first()
                .map_err(|e| McpToolError::UiAutomation(format!("find_element failed: {}", e)))?;

            let info = automation::element_to_info(&element, vec![]);
            Ok((window_handle, info))
        })
        .await?
        .map_err(|e: McpToolError| e)?;

    let (wh, elem_info) = info;

    // Update context with a reference to the found element
    *context = Some(build_element_ref_json(
        wh,
        &elem_info.automation_id,
        &elem_info.runtime_id,
        &elem_info.name,
        &elem_info.control_type,
    ));

    Ok(format!(
        "Found element: '{}' ({})",
        elem_info.name, elem_info.control_type
    ))
}

async fn execute_click(com: &ComThreadHandle, element_ref_json: &str) -> Result<String, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    com.run(move |auto| {
        let element = automation::resolve_element(auto, &element_ref)?;

        if let Ok(invoke) = element.get_pattern::<UIInvokePattern>() {
            invoke
                .invoke()
                .map_err(|e| McpToolError::UiAutomation(format!("Invoke failed: {}", e)))?;
            return Ok("Clicked via InvokePattern".to_string());
        }

        element
            .click()
            .map_err(|e| McpToolError::UiAutomation(format!("Click failed: {}", e)))?;
        Ok("Clicked via mouse click".to_string())
    })
    .await?
    .map_err(|e: McpToolError| e)
}

async fn execute_set_value(
    com: &ComThreadHandle,
    element_ref_json: &str,
    value: String,
) -> Result<String, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    com.run(move |auto| {
        let element = automation::resolve_element(auto, &element_ref)?;

        if let Ok(vp) = element.get_pattern::<UIValuePattern>() {
            vp.set_value(&value)
                .map_err(|e| McpToolError::UiAutomation(format!("SetValue failed: {}", e)))?;
            return Ok(format!("Set value: '{}'", value));
        }

        element
            .set_focus()
            .map_err(|e| McpToolError::UiAutomation(format!("SetFocus failed: {}", e)))?;
        element
            .send_keys("{Ctrl}a", 10)
            .map_err(|e| McpToolError::UiAutomation(format!("Ctrl+A failed: {}", e)))?;
        element
            .send_keys(&value, 10)
            .map_err(|e| McpToolError::UiAutomation(format!("send_keys failed: {}", e)))?;

        Ok(format!("Set value via keys: '{}'", value))
    })
    .await?
    .map_err(|e: McpToolError| e)
}

async fn execute_get_value(
    com: &ComThreadHandle,
    element_ref_json: &str,
) -> Result<String, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    com.run(move |auto| {
        let element = automation::resolve_element(auto, &element_ref)?;

        if let Ok(vp) = element.get_pattern::<UIValuePattern>() {
            if let Ok(val) = vp.get_value() {
                return Ok(format!("Value: '{}'", val));
            }
        }

        let name = element.get_name().unwrap_or_default();
        Ok(format!("Value: '{}'", name))
    })
    .await?
    .map_err(|e: McpToolError| e)
}

async fn execute_send_keys(
    com: &ComThreadHandle,
    element_ref_json: Option<String>,
    keys: String,
    interval_ms: Option<u64>,
) -> Result<String, McpToolError> {
    let element_ref = match element_ref_json {
        Some(ref json) => Some(parse_element_ref(json)?),
        None => None,
    };

    let interval = interval_ms.unwrap_or(10);

    com.run(move |auto| {
        let element = match element_ref {
            Some(ref eref) => automation::resolve_element(auto, eref)?,
            None => auto
                .get_focused_element()
                .map_err(|e| McpToolError::UiAutomation(format!("get_focused_element: {}", e)))?,
        };

        let _ = element.set_focus();

        element
            .send_keys(&keys, interval)
            .map_err(|e| McpToolError::UiAutomation(format!("send_keys failed: {}", e)))?;

        Ok(format!("Sent keys: '{}'", keys))
    })
    .await?
    .map_err(|e: McpToolError| e)
}

async fn execute_wait_for_element(
    com: &ComThreadHandle,
    window_handle: i64,
    name: Option<String>,
    automation_id: Option<String>,
    control_type: Option<String>,
    timeout_ms: Option<u64>,
    context: &mut Option<String>,
) -> Result<String, McpToolError> {
    if name.is_none() && automation_id.is_none() && control_type.is_none() {
        return Err(McpToolError::InvalidParameter(
            "wait_for_element: at least one of name, automation_id, or control_type required"
                .to_string(),
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
                    matcher = matcher.filter_fn(Box::new(
                        move |element: &uiautomation::UIElement| -> Result<bool, uiautomation::Error> {
                            let elem_aid = element.get_automation_id().unwrap_or_default();
                            Ok(elem_aid == aid_clone)
                        },
                    ));
                }

                match matcher.find_first() {
                    Ok(el) => {
                        let info = automation::element_to_info(&el, vec![]);
                        Ok(Some((window_handle, info)))
                    }
                    Err(_) => Ok(None),
                }
            })
            .await?
            .map_err(|e: McpToolError| e)?;

        if let Some((wh, elem_info)) = result {
            *context = Some(build_element_ref_json(
                wh,
                &elem_info.automation_id,
                &elem_info.runtime_id,
                &elem_info.name,
                &elem_info.control_type,
            ));

            return Ok(format!(
                "Found element: '{}' ({})",
                elem_info.name, elem_info.control_type
            ));
        }

        if start.elapsed().as_millis() as u64 >= timeout {
            return Err(McpToolError::Timeout(timeout));
        }

        tokio::time::sleep(poll_interval).await;
    }
}

async fn execute_screenshot(
    com: &ComThreadHandle,
    window_handle: i64,
    extra_content: &mut Vec<Content>,
) -> Result<String, McpToolError> {
    let result = crate::tools::screenshot_window_impl(com, window_handle).await?;

    // Extract image content from the screenshot result and add to extra_content
    for c in result.content {
        match &c.raw {
            rmcp::model::RawContent::Image(_) => {
                extra_content.push(c);
            }
            _ => {}
        }
    }

    Ok("Screenshot captured".to_string())
}

async fn execute_expand_collapse(
    com: &ComThreadHandle,
    element_ref_json: &str,
    action: &str,
) -> Result<String, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;
    let action_str = action.to_lowercase();

    if action_str != "expand" && action_str != "collapse" {
        return Err(McpToolError::InvalidParameter(
            "action must be 'expand' or 'collapse'".to_string(),
        ));
    }

    com.run(move |auto| {
        let element = automation::resolve_element(auto, &element_ref)?;

        let pattern = element
            .get_pattern::<UIExpandCollapsePattern>()
            .map_err(|_| McpToolError::PatternNotSupported("ExpandCollapsePattern".to_string()))?;

        if action_str == "expand" {
            pattern
                .expand()
                .map_err(|e| McpToolError::UiAutomation(format!("Expand failed: {}", e)))?;
        } else {
            pattern
                .collapse()
                .map_err(|e| McpToolError::UiAutomation(format!("Collapse failed: {}", e)))?;
        }

        let new_state = pattern
            .get_state()
            .map(|s| format!("{:?}", s))
            .unwrap_or_else(|_| "Unknown".to_string());

        Ok(format!(
            "{}d element. State: {}",
            if action_str == "expand" { "Expande" } else { "Collapse" },
            new_state
        ))
    })
    .await?
    .map_err(|e: McpToolError| e)
}

async fn execute_select_item(
    com: &ComThreadHandle,
    element_ref_json: &str,
) -> Result<String, McpToolError> {
    let element_ref = parse_element_ref(element_ref_json)?;

    com.run(move |auto| {
        let element = automation::resolve_element(auto, &element_ref)?;

        let pattern = element
            .get_pattern::<UISelectionItemPattern>()
            .map_err(|_| McpToolError::PatternNotSupported("SelectionItemPattern".to_string()))?;

        pattern
            .select()
            .map_err(|e| McpToolError::UiAutomation(format!("Select failed: {}", e)))?;

        let name = element.get_name().unwrap_or_default();
        Ok(format!("Selected item: '{}'", name))
    })
    .await?
    .map_err(|e: McpToolError| e)
}
