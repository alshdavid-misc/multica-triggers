use serde::Serialize;

use crate::context::AppContext;
use crate::context::PendingRun;
use crate::context::PendingRunCleanup;
use crate::context::PullRequestKey;
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

  // Registered before reacting so a finishing older run sees this one and leaves the shared reaction in place.
  {
    let key = PullRequestKey {
      repo,
      number: pull_request,
    };

    let mut pending_runs = pending_runs.lock().unwrap();

    for run in pending_runs.values_mut() {
      if run.pull_request.as_ref() == Some(&key) {
        run.cancel_requested = true;
      }
    }

    pending_runs.insert(
      run_id,
      PendingRun {
        autopilot_id,
        cleanup: PendingRunCleanup::RemoveEyesReaction {
          subject_id: subject_id.clone(),
        },
        pull_request: Some(key),
        cancel_requested: false,
      },
    );
  }

  poll_runs.notify_one();

  github::add_eyes_reaction(&github, &subject_id).await
}
