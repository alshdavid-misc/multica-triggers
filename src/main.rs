mod cmd;
mod platform;

use clap::Parser;
use clap::Subcommand;

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
  match cmd.command {
    Commands::Update(args) => cmd::update::main(args).await,
    Commands::ActionGithubIssue(args) => cmd::action_github_issue::main(cmd.config, args).await,
    Commands::ActionGithubPullRequestComment(args) => {
      cmd::action_github_pull_request_comment::main(cmd.config, args).await
    }
    Commands::Start(args) => cmd::start::main(cmd.config, args).await,
  }
}
