use crate::context::AppContext;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestSyncCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,
}

pub async fn main(
  _ctx: AppContext,
  _args: ActionGithubPullRequestSyncCommand,
) -> anyhow::Result<()> {
  Ok(())
}
