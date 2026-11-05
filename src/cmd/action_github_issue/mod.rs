use serde::Serialize;

use crate::cmd::acknowledge::acknowledge;
use crate::context::AppContext;
use crate::platform::github;
use crate::platform::multica;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubIssueCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub issue number
  #[clap(long)]
  pub issue: u64,

  /// GitHub issue comment identifier, when the mention is in a comment rather than the issue body
  #[clap(long)]
  pub comment_id: Option<u64>,
}

#[derive(Serialize)]
struct IssueMention<'a> {
  repo: &'a str,
  issue: u64,
  issue_url: &'a str,
  issue_title: &'a str,
  issue_author: &'a str,
  issue_body: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_id: Option<u64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_url: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_author: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_body: Option<&'a str>,
}

pub async fn main(
  AppContext {
    github,
    multica_issue_mention_webhook,
    pending_runs,
    ..
  }: AppContext,
  ActionGithubIssueCommand {
    repo,
    issue,
    comment_id,
  }: ActionGithubIssueCommand,
) -> anyhow::Result<()> {
  let Some(webhook) = multica_issue_mention_webhook else {
    return Ok(());
  };

  let github_issue = github::get_issue(&github, &repo, issue).await?;

  let comment = match comment_id {
    Some(comment_id) => Some(github::get_issue_comment(&github, &repo, comment_id).await?),
    None => None,
  };

  let response = multica::send_webhook(
    &webhook,
    &IssueMention {
      repo: &repo,
      issue,
      issue_url: github_issue.html_url.as_str(),
      issue_title: &github_issue.title,
      issue_author: &github_issue.user.login,
      issue_body: github_issue.body.as_deref().unwrap_or_default(),
      comment_id,
      comment_url: comment.as_ref().map(|comment| comment.html_url.as_str()),
      comment_author: comment.as_ref().map(|comment| comment.user.login.as_str()),
      comment_body: comment
        .as_ref()
        .map(|comment| comment.body.as_deref().unwrap_or_default()),
    },
  )
  .await?;

  let target = match comment_id {
    Some(comment_id) => github::ReactionTarget::IssueComment(comment_id),
    None => github::ReactionTarget::Issue(issue),
  };

  acknowledge(&github, &pending_runs, &repo, target, response).await
}
