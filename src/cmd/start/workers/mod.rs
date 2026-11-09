mod poll_autopilot_runs;

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;

pub use self::poll_autopilot_runs::*;
use crate::context::AppContext;

pub fn schedule<F, Fut>(
  app: AppContext,
  task: F,
  interval: Duration,
  wake: Arc<Notify>,
) where
  F: Fn(AppContext) -> Fut + Send + 'static,
  Fut: Future<Output = anyhow::Result<()>> + Send,
{
  tokio::task::spawn(async move {
    loop {
      if let Err(message) = task(app.clone()).await {
        eprintln!("Task failed: {:?}", message);
      }
      tokio::select! {
        _ = tokio::time::sleep(interval) => {}
        _ = wake.notified() => {}
      }
    }
  });
}
