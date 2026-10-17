use crate::CommandConfig;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestCommentCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long, env = "GITHUB_REPO", hide_env_values = true)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long, env = "GITHUB_PULL_REQUEST", hide_env_values = true)]
  pub pull_request: u64,
}

pub async fn main(
  _config: CommandConfig,
  _args: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  Ok(())
}
