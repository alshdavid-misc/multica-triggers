use crate::context::AppContext;

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
  _ctx: AppContext,
  _args: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  Ok(())
}
