#![allow(dead_code)]

mod automation;
mod com_thread;
mod server;
mod tools;
mod types;

use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr so stdout stays clean for MCP JSON-RPC
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting desktop-automation server");

    // Spawn the dedicated COM thread with UIAutomation
    let com_thread = com_thread::ComThreadHandle::spawn()
        .expect("Failed to initialize COM thread - UIAutomation unavailable");

    // Create the MCP server
    let service = server::DesktopAutomationServer::new(com_thread);

    // Serve via stdio transport
    let server = service
        .serve(rmcp::transport::stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Server error: {:?}", e);
        })?;

    // Wait for shutdown
    server.waiting().await?;

    tracing::info!("desktop-automation server stopped");
    Ok(())
}
