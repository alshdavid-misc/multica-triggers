mod prompt;

use std::collections::HashMap;

use uuid::Uuid;

use crate::CommandConfig;
use crate::context::AppContext;
use crate::platform::crypto;
use crate::platform::github;
use crate::platform::multica;
use crate::platform::multica::CreateIssue;
use crate::platform::multica::CreateIssueOutcome;
use crate::platform::multica::IssueStatus;
use crate::platform::multica::MulticaIssue;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestCommentCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,

  /// GitHub pull request number
  #[clap(long)]
  pub comment_id: u64,

  /// Where the comment was left on the pull request
  #[clap(long, value_enum)]
  pub kind: PullRequestCommentKind,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PullRequestCommentKind {
  /// Conversation tab comment, not tied to a file or line
  General,
  /// Review comment on a line of the diff
  Inline,
}

pub async fn main(
  AppContext {
    github,
    multica_url,
    multica_token,
    multica_workspace_id,
    multica_project_id,
    multica_agent,
    github_issue_label: label,
    ..
  }: AppContext,
  ActionGithubPullRequestCommentCommand {
    repo,
    pull_request,
    comment_id,
    kind,
  }: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  println!("Comment left");

  let rendered = match kind {
    PullRequestCommentKind::General => github::get_issue_comment(&github, &repo, comment_id)
      .await
      .and_then(|comment| {
        let prompt =
          prompt::prompt_pull_request_general_comment(comment.body.as_deref().unwrap_or_default())?;
        Ok((prompt, None))
      }),
    PullRequestCommentKind::Inline => github::get_pr_comment(&github, &repo, comment_id)
      .await
      .and_then(|comment| {
        let prompt = prompt::prompt_pull_request_inline_comment(&comment)?;
        let reply = comment
          .in_reply_to_id
          .map(|id| (id.into_inner(), comment.body));
        Ok((prompt, reply))
      }),
  };

  let (prompt, reply) = match rendered {
    Ok(rendered) => rendered,
    Err(err) => {
      eprintln!("{:?}", err);
      return Ok(());
    }
  };

  let github_pull_request = github::get_pull_request(&github, &repo, pull_request).await?;
  let key = crypto::hash_key(&format!("{repo}/pulls/{pull_request}"));

  let multica_issue = create_or_get_multica_ticket(
    &key,
    &repo,
    &github_pull_request.head.ref_field,
    &multica_url,
    &multica_token,
    &multica_workspace_id,
    multica_project_id.as_deref(),
  )
  .await?;

  let parent_id = match &reply {
    Some((reply_to, _)) => {
      let marker = format!(
        "<comment_id>{}</comment_id>",
        crypto::hash_key(&reply_to.to_string())
      );
      multica::list_root_comments(
        &multica_url,
        &multica_token,
        &multica_workspace_id,
        &multica_issue.id,
      )
      .await?
      .into_iter()
      .find(|comment| comment.content.contains(&marker))
      .map(|comment| comment.id)
    }
    None => None,
  };

  // A reply without a thread to sit in is posted top-level, where it needs the
  // full prompt for the agent to know which code it refers to.
  let multica_comment = match (&parent_id, reply) {
    (Some(_), Some((_, body))) => body,
    _ => prompt,
  };

  multica::add_issue_comment(
    &multica_url,
    &multica_token,
    &multica_workspace_id,
    &multica_issue.id,
    &multica_comment,
    parent_id.as_deref(),
  )
  .await?;

  // let reference = format!("{}/pulls/{}", args.repo, args.pull_request);
  // let key = crypto::hash_key(&reference);

  // let mut metadata = HashMap::new();
  // metadata.insert(key.clone(), key.clone());

  // let found = multica::find_issues_by_metadata(
  //   &multica_url,
  //   &multica_token,
  //   &multica_workspace_id,
  //   &metadata,
  // )
  // .await?;

  // match found.first() {
  //   Some(issue) => {
  //     multica::add_issue_comment(
  //       &multica_url,
  //       &multica_token,
  //       &multica_workspace_id,
  //       &issue.id,
  //       &description_comment(&args.repo, args.pull_request),
  //     )
  //     .await?;
  //   }
  //   None => {
  //     let outcome = multica::create_issue(
  //       &multica_url,
  //       &multica_token,
  //       &multica_workspace_id,
  //       CreateIssue {
  //         title: &format!("Address comments on PR: {}", reference),
  //         description: &description_initial(&args.repo, args.pull_request),
  //         project_id: multica_project_id.as_deref(),
  //         agent: &multica_agent,
  //         metadata,
  //       },
  //     )
  //     .await?;

  //     match outcome {
  //       CreateIssueOutcome::Created(issue) => {
  //         multica::update_issue_status(
  //           &multica_url,
  //           &multica_token,
  //           &multica_workspace_id,
  //           &issue.id,
  //           IssueStatus::Todo,
  //           false,
  //         )
  //         .await?;
  //       }
  //       CreateIssueOutcome::Duplicate(issue) => {
  //         multica::add_issue_comment(
  //           &multica_url,
  //           &multica_token,
  //           &multica_workspace_id,
  //           &issue.id,
  //           &description_comment(&args.repo, args.pull_request),
  //         )
  //         .await?;
  //       }
  //     }
  //   }
  // };

  Ok(())
}

async fn create_or_get_multica_ticket(
  key: &str,
  repo: &str,
  branch: &str,
  multica_url: &str,
  multica_token: &str,
  multica_workspace_id: &str,
  multica_project_id: Option<&str>,
) -> anyhow::Result<MulticaIssue> {
  let mut metadata = HashMap::<String, String>::new();
  metadata.insert(key.to_string(), key.to_string());

  let found =
    multica::find_issues_by_metadata(multica_url, multica_token, multica_workspace_id, &metadata)
      .await?;

  if let Some(issue) = found.into_iter().next() {
    return Ok(issue);
  }

  let outcome = multica::create_issue(
    multica_url,
    multica_token,
    multica_workspace_id,
    CreateIssue {
      title: &format!("Review feedback: {branch} - {}", Uuid::new_v4()),
      description: &prompt::prompt_top_level(repo, branch)?,
      project_id: multica_project_id,
      agent: None,
      metadata,
    },
  )
  .await?;

  match outcome {
    CreateIssueOutcome::Created(issue) | CreateIssueOutcome::Duplicate(issue) => Ok(issue),
  }
}
