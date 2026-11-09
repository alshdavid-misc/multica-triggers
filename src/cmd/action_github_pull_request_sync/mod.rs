use serde::Serialize;

use crate::context::AppContext;
use crate::context::PendingRun;
use crate::context::pull_request_key;
use crate::context::run_key;
use crate::platform::github;
use crate::platform::multica;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestSyncCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,
}

#[derive(Serialize)]
struct PullRequestSync<'a> {
  repo: &'a str,
  pull_request: u64,
  pull_request_url: &'a str,
}

pub async fn main(
  AppContext {
    github,
    multica_pull_request_sync_webhook,
    pending_runs,
    poll_runs,
    cleanup_lock,
    ..
  }: AppContext,
  ActionGithubPullRequestSyncCommand { repo, pull_request }: ActionGithubPullRequestSyncCommand,
) -> anyhow::Result<()> {
  let Some(webhook) = multica_pull_request_sync_webhook else {
    return Ok(());
  };

  let github_pull_request = github::get_pull_request(&github, &repo, pull_request).await?;
  let pull_request_url = github_pull_request
    .html_url
    .ok_or_else(|| anyhow::anyhow!("pull request {repo}#{pull_request} has no html url"))?;

  let response = multica::send_webhook(
    &webhook,
    &PullRequestSync {
      repo: &repo,
      pull_request,
      pull_request_url: pull_request_url.as_str(),
    },
  )
  .await?;

  if response.status != "accepted" {
    eprintln!(
      "webhook not accepted ({}) for {repo}#{pull_request}",
      response.status
    );
    return Ok(());
  }

  // Without a run to poll the reaction could never be removed, so it is not added.
  let (Some(run_id), Some(autopilot_id)) = (response.run_id, response.autopilot_id) else {
    return Ok(());
  };

  let subject_id = github_pull_request
    .node_id
    .ok_or_else(|| anyhow::anyhow!("pull request {repo}#{pull_request} has no node id"))?;

  let _cleanup_lock = cleanup_lock.lock().await;

  if let Err(error) = github::add_eyes_reaction(&github, &subject_id).await {
    eprintln!("failed to add reaction to {repo}#{pull_request}: {error:?}");
  }

  {
    let mut pending_runs = pending_runs.lock();

    let replaced = pending_runs.insert(
      pull_request_key(&repo, pull_request),
      PendingRun {
        run_id,
        autopilot_id,
        cleanup: Some(Box::new(move || {
          Box::pin(async move { github::remove_eyes_reaction(&github, &subject_id).await })
        })),
        cancel_requested: false,
      },
    );

    // The replaced run shares this 👀, so it is cancelled without its cleanup.
    if let Some(mut replaced) = replaced {
      replaced.cleanup = None;
      replaced.cancel_requested = true;
      pending_runs.insert(run_key(&replaced.run_id), replaced);
    }
  }

  poll_runs.notify_one();

  Ok(())
}
