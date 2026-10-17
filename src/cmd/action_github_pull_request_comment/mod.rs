use std::collections::HashMap;

use crate::CommandConfig;
use crate::platform::crypto;
use crate::platform::multica;
use crate::platform::multica::CreateIssue;
use crate::platform::multica::CreateIssueOutcome;
use crate::platform::multica::IssueStatus;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestCommentCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,
}

pub async fn main(
  config: CommandConfig,
  args: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  let reference = format!("{}/pulls/{}", args.repo, args.pull_request);
  let key = crypto::hash_key(&reference);

  let mut metadata = HashMap::new();
  metadata.insert(key.clone(), key.clone());

  let found = multica::find_issues_by_metadata(
    &config.multica_url,
    &config.multica_token,
    &config.multica_workspace_id,
    &metadata,
  )
  .await?;

  match found.first() {
    Some(issue) => {
      multica::add_issue_comment(
        &config.multica_url,
        &config.multica_token,
        &config.multica_workspace_id,
        &issue.id,
        &description_comment(),
      )
      .await?;
    }
    None => {
      let outcome = multica::create_issue(
        &config.multica_url,
        &config.multica_token,
        &config.multica_workspace_id,
        CreateIssue {
          title: &format!("Address comments on PR: {}", reference),
          description: &description_initial(),
          project_id: config.multica_project_id.as_deref(),
          agent: &config.multica_agent,
          metadata,
        },
      )
      .await?;

      match outcome {
        CreateIssueOutcome::Created(issue) => {
          multica::update_issue_status(
            &config.multica_url,
            &config.multica_token,
            &config.multica_workspace_id,
            &issue.id,
            IssueStatus::Todo,
            false,
          )
          .await?;
        }
        CreateIssueOutcome::Duplicate(issue) => {
          multica::add_issue_comment(
            &config.multica_url,
            &config.multica_token,
            &config.multica_workspace_id,
            &issue.id,
            &description_comment(),
          )
          .await?;
        }
      }
    }
  };

  Ok(())
}

fn description_initial() -> String {
  "".to_string()
}

fn description_comment() -> String {
  "".to_string()
}
