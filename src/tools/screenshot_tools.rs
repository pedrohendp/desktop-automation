use base64::Engine;
use image::{ImageBuffer, Rgba};
use rmcp::model::{CallToolResult, Content};
use win_screenshot::capture::{capture_window_ex, Area, Using};

use crate::com_thread::ComThreadHandle;
use crate::types::McpToolError;

/// Capture a screenshot of a window and return it as a base64-encoded PNG.
pub async fn screenshot_window_impl(
    com: &ComThreadHandle,
    window_handle: i64,
) -> Result<CallToolResult, McpToolError> {
    let hwnd = window_handle;

    // Validate the window exists via COM thread (UIAutomation element_from_handle)
    // Return () instead of UIElement since UIElement is !Send
    com.run(move |automation| {
        let handle = uiautomation::types::Handle::from(hwnd as isize);
        automation
            .element_from_handle(handle)
            .map(|_| ()) // discard UIElement, just validate it exists
            .map_err(|_| McpToolError::WindowNotFound(hwnd))
    })
    .await??;

    // Check if window is visible and not minimized using Win32 API
    let is_visible = unsafe {
        windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(
            windows::Win32::Foundation::HWND(hwnd as *mut _),
        )
    };
    if !is_visible.as_bool() {
        return Err(McpToolError::ScreenshotFailed(
            "Window is not visible (may be minimized or hidden)".to_string(),
        ));
    }

    let is_iconic = unsafe {
        windows::Win32::UI::WindowsAndMessaging::IsIconic(
            windows::Win32::Foundation::HWND(hwnd as *mut _),
        )
    };
    if is_iconic.as_bool() {
        return Err(McpToolError::ScreenshotFailed(
            "Window is minimized".to_string(),
        ));
    }

    // Capture the window screenshot (does not need COM thread)
    // Use PrintWindow as it works better for background windows
    let buf = capture_window_ex(hwnd as isize, Using::PrintWindow, Area::Full, None, None)
        .map_err(|e| McpToolError::ScreenshotFailed(format!("{:?}", e)))?;

    if buf.width == 0 || buf.height == 0 {
        return Err(McpToolError::ScreenshotFailed(
            "Captured image has zero dimensions".to_string(),
        ));
    }

    // The pixels from win-screenshot are RGBA (32-bit, 4 bytes per pixel)
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(buf.width, buf.height, buf.pixels).ok_or_else(|| {
            McpToolError::ScreenshotFailed("Failed to create image from pixel data".to_string())
        })?;

    // Encode as PNG to a buffer
    let mut png_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| McpToolError::ScreenshotFailed(format!("PNG encoding failed: {}", e)))?;

    // Encode to base64
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    // Return as image content
    Ok(CallToolResult::success(vec![
        Content::text(format!(
            "Screenshot captured: {}x{} pixels, {} bytes PNG",
            buf.width,
            buf.height,
            png_bytes.len()
        )),
        Content::image(b64, "image/png"),
    ]))
}
