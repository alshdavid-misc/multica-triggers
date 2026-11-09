use std::time::Duration;

use crate::context::AppContext;
use crate::platform::multica;

const FOLLOW_UP_DELAY: Duration = Duration::from_secs(2);

pub async fn poll_autopilot_runs(ctx: AppContext) -> anyhow::Result<()> {
  let pending: Vec<_> = ctx
    .pending_runs
    .lock()
    .iter()
    .map(|(key, run)| {
      (
        key.clone(),
        run.run_id.clone(),
        run.autopilot_id.clone(),
        run.cancel_requested,
      )
    })
    .collect();

  let mut follow_up = false;

  for (key, run_id, autopilot_id, cancel_requested) in pending {
    let autopilot_run = multica::get_autopilot_run(
      &ctx.multica_url,
      &ctx.multica_token,
      &ctx.multica_workspace_id,
      &autopilot_id,
      &run_id,
    )
    .await;

    match autopilot_run {
      Ok(Some(autopilot_run))
        if matches!(
          autopilot_run.status.as_str(),
          "pending" | "issue_created" | "running"
        ) =>
      {
        // A superseded run without a task yet is retried on the next poll once Multica creates one.
        if cancel_requested
          && let Some(task_id) = &autopilot_run.task_id
          && let Err(error) = multica::cancel_task(
            &ctx.multica_url,
            &ctx.multica_token,
            &ctx.multica_workspace_id,
            task_id,
          )
          .await
        {
          eprintln!("failed to cancel superseded autopilot run {run_id}: {error:?}");
        }
        follow_up |= cancel_requested;
        continue;
      }
      Ok(_) => {}
      Err(error) => {
        eprintln!("failed to poll autopilot run {run_id}: {error:?}");
        continue;
      }
    }

    let _cleanup_lock = ctx.cleanup_lock.lock().await;

    // The key may have been handed to a newer run since the snapshot; that run's entry is not ours to take.
    let cleanup = {
      let mut pending_runs = ctx.pending_runs.lock();
      if pending_runs
        .get(&key)
        .is_some_and(|run| run.run_id == run_id)
      {
        pending_runs.remove(&key).and_then(|run| run.cleanup)
      } else {
        None
      }
    };

    if let Some(cleanup) = cleanup
      && let Err(error) = cleanup().await
    {
      eprintln!("failed to clean up autopilot run {run_id}: {error:?}");
    }
  }

  // Multica settles a cancelled run asynchronously, so a superseded run is
  // re-polled shortly rather than waiting out the full interval.
  if follow_up {
    let poll_runs = ctx.poll_runs.clone();
    tokio::spawn(async move {
      tokio::time::sleep(FOLLOW_UP_DELAY).await;
      poll_runs.notify_one();
    });
  }

  Ok(())
}
