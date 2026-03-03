use std::panic::AssertUnwindSafe;
use std::thread;

use tokio::sync::{mpsc, oneshot};

use crate::types::McpToolError;

type WorkFn = Box<dyn FnOnce(&uiautomation::UIAutomation) + Send>;

struct WorkItem {
    work: WorkFn,
}

/// Handle to the dedicated COM thread that owns the UIAutomation instance.
#[derive(Clone)]
pub struct ComThreadHandle {
    sender: mpsc::UnboundedSender<WorkItem>,
}

impl ComThreadHandle {
    /// Spawn a new OS thread with COM initialized and a UIAutomation instance.
    /// Blocks until the COM thread is ready or returns an error if initialization fails.
    pub fn spawn() -> Result<Self, String> {
        let (tx, mut rx) = mpsc::unbounded_channel::<WorkItem>();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();

        thread::Builder::new()
            .name("desktop-automation-com".into())
            .spawn(move || {
                // UIAutomation::new() handles CoInitializeEx internally (MTA mode).
                // Do NOT call CoInitializeEx manually to avoid RPC_E_CHANGED_MODE.
                let automation = match uiautomation::UIAutomation::new() {
                    Ok(a) => a,
                    Err(e) => {
                        let msg = format!("Failed to create UIAutomation: {}", e);
                        tracing::error!("{}", msg);
                        let _ = ready_tx.send(Err(msg));
                        return;
                    }
                };

                tracing::info!("COM thread started with UIAutomation instance");
                let _ = ready_tx.send(Ok(()));

                // Block this thread and process work items
                while let Some(item) = rx.blocking_recv() {
                    // Catch panics so the COM thread survives bad closures
                    let work = item.work;
                    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        work(&automation);
                    }));
                    if let Err(panic_info) = result {
                        let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "unknown panic".to_string()
                        };
                        tracing::error!("Panic on COM thread (recovered): {}", msg);
                    }
                }

                tracing::info!("COM thread shutting down");
            })
            .map_err(|e| format!("Failed to spawn COM thread: {}", e))?;

        // Wait for the COM thread to signal readiness
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(ComThreadHandle { sender: tx }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("COM thread died before signaling readiness".to_string()),
        }
    }

    /// Dispatch a closure to the COM thread and await the result.
    pub async fn run<F, R>(&self, f: F) -> Result<R, McpToolError>
    where
        F: FnOnce(&uiautomation::UIAutomation) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (result_tx, result_rx) = oneshot::channel();

        let work: WorkFn = Box::new(move |automation| {
            let result = f(automation);
            let _ = result_tx.send(result);
        });

        self.sender
            .send(WorkItem { work })
            .map_err(|_| McpToolError::ComThreadDead)?;

        result_rx.await.map_err(|_| McpToolError::ComThreadDead)
    }
}
