use octocrab::models::Author;
use octocrab::models::Repository;
use octocrab::models::webhook_events::WebhookEvent;
use octocrab::models::webhook_events::WebhookEventPayload;
use octocrab::models::webhook_events::payload::DiscussionCommentWebhookEventAction;
use octocrab::models::webhook_events::payload::DiscussionCommentWebhookEventPayload;
use octocrab::models::webhook_events::payload::DiscussionWebhookEventAction;
use octocrab::models::webhook_events::payload::DiscussionWebhookEventPayload;
use octocrab::models::webhook_events::payload::IssueCommentWebhookEventAction;
use octocrab::models::webhook_events::payload::IssueCommentWebhookEventPayload;
use octocrab::models::webhook_events::payload::IssuesWebhookEventAction;
use octocrab::models::webhook_events::payload::IssuesWebhookEventPayload;
use octocrab::models::webhook_events::payload::PullRequestReviewCommentWebhookEventAction;
use octocrab::models::webhook_events::payload::PullRequestReviewCommentWebhookEventPayload;
use octocrab::models::webhook_events::payload::PullRequestWebhookEventAction;
use octocrab::models::webhook_events::payload::PullRequestWebhookEventPayload;
use uhttp::HandlerResponse;
use uhttp::StatusCode;

use super::super::handler_context::HandlerContext;
use crate::context::AppContext;
use crate::platform::crypto;
use crate::platform::github;

const REVIEW_COMMAND: &str = "review";

pub async fn triggers_github_post(
  mut req: uhttp::Request,
  res: uhttp::Response,
  HandlerContext { ctx, .. }: HandlerContext,
) -> uhttp::Result<HandlerResponse> {
  // ASSUMPTION: `req.headers` is an `http::HeaderMap`. Adjust to uhttp's API.
  let Some(event_name) = req
    .headers
    .get("x-github-event")
    .and_then(|v| v.to_str().ok())
    .map(str::to_owned)
  else {
    return res.status(StatusCode::BAD_REQUEST).body(vec![]);
  };

  let Some(signature) = req
    .headers
    .get("x-hub-signature-256")
    .and_then(|v| v.to_str().ok())
    .map(str::to_owned)
  else {
    return res.status(StatusCode::UNAUTHORIZED).body(vec![]);
  };

  let body = uhttp::body::bytes(&mut req.body).await?;

  if !crypto::verify_github_signature(&ctx.github_webhook_secret, &body, &signature) {
    return res.status(StatusCode::UNAUTHORIZED).body(vec![]);
  }

  let event = WebhookEvent::try_from_header_and_body(&event_name, &body)?;
  let repository = event.repository;
  let sender = event.sender;

  // eprintln!("github event: {event_name}");

  match event.specific {
    WebhookEventPayload::Issues(payload)
      if matches!(payload.action, IssuesWebhookEventAction::Opened) =>
    {
      tokio::spawn(on_issue_opened(ctx, repository, sender, payload));
    }

    WebhookEventPayload::Discussion(payload)
      if matches!(payload.action, DiscussionWebhookEventAction::Created) =>
    {
      tokio::spawn(on_discussion_created(ctx, repository, sender, payload));
    }

    WebhookEventPayload::DiscussionComment(payload)
      if matches!(payload.action, DiscussionCommentWebhookEventAction::Created) =>
    {
      tokio::spawn(on_discussion_comment(ctx, repository, payload));
    }

    WebhookEventPayload::PullRequest(payload) => match &payload.action {
      PullRequestWebhookEventAction::Opened
      | PullRequestWebhookEventAction::Synchronize
      | PullRequestWebhookEventAction::Reopened
      | PullRequestWebhookEventAction::ReadyForReview => {
        tokio::spawn(on_pr_sync(ctx, repository, sender, payload));
      }
      _ => {}
    },

    WebhookEventPayload::PullRequestReviewComment(payload)
      if matches!(
        payload.action,
        PullRequestReviewCommentWebhookEventAction::Created
      ) =>
    {
      tokio::spawn(on_pr_review_comment(ctx, repository, payload));
    }

    WebhookEventPayload::IssueComment(payload) => {
      // `issue_comment` fires for both issues and PRs. For a PR,
      // `issue.pull_request` is populated.
      let on_pr = payload.issue.pull_request.is_some();

      match (&payload.action, on_pr) {
        (IssueCommentWebhookEventAction::Created, false) => {
          tokio::spawn(on_issue_comment(ctx, repository, payload));
        }
        (IssueCommentWebhookEventAction::Created, true) => {
          tokio::spawn(on_pr_comment(ctx, repository, payload));
        }
        _ => {}
      }
    }

    _ => {}
  }

  res.status(StatusCode::NO_CONTENT).body(vec![])
}

async fn on_issue_opened(
  ctx: AppContext,
  repository: Option<Repository>,
  sender: Option<Author>,
  payload: Box<IssuesWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_issue::ActionGithubIssueCommand;
  use crate::cmd::action_github_issue::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("issue event has no repository"))?;

  if sender.is_some_and(|sender| sender.id.into_inner() == ctx.github_machine_user_id) {
    return Ok(());
  }

  let body = payload.issue.body.as_deref().unwrap_or_default();
  if !mentions_machine_user(&ctx, body).await? {
    return Ok(());
  }

  main(
    ctx,
    ActionGithubIssueCommand {
      repo,
      issue: payload.issue.number,
      comment_id: None,
    },
  )
  .await
}

async fn on_issue_comment(
  ctx: AppContext,
  repository: Option<Repository>,
  payload: Box<IssueCommentWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_issue::ActionGithubIssueCommand;
  use crate::cmd::action_github_issue::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("issue comment event has no repository"))?;

  if payload.comment.user.id.into_inner() == ctx.github_machine_user_id {
    return Ok(());
  }

  let body = payload.comment.body.as_deref().unwrap_or_default();
  if !mentions_machine_user(&ctx, body).await? {
    return Ok(());
  }

  main(
    ctx,
    ActionGithubIssueCommand {
      repo,
      issue: payload.issue.number,
      comment_id: Some(payload.comment.id.into_inner()),
    },
  )
  .await
}

async fn on_pr_comment(
  ctx: AppContext,
  repository: Option<Repository>,
  payload: Box<IssueCommentWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_pull_request_comment::ActionGithubPullRequestCommentCommand;
  use crate::cmd::action_github_pull_request_comment::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("pull request comment event has no repository"))?;

  if payload.comment.user.id.into_inner() == ctx.github_machine_user_id {
    return Ok(());
  }

  let body = payload.comment.body.as_deref().unwrap_or_default();
  let login = machine_user_login(&ctx).await?;

  if github::mentions_command(body, &login, REVIEW_COMMAND) {
    return review_pull_request(ctx, repo, payload.issue.number).await;
  }

  if !github::mentions(body, &login) {
    return Ok(());
  }

  main(
    ctx,
    ActionGithubPullRequestCommentCommand {
      repo,
      pull_request: payload.issue.number,
      comment_id: payload.comment.id.into_inner(),
      review_comment: false,
    },
  )
  .await
}

async fn on_pr_review_comment(
  ctx: AppContext,
  repository: Option<Repository>,
  payload: Box<PullRequestReviewCommentWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_pull_request_comment::ActionGithubPullRequestCommentCommand;
  use crate::cmd::action_github_pull_request_comment::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("pull request review comment event has no repository"))?;

  if payload
    .comment
    .user
    .as_ref()
    .is_some_and(|user| user.id.into_inner() == ctx.github_machine_user_id)
  {
    return Ok(());
  }

  if !mentions_machine_user(&ctx, &payload.comment.body).await? {
    return Ok(());
  }

  main(
    ctx,
    ActionGithubPullRequestCommentCommand {
      repo,
      pull_request: payload.pull_request.number,
      comment_id: payload.comment.id.into_inner(),
      review_comment: true,
    },
  )
  .await
}

async fn on_discussion_created(
  ctx: AppContext,
  repository: Option<Repository>,
  sender: Option<Author>,
  payload: Box<DiscussionWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_discussion::ActionGithubDiscussionCommand;
  use crate::cmd::action_github_discussion::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("discussion event has no repository"))?;

  if sender.is_some_and(|sender| sender.id.into_inner() == ctx.github_machine_user_id) {
    return Ok(());
  }

  let body = payload.discussion["body"].as_str().unwrap_or_default();
  if !mentions_machine_user(&ctx, body).await? {
    return Ok(());
  }

  let discussion = payload.discussion["number"]
    .as_u64()
    .ok_or_else(|| anyhow::anyhow!("discussion event has no discussion number"))?;

  main(
    ctx,
    ActionGithubDiscussionCommand {
      repo,
      discussion,
      comment_node_id: None,
    },
  )
  .await
}

async fn on_discussion_comment(
  ctx: AppContext,
  repository: Option<Repository>,
  payload: Box<DiscussionCommentWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_discussion::ActionGithubDiscussionCommand;
  use crate::cmd::action_github_discussion::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("discussion comment event has no repository"))?;

  if payload.comment["user"]["id"].as_u64() == Some(ctx.github_machine_user_id) {
    return Ok(());
  }

  let body = payload.comment["body"].as_str().unwrap_or_default();
  if !mentions_machine_user(&ctx, body).await? {
    return Ok(());
  }

  let discussion = payload.discussion["number"]
    .as_u64()
    .ok_or_else(|| anyhow::anyhow!("discussion comment event has no discussion number"))?;
  let comment_node_id = payload.comment["node_id"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("discussion comment event has no comment node id"))?
    .to_owned();

  main(
    ctx,
    ActionGithubDiscussionCommand {
      repo,
      discussion,
      comment_node_id: Some(comment_node_id),
    },
  )
  .await
}

async fn machine_user_login(ctx: &AppContext) -> anyhow::Result<String> {
  github::get_user_login(&ctx.github, ctx.github_machine_user_id).await
}

async fn mentions_machine_user(
  ctx: &AppContext,
  body: &str,
) -> anyhow::Result<bool> {
  let login = machine_user_login(ctx).await?;
  Ok(github::mentions(body, &login))
}

async fn review_pull_request(
  ctx: AppContext,
  repo: String,
  pull_request: u64,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_pull_request_sync::ActionGithubPullRequestSyncCommand;
  use crate::cmd::action_github_pull_request_sync::main;

  main(
    ctx,
    ActionGithubPullRequestSyncCommand { repo, pull_request },
  )
  .await
}

async fn on_pr_sync(
  ctx: AppContext,
  repository: Option<Repository>,
  sender: Option<Author>,
  payload: Box<PullRequestWebhookEventPayload>,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_pull_request_sync::ActionGithubPullRequestSyncCommand;
  use crate::cmd::action_github_pull_request_sync::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("pull request event has no repository"))?;

  let sender_id = sender.map(|sender| sender.id.into_inner());

  if let Some(users) = &ctx.github_user_reply_list
    && !sender_id.is_some_and(|id| users.contains(&id))
  {
    return Ok(());
  }

  if sender_id == Some(ctx.github_machine_user_id) {
    return Ok(());
  }

  main(
    ctx,
    ActionGithubPullRequestSyncCommand {
      repo,
      pull_request: payload.number,
    },
  )
  .await
}
