use serde::Serialize;

use crate::cmd::acknowledge::acknowledge;
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

  /// GitHub pull request comment identifier
  #[clap(long)]
  pub comment_id: u64,

  /// Treat `comment_id` as a review comment left on a line of the diff
  #[clap(long)]
  pub review_comment: bool,
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
  #[serde(skip_serializing_if = "Option::is_none")]
  line: Option<u64>,
}

pub async fn main(
  AppContext {
    github,
    multica_pull_request_comment_webhook,
    pending_runs,
    ..
  }: AppContext,
  ActionGithubPullRequestCommentCommand {
    repo,
    pull_request,
    comment_id,
    review_comment,
  }: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  let Some(webhook) = multica_pull_request_comment_webhook else {
    return Ok(());
  };

  let github_pull_request = github::get_pull_request(&github, &repo, pull_request).await?;
  let pull_request_url = github_pull_request
    .html_url
    .ok_or_else(|| anyhow::anyhow!("pull request {repo}#{pull_request} has no html url"))?;

  let (comment_url, comment_author, comment_body, line) = if review_comment {
    let comment = github::get_pr_comment(&github, &repo, comment_id).await?;
    (
      comment.html_url,
      comment.user.map(|user| user.login).unwrap_or_default(),
      comment.body,
      comment.line,
    )
  } else {
    let comment = github::get_issue_comment(&github, &repo, comment_id).await?;
    (
      comment.html_url.to_string(),
      comment.user.login,
      comment.body.unwrap_or_default(),
      None,
    )
  };

  let response = multica::send_webhook(
    &webhook,
    &PullRequestComment {
      repo: &repo,
      pull_request,
      pull_request_url: pull_request_url.as_str(),
      comment_id,
      comment_url: &comment_url,
      comment_author: &comment_author,
      comment_body: &comment_body,
      line,
    },
  )
  .await?;

  let target = if review_comment {
    github::ReactionTarget::ReviewComment(comment_id)
  } else {
    github::ReactionTarget::IssueComment(comment_id)
  };

  acknowledge(&github, &pending_runs, &repo, target, response).await
}
