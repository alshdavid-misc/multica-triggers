use serde::Serialize;

use crate::context::AppContext;
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

  multica::send_webhook(
    &webhook,
    &PullRequestSync {
      repo: &repo,
      pull_request,
      pull_request_url: pull_request_url.as_str(),
    },
  )
  .await
}
