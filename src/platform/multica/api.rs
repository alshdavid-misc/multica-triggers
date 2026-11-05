use reqwest::Client;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AutopilotRunResponse {
  status: String,
}

/// Fetch an autopilot run's status, or `None` when the run no longer exists.
pub async fn get_autopilot_run_status(
  base_url: &str,
  token: &str,
  workspace_id: &str,
  autopilot_id: &str,
  run_id: &str,
) -> anyhow::Result<Option<String>> {
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

  let data: AutopilotRunResponse = response.json().await?;
  Ok(Some(data.status))
}
