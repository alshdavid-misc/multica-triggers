use octocrab::models::Repository;
use octocrab::models::webhook_events::WebhookEvent;
use octocrab::models::webhook_events::WebhookEventPayload;
use octocrab::models::webhook_events::payload::IssueCommentWebhookEventAction;
use octocrab::models::webhook_events::payload::IssueCommentWebhookEventPayload;
use octocrab::models::webhook_events::payload::IssuesWebhookEventAction;
use octocrab::models::webhook_events::payload::IssuesWebhookEventPayload;
use octocrab::models::webhook_events::payload::PullRequestReviewCommentWebhookEventAction;
use octocrab::models::webhook_events::payload::PullRequestReviewCommentWebhookEventPayload;
use uhttp::HandlerResponse;
use uhttp::StatusCode;

use super::super::handler_context::HandlerContext;
use crate::context::AppContext;

/// A comment on a pull request. GitHub delivers these as two different
/// events: conversation-tab comments arrive as `issue_comment`, and inline
/// diff comments (including replies) arrive as `pull_request_review_comment`.
#[derive(Debug)]
pub enum PrComment {
  Conversation(Box<IssueCommentWebhookEventPayload>),
  Review(Box<PullRequestReviewCommentWebhookEventPayload>),
}

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

  let body = uhttp::body::bytes(&mut req.body).await?;

  // let body = uhttp::body::json::<serde_json::Value>(&mut req.body).await?;
  // println!("{}", serde_json::to_string_pretty(&body)?);
  // let body = serde_json::to_vec(&body)?;

  let event = WebhookEvent::try_from_header_and_body(&event_name, &body)?;
  let repository = event.repository;

  match event.specific {
    WebhookEventPayload::Issues(payload) => match &payload.action {
      IssuesWebhookEventAction::Labeled => {
        tokio::spawn(on_issue_labeled(repository, payload));
      }
      _ => {}
    },

    WebhookEventPayload::IssueComment(payload) => {
      // `issue_comment` fires for both issues and PRs. For a PR,
      // `issue.pull_request` is populated.
      let on_pr = payload.issue.pull_request.is_some();

      match (&payload.action, on_pr) {
        (IssueCommentWebhookEventAction::Created, false) => {
          tokio::spawn(on_issue_comment(repository, payload));
        }
        (IssueCommentWebhookEventAction::Created, true) => {
          let comment = PrComment::Conversation(payload);
          tokio::spawn(on_pr_comment(ctx, repository, comment));
        }
        (IssueCommentWebhookEventAction::Edited, true) => {
          let comment = PrComment::Conversation(payload);
          tokio::spawn(on_pr_comment_edited(repository, comment));
        }
        (IssueCommentWebhookEventAction::Deleted, true) => {
          let comment = PrComment::Conversation(payload);
          tokio::spawn(on_pr_comment_deleted(repository, comment));
        }
        _ => {}
      }
    }

    WebhookEventPayload::PullRequestReviewComment(payload) => match &payload.action {
      PullRequestReviewCommentWebhookEventAction::Created => {
        let comment = PrComment::Review(payload);
        tokio::spawn(on_pr_comment(ctx, repository, comment));
      }
      PullRequestReviewCommentWebhookEventAction::Edited => {
        let comment = PrComment::Review(payload);
        tokio::spawn(on_pr_comment_edited(repository, comment));
      }
      PullRequestReviewCommentWebhookEventAction::Deleted => {
        let comment = PrComment::Review(payload);
        tokio::spawn(on_pr_comment_deleted(repository, comment));
      }
      _ => {}
    },

    _ => {}
  }

  res.status(StatusCode::NO_CONTENT).body(vec![])
}

async fn on_issue_labeled(
  _repository: Option<Repository>,
  payload: Box<IssuesWebhookEventPayload>,
) -> anyhow::Result<()> {
  println!("on_issue_labeled");
  // dbg!(&payload);
  Ok(())
}

async fn on_issue_comment(
  _repository: Option<Repository>,
  payload: Box<IssueCommentWebhookEventPayload>,
) -> anyhow::Result<()> {
  println!("on_issue_comment");
  // dbg!(&payload);
  Ok(())
}

async fn on_pr_comment(
  ctx: AppContext,
  repository: Option<Repository>,
  comment: PrComment,
) -> anyhow::Result<()> {
  use crate::cmd::action_github_pull_request_comment::ActionGithubPullRequestCommentCommand;
  use crate::cmd::action_github_pull_request_comment::PullRequestCommentKind;
  use crate::cmd::action_github_pull_request_comment::main;

  let repo = repository
    .and_then(|r| r.full_name)
    .ok_or_else(|| anyhow::anyhow!("pull request comment event has no repository"))?;

  let (pull_request, comment_id, kind) = match &comment {
    PrComment::Conversation(payload) => (
      payload.issue.number,
      payload.comment.id,
      PullRequestCommentKind::General,
    ),
    PrComment::Review(payload) => (
      payload.pull_request.number,
      payload.comment.id,
      PullRequestCommentKind::Inline,
    ),
  };

  main(
    ctx,
    ActionGithubPullRequestCommentCommand {
      repo,
      pull_request,
      comment_id: comment_id.into_inner(),
      kind,
    },
  )
  .await
}

async fn on_pr_comment_edited(
  _repository: Option<Repository>,
  comment: PrComment,
) -> anyhow::Result<()> {
  println!("on_pr_comment_edited");
  // dbg!(&comment);
  Ok(())
}

async fn on_pr_comment_deleted(
  _repository: Option<Repository>,
  comment: PrComment,
) -> anyhow::Result<()> {
  println!("on_pr_comment_deleted");
  // dbg!(&comment);
  Ok(())
}
