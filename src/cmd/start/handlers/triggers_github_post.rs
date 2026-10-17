use serde::Deserialize;
use uhttp::HandlerResponse;
use uhttp::StatusCode;

use super::super::handler_context::HandlerContext;
use crate::CommandConfig;
use crate::cmd::action_github_issue;
use crate::cmd::action_github_issue::ActionGithubIssueCommand;

#[derive(Debug, Deserialize)]
pub struct GithubWebhookPayload {
  pub action: Option<String>,
  pub issue: Option<GithubWebhookIssue>,
  pub label: Option<GithubWebhookLabel>,
  pub repository: Option<GithubWebhookRepository>,
}

#[derive(Debug, Deserialize)]
pub struct GithubWebhookRepository {
  pub full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubWebhookIssue {
  // pub html_url: Option<String>,
  // pub labels: Option<Vec<GithubWebhookLabel>>,
  pub number: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GithubWebhookLabel {
  pub name: Option<String>,
}

pub async fn triggers_github_post(
  mut req: uhttp::Request,
  res: uhttp::Response,
  HandlerContext { config, .. }: HandlerContext,
) -> uhttp::Result<HandlerResponse> {
  // dbg!(uhttp::body::json::<serde_json::Value>(&mut req.body).await?);
  let body = uhttp::body::json::<serde_json::Value>(&mut req.body).await?;
  dbg!(&body);

  let body = serde_json::from_value::<GithubWebhookPayload>(body)?;

  match body.action.as_deref() {
    Some("labeled") => {
      tokio::spawn(on_labeled((*config).clone(), body));
    }
    _ => {}
  }

  res.status(StatusCode::NO_CONTENT).body(vec![])
}

async fn on_labeled(
  config: CommandConfig,
  body: GithubWebhookPayload,
) -> anyhow::Result<()> {
  let is_sentinel =
    body.label.as_ref().and_then(|label| label.name.as_deref()) == Some(config.label.as_str());

  if is_sentinel
    && let Some(repository) = body.repository.as_ref()
    && let Some(full_name) = repository.full_name.as_deref()
    && let Some(number) = body.issue.as_ref().and_then(|issue| issue.number)
  {
    action_github_issue::main(
      config,
      ActionGithubIssueCommand {
        repo: full_name.to_string(),
        issue: number,
      },
    )
    .await?;
  }

  Ok(())
}
