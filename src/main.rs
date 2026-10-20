mod cmd;
mod context;
mod platform;

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use clap::Subcommand;

use crate::context::AppContext;
use crate::platform::kv::KvStore;

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

  /// Multica project identifier
  #[clap(long, env = "MULTICA_PROJECT_ID")]
  pub multica_project_id: Option<String>,

  /// Multica agent identifier
  #[clap(long, env = "MULTICA_AGENT")]
  pub multica_agent: String,

  /// Authentication token for the GitHub API
  #[clap(long, env = "GITHUB_TOKEN", hide_env_values = true)]
  pub github_token: String,

  /// Label applied to the GitHub issue once it has been imported
  #[clap(long, env = "GITHUB_ISSUE_LABEL")]
  pub label: String,

  /// Users that the triggers will respond to
  #[clap(long, env = "USER_REPLY_LIST")]
  pub user_reply_list: String,

  /// Numeric ID of the machine GitHub user whose mentions create Multica issues
  #[clap(long, env = "GITHUB_MACHINE_USER_ID")]
  pub github_machine_user_id: u64,

  #[clap(long, env = "MULTICA_TRIGGER_DB", default_value_os_t = default_db_path())]
  pub multica_trigger_db: PathBuf,
}

fn default_db_path() -> PathBuf {
  std::env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|dir| dir.join("data.db")))
    .unwrap_or_else(|| PathBuf::from("data.db"))
}

#[derive(Debug, Subcommand)]
enum Commands {
  /// Update the binary to the latest release
  #[clap(name = "update", visible_aliases = ["up", "u"])]
  Update(cmd::update::UpdateCommand),

  /// Import a single GitHub issue into Multica
  #[clap(name = "action-github-issue")]
  ActionGithubIssue(cmd::action_github_issue::ActionGithubIssueCommand),

  /// Import a single GitHub pull request comment into Multica
  #[clap(name = "action-github-pull-request-comment")]
  ActionGithubPullRequestComment(
    cmd::action_github_pull_request_comment::ActionGithubPullRequestCommentCommand,
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

  let state = Arc::new(KvStore::new(&cmd.config.multica_trigger_db).await?);

  let ctx = AppContext {
    multica_url: cmd.config.multica_url,
    multica_token: cmd.config.multica_token,
    multica_workspace_id: cmd.config.multica_workspace_id,
    multica_project_id: cmd.config.multica_project_id,
    multica_agent: cmd.config.multica_agent,
    github_token: cmd.config.github_token,
    github_issue_label: cmd.config.label,
    github_user_reply_list: cmd.config.user_reply_list,
    github_machine_user_id: cmd.config.github_machine_user_id,
    github: Arc::new(github),
    state: state,
  };

  match cmd.command {
    Commands::Update(args) => cmd::update::main(args).await,
    Commands::ActionGithubIssue(args) => cmd::action_github_issue::main(ctx, args).await,
    Commands::ActionGithubPullRequestComment(args) => {
      cmd::action_github_pull_request_comment::main(ctx, args).await
    }
    Commands::Start(args) => cmd::start::main(ctx, args).await,
  }
}
