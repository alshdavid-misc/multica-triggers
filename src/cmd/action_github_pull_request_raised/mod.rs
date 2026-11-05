use crate::context::AppContext;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestRaisedCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,
}

pub async fn main(
  _ctx: AppContext,
  _args: ActionGithubPullRequestRaisedCommand,
) -> anyhow::Result<()> {
  Ok(())
}
