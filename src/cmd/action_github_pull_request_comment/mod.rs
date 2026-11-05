use serde::Serialize;

use crate::context::AppContext;
use crate::platform::github;
use crate::platform::multica;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestCommentCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,

  /// GitHub top-level pull request comment identifier
  #[clap(long)]
  pub comment_id: u64,
}

#[derive(Serialize)]
struct PullRequestComment<'a> {
  repo: &'a str,
  pull_request: u64,
  pull_request_url: &'a str,
  comment_id: u64,
  comment_url: &'a str,
  comment_author: &'a str,
  comment_body: &'a str,
}

pub async fn main(
  AppContext {
    github,
    multica_pull_request_comment_webhook,
    ..
  }: AppContext,
  ActionGithubPullRequestCommentCommand {
    repo,
    pull_request,
    comment_id,
  }: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  let Some(webhook) = multica_pull_request_comment_webhook else {
    return Ok(());
  };

  let github_pull_request = github::get_pull_request(&github, &repo, pull_request).await?;
  let pull_request_url = github_pull_request
    .html_url
    .ok_or_else(|| anyhow::anyhow!("pull request {repo}#{pull_request} has no html url"))?;

  let comment = github::get_issue_comment(&github, &repo, comment_id).await?;

  multica::send_webhook(
    &webhook,
    &PullRequestComment {
      repo: &repo,
      pull_request,
      pull_request_url: pull_request_url.as_str(),
      comment_id,
      comment_url: comment.html_url.as_str(),
      comment_author: &comment.user.login,
      comment_body: comment.body.as_deref().unwrap_or_default(),
    },
  )
  .await
}
