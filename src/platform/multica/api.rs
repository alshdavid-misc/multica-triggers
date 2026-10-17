use std::collections::HashMap;

use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;

use super::MulticaIssue;

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
  fn issue(&self) -> &MulticaIssue {
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

#[derive(Debug, Deserialize)]
struct DuplicateResponse {
  code: Option<String>,
  issue: Option<MulticaIssue>,
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
    "open_only": "true",
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
