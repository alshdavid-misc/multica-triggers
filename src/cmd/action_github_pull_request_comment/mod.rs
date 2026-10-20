use std::collections::HashMap;
use std::sync::LazyLock;

use handlebars::Handlebars;
use handlebars::handlebars_helper;
use octocrab::models::pulls::Comment as PullRequestComment;
use octocrab::models::pulls::Review;
use octocrab::models::pulls::ReviewState;
use serde_json::json;

use crate::CommandConfig;
use crate::context::AppContext;
use crate::platform::crypto;
use crate::platform::github;
use crate::platform::multica;
use crate::platform::multica::CreateIssue;
use crate::platform::multica::CreateIssueOutcome;
use crate::platform::multica::IssueStatus;
use crate::platform::multica::MulticaIssue;

#[derive(Debug, clap::Parser)]
pub struct ActionGithubPullRequestCommentCommand {
  /// GitHub repository slug (`owner/name`)
  #[clap(long)]
  pub repo: String,

  /// GitHub pull request number
  #[clap(long)]
  pub pull_request: u64,

  /// GitHub pull request number
  #[clap(long)]
  pub comment_id: u64,
}

pub async fn main(
  AppContext {
    github,
    multica_url,
    multica_token,
    multica_workspace_id,
    multica_project_id,
    multica_agent,
    github_issue_label: label,
    ..
  }: AppContext,
  ActionGithubPullRequestCommentCommand {
    repo,
    pull_request,
    comment_id,
  }: ActionGithubPullRequestCommentCommand,
) -> anyhow::Result<()> {
  println!("Comment left");

  let github_comment = match github::get_pr_comment(&github, &repo, comment_id).await {
    Ok(comment) => comment,
    Err(err) => {
      eprintln!("{:?}", err);
      return Ok(());
    }
  };

  let username = github_comment.user.map(|user| user.login);
  let content = github_comment.body;

  dbg!(&username);
  dbg!(&content);

  // let reference = format!("{}/pulls/{}", args.repo, args.pull_request);
  // let key = crypto::hash_key(&reference);

  // let mut metadata = HashMap::new();
  // metadata.insert(key.clone(), key.clone());

  // let found = multica::find_issues_by_metadata(
  //   &multica_url,
  //   &multica_token,
  //   &multica_workspace_id,
  //   &metadata,
  // )
  // .await?;

  // match found.first() {
  //   Some(issue) => {
  //     multica::add_issue_comment(
  //       &multica_url,
  //       &multica_token,
  //       &multica_workspace_id,
  //       &issue.id,
  //       &description_comment(&args.repo, args.pull_request),
  //     )
  //     .await?;
  //   }
  //   None => {
  //     let outcome = multica::create_issue(
  //       &multica_url,
  //       &multica_token,
  //       &multica_workspace_id,
  //       CreateIssue {
  //         title: &format!("Address comments on PR: {}", reference),
  //         description: &description_initial(&args.repo, args.pull_request),
  //         project_id: multica_project_id.as_deref(),
  //         agent: &multica_agent,
  //         metadata,
  //       },
  //     )
  //     .await?;

  //     match outcome {
  //       CreateIssueOutcome::Created(issue) => {
  //         multica::update_issue_status(
  //           &multica_url,
  //           &multica_token,
  //           &multica_workspace_id,
  //           &issue.id,
  //           IssueStatus::Todo,
  //           false,
  //         )
  //         .await?;
  //       }
  //       CreateIssueOutcome::Duplicate(issue) => {
  //         multica::add_issue_comment(
  //           &multica_url,
  //           &multica_token,
  //           &multica_workspace_id,
  //           &issue.id,
  //           &description_comment(&args.repo, args.pull_request),
  //         )
  //         .await?;
  //       }
  //     }
  //   }
  // };

  Ok(())
}

async fn create_or_get_multica_ticket(
  key: &str,
  repo: &str,
  pull_request: u64,
  branch: &str,
  multica_url: &str,
  multica_token: &str,
  multica_workspace_id: &str,
  multica_project_id: Option<&str>,
) -> anyhow::Result<MulticaIssue> {
  let mut metadata = HashMap::<String, String>::new();
  metadata.insert(key.to_string(), key.to_string());

  let found =
    multica::find_issues_by_metadata(multica_url, multica_token, multica_workspace_id, &metadata)
      .await?;

  if let Some(issue) = found.into_iter().next() {
    return Ok(issue);
  }

  let outcome = multica::create_issue(
    multica_url,
    multica_token,
    multica_workspace_id,
    CreateIssue {
      title: &format!("Github Pull Request: {repo}/pulls/{pull_request}"),
      description: &prompt_top_level(repo, branch)?,
      project_id: multica_project_id,
      agent: None,
      metadata,
    },
  )
  .await?;

  match outcome {
    CreateIssueOutcome::Created(issue) | CreateIssueOutcome::Duplicate(issue) => Ok(issue),
  }
}

static TEMPLATES: LazyLock<Handlebars<'static>> = LazyLock::new(|| {
  let mut handlebars = Handlebars::new();
  handlebars.set_strict_mode(true);
  handlebars.register_escape_fn(handlebars::no_escape);
  handlebars.register_helper("safe", Box::new(safe));

  for (name, template) in [
    ("top_level", include_str!("./prompt_top_level.hbs")),
    (
      "general_comment",
      include_str!("./prompt_pull_request_general_comment.hbs"),
    ),
    (
      "inline_comment",
      include_str!("./prompt_pull_request_inline_comment.hbs"),
    ),
    ("review", include_str!("./prompt_pull_request_review.hbs")),
  ] {
    handlebars
      .register_template_string(name, template)
      .expect("prompt templates are valid");
  }

  handlebars
});

// Comment bodies and diff hunks are reviewer-controlled and are wrapped in XML
// tags, so they must not be able to close or open tags of their own.
handlebars_helper!(safe: |text: str| text
  .replace('&', "&amp;")
  .replace('<', "&lt;")
  .replace('>', "&gt;"));

fn prompt_top_level(
  repo: &str,
  branch: &str,
) -> anyhow::Result<String> {
  Ok(TEMPLATES.render("top_level", &json!({ "repo": repo, "branch": branch }))?)
}

fn prompt_pull_request_general_comment(
  commit_sha: &str,
  body: &str,
) -> anyhow::Result<String> {
  Ok(TEMPLATES.render(
    "general_comment",
    &json!({ "commit_sha": commit_sha, "body": body }),
  )?)
}

fn prompt_pull_request_inline_comment(comment: &PullRequestComment) -> anyhow::Result<String> {
  Ok(TEMPLATES.render(
    "inline_comment",
    &json!({
      "path": comment.path,
      "line": comment.line,
      "start_line": comment.start_line,
      "original_line": comment.original_line,
      "commit_sha": comment.original_commit_id,
      "is_reply": comment.in_reply_to_id.is_some(),
      "thread_id": comment.in_reply_to_id,
      "diff_hunk": comment.diff_hunk,
      "body": comment.body,
    }),
  )?)
}

fn prompt_pull_request_review(review: &Review) -> anyhow::Result<String> {
  let review_state_label = match review.state {
    Some(ReviewState::Approved) => "Approved",
    Some(ReviewState::ChangesRequested) => "Changes requested",
    Some(ReviewState::Commented) => "Commented",
    Some(ReviewState::Dismissed) => "Dismissed",
    Some(ReviewState::Pending) => "Pending",
    _ => "Unknown",
  };

  Ok(TEMPLATES.render(
    "review",
    &json!({
      "review_state_label": review_state_label,
      "commit_sha": review.commit_id.as_deref().unwrap_or_default(),
      "body": review.body.as_deref().unwrap_or_default(),
    }),
  )?)
}
