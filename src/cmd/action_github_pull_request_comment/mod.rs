mod prompt;

use std::collections::HashMap;

use octocrab::models::reactions::ReactionContent;
use uuid::Uuid;

use crate::context::AppContext;
use crate::platform::crypto;
use crate::platform::github;
use crate::platform::multica;
use crate::platform::multica::CreateIssue;

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

pub async fn main(
  AppContext {
    github,
    multica_url,
    multica_token,
    multica_workspace_id,
    multica_project_id,
    github_machine_user_id,
    github_user_reply_list,
    ..
  }: AppContext,
  ActionGithubPullRequestCommentCommand {
    repo,
    pull_request,
    comment_id,
  }: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  let comment = github::get_issue_comment(&github, &repo, comment_id).await?;

  if comment.user.id.into_inner() == github_machine_user_id {
    return Ok(());
  }

  if let Some(users) = &github_user_reply_list
    && !users
      .iter()
      .any(|user| user.eq_ignore_ascii_case(&comment.user.login))
  {
    return Ok(());
  }

  let body = comment.body.as_deref().unwrap_or_default();

  let login = github::get_user_login(&github, github_machine_user_id).await?;
  if !mentions(body, &login) {
    return Ok(());
  }

  let github_pull_request = github::get_pull_request(&github, &repo, pull_request).await?;

  let description = prompt::prompt_top_level(&repo, &github_pull_request.head.ref_field, body)?;

  let key = crypto::hash_key(&format!("{repo}/pulls/{pull_request}"));

  let outcome = multica::create_issue(
    &multica_url,
    &multica_token,
    &multica_workspace_id,
    CreateIssue {
      title: &Uuid::new_v4().to_string(),
      description: &description,
      project_id: multica_project_id.as_deref(),
      agent: None,
      metadata: HashMap::from([(key.clone(), key)]),
    },
  )
  .await?;

  println!("Created  {}", outcome.issue().identifier);

  github::add_issue_comment_reaction(&github, &repo, comment_id, ReactionContent::Eyes).await?;

  Ok(())
}

// GitHub logins are case-insensitive, a longer login sharing the prefix
// (`@bot` vs `@bot-two`) is a different user, and an `@` inside a word is an
// email address rather than a mention.
fn mentions(
  body: &str,
  login: &str,
) -> bool {
  let is_login_char = |c: char| c.is_ascii_alphanumeric() || c == '-';

  body.match_indices('@').any(|(at, _)| {
    if body[..at].chars().next_back().is_some_and(is_login_char) {
      return false;
    }
    let rest = &body[at + 1..];
    let end = rest.find(|c: char| !is_login_char(c)).unwrap_or(rest.len());
    rest[..end].eq_ignore_ascii_case(login)
  })
}
