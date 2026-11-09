use std::time::Duration;

use crate::context::AppContext;
use crate::context::PendingRunCleanup;
use crate::platform::github;
use crate::platform::multica;

const FOLLOW_UP_DELAY: Duration = Duration::from_secs(2);

pub async fn poll_autopilot_runs(ctx: AppContext) -> anyhow::Result<()> {
  let pending: Vec<_> = ctx
    .pending_runs
    .lock()
    .unwrap()
    .iter()
    .map(|(run_id, run)| (run_id.clone(), run.clone()))
    .collect();

  let mut follow_up = false;

  for (run_id, run) in pending {
    let autopilot_run = multica::get_autopilot_run(
      &ctx.multica_url,
      &ctx.multica_token,
      &ctx.multica_workspace_id,
      &run.autopilot_id,
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
        if run.cancel_requested
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
        follow_up |= run.cancel_requested;
        continue;
      }
      Ok(_) => {}
      Err(error) => {
        eprintln!("failed to poll autopilot run {run_id}: {error:?}");
        continue;
      }
    }

    // Runs on the same pull request share one 👀 reaction, so it stays until the last of them finishes.
    let shared = {
      let mut pending_runs = ctx.pending_runs.lock().unwrap();
      pending_runs.remove(&run_id);
      run.pull_request.as_ref().is_some_and(|key| {
        pending_runs
          .values()
          .any(|other| other.pull_request.as_ref() == Some(key))
      })
    };

    if shared {
      continue;
    }

    let PendingRunCleanup::RemoveEyesReaction { subject_id } = &run.cleanup;

    if let Err(error) = github::remove_eyes_reaction(&ctx.github, subject_id).await {
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
