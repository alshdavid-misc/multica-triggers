use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Deserialize)]
pub struct WebhookResponse {
  #[serde(default)]
  pub status: String,
  pub run_id: Option<String>,
  pub autopilot_id: Option<String>,
}

pub async fn send_webhook<T: Serialize>(
  webhook: &str,
  payload: &T,
) -> anyhow::Result<WebhookResponse> {
  let body = serde_json::to_string_pretty(payload)?;

  let response = Client::new()
    .post(webhook)
    .header(CONTENT_TYPE, "application/json")
    .body(body)
    .send()
    .await
    .inspect_err(|error| eprintln!("webhook request failed {webhook}: {error:?}"))?;

  let status = response.status();
  let text = response.text().await.unwrap_or_default();

  if !status.is_success() {
    eprintln!("webhook error {status} {webhook}: {text}");
    anyhow::bail!("webhook error {status}: {text}");
  }

  Ok(serde_json::from_str(&text).unwrap_or_default())
}
