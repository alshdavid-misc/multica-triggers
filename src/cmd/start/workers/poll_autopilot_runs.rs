use crate::context::AppContext;
use crate::platform::github;
use crate::platform::multica;

pub async fn poll_autopilot_runs(ctx: AppContext) -> anyhow::Result<()> {
  let pending: Vec<_> = ctx
    .pending_runs
    .lock()
    .unwrap()
    .iter()
    .map(|(run_id, run)| (run_id.clone(), run.clone()))
    .collect();

  for (run_id, run) in pending {
    let status = multica::get_autopilot_run_status(
      &ctx.multica_url,
      &ctx.multica_token,
      &ctx.multica_workspace_id,
      &run.autopilot_id,
      &run_id,
    )
    .await;

    match status {
      Ok(Some(status)) if matches!(status.as_str(), "pending" | "issue_created" | "running") => {
        continue;
      }
      Ok(_) => {}
      Err(error) => {
        eprintln!("failed to poll autopilot run {run_id}: {error:?}");
        continue;
      }
    }

    if let Err(error) = github::remove_eyes_reaction(&ctx.github, &run.subject_id).await {
      eprintln!("failed to remove reaction for autopilot run {run_id}: {error:?}");
    }

    ctx.pending_runs.lock().unwrap().remove(&run_id);
  }

  Ok(())
}
