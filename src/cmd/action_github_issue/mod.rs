use crate::context::AppContext;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubIssueCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub issue number
  #[clap(long)]
  pub issue: u64,
}

pub async fn main(
  _ctx: AppContext,
  _args: ActionGithubIssueCommand,
) -> anyhow::Result<()> {
  Ok(())
}
