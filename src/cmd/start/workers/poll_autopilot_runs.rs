use crate::context::AppContext;
use crate::context::PendingRunCleanup;
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

    let cleanup = match &run.cleanup {
      PendingRunCleanup::RemoveEyesReaction { subject_id } => {
        github::remove_eyes_reaction(&ctx.github, subject_id).await
      }
      PendingRunCleanup::DeleteIssueComment { repo, comment_id } => {
        github::delete_issue_comment(&ctx.github, repo, *comment_id).await
      }
    };

    if let Err(error) = cleanup {
      eprintln!("failed to clean up autopilot run {run_id}: {error:?}");
    }

    ctx.pending_runs.lock().unwrap().remove(&run_id);
  }

  Ok(())
}
