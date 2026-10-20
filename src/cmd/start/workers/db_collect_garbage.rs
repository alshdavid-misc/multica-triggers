use std::time::Duration;

use crate::context::AppContext;

pub async fn db_collect_garbage(app: AppContext) -> anyhow::Result<()> {
  app.state.collect_garbage(Duration::from_hours(48)).await?;
  Ok(())
}
