mod prompt;

use std::collections::HashMap;

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
  }: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  println!("Comment left");

  let github_comment = match github::get_pr_comment(&github, &repo, comment_id).await {
    Ok(comment) => comment,
    Err(err) => {
      eprintln!("{:?}", err);
      return Ok(());
    }
  };

  let username = github_comment.user.map(|user| user.login);
  let content = github_comment.body;

  let github_pull_request = github::get_pull_request(&github, &repo, pull_request).await?;
  let key = crypto::hash_key(&format!("{repo}/pulls/{pull_request}"));

  let multica_issue = create_or_get_multica_ticket(
    &key,
    &repo,
    pull_request,
    &github_pull_request.head.ref_field,
    &multica_url,
    &multica_token,
    &multica_workspace_id,
    multica_project_id.as_deref(),
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
  pull_request: u64,
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
      title: &format!("Github Pull Request: {repo}/pulls/{pull_request}"),
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
