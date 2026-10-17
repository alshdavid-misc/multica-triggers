use crate::CommandConfig;

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
  _config: CommandConfig,
  _args: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  Ok(())
}
