mod poll_multica_issues;

use std::future::Future;
use std::time::Duration;

pub use self::poll_multica_issues::*;
use crate::context::AppContext;

pub fn schedule<F, Fut>(
  app: AppContext,
  task: F,
  interval: Duration,
) where
  F: Fn(AppContext) -> Fut + Send + 'static,
  Fut: Future<Output = anyhow::Result<()>> + Send,
{
  tokio::task::spawn(async move {
    loop {
      if let Err(message) = task(app.clone()).await {
        eprintln!("Task failed: {:?}", message);
      }
      tokio::time::sleep(interval).await;
    }
  });
}
