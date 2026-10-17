use std::collections::HashMap;

use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;

use super::MulticaIssue;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum IssueStatus {
  Backlog,
  Todo,
  InProgress,
  InReview,
  Done,
  Blocked,
  Cancelled,
}

/// Parameters for creating a Multica issue.
#[derive(Debug)]
pub struct CreateIssue<'a> {
  pub title: &'a str,
  pub description: &'a str,
  pub project_id: Option<&'a str>,
  pub agent: &'a str,
  pub metadata: HashMap<String, String>,
}

/// Outcome of a create-issue request.
#[derive(Debug)]
pub enum CreateIssueOutcome {
  /// The issue was created.
  Created(MulticaIssue),
  /// An equivalent issue is already active and was returned instead.
  Duplicate(MulticaIssue),
}

impl CreateIssueOutcome {
  pub fn issue(&self) -> &MulticaIssue {
    match self {
      CreateIssueOutcome::Created(issue) | CreateIssueOutcome::Duplicate(issue) => issue,
    }
  }
}

/// Create an issue, reporting when an equivalent issue is already active.
///
/// Metadata is not a create field — the API only accepts it one key at a time —
/// so it is written per key once the issue id is known.
pub async fn create_issue(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  issue: CreateIssue<'_>,
) -> anyhow::Result<CreateIssueOutcome> {
  let body = serde_json::json!({
    "title": issue.title,
    "description": issue.description,
    "project_id": issue.project_id,
    "assignee_type": "agent",
    "assignee_id": issue.agent,
    "status": IssueStatus::Backlog,
  });

  let response = Client::new()
    .post(format!("{base_url}/api/issues"))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .json(&body)
    .send()
    .await?;

  let status = response.status();

  let outcome = if status == StatusCode::CONFLICT {
    let data: DuplicateResponse = response.json().await?;
    match data.issue {
      Some(issue) if data.code.as_deref() == Some("active_duplicate_issue") => {
        CreateIssueOutcome::Duplicate(issue)
      }
      _ => anyhow::bail!("Multica API error {status}: active_duplicate_issue"),
    }
  } else if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status}: {text}");
  } else {
    CreateIssueOutcome::Created(response.json().await?)
  };

  for (key, value) in &issue.metadata {
    set_issue_metadata(
      base_url,
      token,
      workspace_id,
      &outcome.issue().id,
      key,
      value,
    )
    .await?;
  }

  Ok(outcome)
}

#[derive(Debug, Deserialize)]
struct DuplicateResponse {
  code: Option<String>,
  issue: Option<MulticaIssue>,
}

/// Set an issue's status.
///
/// Only leaving `backlog` starts the agent's run; `suppress_run` skips that.
pub async fn update_issue_status(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  issue_id: &str,
  status: IssueStatus,
  suppress_run: bool,
) -> anyhow::Result<MulticaIssue> {
  let response = Client::new()
    .put(format!("{base_url}/api/issues/{issue_id}"))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .json(&serde_json::json!({
      "status": status,
      "suppress_run": suppress_run,
    }))
    .send()
    .await?;

  let status_code = response.status();

  if !status_code.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status_code} updating status of {issue_id}: {text}");
  }

  Ok(response.json().await?)
}

#[derive(Debug, Deserialize)]
struct IssueStatusResponse {
  status: String,
}

/// Fetch an issue's current status key.
async fn get_issue_status(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  issue_id: &str,
) -> anyhow::Result<String> {
  let response = Client::new()
    .get(format!("{base_url}/api/issues/{issue_id}"))
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .send()
    .await?;

  let status = response.status();

  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status} fetching {issue_id}: {text}");
  }

  let data: IssueStatusResponse = response.json().await?;
  Ok(data.status)
}

/// Set a single metadata key on an issue.
async fn set_issue_metadata(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  issue_id: &str,
  key: &str,
  value: &str,
) -> anyhow::Result<()> {
  let response = Client::new()
    .put(format!("{base_url}/api/issues/{issue_id}/metadata/{key}"))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .json(&serde_json::json!({ "value": value }))
    .send()
    .await?;

  let status = response.status();

  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status} setting metadata {key}: {text}");
  }

  Ok(())
}

/// Find open issues whose metadata contains every given key/value pair.
pub async fn find_issues_by_metadata(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  metadata: &HashMap<String, String>,
) -> anyhow::Result<Vec<MulticaIssue>> {
  let body = serde_json::json!({
    "metadata": serde_json::to_string(metadata)?,
  });

  let response = Client::new()
    .post(format!("{base_url}/api/issues/query"))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .json(&body)
    .send()
    .await?;

  let status = response.status();

  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status}: {text}");
  }

  let data: QueryIssuesResponse = response.json().await?;
  Ok(data.issues)
}

#[derive(Debug, Deserialize)]
struct QueryIssuesResponse {
  issues: Vec<MulticaIssue>,
}

/// Add a top-level comment and make sure the assigned agent picks it up.
///
/// The issue is moved to `todo` before commenting so the agent never reads a
/// stale `in_review` or `cancelled` status. The comment, not the status change,
/// starts the run, so the status change suppresses its own run.
pub async fn add_issue_comment(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  issue_id: &str,
  comment: &str,
) -> anyhow::Result<()> {
  let current = get_issue_status(base_url, token, workspace_id, issue_id).await?;

  if !matches!(current.as_str(), "todo" | "in_progress") {
    update_issue_status(
      base_url,
      token,
      workspace_id,
      issue_id,
      IssueStatus::Todo,
      true,
    )
    .await?;
  }

  let response = Client::new()
    .post(format!("{base_url}/api/issues/{issue_id}/comments"))
    .header("Content-Type", "application/json")
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .json(&serde_json::json!({ "content": comment }))
    .send()
    .await?;

  let status = response.status();

  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status} commenting on {issue_id}: {text}");
  }

  Ok(())
}
