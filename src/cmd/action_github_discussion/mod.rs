use serde::Serialize;

use crate::cmd::acknowledge::acknowledge;
use crate::context::AppContext;
use crate::platform::github;
use crate::platform::multica;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubDiscussionCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub discussion number
  #[clap(long)]
  pub discussion: u64,

  /// GraphQL node id of the discussion comment, when the mention is in a comment rather than the discussion body
  #[clap(long)]
  pub comment_node_id: Option<String>,
}

#[derive(Serialize)]
struct DiscussionMention<'a> {
  repo: &'a str,
  discussion: u64,
  discussion_node_id: &'a str,
  discussion_url: &'a str,
  discussion_title: &'a str,
  discussion_author: &'a str,
  discussion_body: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_node_id: Option<&'a str>,
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
    multica_discussion_mention_webhook,
    pending_runs,
    ..
  }: AppContext,
  ActionGithubDiscussionCommand {
    repo,
    discussion,
    comment_node_id,
  }: ActionGithubDiscussionCommand,
) -> anyhow::Result<()> {
  let Some(webhook) = multica_discussion_mention_webhook else {
    return Ok(());
  };

  let github_discussion = github::get_discussion(&github, &repo, discussion).await?;

  let comment = match &comment_node_id {
    Some(node_id) => Some(github::get_discussion_comment(&github, node_id).await?),
    None => None,
  };

  let response = multica::send_webhook(
    &webhook,
    &DiscussionMention {
      repo: &repo,
      discussion,
      discussion_node_id: &github_discussion.id,
      discussion_url: &github_discussion.url,
      discussion_title: &github_discussion.title,
      discussion_author: github_discussion
        .author
        .as_ref()
        .map(|author| author.login.as_str())
        .unwrap_or_default(),
      discussion_body: &github_discussion.body,
      comment_node_id: comment.as_ref().map(|comment| comment.id.as_str()),
      comment_url: comment.as_ref().map(|comment| comment.url.as_str()),
      comment_author: comment.as_ref().map(|comment| {
        comment
          .author
          .as_ref()
          .map(|author| author.login.as_str())
          .unwrap_or_default()
      }),
      comment_body: comment.as_ref().map(|comment| comment.body.as_str()),
    },
  )
  .await?;

  let subject_id = match comment {
    Some(comment) => comment.id,
    None => github_discussion.id,
  };

  acknowledge(&github, &pending_runs, subject_id, response).await
}
