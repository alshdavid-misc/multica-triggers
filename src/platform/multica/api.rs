use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AutopilotRun {
  pub status: String,
  pub task_id: Option<String>,
}

/// Fetch an autopilot run, or `None` when the run no longer exists.
pub async fn get_autopilot_run(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  autopilot_id: &str,
  run_id: &str,
) -> anyhow::Result<Option<AutopilotRun>> {
  let response = Client::new()
    .get(format!(
      "{base_url}/api/autopilots/{autopilot_id}/runs/{run_id}"
    ))
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .send()
    .await?;

  let status = response.status();

  if status == StatusCode::NOT_FOUND {
    return Ok(None);
  }

  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status} fetching autopilot run {run_id}: {text}");
  }

  Ok(Some(response.json().await?))
}

pub async fn cancel_task(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  task_id: &str,
) -> anyhow::Result<()> {
  let response = Client::new()
    .post(format!("{base_url}/api/tasks/{task_id}/cancel"))
    .header("Authorization", format!("Bearer {token}"))
    .header("X-Workspace-ID", workspace_id)
    .send()
    .await?;

  let status = response.status();

  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    anyhow::bail!("Multica API error {status} cancelling task {task_id}: {text}");
  }

  Ok(())
}
