use std::collections::HashMap;

use crate::CommandConfig;
use crate::platform::multica;
use crate::platform::multica::CreateIssue;
use crate::platform::multica::IssueStatus;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestCommentCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,

  /// Comments to reply to (may be repeated)
  #[clap(long)]
  pub reply_to: Vec<String>,
}

pub async fn main(
  config: CommandConfig,
  args: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  let key = format!("{}/pulls/{}", args.repo, args.pull_request);

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

    },
    None => {
      multica::create_issue(
        &config.multica_url,
        &config.multica_token,
        &config.multica_workspace_id,
        CreateIssue {
          title: &format!("Address comments on PR: {}", key),
          description: &description_initial(),
          project_id: config.multica_project_id.as_deref(),
          agent: &config.multica_agent,
          metadata,
        },
      )
      .await?;
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
