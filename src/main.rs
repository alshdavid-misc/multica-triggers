mod cmd;
mod context;
mod platform;

use std::sync::Arc;

use clap::Parser;
use clap::Subcommand;

use crate::context::AppContext;

#[derive(Debug, Parser)]
struct Command {
  #[clap(subcommand)]
  command: Commands,

  #[clap(flatten)]
  config: CommandConfig,
}

#[derive(Debug, Clone, Parser)]
struct CommandConfig {
  /// Base URL of the Multica instance
  #[clap(long, env = "MULTICA_URL")]
  pub multica_url: String,

  /// Authentication token for the Multica API
  #[clap(long, env = "MULTICA_TOKEN", hide_env_values = true)]
  pub multica_token: String,

  /// Multica workspace identifier
  #[clap(long, env = "MULTICA_WORKSPACE_ID")]
  pub multica_workspace_id: String,

  /// Authentication token for the GitHub API
  #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
  pub github_token: String,

  #[clap(long, env = "GITHUB_WEBHOOK_SECRET", hide_env_values = true)]
  pub github_webhook_secret: String,

  /// Comma separated numeric IDs of GitHub users the triggers will respond to (everyone when omitted)
  #[clap(long, env = "USER_REPLY_LIST", value_delimiter = ',', value_parser = parse_user_id)]
  pub user_reply_list: Option<Vec<u64>>,

  /// Numeric ID of the machine GitHub user whose mentions create Multica issues
  #[clap(long, env = "GITHUB_MACHINE_USER_ID")]
  pub github_machine_user_id: u64,

  #[clap(long, env = "MULTICA_DISCUSSION_MENTION_WEBHOOK")]
  pub multica_discussion_mention_webhook: Option<String>,

  #[clap(long, env = "MULTICA_ISSUE_MENTION_WEBHOOK")]
  pub multica_issue_mention_webhook: Option<String>,

  #[clap(long, env = "MULTICA_PULL_REQUEST_COMMENT_WEBHOOK")]
  pub multica_pull_request_comment_webhook: Option<String>,

  #[clap(long, env = "MULTICA_PULL_REQUEST_SYNC_WEBHOOK")]
  pub multica_pull_request_sync_webhook: Option<String>,
}

fn parse_user_id(value: &str) -> Result<u64, std::num::ParseIntError> {
  value.trim().parse()
}

#[derive(Debug, Subcommand)]
enum Commands {
  /// Update the binary to the latest release
  #[clap(name = "update", visible_aliases = ["up", "u"])]
  Update(cmd::update::UpdateCommand),

  /// Import a single GitHub issue into Multica
  #[clap(name = "action-github-issue")]
  ActionGithubIssue(cmd::action_github_issue::ActionGithubIssueCommand),

  /// Send a GitHub discussion mention to Multica
  #[clap(name = "action-github-discussion")]
  ActionGithubDiscussion(cmd::action_github_discussion::ActionGithubDiscussionCommand),

  /// Import a single GitHub pull request comment into Multica
  #[clap(name = "action-github-pull-request-comment")]
  ActionGithubPullRequestComment(
    cmd::action_github_pull_request_comment::ActionGithubPullRequestCommentCommand,
  ),

  /// Handle a GitHub pull request being opened, reopened, updated or marked ready for review
  #[clap(name = "action-github-pull-request-sync")]
  ActionGithubPullRequestSync(
    cmd::action_github_pull_request_sync::ActionGithubPullRequestSyncCommand,
  ),

  /// Start the service
  #[clap(name = "start")]
  Start(cmd::start::StartCommand),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cmd = Command::parse();

  let github = octocrab::Octocrab::builder()
    .personal_token(cmd.config.github_token.clone())
    .build()?;

  let ctx = AppContext {
    multica_url: cmd.config.multica_url,
    multica_token: cmd.config.multica_token,
    multica_workspace_id: cmd.config.multica_workspace_id,
    github_webhook_secret: cmd.config.github_webhook_secret,
    github_user_reply_list: cmd.config.user_reply_list,
    github_machine_user_id: cmd.config.github_machine_user_id,
    multica_discussion_mention_webhook: cmd.config.multica_discussion_mention_webhook,
    multica_issue_mention_webhook: cmd.config.multica_issue_mention_webhook,
    multica_pull_request_comment_webhook: cmd.config.multica_pull_request_comment_webhook,
    multica_pull_request_sync_webhook: cmd.config.multica_pull_request_sync_webhook,
    github: Arc::new(github),
    pending_runs: Default::default(),
    poll_runs: Default::default(),
    cleanup_lock: Default::default(),
  };

  match cmd.command {
    Commands::Update(args) => cmd::update::main(args).await,
    Commands::ActionGithubDiscussion(args) => cmd::action_github_discussion::main(ctx, args).await,
    Commands::ActionGithubIssue(args) => cmd::action_github_issue::main(ctx, args).await,
    Commands::ActionGithubPullRequestComment(args) => {
      cmd::action_github_pull_request_comment::main(ctx, args).await
    }
    Commands::ActionGithubPullRequestSync(args) => {
      cmd::action_github_pull_request_sync::main(ctx, args).await
    }
    Commands::Start(args) => cmd::start::main(ctx, args).await,
  }
}
